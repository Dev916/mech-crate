---
title: Framework
description: The router, recipes, compose and env conventions, infra credentials, upgrade, testing and deploy.
sidebar:
  order: 1
---

The [Start](/docs/start/) section gets a service answering on a hostname. This
section is what is underneath: the machinery that makes the folder contract
operational, and the honest state of each part.

| Page | What it covers |
|---|---|
| [The router](/docs/framework/router/) | One Traefik instance per workstation; hostname routing; the labels that connect a service |
| [Recipes](/docs/framework/recipes/) | What ships, what each recipe carries, measured status, and how to author one |
| [Compose &amp; env conventions](/docs/framework/compose-env/) | Baseline + dev override layering; the three env layers and their real precedence |
| [Infra credentials](/docs/framework/infra-credentials/) | `mx infra` — where provider credentials live, global versus per-project |
| [Upgrade](/docs/framework/upgrade/) | Keeping a project current with the templates — and why it is currently broken |
| [Testing](/docs/framework/testing/) | The gates, why they are proven rather than assumed, and the known-broken lane |
| [Cloudflare deploy](/docs/framework/cloudflare-deploy/) | The `cf-*` targets, and how this site is meant to ship |
| [Remote blueprints](/docs/framework/unyform/) | The optional Unyform integration |
| [Architecture diagrams](/docs/framework/diagrams/) | All five diagrams on one page |

Pages here stay short on purpose. Where a corpus document already covers a topic
in depth, the page links to it rather than restating it — each corpus document
renders exactly once, under [Techniques Corpus](/docs/corpus/).
