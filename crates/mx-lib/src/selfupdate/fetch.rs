//! Self-update: downloading a release asset and verifying it (effectful).
//!
//! A download streams to `<name>.part` inside the destination directory and is
//! renamed into place only after the body is complete, so a killed process can
//! never leave a truncated file that looks finished (spec §3.4). Verification
//! is a separate step: on a checksum mismatch the file is removed and nothing
//! else on disk is touched (§4).

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::selfupdate::index::Asset;

/// Progress sink for [`download`]: called with the number of bytes received so
/// far (cumulative, not a delta), so an `indicatif::ProgressBar` can be driven
/// with `set_position`. `Send + Sync` keeps the returned future `Send`.
pub type ProgressFn<'a> = &'a (dyn Fn(u64) + Send + Sync);

/// Bytes hashed per read in [`verify`]; the file is never held in memory.
const HASH_CHUNK: usize = 64 * 1024;

/// Download `asset` into `dest_dir`, returning the path written.
///
/// The body streams to `dest_dir/<asset.name>.part` and is renamed to
/// `dest_dir/<asset.name>` once complete. A stale `.part` from an earlier
/// interrupted run is truncated, and a failed transfer leaves neither file
/// behind. `progress`, if given, is called with the cumulative byte count.
pub async fn download(
    client: &reqwest::Client,
    asset: &Asset,
    dest_dir: &Path,
    progress: Option<ProgressFn<'_>>,
) -> Result<PathBuf> {
    std::fs::create_dir_all(dest_dir)?;
    let final_path = dest_dir.join(&asset.name);
    let part_path = dest_dir.join(format!("{}.part", asset.name));

    if let Err(e) = stream_to(client, &asset.url, &part_path, progress).await {
        let _ = std::fs::remove_file(&part_path);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&part_path, &final_path) {
        let _ = std::fs::remove_file(&part_path);
        return Err(Error::Io(e));
    }
    Ok(final_path)
}

/// Stream the body of `url` into `part_path`, truncating anything already
/// there. The caller is responsible for removing `part_path` on error.
async fn stream_to(
    client: &reqwest::Client,
    url: &str,
    part_path: &Path,
    progress: Option<ProgressFn<'_>>,
) -> Result<()> {
    let mut resp = send(client, url).await?;
    let mut file = File::create(part_path)?;
    let mut received: u64 = 0;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
        received += chunk.len() as u64;
        if let Some(cb) = progress {
            cb(received);
        }
    }
    file.flush()?;
    Ok(())
}

/// Download a `.sha256` sidecar asset into memory and parse the digest out of
/// it. The body is small by construction (one shasum line).
pub async fn fetch_checksum(client: &reqwest::Client, asset: &Asset) -> Result<String> {
    let resp = send(client, &asset.url).await?;
    parse_sha256(&resp.text().await?)
}

/// Extract a lowercase sha256 hex digest from the contents of a `.sha256` file.
///
/// Accepts the shasum formats `<hex>  <name>` and `<hex> <name>` as well as a
/// bare `<hex>`, with any surrounding whitespace. Anything that is not exactly
/// 64 hex characters is rejected rather than guessed at.
pub fn parse_sha256(text: &str) -> Result<String> {
    let first = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .ok_or_else(|| Error::SelfUpdate("empty checksum file".to_string()))?;
    let token = first.split_whitespace().next().unwrap_or(first);
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::SelfUpdate(format!(
            "not a sha256 checksum: '{first}'"
        )));
    }
    Ok(token.to_ascii_lowercase())
}

/// Stream `path` through sha256 and compare with `expected_hex`
/// (case-insensitive). On a mismatch the file is removed and
/// [`Error::ChecksumMismatch`] is returned; nothing else is touched.
pub fn verify(path: &Path, expected_hex: &str) -> Result<()> {
    let actual = sha256_file(path)?;
    let expected = expected_hex.trim().to_ascii_lowercase();
    if actual == expected {
        return Ok(());
    }
    let _ = std::fs::remove_file(path);
    Err(Error::ChecksumMismatch { expected, actual })
}

