---
title: "Secure Python: Supply Chain, Input Handling, Secrets, and Runtime Hardening"
category: security
languages: [python]
complexity: advanced
use_cases:
  - hardening how a Python service resolves, pins, and installs its dependencies
  - picking the correct idiom per injection class (SQL, subprocess, path, template, regex)
  - choosing password hashing, token generation, and TLS defaults that survive review
  - deciding how to run untrusted Python, and why in-process sandboxing is not an option
  - wiring security linters and dependency gates into CI without drowning in findings
summary: "Primary-source audit of Python security as of 2026-09: what Trusted Publishing and PEP 740 attestations assert, hash-pinned installs and index priority, the modules that execute on load, the right idiom per injection class, password and TLS defaults, and why CPython cannot sandbox itself."
provenance: researched
researched: 2026-09-18
sources:
  - https://docs.pypi.org/trusted-publishers/
  - https://docs.pypi.org/trusted-publishers/using-a-publisher/
  - https://peps.python.org/pep-0740/
  - https://docs.pypi.org/attestations/publish/v1/
  - https://github.com/pypa/gh-action-pypi-publish
  - https://peps.python.org/pep-0751/
  - https://pip.pypa.io/en/stable/news/
  - https://pip.pypa.io/en/stable/topics/secure-installs/
  - https://docs.astral.sh/uv/concepts/indexes/
  - https://github.com/pypa/pip-audit
  - https://google.github.io/osv-scanner/supported-languages-and-lockfiles/
  - https://blog.pypi.org/posts/2024-12-30-quarantine/
  - https://blog.pypi.org/posts/2025-07-31-incident-report-phishing-attack/
  - https://docs.python.org/3/library/pickle.html
  - https://docs.python.org/3/library/marshal.html
  - https://docs.python.org/3/library/shelve.html
  - https://pyyaml.org/wiki/PyYAMLDocumentation
  - https://docs.python.org/3/library/ast.html
  - https://docs.python.org/3.14/library/tarfile.html
  - https://docs.python.org/3/library/zipfile.html
  - https://docs.python.org/3/library/xml.html
  - https://github.com/tiran/defusedxml
  - https://www.psycopg.org/psycopg3/docs/basic/params.html
  - https://www.psycopg.org/psycopg3/docs/basic/from_pg2.html
  - https://docs.python.org/3/library/subprocess.html
  - https://docs.python.org/3/library/shlex.html
  - https://docs.python.org/3/library/pathlib.html
  - https://jinja.palletsprojects.com/en/stable/sandbox/
  - https://docs.python.org/3/library/re.html
  - https://docs.python.org/3/library/secrets.html
  - https://docs.python.org/3/library/hmac.html
  - https://docs.python.org/3/library/hashlib.html
  - https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
  - https://argon2-cffi.readthedocs.io/en/stable/api.html
  - https://cryptography.io/en/latest/fernet/
  - https://cryptography.io/en/latest/hazmat/primitives/aead/
  - https://docs.python.org/3/library/ssl.html
  - https://github.com/vstinner/pysandbox
  - https://peps.python.org/pep-0578/
  - https://docs.python.org/3/using/cmdline.html
  - https://gvisor.dev/docs/architecture_guide/intro/
  - https://docs.python.org/3/library/resource.html
  - https://docs.astral.sh/ruff/rules/
  - https://docs.semgrep.dev/writing-rules/data-flow/taint-mode/overview
  - https://codeql.github.com/codeql-query-help/python/
  - https://github.com/actions/dependency-review-action
  - https://www.python.org/downloads/release/python-3140/
  - https://github.com/yaml/pyyaml/blob/main/CHANGES
---

# Secure Python: Supply Chain, Input Handling, Secrets, and Runtime Hardening

State of practice as of 2026-09. The seed sources are the CPython standard library reference pages that carry explicit security warnings (pickle, marshal, shelve, tarfile, zipfile, xml, subprocess, shlex, pathlib, re, secrets, hmac, hashlib, ssl, resource, and the command line page) [14][15][16][19][20][21][25][26][27][29][30][31][32][37][40][42], the packaging PEPs and PyPA documentation that define the publish and install trust chain (PEP 740, PEP 751, PEP 578, Trusted Publishing, pip secure installs, pip-audit) [3][6][39][1][8][10], and the vendor docs for the libraries that own the remaining sharp edges (uv, psycopg 3, Jinja2, cryptography, argon2-cffi, defusedxml, ruff, Semgrep, CodeQL, gVisor) [9][23][28][35][34][22][43][44][45][41]. Inline `[n]` cites key to `sources`. Baseline runtime: Python 3.14.0 was released on 2025-10-07 [47]; the latest pip release listed in pip's news file is 26.2.1, dated 2026-08-04 [7]. Container hardening (non-root users, minimal base images, build context hygiene) is out of scope here and lives in `docker-assembly-guide.md`; SQL schema and query concerns beyond injection live in `database-design-guide.md`.

Everything outside `## Synthesis (inferred)` is a claim traceable to a cited page. Code is illustrative unless stated otherwise; every fence in this doc was compiled and linted (see the verification note at the end of section 6).

## 1. Supply chain: the resolver is the attack surface

### 1.1 Publishing: what Trusted Publishing and attestations actually prove

