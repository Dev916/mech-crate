---
title: Install
description: Get the mx binary onto a machine from the release channel (or from source), keep it updated, and bring up the workstation router.
sidebar:
  order: 2
---

`mx` ships as a signed tarball per platform on the release channel
(`unyform-ai/mech-crate-releases`). The installer below downloads the one
for your machine, verifies its checksum, and installs it under your home
directory with no `sudo`. Contributors build from a checkout instead.

## Requirements

| Tool | Why |
|---|---|
| Docker + Compose v2 | runs services and the router |
| GNU Make | the project verb layer |
| Rust (stable) | only if you build from source |

`mx doctor` and `make doctor` both check these and print what they find, so you
do not have to guess whether a version is acceptable.

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

Pin a version with `MX_VERSION=0.1.2` in front of the command.

### Homebrew (macOS, Linux)

Coming with the tap (`brew install unyform-ai/tap/mx`). Once installed that way,
`mx self-update` hands off to `brew upgrade mx`.

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
a Homebrew install runs `brew upgrade mx`; a source checkout is rebuilt
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

`mx router status` prints the dashboard URL — Traefik allocates it from
`7680-7799` unless you pin it. Full detail is on
[The router](/docs/framework/router/).

## Next

Create something: [Your first project](/docs/start/first-project/).