/// The sha256 of a file's contents, as lowercase hex.
fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; HASH_CHUNK];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// GET `url` with mx's User-Agent, mapping a non-2xx status onto the same
/// error variants the index client uses.
async fn send(client: &reqwest::Client, url: &str) -> Result<reqwest::Response> {
    let resp = client
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            concat!("mx/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await?;
    let status = resp.status();
    if status.is_success() {
        Ok(resp)
    } else if status.as_u16() == 404 {
        Err(Error::NotFound(url.to_string()))
    } else {
        Err(Error::Api(format!("{status} from {url}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::selfupdate::index::Asset;
    use sha2::{Digest, Sha256};
    use std::sync::atomic::{AtomicU64, Ordering};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const BODY: &[u8] = b"a small pretend tarball\n";

    fn digest_of(bytes: &[u8]) -> String {
        hex::encode(Sha256::digest(bytes))
    }

    async fn serving(name: &str, body: &'static [u8]) -> (MockServer, Asset) {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(format!("/download/{name}")))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
            .mount(&server)
            .await;
        let asset = Asset {
            name: name.to_string(),
            url: format!("{}/download/{name}", server.uri()),
            size: body.len() as u64,
        };
        (server, asset)
    }

    #[test]
    fn parse_sha256_accepts_the_shasum_two_space_format() {
        let hex = digest_of(BODY);
        let line = format!("{hex}  mx-v0.1.2-universal-apple-darwin.tar.gz\n");
        assert_eq!(parse_sha256(&line).unwrap(), hex);
    }

    #[test]
    fn parse_sha256_accepts_a_single_space_before_the_name() {
        let hex = digest_of(BODY);
        let line = format!("{hex} mx-v0.1.2-universal-apple-darwin.tar.gz");
        assert_eq!(parse_sha256(&line).unwrap(), hex);
    }

    #[test]
    fn parse_sha256_accepts_bare_hex() {
        let hex = digest_of(BODY);
        assert_eq!(parse_sha256(&format!("  {hex}\n")).unwrap(), hex);
    }

    #[test]
    fn parse_sha256_normalises_uppercase_hex() {
        let hex = digest_of(BODY);
        let line = format!("{}  file.tar.gz", hex.to_uppercase());
        assert_eq!(parse_sha256(&line).unwrap(), hex);
    }

    #[test]
    fn parse_sha256_rejects_garbage() {
        assert!(parse_sha256("").is_err());
        assert!(parse_sha256("not a checksum\n").is_err());
        // 63 hex chars: one short.
        assert!(parse_sha256(&"a".repeat(63)).is_err());
        // 64 chars, but not all hex.
        assert!(parse_sha256(&format!("{}zz", "a".repeat(62))).is_err());
        // A sha512 sum must not pass as a sha256.
        assert!(parse_sha256(&"a".repeat(128)).is_err());
    }

    #[tokio::test]
    async fn download_writes_the_body_and_verify_accepts_the_matching_digest() {
        let (_server, asset) = serving("mx-v0.1.2-universal-apple-darwin.tar.gz", BODY).await;
        let dir = tempfile::tempdir().unwrap();
        let client = reqwest::Client::new();

        let out = download(&client, &asset, dir.path(), None).await.unwrap();

        assert_eq!(out, dir.path().join(&asset.name));
        assert_eq!(std::fs::read(&out).unwrap(), BODY);
        verify(&out, &digest_of(BODY)).unwrap();
        assert!(out.exists(), "a verified file must survive");
    }

    #[tokio::test]
    async fn download_leaves_no_part_file_behind() {
        let (_server, asset) = serving("bundle.tar.gz", BODY).await;
        let dir = tempfile::tempdir().unwrap();
        download(&reqwest::Client::new(), &asset, dir.path(), None)
            .await
            .unwrap();
        assert!(!dir.path().join("bundle.tar.gz.part").exists());
    }

    #[tokio::test]
    async fn download_reports_cumulative_progress() {
        let (_server, asset) = serving("progress.tar.gz", BODY).await;
        let dir = tempfile::tempdir().unwrap();
        let seen = AtomicU64::new(0);
        let cb = |n: u64| {
            seen.store(n, Ordering::SeqCst);
        };

        download(&reqwest::Client::new(), &asset, dir.path(), Some(&cb))
            .await
            .unwrap();

        assert_eq!(seen.load(Ordering::SeqCst), BODY.len() as u64);
    }

    #[tokio::test]
    async fn download_of_a_missing_asset_is_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/download/gone.tar.gz"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let asset = Asset {
            name: "gone.tar.gz".to_string(),
            url: format!("{}/download/gone.tar.gz", server.uri()),
            size: 0,
        };
        let dir = tempfile::tempdir().unwrap();

        let err = download(&reqwest::Client::new(), &asset, dir.path(), None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound(_)), "got {err:?}");
        assert!(!dir.path().join("gone.tar.gz.part").exists());
        assert!(!dir.path().join("gone.tar.gz").exists());
    }

    #[test]
    fn verify_rejects_a_wrong_digest_and_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bundle.tar.gz");
        std::fs::write(&file, BODY).unwrap();
        let wrong = "0".repeat(64);

        let err = verify(&file, &wrong).unwrap_err();
        match err {
            Error::ChecksumMismatch { expected, actual } => {
                assert_eq!(expected, wrong);
                assert_eq!(actual, digest_of(BODY));
            }
            other => panic!("expected ChecksumMismatch, got {other:?}"),
        }
        assert!(!file.exists(), "a mismatched file must be removed");
    }

    #[test]
    fn verify_is_case_insensitive_about_the_expected_hex() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("bundle.tar.gz");
        std::fs::write(&file, BODY).unwrap();
        verify(&file, &digest_of(BODY).to_uppercase()).unwrap();
        assert!(file.exists());
    }

    #[test]
    fn verify_of_a_missing_file_is_an_io_error() {
        let dir = tempfile::tempdir().unwrap();
        let err = verify(&dir.path().join("absent"), &digest_of(BODY)).unwrap_err();
        assert!(matches!(err, Error::Io(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn fetch_checksum_downloads_and_parses_the_sidecar() {
        let server = MockServer::start().await;
        let hex = digest_of(BODY);
        let body = format!("{hex}  mx-v0.1.2-universal-apple-darwin.tar.gz\n");
        Mock::given(method("GET"))
            .and(path("/download/mx.tar.gz.sha256"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
        let asset = Asset {
            name: "mx.tar.gz.sha256".to_string(),
            url: format!("{}/download/mx.tar.gz.sha256", server.uri()),
            size: 101,
        };

        let got = fetch_checksum(&reqwest::Client::new(), &asset)
            .await
            .unwrap();
        assert_eq!(got, hex);
    }

    #[tokio::test]
    async fn fetch_checksum_of_a_non_2xx_is_an_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/download/x.sha256"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;
        let asset = Asset {
            name: "x.sha256".to_string(),
            url: format!("{}/download/x.sha256", server.uri()),
            size: 0,
        };

        let err = fetch_checksum(&reqwest::Client::new(), &asset)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Api(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn download_replaces_a_stale_part_file() {
        let (_server, asset) = serving("stale.tar.gz", BODY).await;
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("stale.tar.gz.part"), b"leftover junk").unwrap();

        let out = download(&reqwest::Client::new(), &asset, dir.path(), None)
            .await
            .unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), BODY);
    }
}
