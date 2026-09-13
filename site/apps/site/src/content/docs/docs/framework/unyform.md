---
title: Remote blueprints
description: The optional Unyform integration. Organization-connected recipes, and why nothing in mx depends on it.
sidebar:
  order: 9
---

Local recipes are fully self-sufficient. Scaffolding, the router, the compose and
env conventions, `mx docs` and `mx doctor` need no account, no login, and no
network call. If you never read past this paragraph you lose nothing.

For teams that want organization-connected scaffolding, mx integrates with
[unyform](https://unyform.ai): `mx login` links your organization, and blueprints
generated from your connected repositories become available alongside the local
recipes. That is the extent of it: an optional cloud *source of recipes*, not a
dependency.

## The commands

```bash
mx login       # link your organization
mx whoami      # who you are logged in as
mx logout      # unlink
```

The same three also live under `mx unyform login | logout | whoami`.

Once linked, the remote recipe verbs become useful:

```bash
mx recipes pull <name>       # fetch a blueprint
mx recipes versions <name>   # what versions exist
mx recipes cache             # manage the local cache
mx recipes apply <name>      # apply it to the current project
```

Pulled blueprints appear in `mx recipes list` next to the local ones and are
applied by the same `mx add --recipe <name>`, so nothing downstream of the recipe
knows or cares where it came from.

`mx cc-plugin` installs or uninstalls the Unyform Claude Code plugin hooks. Also
optional, also inert if you skip it.

## Honest state

Two defects on this path are open in the
[known-broken lane](https://github.com/Dev916/mech-crate/blob/main/tests/KNOWN_BROKEN.md),
each with a red test asserting the fixed behaviour:

- **`mech-crate-rnj`**: the CLI client and the MCP server client request
  different `/v1/orgs/<segment>/recipes` paths (organization **id** versus
  **slug**) against the same API.
- **`mech-crate-9be`**: `mx recipes apply <r> --fix` produces output identical to
  a plain apply, i.e. the advertised dependency-drift comparison is not happening.

Neither affects local recipes. See [Testing](/docs/framework/testing/) for what
that lane is and why these are published rather than hidden.

## Related

- [Recipes](/docs/framework/recipes/): the local recipes, and how to write one
- [Infra credentials](/docs/framework/infra-credentials/): a different kind of
  credential, kept in a different place