Trusted Publishing uses "the OpenID Connect (OIDC) standard to exchange short-lived identity tokens between a trusted third-party service and PyPI", removing the need for manually created API tokens; the API key PyPI mints in exchange is "only valid for 15 minutes from time of creation" [1]. The supported providers are GitHub Actions, Google Cloud, ActiveState, and GitLab CI/CD [2]. The exchange is three steps: retrieve an OIDC token from the identity provider, submit it to PyPI for a short-lived API key, then use that key as you normally would [2]. On GitHub Actions the workflow job needs `id-token: write`, which the docs call mandatory for trusted publishing, and PyPA recommends its own `pypi-publish` action over a hand-rolled exchange [2][5].

PEP 740 (Final, adopted 2024-07-17) layers signed attestations on top of that identity [3]. What a PyPI Publish Attestation asserts is narrow and worth quoting: that a particular release distribution "was, in fact, uploaded via a Trusted Publisher and not some other publishing mechanism", and that a *specific* Trusted Publisher identity was used [4]. The predicate body itself "MUST be either empty (meaning an empty JSON object, `{}`) or not supplied" [4]. The docs frame the property as integrity but "not necessarily trustworthiness" [4]. PEP 740 also explicitly declines to make a policy recommendation "around mandatory digital attestations on release uploads or their subsequent verification by installing clients like pip" [3].

| Mechanism | Proves | Does not prove |
|---|---|---|
| Trusted Publishing [1][2] | Upload came from a configured OIDC identity, using a credential valid 15 minutes [1] | Anything about the code inside the artifact |
| PEP 740 attestation [3][4] | This file was uploaded via a Trusted Publisher, by a specific publisher identity [4] | That contents are safe; the predicate body is empty by design [4] |
| Hash pinning [8] | The bytes you install match the bytes you reviewed [8] | That the bytes were ever benign |

Practical default for a library you publish: enable Trusted Publishing and leave attestations alone, because generating and uploading signed attestations for all distribution files "is now on by default for all projects using Trusted Publishing" in the PyPA action, and is "currently limited to Trusted Publishing flows using PyPI or TestPyPI" [5]. You turn it off with `attestations: false`, which is a step backwards unless a downstream tool chokes [5].

### 1.2 Installing: lockfiles, hashes, and index priority

PEP 751 (Final, resolved 2025-03-31) standardises the lockfile as `pylock.toml`, or `pylock.{name}.toml` matching `^pylock\.([^.]+)\.toml$` [6]. Hashes are mandatory: the hashes table "MUST contain at least one entry", and "At least one secure algorithm from `hashlib.algorithms_guaranteed` SHOULD always be included (at time of writing, sha256 specifically is recommended)" [6]. The PEP is candid that a lockfile does not stop name-level attacks, acknowledging it cannot prevent typosquatting [6].

pip's support is still labelled experimental on both sides: 25.1 (2025-04-26) added "a new, *experimental*, `pip lock` command, implementing PEP 751", and 26.1 (2026-04-26) added "experimental support to read requirements from standardized pylock.toml files (`-r pylock.toml`)" [7]. 26.2 (2026-07-29) tightened it further, including rejecting "a package `path` in a `pylock.toml` fetched from a URL when it resolves outside the lock file's own location" [7].

Hash-checking mode is the part that is not experimental. `--require-hashes` forces it on, "Requirements must be pinned (either to a URL, filesystem path or using `==`)", and "Hashes are required for *all* dependencies" so an unlisted transitive dependency is an error rather than a silent hole [8]. Note the implicit trigger: "Specifying `--hash` against *any* requirement will activate this mode globally" [8], and pip 26.2 added `--no-require-hashes` to disable that automatic enablement [7]. pip's secure-installs page lists `--only-binary :all:` as one of its two methods for a secure install, alongside hash checking [8].

Index priority, not the presence of a private index, is what decides where a name resolves [9]. uv's default `--index-strategy` is `first-index`: "Search for each package across all indexes, limiting the candidate versions to those present in the first index that contains the package", which is what prevents dependency confusion [9]. The alternatives are named for what they are: `unsafe-first-match` and `unsafe-best-match`, and the docs call the latter "the closest to pip's behavior" while noting that it "exposes users to the risk of 'dependency confusion' attacks" [9]. The stronger control is pinning a package to one index with `explicit = true`, which prevents packages "from being installed from that index unless explicitly pinned to it" [9]:

```toml
[tool.uv.sources]
internal-widgets = { index = "corp" }

[[tool.uv.index]]
name = "corp"
url = "https://packages.internal.example/simple"
explicit = true
```

Decision rule: `--extra-index-url` is a request for whichever index answers first or best, so treat it as a smell; name each index and pin internal packages to theirs [9].

### 1.3 Scanning: pip-audit, OSV, and the gate in CI

pip-audit is maintained under the Python Packaging Authority and "uses the Python Packaging Advisory Database via the PyPI JSON API as a source of vulnerability reports", with OSV and ESMS selectable via `--vulnerability-service` [10]. It emits SBOMs directly: `--format` accepts `columns, json, cyclonedx-json, cyclonedx-xml, markdown` [10]. `--require-hashes` gives "a hash to check each requirement against, for repeatable audits", `-S/--strict` fails "the entire audit if dependency collection fails on any dependency", and `--fix` will "automatically upgrade dependencies with known vulnerabilities" [10].

Read pip-audit's own caveats before you treat it as a malware gate [10]. It is "not a static code analyzer", it cannot "defend you against malicious packages", and its resolution behaviour carries the warning "If you wouldn't `pip install` it, you should not `pip audit` it" [10]. That last line is the whole point: a resolver that can install a package is a resolver that can be made to run one, so auditing an untrusted tree is not a read-only act [10].

