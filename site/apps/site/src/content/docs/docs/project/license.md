---
title: License
description: 'MechCrate is dual-licensed under MIT or Apache-2.0, at your option: the standard Rust-ecosystem arrangement, including for contributions.'
sidebar:
  order: 4
---

MechCrate is **dual-licensed**. In the words of the repository's own README:

> Licensed under either of
> [Apache License, Version 2.0](https://github.com/Dev916/mech-crate/blob/main/LICENSE-APACHE)
> or [MIT license](https://github.com/Dev916/mech-crate/blob/main/LICENSE-MIT)
> at your option.

The workspace manifest carries the same thing in SPDX form:

```toml
license = "MIT OR Apache-2.0"
```

`OR` is the operative word. You pick one. You do not have to satisfy both.

## What you get

| | MIT | Apache-2.0 |
|---|---|---|
| Use, modify, distribute, sell | yes | yes |
| Must keep the notice | yes | yes |
| Express patent grant | no | yes |
| Explicit trademark carve-out | no | yes |
| Length | one page | long |

Choosing MIT gets you the shortest permissive licence in common use. Choosing
Apache-2.0 gets you an explicit patent grant from contributors and clearer
handling of modifications and trademarks, at the cost of reading more.

This pairing is the Rust ecosystem's default, and the reason is compatibility in
both directions: MIT lets the code be used from projects that cannot take
Apache-2.0's terms, and Apache-2.0 lets it be used where an express patent grant
is required. Offering both means neither constraint blocks you.

The copyright notice reads *Copyright (c) 2026 PRICELOVE, LLC and mech-crate
contributors*.

## Contributing

Contributions carry the same dual licence, automatically:

> Unless you explicitly state otherwise, any contribution intentionally
> submitted for inclusion in this work by you, as defined in the Apache-2.0
> license, shall be dual licensed as above, without any additional terms or
> conditions.

There is no CLA to sign and no copyright assignment. Opening a pull request is
the whole ceremony.

## The documents on this site

The [techniques corpus](/docs/corpus/) and these guides are files in the same
repository (`docs/development/` and `site/apps/site/src/content/`), so the same
terms cover them. Individual corpus documents cite external sources in their
provenance footers; those citations point at their own authors' work under their
own terms, and the corpus document is the summary and analysis, not a
reproduction.

**→ [LICENSE-MIT](https://github.com/Dev916/mech-crate/blob/main/LICENSE-MIT)**
· **[LICENSE-APACHE](https://github.com/Dev916/mech-crate/blob/main/LICENSE-APACHE)**
