---
title: Upgrade
description: How mx upgrade is meant to keep projects current with the templates, and its current, broken, state.
sidebar:
  order: 6
---

The premise of the folder contract is that mx owns the verb layer, so
improvements to `make dev` can reach a project scaffolded a year ago. `mx
upgrade` is the command that carries them.

```bash
mx upgrade --dry-run   # show what would be done, change nothing
mx upgrade --diff      # show a diff for each changed file
mx upgrade             # walk the changes interactively
mx upgrade --yes       # accept everything, non-interactive
```

:::danger[Currently broken. Do not rely on this]
On the current build, `mx upgrade` fails before it does anything:

```
$ mx upgrade --dry-run
   🦝 MechCrate Project Upgrade

→ Upgrading project at: /path/to/your-project

Checking directories...

Scanning templates...
error: Configuration error: Project templates not found at ~/.mech-crate/templates/project
```

Exit code `1`. `--diff` fails identically. The discovery step looks for a
`templates/project/` directory that the shipped template layout does not
contain. The scaffold templates live at the root of `templates/`, not under a
`project/` subdirectory.

Tracked as **`mech-crate-z5i`**. The red test
`upgrade::tests::upgrade_discovery_works_against_real_templates_layout` in
`crates/mx-lib/src/upgrade/mod.rs` asserts that `discover_upgrades()` succeeds
against the real `templates/` layout. It is failing today, on purpose: writing
the failing test *first* is how the defect is defined, and un-ignoring it is the
fix's definition of done. See [Testing](/docs/framework/testing/) and the
[known-broken lane](https://github.com/Dev916/mech-crate/blob/main/tests/KNOWN_BROKEN.md).

Until it lands, updating a project's tooling means copying the relevant files
from `templates/` by hand. The categories below tell you which ones are safe to
copy over.
:::

## What upgrade is designed to do

The interesting part of the design survives the defect, and it is worth
understanding because it is the same rule that governs who owns what in the
[folder contract](/docs/start/folder-contract/). Every template file is
categorised:

| Category | Paths | Behaviour |
|---|---|---|
| **Tooling** | `Makefile`, `make/*.mk`, `scripts/*.sh`, `scripts/*.mjs` | Offered for update (these are mx's) |
| **Config** | `docker/compose/`, `docker/dockerfiles/`, `docker/system/` | Added if missing, **never overwritten** (these are yours) |
| **Conditional** | `make/cloudflare.mk`, `scripts/cf-*.sh`, `infra/cloudflare/` | Only touched if the project has `infra/cloudflare/` |
| **Skip** | `recipes/`, `router/` | Not part of a project upgrade |

The line that matters: **your compose files and dockerfiles are never replaced.**
An upgrade can improve how `make dev` composes them; it cannot rewrite what they
say. That is what makes accepting an upgrade a low-stakes decision rather than a
merge conflict.

Conditional files are keyed on evidence rather than a config flag. The
Cloudflare set is in scope exactly when `infra/cloudflare/` exists on disk, so a
project that never opted in never sees those files offered.

## Recipes are upgraded separately

`mx upgrade` covers the project skeleton. Service-level scaffolding comes from
recipes and has its own path: `mx recipes apply <name>` re-applies a recipe to
the current project, and `mx recipes versions <name>` lists what is available.
See [Recipes](/docs/framework/recipes/).