OSV-Scanner covers the lockfile side and already reads the new formats: for Python it supports `Pipfile.lock`, `poetry.lock`, `requirements.txt`, `pdm.lock`, `pylock.toml`, and `uv.lock` [11]. For pull requests, GitHub's dependency-review action "scans your pull requests for dependency changes, and will raise an error if any vulnerabilities or invalid licenses are being introduced", runs on `pull_request` events, and defaults its failure threshold to `low` severity [46].

| Gate | Catches | Blind to |
|---|---|---|
| pip-audit in CI [10] | Known advisories in the resolved tree [10] | Malicious packages; it is "not a static code analyzer" [10] |
| OSV-Scanner on the lockfile [11] | Known advisories without resolving [11] | Anything absent from OSV |
| dependency-review on the PR [46] | Newly introduced vulnerable or wrongly licensed deps [46] | Vulnerabilities that were already there |
| `--require-hashes` [8] | Substituted artifacts, unlisted transitives [8] | A compromised release you pinned to |

### 1.4 What PyPI itself does, and why you cannot outsource this

PyPI's Project Quarantine shipped in 2024 and is deliberately blunt: a quarantined project "is not installable (hidden from simple index) while in quarantine", "is not modifiable by the project owner while in quarantine", its state is "visible to Project Owners, security researchers, and PyPI Administrators", and the state "can be reverted by a PyPI Administrator to restore general visibility" [12]. Scale from the announcement: since the August rollout, administrators had marked approximately 140 reported projects as quarantined, of which only one was cleared and the rest were removed entirely [12]. The same post states plainly that at the time "the current full-time security staff for PyPI == 1" [12].

The credential-theft path is not hypothetical [13]. PyPI's own incident report covers a phishing campaign in late July 2025 using the typosquatted domain `pypj.org`, in which four user accounts were successfully phished and malicious `num2words` versions 0.5.15 and 0.5.16 were uploaded [13]. PyPI's recommendation is phishing-resistant 2FA: "Use WebAuthn via browser or hardware security keys for 2FA on your PyPI account. This will help protect your account from phishing attacks, as the attacker would need access to the second factor to complete the login process" [13]. The report's reading of this specific incident is that "the attacker would not have been able to use the second factor, as the WebAuthn protocol requires the user to physically interact with a hardware security key" [13].

## 2. Deserialisation and evaluation: which loads execute

The rule is not "avoid pickle". It is that some Python loaders can "execute arbitrary code" on load and some cannot, and the stdlib says which is which on each module's own page [14][15][16].

| Input format | API | Executes attacker code? | Use instead |
|---|---|---|---|
| pickle | `pickle.load/loads` | Yes: "malicious pickle data which will execute arbitrary code during unpickling" [14] | JSON; "deserializing untrusted JSON does not in itself create an arbitrary code execution vulnerability" [14] |
| shelve | `shelve.open` | Yes: backed by pickle, "loading a shelf can execute arbitrary code" [16] | A real database |
| marshal | `marshal.load/loads` | Unsafe: "not intended to be secure against erroneous or maliciously constructed data" [15] | pickle for persistence, since marshal's format may change incompatibly [15] |
| YAML | `yaml.load` | Yes: "`yaml.load` is as powerful as `pickle.load` and so may call any Python function" [17] | `yaml.safe_load`; of its loader class the docs say "SafeLoader(stream) supports only standard YAML tags and thus it does not construct class instances" [17] |
| Python literals | `eval` | Yes | `ast.literal_eval`, with the caveat below [18] |

Two details that get misremembered. First, the `Loader` argument became mandatory in two steps, not one: PyYAML 5.1 (2019-03-13) shipped "Deprecate yaml.load and add FullLoader and UnsafeLoader classes", and 6.0 (2021-10-13) shipped "always require `Loader` arg to `yaml.load()`" [48]. On any PyYAML at or above 6.0 the unsafe path is therefore a deliberate act rather than a default [48]. Second, `ast.literal_eval` is not a sandbox: the docs retracted the old "safe" wording as "misleading" and state that "A relatively small input can lead to memory exhaustion or to C stack exhaustion, crashing the process", with "the possibility for excessive CPU consumption denial of service on some inputs", concluding that "Calling it on untrusted data is thus not recommended" [18]. It permits only strings, bytes, numbers, tuples, lists, dicts, sets, booleans, `None` and `Ellipsis` [18].

If a trust boundary must carry pickled data at all, the stdlib's own mitigation is authentication rather than parsing: "Consider signing data with `hmac` if you need to ensure that it has not been tampered with" [14]. The following example is the intended pattern, and the `noqa` marks the pickle call that a security linter will flag on sight [43]:

```python
import hmac
import pickle
from hashlib import sha256


def seal(obj: object, key: bytes) -> bytes:
    body = pickle.dumps(obj)
    return hmac.new(key, body, sha256).digest() + body


def unseal(blob: bytes, key: bytes) -> object:
    tag, body = blob[:32], blob[32:]
    expected = hmac.new(key, body, sha256).digest()
    if not hmac.compare_digest(tag, expected):
        raise ValueError("bad signature")
    return pickle.loads(body)  # noqa: S301 - reached only after HMAC verification
```

This buys integrity against outsiders, not safety: anyone holding the key can still execute code in your process [14].

### 2.1 Archives: tarfile's new default, zipfile, and bombs

