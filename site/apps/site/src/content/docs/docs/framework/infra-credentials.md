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
mx infra unlink cloudflare  # go back to project-local
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

1. **Project-local**: `./infra/<provider>/.env.<provider>`, used when it exists
   and the project is not linked to global.
2. **Global**: `~/.mech-crate/config/infra/<provider>.env`, used otherwise.

Linking is the normal case; project-local config is the escape hatch for a
project that needs a different account than the rest of the workstation.

Credential files are gitignored. Nothing in this flow puts a token in a committed
file, and nothing sends one anywhere except the provider.

### Cloudflare, specifically

The Cloudflare deploy toolchain (`make/cloudflare.mk` and the `cf-*` scripts)
reads **both** scopes on every invocation, with no linking required:

1. the environment, or the `make` command line
2. project `infra/cloudflare/.env.cloudflare`
3. global `~/.mech-crate/config/infra/cloudflare.env`

One `mx infra setup cloudflare` is therefore a complete setup for every project
on the workstation, and `make cf-setup` is the per-project override. The two
variables are wrangler's own names, `CLOUDFLARE_ACCOUNT_ID` and
`CLOUDFLARE_API_TOKEN`, so the value in a file reaches the deploy untranslated.
Each scope is read on its own, so a project file is what wins, whichever spelling
it uses: the older `CF_ACCOUNT_ID` / `CF_API_TOKEN` names are deprecated aliases,
still read so an existing config keeps working, and written by nothing.

Two targets make all of that visible rather than guessable:

```bash
make cf-vars              # the resolved id, which scope supplied it, both file paths
make cf-check-credentials # fails loudly, naming both paths and both fixes, when nothing resolves
```

## The full guide

Provider-by-provider setup, the resolution flow in detail, staging-versus-production
credential files, and the per-provider variable names are all in the corpus:

**→ [Infrastructure Configuration Guide](/docs/corpus/infra/infra-config/)**

:::caution[Two things that guide gets ahead of the code on]
The corpus guide shows `mx cf setup` as a project-local alternative. There is no
`mx cf` subcommand in the current build. Running it exits with `unrecognized
subcommand`. It is tracked as `mech-crate-vxq` with a red test in the
[known-broken lane](https://github.com/Dev916/mech-crate/blob/main/tests/KNOWN_BROKEN.md);
the fix may equally turn out to be deleting the doc references.

One adjacent defect is still open in the same lane and worth knowing about before
you lean on this path: `mech-crate-066`, where `mx infra link` does not write the
marker `mx infra inspect` looks for. It has a red test asserting the fixed
behaviour. See [Testing](/docs/framework/testing/) for what that lane is and why
the defects are published rather than hidden.

`mech-crate-wd9` used to sit beside it: the account-id variable
`mx infra setup cloudflare` wrote was not the one `templates/make/cloudflare.mk`
read, and the mk consulted the project file only, so global credentials reached
no `cf-*` target at all. Both halves are fixed, and the resolution order above is
the shipped behaviour rather than the intended one. Linking is what
`mech-crate-066` still blocks, and the Cloudflare path no longer needs it.
:::

## Related

- [Cloudflare deploy](/docs/framework/cloudflare-deploy/): what the credentials
  are for
- [Compose &amp; env conventions](/docs/framework/compose-env/): the project-level
  env layers, which are a different thing
