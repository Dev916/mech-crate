# mx self-update: a real upgrade path for the mx client

**Date:** 2026-09-02
**Status:** proposed
**Epic:** `bd show mech-crate-4vp` (children 4vp.1–4vp.14; branch `feat/self-update`)

## 1. Problem

`mx self-update` exists today, but it is not an upgrade path. It is a
developer convenience: find a mech-crate checkout, run `cargo build --release`,
copy binaries into `<repo>/bin/`, and sudo-symlink them into `/usr/local/bin`
(`crates/mx-cli/src/commands/self_update.rs`). It requires a Rust toolchain, a
checkout, and sudo. It cannot tell the user whether a newer version exists, and
its checkout-discovery fallback reads a marker file `mx init` never writes
(bd `mech-crate-gjl`).

The release side is equally unfinished:

- `.github/workflows/release.yml` builds, signs, notarizes and packages a
  universal macOS tarball and publishes it to `unyform-ai/mech-crate-releases`,
  but it has **never run**. There are no git tags and no releases.
- The packaging script it calls, `scripts/package.sh`, is **untracked**
  (bd `mech-crate-d5c`, P1). A tag build on a clean clone fails.
- Linux builds, a draft-then-publish flow, and the Homebrew tap bump are
  `TODO` comments at the bottom of the workflow. `unyform-ai/homebrew-tap` is
  an empty repository.
- Nothing in `mx` ever compares its version to anything. `~/.mech-crate/version`
  on this machine says `0.1.0`; the binary says `0.1.1`.

The request is a frictionless path: `mx` should know when it is out of date,
say so, and update itself with one command, on every install kind we support.

## 2. Goals and non-goals

**Goals**

1. `mx self-update` upgrades the running installation to the latest published
   release with one command, verifying what it downloads, and never leaving a
   half-installed state.
2. `mx self-update --check` (and a passive, once-a-day hint on normal
   commands) tells the user a newer version exists.
3. One command works for every install kind: release tarball, Homebrew,
   source checkout, and a bare copied binary.
4. Rollback to the previous release is one command.
5. The release channel actually exists: tagged builds land on the releases
   repo for macOS and Linux, and a curl installer and Homebrew formula
   consume the same tarballs.

**Non-goals (explicitly out of scope for this epic)**

- Release channels other than stable (no nightly, no pre-release opt-in).
- Delta or patch updates; every update downloads a full tarball.
- Auto-applying updates without the user running a command.
- Windows.
- Detached signatures (minisign/cosign). sha256 plus macOS codesign
  verification is the integrity story for now; a signature scheme is a
  follow-up once the pipeline is proven.