Python 3.14 changed the tarfile default: "Set the default extraction filter to `data`, which disallows some dangerous features such as links to absolute paths or paths outside of the destination. Previously, the filter strategy was equivalent to `fully_trusted`" [19]. The `data` filter refuses absolute paths (`AbsolutePathError`), paths escaping the destination (`OutsideDestinationError`), links to absolute paths or outside the destination (`AbsoluteLinkError`, `LinkOutsideDestinationError`), and device files including pipes (`SpecialFileError`); it also clears high mode bits and sets uid/gid/uname/gname to `None` [19]. The `tar` filter is the weaker middle option: it strips leading slashes, normalises `..`, refuses absolute and escaping paths, and clears setuid/setgid/sticky plus group and other write bits, but does not police link targets or device files [19].

Pass `filter="data"` explicitly anyway. It is correct on every version that has the argument, and the docs are clear that the new default is not a solution: "Since Python 3.14, the default (`data`) will prevent the most dangerous security issues. However, it will not prevent *all* unintended or insecure behavior", and "None of the available filters blocks *all* dangerous archive features. Never extract archives from untrusted sources without prior inspection" [19].

zipfile is less dangerous by construction but not safe [20]. `extract()` strips drive letters and leading separators from absolute member names and removes all `".."` components, and on Windows replaces illegal characters with underscores [20]; `extractall()` still carries "Never extract archives from untrusted sources without prior inspection" [20]. The uncovered failure is volumetric: the docs note decompression bombs "apply to zipfile library that can cause disk volume exhaustion" [20]. Cap expansion in the caller, since the documented failure there is volumetric [20]:

```python
import zipfile

MAX_TOTAL = 512 * 1024 * 1024
MAX_RATIO = 100


def safe_members(zf: zipfile.ZipFile) -> list[zipfile.ZipInfo]:
    total = 0
    picked = []
    for info in zf.infolist():
        if info.compress_size and info.file_size / info.compress_size > MAX_RATIO:
            raise ValueError(f"suspicious compression ratio: {info.filename}")
        total += info.file_size
        if total > MAX_TOTAL:
            raise ValueError("archive expands beyond budget")
        picked.append(info)
    return picked
```

### 2.2 XML: know which parser you have

CPython's XML parsers "rely on the library libexpat, commonly called Expat" and "By default, Expat itself does not access local files or create network connections" [21]. The version boundary is the thing to check in CI, not a library choice: "Expat versions lower than 2.7.2 may be vulnerable to the 'billion laughs', 'quadratic blowup' and 'large tokens' vulnerabilities, or to disproportional use of dynamic memory", and whether you get the bundled or a system Expat depends on how the interpreter was configured, so "Check `pyexpat.EXPAT_VERSION`" [21]. Separately, `xmlrpc` is called out as "**vulnerable** to the 'decompression bomb' attack" [21].

defusedxml remains the belt for untrusted XML, with three switches: `forbid_dtd` blocks `<!DOCTYPE>`, `forbid_entities` blocks `<!ENTITY>` and is enabled by default, and `forbid_external` disables external resource access for DTDs and entities and is enabled by default [22]. Its own README is honest that it "modules are not drop-in replacements of their stdlib counterparts" and only covers parsing and loading [22].

```python
import pyexpat

MIN_EXPAT = (2, 7, 2)


def expat_is_current() -> bool:
    raw = pyexpat.EXPAT_VERSION.removeprefix("expat_")
    parts = tuple(int(p) for p in raw.split("."))
    return parts >= MIN_EXPAT
```

## 3. Injection: one correct idiom per class

| Class | The wrong shape | The idiom | Source |
|---|---|---|---|
| SQL values | String formatting into the query | `%s` / `%(name)s` placeholders with a parameter sequence or mapping | [23] |
| SQL identifiers | Formatting a table name into the query | `psycopg.sql.SQL(...).format(Identifier(...))` | [23] |
| Shell | `shell=True` with an f-string | A list of args, `shell=False` (the default) | [25] |
| Shell, unavoidable string | Manual quoting | `shlex.quote`, Unix only | [26] |
| Path traversal | Joining user input onto a base dir | `Path.resolve()` then `is_relative_to(base)` | [27] |
| Template (SSTI) | `Environment` on user templates | `SandboxedEnvironment` plus output-context escaping | [28] |
| Regex DoS | Nested quantifiers on user input | Possessive quantifiers or atomic groups | [29] |

**SQL.** psycopg 3 supports `%s` positional placeholders with a sequence and `%(name)s` named placeholders with a mapping, and the documentation is emphatic that values must never be merged into the query by concatenation or the `%` operator [23]. Dynamic identifiers go through `psycopg.sql`: `SQL("INSERT INTO {} VALUES (%s)").format(Identifier('tablename'))` [23]. The structural reason this holds in psycopg 3 is that it "sends the query and the parameters to the server separately, instead of merging them on the client side" [24]. Know the exceptions the same page lists: server-side binding "doesn't work with `SET` or with `NOTIFY`" or "with any data definition statement", and the documented fallbacks are `psycopg.sql` or `ClientCursor` [24].

**Shell.** subprocess "will not implicitly choose to call a system shell", so "all characters, including shell metacharacters, can safely be passed to child processes"; `shell` defaults to `False`, and "Unless otherwise stated, it is recommended to pass *args* as a sequence" [25]. With `shell=True` "it is the application's responsibility to ensure that all whitespace and metacharacters are quoted appropriately to avoid shell injection vulnerabilities" [25]. `shlex.quote` is the escape hatch, not the plan: the shlex docs warn "The `shlex` module is **only designed for Unix shells**", that `quote()` "is not guaranteed to be correct on non-POSIX compliant shells or shells from other operating systems such as Windows", and that executing such commands "can open up the possibility of a command injection vulnerability", recommending list-based `subprocess.run()` with `shell=False` instead [26]. One Windows-specific trap runs the other way: batch files "may be launched by the operating system in a system shell regardless of the arguments passed to this library", and for untrusted arguments the docs suggest passing `shell=True` so Python escapes special characters [25].

