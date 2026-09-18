---
title: Install
description: Get the mx binary onto a machine from the release channel (or from source), keep it updated, and bring up the workstation router.
sidebar:
  order: 2
---

`mx` ships as a tarball per platform on the release channel
(`unyform-ai/mech-crate-releases`): one universal macOS build and two static
Linux builds. The installer below downloads the one for your machine, verifies
its checksum, and installs it under your home directory with no `sudo`.
Homebrew installs the same bundle. Contributors build from a checkout instead.

## Requirements

| Tool | Why |
|---|---|
| Docker + Compose v2 | runs services and the router |
| GNU Make | the project verb layer |
| Rust (stable) | only if you build from source |

`mx doctor` and `make doctor` both check these and print what they find, so you
do not have to guess whether a version is acceptable.

## Supported platforms

Both install paths cover the same builds. "Verified" means the release was
installed, updated and rolled back on that platform; "built in CI" means the
release workflow produced and checked the binary but nobody has run it on that
hardware yet.

| Platform | Architecture | `install.sh` | Homebrew | Status |
|---|---|---|---|---|
| macOS 11 (Big Sur) or later | Apple silicon | yes | yes | verified on macOS 26 (install, update, rollback, `brew install` and `brew upgrade`) |
| macOS 11 (Big Sur) or later | Intel | yes, same universal binary | yes | built in CI, not yet run on Intel hardware |
| Linux | x86_64 | yes | yes, with [Homebrew on Linux](https://docs.brew.sh/Homebrew-on-Linux) | static build, runs on Alpine 3.20 and Debian 12 containers; `brew install` verified in the `homebrew/brew` container |
| Linux | aarch64 | yes | yes, with Homebrew on Linux | static build, runs on Alpine 3.20 and Debian 12 containers; `brew install` not yet run on aarch64 |
| Windows | x86_64, arm64 | through WSL 2 | through WSL 2 | not tested: inside WSL 2 the Linux build applies, with Docker Desktop's WSL 2 backend for the router and services |
| Anything else | | no, the installer stops with "no mx release is published for ..." | no | build from source with stable Rust (below) |

A few details behind the table:

- **macOS.** The universal binary's Apple silicon slice targets macOS 11 and
  the Intel slice 10.12, so 11 is the floor. Binaries are signed ad hoc today:
  the installer and Homebrew never trip Gatekeeper, but a tarball downloaded
  through a browser would be quarantined. Developer ID signing and
  notarization are tracked as `mech-crate-4vp.15`.
- **Linux.** Both builds are fully static (musl), so there is no glibc or
  distribution requirement for `mx` itself. The router and the services still
  need Docker Engine with Compose v2, and the project verbs need GNU Make.
  Homebrew's own platform policy decides whether `brew` runs on your
  distribution and architecture; the formula carries both Linux tarballs.
- **Windows.** There is no native build. The Linux build is expected to work
  inside WSL 2 exactly as on Linux, but that has not been exercised, so treat
  it as unverified until it is.

## Install

### curl (macOS, Linux)

```bash
curl -fsSL https://mechcrate.dev/install.sh | sh
```

The script resolves the latest release, downloads `mx-v<version>-<platform>.tar.gz`
and its `.sha256`, verifies them, extracts, and runs the bundled binary's own
installer (`mx self-update --from-dir`). It prints a `PATH` hint if
`~/.local/bin` is not already on your `PATH`. Then:

```bash
mx --version
mx doctor
```

Pin a version with `MX_VERSION=0.1.3` in front of the command.

`mx --version` prints the crate version. `mx doctor` checks Docker, Compose and
Make. Run inside a project, it also checks the folder contract and the service
list.

### Homebrew (macOS, Linux)

```bash
brew install unyform-ai/tap/mechcrate
```

The formula installs the same release bundle (macOS universal, Linux x86_64
and aarch64) into the Cellar and links `mx` and `mx-mcp` into `brew --prefix`.
The formula is named `mechcrate` because homebrew-core already ships an
unrelated `mx`; the commands it installs are still `mx` and `mx-mcp`.
Installed this way, `mx self-update` hands off to `brew upgrade mechcrate`; the tap
is bumped automatically for every stable release. On Linux this needs
[Homebrew on Linux](https://docs.brew.sh/Homebrew-on-Linux) first.

### From source (contributors)

```bash
git clone https://github.com/Dev916/mech-crate.git
cd mech-crate
make install-local    # release build, installs mx to ~/.local/bin (no sudo)
```

`make install-local` runs a release build first, so the first run takes a few
minutes. `make install` installs to a system prefix instead and needs elevated
permissions. Set `MECH_CRATE_ROOT` to the repository root when you want a
command to resolve templates straight out of the checkout.

## Keeping mx up to date

```bash
mx self-update --check     # is a newer release out? (exit 10 when yes)
mx self-update             # show the plan, confirm, apply
mx self-update --yes       # no prompt
mx self-update --to 0.1.2  # a specific version, up or down
mx self-update --rollback  # back to the previously installed release
mx self-update --dry-run   # the plan only, nothing changes
```

`self-update` works out how this copy of `mx` was installed and does the
matching thing: a release install downloads and verifies the next tarball;
a Homebrew install runs `brew upgrade mechcrate`; a source checkout is rebuilt
(`--pull` to `git pull --rebase` first).

Every update is verified before it goes live. The tarball's sha256 must match
the published sidecar, the new binary must report the version it claims, and
on macOS its code signature is checked when `codesign` is available. A failure
at any of those points leaves the current install untouched.

### The daily hint

Once a day, on the first command you run, `mx` refreshes a small cache in the
background and, if a newer release exists, prints one line to stderr after the
command's own output:

```
mx 0.1.3 is available (you have 0.1.2). Run: mx self-update
```

It never delays a command (the check is a detached background process), never
prints when stderr is not a terminal, and stays quiet under `CI`. Turn it off
with `MX_NO_UPDATE_CHECK=1`, or permanently in `~/.mech-crate/config/update.toml`:

```toml
check = false
```

### Where things live

```
~/.mech-crate/
  releases/mx-v0.1.2/   # each installed release, exactly as packaged
  current -> releases/mx-v0.1.2
  templates/            # refreshed from current/templates on every update
  version
~/.local/bin/mx      -> ~/.mech-crate/current/bin/mx
~/.local/bin/mx-mcp  -> ~/.mech-crate/current/bin/mx-mcp
```

An update extracts the new release beside the old one and then swaps the
`current` symlink, so the running process finishes normally and `--rollback`
is a symlink flip back. The previous release is kept; older ones are pruned.

## Templates

The first `mx` command that needs them installs the project templates and
recipes to `~/.mech-crate`; updates refresh them automatically. You can do it
explicitly:

```bash
mx init             # install templates to ~/.mech-crate
mx init --update    # refresh templates, keep config
mx init --force     # overwrite an existing install
```

## The router

The router is installed once per machine, not once per project:

```bash
mx router install    # copies the Traefik config, creates the devmesh-traefik network
mx router up         # start it
mx router status     # installed / running / network / dashboard URL
```

`mx router status` prints the dashboard URL. Traefik allocates it from
`7680-7799` unless you pin it. Full detail is on
[The router](/docs/framework/router/).

## Next

Create something: [Your first project](/docs/start/first-project/).