- Updating recipe-generated project files. That is `mx upgrade`, a different
  command with its own broken state (see the site's Upgrade page).

## 3. Design decisions

Corpus guidance consulted (`mcp__mx__rag_context`, lexical-only at the time):
the house rules that apply are *atomic operations (temp file → rename)*,
*idempotency by default*, *pure core, effectful shell*, *contract tests at IO
boundaries*, and *minimal dependencies*. Each decision below cites the rule it
serves.

### 3.1 Release channel: GitHub Releases on `unyform-ai/mech-crate-releases`

The releases repo is **public**, so update checks and downloads need no
token. `GET /repos/unyform-ai/mech-crate-releases/releases/latest` returns
the newest published, non-draft, non-prerelease release. That endpoint's
semantics are exactly what a self-updater wants, which is why the pipeline
must move to draft-then-publish (§3.7): a half-uploaded release must never be
`latest`.

- Asset names are already fixed by `scripts/package.sh`:
  `mx-v<version>-<triple>.tar.gz` and `mx-v<version>-<triple>.tar.gz.sha256`
  (shasum format: `<hex>  <filename>`).
- Triples: `universal-apple-darwin`, `x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl`. The client picks one from
  `cfg!(target_os)` / `cfg!(target_arch)`.
- `GITHUB_TOKEN` or `GH_TOKEN`, if set, is sent as a bearer to raise the
  unauthenticated rate limit. Never required.
- `MX_RELEASES_API` overrides the API base URL. This is the seam that lets
  wiremock stand in for GitHub in tests (*contract tests at IO boundaries*)
  and lets a private mirror serve releases later.
- Every request sends a `User-Agent: mx/<version>` header; GitHub rejects
  requests without one.

### 3.2 Install layout for release installs

```
~/.mech-crate/
  releases/
    mx-v0.1.1/            # an extracted tarball, exactly as packaged
      bin/{mx,mx-mcp,mx-ingest}
      bin/lib/*.sh
      templates/
      scripts/
      LICENSE.txt
      VERSION
    mx-v0.1.2/
  current -> releases/mx-v0.1.2      # the only thing an update changes
  templates/                         # refreshed from current/templates
  version                            # mirrors current/VERSION
~/.local/bin/mx      -> ~/.mech-crate/current/bin/mx
~/.local/bin/mx-mcp  -> ~/.mech-crate/current/bin/mx-mcp
~/.local/bin/mx-ingest -> ~/.mech-crate/current/bin/mx-ingest
```

Why this shape:

- **The tarball is already a valid MechCrate root.** `paths::mech_crate_root`
  walks up from the canonicalized executable until it finds `scripts/`;
  `source_templates_dir` does the same for `templates/`. An extracted tarball
  satisfies both without any code change, and `mx-mcp` resolves `<root>/bin/mx`
  the same way. Keeping the tarball intact on disk means no second layout to
  maintain.
- **Atomic swap.** An update extracts into a temp directory under
  `~/.mech-crate/tmp/`, renames it into `releases/`, then replaces the
  `current` symlink with a rename (`current.new` → `current`). At no point does
  a partially extracted tree sit behind `current`. The running process keeps
  its inode and finishes normally (*atomic operations*).
- **Rollback is a symlink flip.** The previous release directory is kept
  (retention: current plus one previous). `mx self-update --rollback` points
  `current` back at it.
- **No sudo.** `~/.local/bin` is the shim directory. The installer prints the
  PATH hint if it is not on PATH. The dev-only `/usr/local/bin` sudo symlink
  dance stays confined to the source-checkout strategy.

### 3.3 Install kinds and strategies

`InstallKind` is a pure function of the canonicalized executable path, the
MechCrate home, and (if `brew` exists) the Homebrew prefix. It is unit-tested
against tempdir layouts (*pure core*).

| Kind | Detected when the exe lives under… | Strategy |
|---|---|---|
| `Release` | `~/.mech-crate/releases/mx-v*/bin/` | Download, verify, extract, flip `current`, refresh templates, prune. |
| `Homebrew` | `<brew prefix>/Cellar/mx/` | Run `brew upgrade mx` (after confirmation, or with `--yes`). Never write into the Cellar ourselves. |
| `Source` | a directory tree containing `Cargo.toml` and `crates/mx-cli` (`<repo>/target/release/mx` or `<repo>/bin/mx`) | Today's rebuild flow, kept: optional `git pull --rebase`, `cargo build --release`, copy to `<repo>/bin/`, ensure symlinks. Fixes gjl by reading `paths::recorded_source_root`. |
| `Bare` | anything else (e.g. a real file at `/usr/local/bin/mx` from `scripts/install.sh`) | Adopt into the release layout (§3.2), then re-point the file at the old path to `current/bin/mx` if its directory is writable; otherwise print the two-line fix. |

`--check` and `--dry-run` print the kind, the current and latest versions,
and the plan, then exit. `mx doctor` gains the same one-line summary.

### 3.4 Update engine: `mx_lib::selfupdate`

New module in `mx-lib` so the MCP server and any future surface can reuse it.
Pure and effectful halves are separate files:

- `version.rs` — semver parse and compare (`semver` crate), current version
  from `CARGO_PKG_VERSION`.
- `target.rs` — triple selection, asset name construction.
- `kind.rs` — `InstallKind` detection.
- `plan.rs` — `UpdatePlan` derivation: `UpToDate | Download {asset, checksum}
  | DelegateBrew | RebuildSource | Adopt {..}`. All inputs are values;
  no IO.
- `index.rs` — `ReleaseIndex` client (reqwest): `latest()` → `Release
  {version, assets}`. The one place that knows GitHub's JSON.
- `fetch.rs` — download an asset to a temp file with a progress bar,
  read the `.sha256` sibling, verify with `sha2` (already a dependency).
  Mismatch is an error and the temp file is removed; nothing else is touched.
- `layout.rs` — the only writer of `~/.mech-crate/{releases,current,tmp}`:
  `adopt_bundle(extracted_dir, version)`, `flip_current(version)`,
  `previous()`, `prune(keep = 2)`, `ensure_shims(bin_dir)`. Every operation is
  idempotent and safe to re-run after a crash (*idempotency by default*).
- `verify.rs` — post-extract checks: `bin/mx --version` of the **new** binary
  reports the expected version; on macOS, `codesign --verify --strict` if
  `codesign` is on PATH (warn, do not fail, if it is absent).

Extraction uses the `flate2` and `tar` crates rather than shelling out. Both
are small, and the hermetic test suite already stubs PATH, so an in-process
extractor keeps tests deterministic (*minimal dependencies*, weighed against
testability; two well-known crates win).

Post-update side effects, in order, each idempotent:

1. Refresh `~/.mech-crate/templates` from `current/templates` (the same copy
   `mx init --update` does; factor that copy into `mx_lib` so both call it).
2. Write `~/.mech-crate/version` from `current/VERSION`.
3. If `~/.mech-crate/mcp/mx-mcp-wrapper.sh` exists, regenerate it to
   `MECH_CRATE_ROOT=~/.mech-crate/current` and `exec current/bin/mx-mcp`, so
   MCP clients keep working across updates without editing their config.
4. Prune releases beyond current-plus-one.

### 3.5 The `self-update` command surface

```
mx self-update                 # detect kind, show plan, confirm, apply
mx self-update --check         # print current/latest/kind; exit 0 up to date, 10 if update available
mx self-update --dry-run       # print the plan, change nothing
mx self-update --yes           # no confirmation
mx self-update --to 0.1.2      # pin a specific release instead of latest (--version is clap's global flag)
mx self-update --rollback      # flip current back to the previous kept release
mx self-update --from-dir DIR  # adopt an already-extracted bundle (used by the curl installer)
mx self-update --pull          # Source kind only: git pull --rebase before building (kept from today)
mx self-update --refresh-cache # internal: refresh the notifier cache and exit (see §3.6)
```

Exit code 10 on `--check` lets scripts and the notifier's background process
branch without parsing text. `--from-dir` makes the Rust code the single
writer of the layout; the shell installer only fetches, verifies and extracts.

### 3.6 Passive update notification

The point of the epic is that nobody has to remember to check.

- **When it runs:** any `mx` invocation except `self-update`, `--version`,
  `--help`, and `mcp run`; only when stderr is a TTY; skipped when `CI` is
  set, when `MX_NO_UPDATE_CHECK=1`, or when
  `~/.mech-crate/config/update.toml` has `check = false`.
- **Never blocks:** the foreground command only reads
  `~/.mech-crate/cache/update-check.json`
  (`{checked_at, next_check_at, latest, current_at_check}`). If the cache is
  older than 24 h it spawns `mx self-update --refresh-cache` detached, with
  stdio to `/dev/null`, and continues. Added latency is one small file read.
- **Offline:** the detached refresh fails silently and writes
  `next_check_at = now + 1h` so an offline laptop does not fork a process on
  every command. Nothing is ever printed about failures.
- **Surface:** one line on stderr after the command's own output:
  `mx 0.1.3 is available (you have 0.1.1). Run: mx self-update` — with
  `brew upgrade mx` substituted for Homebrew installs. Printed at most once
  per 24 h per newer version.

### 3.7 Distribution: making the channel real

- **Commit `scripts/package.sh`** as-is after review (closes d5c). It is the
  contract for asset names and tarball layout that everything above relies
  on.
- **Pipeline dry run.** Confirm the six release secrets exist on
  `Dev916/mech-crate` (`gh secret list` returned nothing, which may be a
  permission limit rather than absence), run `release.yml` by
  `workflow_dispatch` with an rc version, and confirm the tarball and
  checksum land on the releases repo. Only then tag `v0.1.2` (bump the
  workspace version in `Cargo.toml`).
- **Linux job** from the workflow's TODO: `cross` builds for both musl
  triples, packaged and uploaded with the same `gh release` flow.
  `bin/lib/*.sh` are bash, so the bundle works unchanged.
- **Draft-then-publish:** each platform job uploads to a draft; a final
  `publish` job that `needs` all of them flips it published.
- **Homebrew formula** in `unyform-ai/homebrew-tap` (`Formula/mx.rb`): the
  formula installs the whole bundle into `libexec` and symlinks the three
  binaries into `bin`, so `mech_crate_root()` resolves through
  `Cellar/mx/<v>/libexec`. A `tap-bump` job in `release.yml` opens the
  version-bump PR (`mislav/bump-homebrew-formula-action`).
- **Curl installer** at `https://mechcrate.dev/install.sh` (a static file in
  `site/apps/site/public/`): POSIX sh; detects OS and arch, resolves the latest
  tag, downloads tarball and checksum, verifies, extracts to a temp dir, and
  runs `<tmp>/bin/mx self-update --from-dir <tmp> --yes`. Prints the PATH hint.
  Follows `docs/development/SHELL_SCRIPTING_GUIDE.md`.

### 3.8 Documentation

- Site: new `docs/start/install.md` (brew / curl / from source, and how
  updates work), a row per new flag in `cli-reference.md`.
- Repo: README install section switches to the curl and brew one-liners with
  `make install-local` kept for contributors; the Deployment section of
  `docs/development/MX_RUST_CLI_AND_MCP_SERVER.md` documents the release
  layout and the pipeline so the corpus can answer "how do I release mx".

## 4. Error handling

- Network failure or non-200 from the index: a single clear error naming the
  URL; nothing on disk changes. In the notifier path, silence.
- Checksum mismatch: error, temp file deleted, `current` untouched.
- Extraction failure: temp dir deleted, `current` untouched.
- Post-extract verification failure (wrong version, codesign rejects): the
  new release dir is deleted, `current` untouched, error names the check.
- `current` flip fails (permissions): error; the previous `current` is still
  valid because the flip is a rename.
- Homebrew kind but `brew` missing from PATH: error with the exact command to
  run.
- Source kind but `cargo` missing: error pointing at rustup, same as today.
- Every failure path is exercised by a test (§5).

## 5. Testing

House style: `assert_cmd` + `predicates`, hermetic `HOME`, stub-bin PATH,
`wiremock` for HTTP (`crates/mx-cli/tests/cli_surface.rs` is the model).

- **Unit (mx-lib):** semver compare including pre-release ordering; triple
  selection per cfg; `InstallKind` for each layout in tempdirs (release,
  cellar, source `target/release`, source `bin/`, bare); `UpdatePlan` for
  every kind × {up to date, newer, pinned older}; `.sha256` parsing; notifier
  cache TTL and backoff arithmetic.
- **Contract (wiremock):** `ReleaseIndex::latest()` against recorded GitHub
  JSON; asset download with matching and mismatching checksum; 404 asset;
  rate-limit 403 surfaces GitHub's message.
- **Integration (assert_cmd):** `--dry-run` output per kind; `--from-dir`
  adopts a fixture bundle, `~/.local/bin/mx --version` through the shim
  reports the bundle's version, `templates/` and `version` refreshed, wrapper
  script rewritten; second `--from-dir` of a newer bundle then `--rollback`
  restores the first; prune keeps exactly two; notifier prints exactly one
  line on a TTY with a stale cache and a newer version, and nothing with
  `MX_NO_UPDATE_CHECK`, `CI`, a fresh cache, or non-TTY stderr.
- **Known-broken lane:** `kb_self_update_finds_the_source_root_recorded_by_init`
  is un-ignored and must pass (closes gjl).
- **Release pipeline:** the rc dry run in §3.7 is the end-to-end test; its
  evidence (asset URLs) goes in the bd issue.

## 6. Security notes

- Downloads are HTTPS to `api.github.com` and `objects.githubusercontent.com`
  only; the API base override is an env var, not a config file, so a
  tampered config cannot redirect downloads.
- sha256 verification is mandatory and not skippable by flag.
- macOS binaries are Developer ID signed and notarized by the pipeline; the
  updater verifies the signature when `codesign` is present. Files written by
  `mx` carry no quarantine attribute, so Gatekeeper would not otherwise check
  them.
- The updater never elevates. The only sudo in the tree stays in the
  source-checkout strategy, unchanged from today.
- Tokens are read from the environment only and are never written to the
  cache file.

## 7. Dependencies added

`semver`, `flate2`, `tar` in `mx-lib`. `reqwest`, `sha2`, `hex`, `dirs`,
`indicatif` are already present.

## 8. Phasing (maps to bd priorities)

- **P1 — a working updater on macOS:** commit package.sh; pipeline dry run and
  first tag; `mx_lib::selfupdate` (kind, plan, index, fetch, layout, verify);
  `self-update` command rewrite with `--check/--dry-run/--yes/--to/
  --rollback/--from-dir`; Source strategy kept and gjl fixed; cli-reference
  row.
- **P2 — frictionless:** passive notifier; Linux release job; draft-then-
  publish; curl installer and site install page; README and dev-guide docs.
- **P3 — ecosystem:** Homebrew formula and tap-bump job; `mx doctor` install
  kind and update line.

## 9. Open questions surfaced for the owner

None block P1. The owner may override any of these defaults:

1. Shim directory `~/.local/bin` (versus `/usr/local/bin` with sudo).
2. Retention of one previous release (versus more).
3. Notifier TTL of 24 h and the exit code 10 for `--check`.
4. Whether the Homebrew tap should be the recommended install on macOS once
   it exists (the spec treats curl and brew as equals).