Resolve the executable too. A bare `"git"` is looked up on `PATH` at call time, which is why ruff ships `S607` `start-process-with-partial-path` alongside the shell rules [43]:

```python
import shutil
import subprocess

GIT = shutil.which("git")


def git_show(repo: str, rev: str) -> str:
    if GIT is None:
        raise RuntimeError("git not found on PATH")
    proc = subprocess.run(  # noqa: S603 - list args, shell=False, resolved executable
        [GIT, "-C", repo, "show", "--stat", "--", rev],
        capture_output=True,
        text=True,
        check=True,
        timeout=30,
    )
    return proc.stdout
```

**Paths.** The two pathlib methods pair because each covers the other's gap. `Path.resolve()` makes the path absolute and resolves symlinks, and "`..` components are also eliminated (this is the only method to do so)" [27]. `is_relative_to()` is the containment check, but on its own it "is string-based; it neither accesses the filesystem nor treats `..` segments specially" [27]. Resolve first, then compare. Note `resolve(strict=False)` resolves "as far as possible" and appends the remainder without checking existence, which is what you want when creating a new file [27].

```python
from pathlib import Path


def resolve_under(base: Path, user_path: str) -> Path:
    root = base.resolve()
    target = (root / user_path).resolve()
    if not target.is_relative_to(root):
        raise ValueError("path escapes the base directory")
    return target
```

**Templates.** Jinja2's `SandboxedEnvironment` "can be used to render untrusted templates", intercepting attribute access, method calls, operators, mutation and string formatting, and blocking "all attributes starting with an underscore" plus "special attributes of internal python objects"; `ImmutableSandboxedEnvironment` additionally "does not permit modifications on the builtin mutable objects `list`, `set`, and `dict`" [28]. Its limits are documented and matter for capacity planning as much as for security: "The sandbox alone is not a solution for perfect security", "It is possible to construct a relatively small template that renders to a very large amount of output, which could correspond to a high use of CPU or memory", and "Jinja only renders text, it does not understand, for example, JavaScript code", so output context still needs its own postprocessing [28].

**Regex.** Verify this one rather than trusting a blog post: `re` has no timeout [29]. The signatures are `re.compile(pattern, flags=0)`, `re.match(pattern, string, flags=0)` and `re.search(pattern, string, flags=0)`, with no timeout parameter [29]. What Python 3.11 added is the structural fix: atomic grouping `(?>...)`, where "once exited, the expression ... has thrown away all stack points within itself", and possessive quantifiers `*+`, `++`, `?+`, `{m,n}+`, which "do not allow back-tracking when the expression following it fails to match"; `x*+`, `x++` and `x?+` are equivalent to `(?>x*)`, `(?>x+)` and `(?>x?)` [29]. With no timeout parameter to fall back on, the mitigations the language offers are the possessive and atomic forms above [29].

```python
import re

# Backtracking-free: the possessive quantifier refuses to give characters back.
SLUG = re.compile(r"^[a-z0-9]++(?:-[a-z0-9]++)*+$")
```

**Headers and logs.** Both are the same bug as the rows above, attacker-controlled text reinterpreted by a downstream parser, and both fall under the same rule: never format untrusted text into a structured serialisation you did not parameterise [23][25].

## 4. Secrets and crypto: the short list

**Generating.** `secrets` "should be used in preference to the default pseudo-random number generator in the `random` module, which is designed for modelling and simulation, not security or cryptography" [30]. On sizing, the docs state: "As of 2015, it is believed that 32 bytes (256 bits) of randomness is sufficient for the typical use-case expected for the `secrets` module" [30].

**Comparing.** Use `hmac.compare_digest`, which "uses an approach designed to prevent timing analysis by avoiding content-based short circuiting behaviour", with the documented residual leak: "If *a* and *b* are of different lengths, or if an error occurs, a timing attack could theoretically reveal information about the types and lengths of *a* and *b*, but not their values" [31]. The docs recommend it over `==` whenever comparing a digest to an externally supplied one [31]. `secrets.compare_digest` documents the same constant-time compare and points at `hmac.compare_digest` for the details [30].

**Password hashing.** The stdlib is explicit that general-purpose hashes are the wrong tool: "Salted hashing (or just hashing) with BLAKE2 or any other general-purpose cryptographic hash function, such as SHA-256, is not suitable for hashing passwords" [32]. OWASP's ordering and exact parameters:

| Algorithm | Parameters [33] | Notes |
|---|---|---|
| argon2id (first choice) | "a minimum configuration of 19 MiB of memory, an iteration count of 2, and 1 degree of parallelism" [33] | Equivalent profiles listed include m=47104/t=1/p=1 and m=7168/t=5/p=1 [33] |
| scrypt (if argon2id unavailable) | "a minimum CPU/memory cost parameter of (2^17), a minimum block size of 8 (1024 bytes), and a parallelization parameter of 1" [33] | Stdlib signature is `hashlib.scrypt(password, *, salt, n, r, p, maxmem=0, dklen=64)` [32] |
| bcrypt | "work factor should be as large as verification server performance will allow, with a minimum of 10" [33] | "bcrypt has a maximum length input length of 72 bytes" [33] |
| PBKDF2-HMAC-SHA256 | 600,000 iterations minimum [33] | Stdlib docs say only "As of 2022, hundreds of thousands of iterations of SHA-256 are suggested" [32] |

