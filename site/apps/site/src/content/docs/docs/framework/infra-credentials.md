---
title: Infra credentials
description: Where provider credentials live, how a project resolves them, and what to watch out for.
sidebar:
  order: 5
---

Infrastructure credentials (Cloudflare, DigitalOcean, AWS, Hetzner) are
deliberately *not* part of a project's `docker/.config/` env layers. They are set
up once per workstation and resolved hierarchically, so a laptop with six mx
projects has one place its Cloudflare token lives.

```bash
mx infra setup cloudflare   # global credentials, once per workstation
mx infra list               # what is configured
mx infra inspect cloudflare # where the values resolve from

cd myproject
mx infra link cloudflare    # use the global credentials here
mx infra unlink cloudflare  # the project's own values apply again
mx infra remove cloudflare  # drop a provider's configuration
```

`mx infra list` reports every known provider and whether it is configured:

```
Infrastructure Providers
────────────────────────────────────────
  • cloudflare - not configured
  • digitalocean - not configured
  • aws - not configured
  • hetzner - not configured
```

## Resolution order

Every credential resolves on its own, highest precedence first:

1. **The environment**, or the `make` command line.
2. **Project-local**: `./infra/<provider>/.env.<provider>`, unless the project
   is linked to global.
3. **Global**: `~/.mech-crate/config/infra/<provider>.env`, shared by every
   project on the workstation.

Precedence is by scope, never by spelling: within a scope the canonical name is
read before a deprecated alias, but a project file carrying only the alias still
beats a global file carrying the canonical name. A project with no project file
simply falls through to global, so one `mx infra setup` is a complete setup.

Credential files are gitignored. Nothing in this flow puts a token in a committed
file, and nothing sends one anywhere except the provider.

## Seeing where a value came from

`mx infra inspect <provider>` prints the resolved set, each key tagged with the
scope that supplied it, plus the link state and both file paths. Secrets are
masked:

```
Configuration for cloudflare
────────────────────────────────────────
  link:    not linked (project values win over global)
  global:  ~/.mech-crate/config/infra/cloudflare.env (present)
  project: ./infra/cloudflare/.env.cloudflare (present)

  CLOUDFLARE_ACCOUNT_ID = 4f2a...  [project, deprecated alias CF_ACCOUNT_ID]
  CLOUDFLARE_API_TOKEN  = ********  [global]
```

It fails, naming `mx infra setup`, when nothing resolves.

## Linking

A project is **linked** when its `.env.<provider>` contains the exact line
`MX_INFRA_USE_GLOBAL=true`. A linked project's file is not read for values, so
its credentials resolve from the environment, then the global config. That one
line is the whole mechanism, and every reader honors it: `mx infra inspect`,
`mx cf vars` / `make cf-vars` (which print `CF_PROJECT_LINKED=true`), every
`cf-*` target, and `scripts/cf-init-app.sh`.

- `mx infra link <provider>` adds the line, creating the file if needed and
  keeping everything else in it. It needs a project and an existing global
  config, and is idempotent.
- `mx infra unlink <provider>` removes the line, restoring the file byte for
  byte, so the project's own values apply again. A file that held nothing but
  the link is deleted. Unlinking an unlinked project is a no-op.

Linking is only needed to make a project that has its own `.env.<provider>`
defer to global without deleting that file. The older `.env.linked` marker file
is deprecated and ignored: `mx infra inspect` warns about a stray one, and
`link` / `unlink` delete it.

### Cloudflare, specifically

The Cloudflare deploy toolchain (`make/cloudflare.mk` and the `cf-*` scripts)
implements the same order on every invocation. Its two variables are wrangler's
own names, `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN`, so the value in a
file reaches the deploy untranslated. The older `CF_ACCOUNT_ID` / `CF_API_TOKEN`
names are deprecated aliases, still read so an existing config keeps working,
and written by nothing. `mx cf setup` (`make cf-setup`) writes the per-project
override.

Two commands make the deploy side visible rather than guessable:

```bash
mx cf vars              # the resolved id, which scope supplied it, both file paths
mx cf check-credentials # fails loudly, naming both paths and both fixes, when nothing resolves
```

Each is a thin wrapper over the `make cf-*` target of the same name, so the
output is identical either way.

## The full guide

Provider-by-provider setup, staging-versus-production credential files, and the
per-provider variable names are all in the corpus:

**→ [Infrastructure Configuration Guide](/docs/corpus/infra/infra-config/)**

:::note[What changed]
Three defects used to sit on this page. `mech-crate-wd9`: the account-id name
`mx infra setup` wrote was not the one `cloudflare.mk` read, and the mk ignored
the global file. `mech-crate-vxq`: the guide taught an `mx cf` subcommand the
CLI did not have. `mech-crate-066`: `mx infra link` and `unlink` were stubs, and
two link markers disagreed. All three are fixed, each against a test that now
runs in the gate, and the behaviour above is what ships. See
[Testing](/docs/framework/testing/) for how that lane works.
:::

## Related

- [Cloudflare deploy](/docs/framework/cloudflare-deploy/): what the credentials
  are for
- [Compose &amp; env conventions](/docs/framework/compose-env/): the project-level
  env layers, which are a different thing
