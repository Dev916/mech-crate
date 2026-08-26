---
title: Install
description: Build mx from source, install it to ~/.local/bin, and bring up the workstation router.
sidebar:
  order: 2
---

There is no published binary yet. `mx` is built from the repository, which also
carries the templates, recipes and router configuration it installs from.

## Requirements

| Tool | Why |
|---|---|
| Rust (stable) | builds the `mx` binary |
| Docker + Compose v2 | runs services and the router |
| GNU Make | the project verb layer |

`mx doctor` and `make doctor` both check these and print what they find, so you
do not have to guess whether a version is acceptable.

## Build and install

```bash
git clone https://github.com/Dev916/mech-crate.git
cd mech-crate
make install-local    # installs mx to ~/.local/bin (no sudo)
```

`make install-local` runs a release build first, so the first run takes a few
minutes. Make sure `~/.local/bin` is on your `PATH`, then confirm:

```bash
mx --version
mx doctor
```

`mx --version` prints the crate version (`mx 0.1.1` at the time of writing).
`mx doctor` checks Docker, Compose and Make, and — when run inside a project —
the folder contract and the service list.

:::note[Installing system-wide]
`make install` installs to a system prefix instead and needs elevated
permissions. `make install-local` is the one to reach for on a laptop.
:::

## Templates

The first `mx` command that needs them installs the project templates and
recipes to `~/.mech-crate`. You can do it explicitly:

```bash
mx init             # install templates to ~/.mech-crate
mx init --update    # refresh templates, keep config
mx init --force     # overwrite an existing install
```

If you are working *on* mx itself and want a command to resolve templates
straight out of a checkout rather than `~/.mech-crate`, set `MECH_CRATE_ROOT` to
the repository root.

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