Watch the gap in that last row: the CPython page and the OWASP page do not say the same thing, and the concrete 600,000 figure is OWASP's [32][33]. In Python the usual implementation is argon2-cffi, whose `PasswordHasher` defaults are `time_cost=3, memory_cost=65536, parallelism=4, hash_len=32, salt_len=16, encoding='utf-8', type=Type.ID` [34]. Wire up `check_needs_rehash`, which checks "whether *hash* was created using the instance's parameters" and exists precisely because "Whenever your Argon2 parameters, or *argon2-cffi*'s defaults, change, you should rehash your passwords at the next opportunity" [34]. `verify()` raises `VerifyMismatchError` on a wrong password rather than returning `False` [34].

```python
from argon2 import PasswordHasher
from argon2.exceptions import VerifyMismatchError

ph = PasswordHasher()


def check_login(stored_hash: str, supplied: str) -> tuple[bool, str | None]:
    try:
        ph.verify(stored_hash, supplied)
    except VerifyMismatchError:
        return False, None
    if ph.check_needs_rehash(stored_hash):
        return True, ph.hash(supplied)
    return True, None
```

**Symmetric encryption.** Fernet is the sane default for "encrypt this blob": it is "symmetric (also known as 'secret key') authenticated cryptography" and "guarantees that a message encrypted using it cannot be manipulated or read without the key" [35]. Keys come from `Fernet.generate_key()`, and the docs are blunt about custody: "If you lose it you'll no longer be able to decrypt messages; if anyone else gains access to it, they'll be able to decrypt all of your messages" [35]. Rotation is `MultiFernet`, which encrypts with the first key in the list and tries each key in turn when decrypting, so you "add your new key at the front of the list to start encrypting new messages, and remove old keys as they are no longer needed"; `rotate()` re-encrypts under the primary key while preserving the original timestamp [35].

Drop to raw AEAD only when you need the `associated_data` parameter or a wire format you do not control [36]. `AESGCM.generate_key(bit_length)` takes 128, 192 or 256; `encrypt(nonce, data, associated_data)` carries the non-negotiable rule "Reuse of a nonce with a given key compromises the security of any message with that nonce and key pair", with NIST recommending a 96-bit (12-byte) IV [36]. `ChaCha20Poly1305` takes a fixed 32-byte key and 12-byte nonces, and `AESGCMSIV` (RFC 8452) is the nonce-misuse-resistant option "without the strict nonce-reuse restriction that standard modes require" [36]. If you cannot prove nonce uniqueness across all writers, that last row is the one to pick [36].

**TLS.** `ssl.create_default_context()` gives `PROTOCOL_TLS_CLIENT` or `PROTOCOL_TLS_SERVER`, `OP_NO_SSLv2` and `OP_NO_SSLv3`, "high encryption cipher suites without RC4 and without unauthenticated cipher suites"; for `Purpose.SERVER_AUTH` it sets `verify_mode` to `CERT_REQUIRED` and loads CA certificates, and `PROTOCOL_TLS_CLIENT` "enables hostname checking by default" [37]. The constructor is the wrong entry point: the convenience function's settings "usually represent a higher security level than when calling the `SSLContext` constructor directly" [37]. The module's own warning is worth keeping in review checklists: "Don't use this module without reading the Security considerations. Doing so may lead to a false sense of security, as the default settings of the ssl module are not necessarily appropriate for your application" [37]. Also note the docs reserve the right to change defaults to more restrictive values "anytime without prior deprecation", so pinning behaviour you depend on means setting it yourself [37].

**What not to log.** There is no stdlib control for this; the closest mechanical helps are the linter rules that flag literal credentials in source, `S105` `hardcoded-password-string`, `S106` `hardcoded-password-func-arg` and `S107` `hardcoded-password-default` [43].

## 5. Runtime hardening: CPython does not sandbox itself

Start from the settled conclusion rather than rediscovering it [38]. The author of pysandbox retired the project with the banner "pysandbox is BROKEN BY DESIGN, please move to a new sandboxing solution: run python in a sandbox, not the opposite!", explaining that it was "a sandbox for the Python namespace, not a sandbox between Python and the operating system", with escapes through arbitrary code-object creation, function closures, globals, defaults, subclass traversal and attributes like `method.__self__` [38]. That escape list, closures, globals, defaults, subclass traversal and attributes like `method.__self__`, is precisely the surface a restricted-builtins design would have to close [38].

What CPython does offer is observability, not containment [39]. PEP 578 (Final, Python 3.8) added `sys.audit` and `sys.addaudithook`, plus `io.open_code` and `PyFile_SetOpenCodeHook` for the verified-open path, and states its scope directly: "This proposal does not attempt to restrict functionality, but simply exposes the fact that the functionality is being used", with the rationale that "detection is significantly more important than early prevention" [39]. Use audit hooks to alarm and to log, not to deny [39].

Interpreter flags reduce a different risk: entries on `sys.path` you did not intend [40].

| Control | Effect | Added |
|---|---|---|
| `-I` | Isolated mode; implies `-E`, `-P` and `-s`; `sys.path` "contains neither the script's directory nor the user's site-packages directory", and "All `PYTHON*` environment variables are ignored" [40] | 3.4 [40] |
| `-P` | Do not prepend a potentially unsafe path to `sys.path`: not the cwd for `-m`, not the script's directory for a script, not an empty string for `-c` or the REPL [40] | 3.11 [40] |
| `PYTHONSAFEPATH` | Same as `-P` when set to a non-empty string [40] | 3.11 [40] |
| `-E` | Ignore all `PYTHON*` environment variables [40] | n/a [40] |

For actually running untrusted code the recommendation from that same README is to "run python in a sandbox, not the opposite" [38]. gVisor "is an open-source workload isolation solution to safely run untrusted code, containers and applications" and "acts as an application kernel, but runs in userspace"; its Sentry "intercepts and handles system calls and page faults from the sandboxed workload ... entirely within gVisor code, written in memory-safe Go", and "gVisor never passes through any system call to the host" [41]. Note how it treats the usual primitives: syscall filtering, namespaces and cgroups are "used for defense-in-depth rather than as a primary layer of defense" [41]. Its threat model is explicit about what it does not cover, including CPU side-channel attacks and vulnerabilities within the workload itself [41].

Inside whatever boundary you pick, cap resources [42]. `resource.setrlimit(resource, (soft, hard))` sets a soft and hard limit, where "The soft limit can never exceed the hard limit" and "Only processes with the effective UID of the super-user can raise a hard limit" [42]. The four that matter for a worker process: `RLIMIT_CPU` ("the maximum amount of processor time (in seconds) that a process can use", after which `SIGXCPU` is sent), `RLIMIT_AS` ("the maximum area (in bytes) of address space"), `RLIMIT_NPROC` ("the maximum number of processes the current process may create"), and `RLIMIT_FSIZE` ("the maximum size of a file which the process may create") [42].

```python
import resource


def clamp_worker(cpu_seconds: int, address_space_bytes: int) -> None:
    """Call in a child process before exec'ing or importing untrusted work."""
    resource.setrlimit(resource.RLIMIT_CPU, (cpu_seconds, cpu_seconds))
    resource.setrlimit(resource.RLIMIT_AS, (address_space_bytes, address_space_bytes))
    resource.setrlimit(resource.RLIMIT_NPROC, (0, 0))
```

## 6. Static analysis and CI gates

ruff's `S` prefix implements flake8-bandit [43]. The rules worth knowing by number, because they are the ones that fire on real code:

| Codes | Names | Section above |
|---|---|---|
| `S301`, `S403`, `S307`, `S506` | suspicious-pickle-usage / -import, suspicious-eval-usage, unsafe-yaml-load [43] | 2 |
| `S202` | tarfile-unsafe-members [43] | 2.1 |
| `S602`-`S607` | subprocess-popen-with-shell-equals-true, subprocess-without-shell-equals-true, call-with-shell-equals-true, start-process-with-a-shell, start-process-with-no-shell, start-process-with-partial-path [43] | 3 |
| `S608`, `S610`, `S611` | hardcoded-sql-expression, django-extra, django-raw-sql [43] | 3 |
| `S701` | jinja2-autoescape-false [43] | 3 |
| `S105`, `S106`, `S107` | hardcoded-password-string / -func-arg / -default [43] | 4 |
| `S324` | hashlib-insecure-hash-function [43] | 4 |
| `S113`, `S501`-`S504` | request-without-timeout, request-with-no-cert-validation, ssl-insecure-version, ssl-with-bad-defaults, ssl-with-no-version [43] | 4 |

The linters differ in what they can see: ruff's `S` rules are syntactic, Semgrep adds dataflow, CodeQL ships a curated security suite [43][44][45]. Semgrep's taint mode ("`mode: taint`") tracks "the flow of untrusted, or tainted, data throughout the body of a function or method" between sources and sinks, with sanitizers neutralising it; propagators "only work intraprocedurally", `--pro-intrafile` gives "interprocedural (across functions), intra-file (within one file) analysis", and cross-file tracking needs `--pro` plus `interfile: true` and is "only supported for a subset of languages" [44]. CodeQL ships a Python query pack (`codeql/python-queries`) with a default suite and a `security-extended` suite described as "queries from `default`, plus extra security queries with slightly lower precision and severity", including `py-code-injection`, `py-sql-injection`, `py-path-injection` and `py-unsafe-deserialization` [45].

A workable ladder, cheapest first: ruff `S` on every commit as a pre-commit hook [43], pip-audit and OSV-Scanner on the lockfile in CI [10][11], dependency-review on pull requests with a chosen severity threshold [46], then Semgrep or CodeQL for taint-shaped bugs the first two categorically cannot see [44][45].

```toml
# pyproject.toml
[tool.ruff.lint]
select = ["E", "F", "B", "S"]

[tool.ruff.lint.per-file-ignores]
"tests/**" = ["S101"]  # assert is the point of a test
```

Every `python` fence in this document was extracted, compiled with `python3 -m py_compile`, and linted clean with `ruff check --select E,F,B,S` on Python 3.13.5 and ruff 0.15.22; the only suppressions are the two `# noqa` comments shown inline, each annotated with why the flagged call is nonetheless the intended pattern [43].

## Agreed vs folklore (compressed)

**Agreed** (each backed by a primary source above): pickle, shelve, marshal and `yaml.load` execute attacker code and JSON does not [14][15][16][17] - `subprocess` with a list and `shell=False` is the default and the correct shape [25] - parameterised SQL with server-side binding removes the value-injection class outright [23][24] - `resolve()` then `is_relative_to()` is the path-containment idiom [27] - argon2id is the first-choice password hash with published minimum parameters [33] - `hmac.compare_digest` over `==` for secrets [31] - `ssl.create_default_context()` over a bare `SSLContext()` [37] - Trusted Publishing plus attestations is the current publishing baseline, and attestations prove identity rather than safety [1][4] - hash-checking mode requires pinned, fully enumerated dependencies [8] - index priority, not `--extra-index-url`, is what stops dependency confusion [9].

**Folklore** (each contradicted by a cited page):

- *"Pin everything and you are safe."* Pinning plus hashes stops substitution [8], but PEP 751 itself concedes lockfiles cannot prevent typosquatting [6], and pip-audit states it "cannot defend you against malicious packages" [10].
- *"YAML is fine if it is your own file."* `yaml.load` "is as powerful as `pickle.load` and so may call any Python function", regardless of provenance [17]; `safe_load` costs nothing and removes the class [17].
- *"You can sandbox Python with restricted builtins."* The canonical attempt was withdrawn as "BROKEN BY DESIGN", with the author's own recommendation being to "run python in a sandbox, not the opposite" [38]; PEP 578 likewise "does not attempt to restrict functionality" [39].
- *"SHA-256 plus a salt is fine for passwords."* The hashlib docs state directly that salted hashing with "any other general-purpose cryptographic hash function, such as SHA-256, is not suitable for hashing passwords" [32].
- *"Set a regex timeout."* `re.match`, `re.search` and `re.compile` take no timeout parameter [29]; the actual controls are possessive quantifiers and atomic groups added in 3.11 [29].
- *"`ast.literal_eval` is safe."* The docs retract that wording as "misleading" and say "Calling it on untrusted data is thus not recommended" because of memory, C stack and CPU exhaustion [18].
- *"Python 3.14 fixed tar extraction."* It changed the default to `data` [19], but the same page says that will "not prevent *all* unintended or insecure behavior" and "None of the available filters blocks *all* dangerous archive features" [19].

## Synthesis (inferred)

Nothing below is cited; it is the ordering and the local application inferred from the sections above.

**Threat-model-ordered checklist for a typical Python service.** Do these in order, because each one closes a class that the next one assumes closed.

1. **Stop installing arbitrary code.** One index, named and pinned, no `--extra-index-url`; internal packages marked `explicit = true`; a committed lockfile with hashes; `--require-hashes` in CI and in the production image build. This is first because everything else runs after the resolver.
2. **Make publish credentials unstealable.** Trusted Publishing instead of long-lived tokens, hardware-key 2FA on the accounts, attestations left on. This is second because a stolen token turns step 1 into a guarantee of delivering the attacker's code.
3. **Audit the boundary functions once, mechanically.** `grep` the codebase for `pickle`, `yaml.load`, `eval`, `exec`, `shell=True`, `extractall`, raw SQL string building, and `Environment(` on user templates. Each has exactly one correct replacement listed in sections 2 and 3; there is no judgement call to make. Regexes are the one case with no call-site remedy, so any pattern that touches attacker-controlled input must be made backtracking-free or moved off the request path.
4. **Fix the crypto defaults, and decide what never reaches a log.** argon2-cffi `PasswordHasher` with `check_needs_rehash` wired into the login path, `secrets` for tokens, `hmac.compare_digest` for every comparison of a secret, `ssl.create_default_context()` everywhere a socket is created by hand. Then the logging half: placeholder-shaped values (`sk-...`, `AKIA...`, `Bearer <token>`, `postgres://localhost/...`) belong in examples, and real ones belong in a secret store and in no log line. Nothing in the stdlib enforces that, so it stays a review rule backed by the `S105`/`S106`/`S107` linter rules rather than a mechanism.
5. **Add the cheap gates.** ruff `S` in pre-commit, pip-audit and OSV-Scanner in CI, dependency-review on PRs. Adding these before steps 3 and 4 produces a backlog nobody triages; adding them after produces a wall you can keep clean.
6. **Only now consider runtime hardening.** `-I` or `PYTHONSAFEPATH` for anything invoked as a tool, audit hooks for alarming, rlimits on workers, and a real OS-level sandbox (gVisor, microVM, or at minimum a locked-down container) if and only if you genuinely execute third-party code.

**Where our own tooling should change.**

- The corpus's Docker guidance (`docker-assembly-guide.md`) covers runtime container hardening; the gap it does not cover is the build step. Any image build that runs `pip install` without `--require-hashes` is the weakest link in an otherwise hardened image, and that check belongs in the recipe templates rather than in each project.
- mx recipes that scaffold Python projects should emit `[tool.ruff.lint] select = ["E", "F", "B", "S"]` by default with `S101` ignored under `tests/`. The `S` rules are free once, expensive to retrofit, and the per-file ignore is the only tuning most projects ever need.
- Our RAG and embedding pipelines deserialise model artifacts and remote payloads; anywhere that path touches pickle (including transitively, through a framework's checkpoint loader) is a code-execution boundary that should be either hash-pinned or replaced with a non-executing format. This is the highest-value single audit in the corpus's Python surface.
- Anything that runs agent-authored or user-supplied Python (a devloop sandbox, a notebook executor, an MCP tool that evaluates expressions) should be assumed unsandboxable in-process, and given a process boundary with rlimits plus an OS-level sandbox from day one. Retrofitting that boundary after the feature ships is the expensive order.
- Publishing: any package we push to PyPI should be on Trusted Publishing before it has more than one release, because migrating a project off long-lived tokens after consumers exist is strictly harder than starting there.
