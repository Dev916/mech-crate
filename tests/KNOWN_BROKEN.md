# Known-Broken TDD Lane

Every tracked, testable defect in this repo has a test that asserts its **fixed**
behavior *today*, annotated `#[ignore = "bd:mech-crate-<id> <reason>"]`. The
gate (`make test`) never runs them; `make test-known-broken` runs only them and
scoreboards the result.

```bash
make test-known-broken   # cargo nextest run --workspace --profile ci --run-ignored only
```

**Expected state: all red.** A lane test that turns **green** is the signal that
a fix landed without bookkeeping — surfaced, not silently green.

## Fix workflow (definition of done for each bd issue)

1. `bd update mech-crate-<id> --claim`
2. Make the lane test pass.
3. Delete its `#[ignore]` attribute — the test joins the gate.
4. Remove its row from the table below.
5. `bd close mech-crate-<id>`.

## House rules

- `#[ignore]` is reserved **exclusively** for this lane. DB-gated tests skip via
  an env-var early return, never `#[ignore]`.
- Every lane test's *arrange* half must succeed today: a lane test that dies on
  a missing fixture tells us nothing about the defect. Setup assertions in this
  lane carry a `setup:` prefix so the two failure classes stay distinguishable.
- Lane tests live **beside the suite that owns their subject**, not in one
  central file — the test is the first thing the fixer should read.

## Mapping

| bd id | Test | Where | Asserts (once fixed) | Tier |
|---|---|---|---|---|
| mech-crate-9be | `kb_recipes_apply_fix_changes_behavior` | `crates/mx-cli/tests/known_broken.rs` | `mx recipes apply <r> --fix` output differs from a plain apply — i.e. `--fix` performs the advertised dependency-drift comparison (wiremock Unyform stub) | integration |
| mech-crate-dpi | `kb_build_platform_reaches_the_build_script` | `crates/mx-cli/tests/known_broken.rs` | `mx build svc --platform linux/arm64` reaches `scripts/build.sh` through the real `make _build` (recording build.sh stub) | integration |
| mech-crate-fr6 | `kb_make_rebuild_is_a_real_target` | `crates/mx-cli/tests/known_broken.rs` | `make rebuild` is a real target: `make -n rebuild` prints at least one command instead of nothing | integration |
| mech-crate-ten | `kb_nuxt_recipe_ships_the_release_scripts_make_invokes` | `crates/mx-lib/tests/recipes_conformance.rs` | The nuxt recipe's app `package.json` declares every `yarn release*` script `templates/make/release.mk` invokes. *Retire if the issue is closed by making `make release` degrade gracefully instead.* | integration |
| mech-crate-3jt | `kb_recipes_info_shows_the_recipe_version` | `crates/mx-cli/tests/known_broken.rs` | `mx recipes info nuxt` surfaces `recipe.json`'s `version` (read from the recipe so it cannot go stale) | integration |
| mech-crate-o5f | `kb_router_inspect_lists_connected_services` | `crates/mx-cli/tests/known_broken.rs` | `mx router inspect` lists the containers attached to `devmesh-traefik` (stub-bin `docker` returns a 2-container network inspect) | integration |
| mech-crate-dqw | `kb_doctor_reports_router_status` | `crates/mx-cli/tests/known_broken.rs` | `mx doctor` reports on the router (network / container / port 80) alongside its structure and docker checks | integration |
| mech-crate-4jw | `corpus::store::tests::kb_lexical_arm_separates_relevant_from_irrelevant` | `crates/mx-lib/src/corpus/store.rs` | With the vector arm held equal (identical embeddings, orthogonal to the query), the lexical arm separates a relevant from an irrelevant ~1.2KB chunk by ≥5× and ≥0.05 of final score. Measured today: **2.18× / 0.0062** | integration (DB) |

8 lane tests, all named `kb_*` (the exceptions, `z5i`, `wd9`, `vxq` and `066`,
left the lane when their fixes landed; `rnj` kept its `kb_` name when it
graduated).

**Scoreboard** (`make test-known-broken`): `8 tests run: 0 passed, 8 failed,
492 skipped`: 8 rows above, 8 red, zero bookkeeping debt. The gate suite
(`make test`) in the same tree: `492 passed, 8 skipped`. Those two numbers
partition the workspace; if they stop summing to 500, either a lane test lost
its `#[ignore]` or a gate test grew one.

The lane held at 14 across the first-touch-killer fixes (bd:mech-crate-bj4,
bd:mech-crate-0tj, bd:mech-crate-290, bd:mech-crate-ads): none of those defects
had a lane test, so they were fixed against fresh red tests that joined the gate
directly, taking it from 162 to 189.

**bd:mech-crate-z5i** (Wave 1, `mx upgrade` discovery against the shipped
`templates/` layout) is the first lane row retired the intended way: its test
lost `#[ignore]` and joined the gate, taking the lane 14 → 13 and the gate
189 → 191 (the fix also added `upgrade_discovery_scope_mirrors_mx_new`, which is
why the workspace total moved 203 → 204).

The rest of Wave 1 had no lane rows to retire, so each fix landed against fresh
red tests that joined the gate directly: compose project isolation plus the
`scripts/.bashrc` delivery gap (bd:mech-crate-71u, bd:mech-crate-12p) took it
191 → 202, the astro `include` fix and its compose-config conformance net
(bd:mech-crate-eic, bd:mech-crate-pos) 202 → 205, and the `init_app` ordering fix
(bd:mech-crate-0uq) 205 → 217. The lane stayed at 13 throughout, so the workspace
total moved 204 → 230.

Wave 2 repeated the pattern: none of its defects had a lane row either, so all
five fixes landed against fresh red tests that joined the gate directly, taking it
217 → 252 and the workspace total 230 → 265. The lane is untouched at 13. Four new
conformance suites carry most of that growth, each of them a net over a *class*
rather than over the files that happened to be wrong:

| Suite | Holds |
|---|---|
| `templates_env_precedence.rs` | every `env_file` list in `templates/` follows `.env.shared` → `.env.secrets` → `.env.<service>`, and anything reading shared also reads secrets (bd:mech-crate-lwe) |
| `templates_secret_generation.rs` | `make init` generates real dev credentials for every db-bearing recipe, idempotently, and no recipe ships an unconsumed `__GENERATE_*__` placeholder (bd:mech-crate-rqc) |
| `templates_multi_service.rs` | `s="a b"` survives the make layer as one argument, list-capable targets pass every name to compose, and single-service targets refuse a list loudly (bd:mech-crate-3kq) |
| `templates_compose_hygiene.rs` | no shipped compose file pins a `container_name` or joins an `external: true` network mx never creates, and the db healthcheck probes the configured role at container runtime (bd:mech-crate-4n4, bd:mech-crate-xhf, bd:mech-crate-v6z) |

**bd:mech-crate-gjl** is the second lane row retired the intended way, and it
retired quietly: `kb_self_update_finds_the_source_root_recorded_by_init` lost its
`#[ignore]` and joined the gate when `mx self-update` landed, taking the lane
13 → 12, but the counts in this file were not moved with it. They are moved now.
That is exactly the bookkeeping debt this scoreboard exists to catch, so it is
recorded rather than quietly corrected.

Wave 3 repeated Wave 2's pattern once more: none of its four defects had a lane
row, so all four landed against fresh red tests that joined the gate directly,
taking it 417 → 427 and the workspace total 429 → 439. The lane is untouched at
12. Two new conformance suites carry the growth, both of them nets over a class
and both walking `templates/` **and** the repo's own `site/docker/compose/`, so
the dogfood copy cannot drift away from what the recipes ship:

| Suite | Holds |
|---|---|
| `templates_compose_ports.rs` | every host-side port publish is `${VAR:-<default>}`, with the default fixed per class: `0` for a port tooling dials on demand, a real number for one a browser dials from a URL it already holds. A container port the classification table has never seen fails deliberately, because publishing a host port is a design call (bd:mech-crate-1a0, with the legacy-template overlap of bd:mech-crate-qpy) |
| `templates_traefik_labels.rs` | every `traefik.http.{routers,services,middlewares}` name is `${COMPOSE_PROJECT_NAME}`-qualified, definition and reference alike, and every router rule's hostname sits in the default position of a `<SERVICE_UPPER>_ROUTER_HOST` override. `templates/router/` is pinned as the deliberate file-configured singleton exception (bd:mech-crate-298) |

**bd:mech-crate-wd9** (Cloudflare credential resolution) is the third lane row
retired the intended way, and the first retired by *replacing* its test rather
than un-ignoring it. The lane test asserted the contract through a heuristic that
only holds for a makefile which reads a credential name verbatim out of a single
`-include`d file: "a `*_ACCOUNT_ID` the mk dereferences but never assigns". The
fix makes `cloudflare.mk` resolve two scopes per credential, so it now *does*
assign the canonical name after reading each file, and the heuristic would have
failed against a correct fix. Its subject moved to a suite of its own,
`crates/mx-lib/tests/templates_cloudflare_credentials.rs`, which holds the same
contract behaviorally (real `make` against scratch `HOME` and project scopes)
plus the static name contract across the Rust writer and the shipped templates.
Lane 12 → 11, gate 427 → 438 (8 in the new suite, 3 unit tests on the
`mx infra setup cloudflare` writer), workspace total 439 → 449.

| Suite | Holds |
|---|---|
| `templates_cloudflare_credentials.rs` | one canonical credential pair (`CLOUDFLARE_ACCOUNT_ID` + `CLOUDFLARE_API_TOKEN`, the names wrangler reads from the environment) that nothing writes under a second spelling; `cloudflare.mk` reading the global credentials file before the project one so the project wins per scope, deprecated `CF_ACCOUNT_ID` alias included; and an unconfigured machine failing loudly with a message naming `mx infra setup cloudflare` (bd:mech-crate-wd9) |

The same fix's docs pass found the second half of the defect's blast radius:
`site/scripts/cf-setup.sh` and `site/scripts/cf-init-app.sh` were byte-identical
copies of the templates and had just gone stale, still writing the deprecated
spelling with nothing failing. They were re-synced and the copy is now pinned, so
the next template change to either script cannot rot its twin in silence. Gate
438 → 441, workspace total 449 → 452.

| Suite | Holds |
|---|---|
| `site_scripts_sync.rs` | the `site/scripts/` copies of `templates/scripts/cf-setup.sh` and `cf-init-app.sh` are byte-identical to the templates they came from, and both carry the canonical credential name. The pinned list is explicit: the rest of `site/scripts/` has legitimately diverged, and a test that discovered its own expectation would assert nothing (bd:mech-crate-wd9) |

The two fixes that carry no suite of their own extended existing ones:
bd:mech-crate-fq3 (rust-worker's unexpanded `{{RUST_VERSION}}`) added an
unexpanded-placeholder net to `recipes_conformance.rs` in the shape of Wave 2's
`__GENERATE_*__` net, and bd:mech-crate-q1w (non-dev `DATABASE_URL` rendering
empty) extended `templates_env_precedence.rs` to refuse a compose `environment:`
value that interpolates a name only an `env_file` layer defines.

**bd:mech-crate-vxq** (the phantom `mx cf` command) is the fourth lane row
retired. The issue allowed either fix, and its lane test said to retire it if the
docs were purged instead; the command was implemented, so the test's subject
held, and it was deleted rather than un-ignored because two gate suites now pin
it more strongly: `cli_surface.rs` sweeps `cf` and five nested `cf` subcommands
through `--help`, and the new suite below runs the wrapper for real. `mx cf <cmd>`
is a thin proxy that runs `make cf-<cmd>` in the project root (an app becomes
`a=<app>`), on the same code path as `mx dev/up/down/logs`. Lane 11 → 10, gate
441 → 458 (9 in `cf_wrapper.rs`, 3 in `docs_drift.rs`, 5 unit tests on the
subcommand-to-target mapping), workspace total 452 → 468.

| Suite | Holds |
|---|---|
| `cf_wrapper.rs` | `mx cf vars` / `config` / `check-credentials` reach the real shipped `cloudflare.mk` through the wrapper against scratch `HOME` and project scopes; app arguments and `--type` are validated by clap; a project without `make/cloudflare.mk` gets an error naming it (bd:mech-crate-vxq) |
| `docs_drift.rs` | every `mx ...` invocation `INFRA_CONFIG.md`, `mx-cloudflare-deploy.md` and the MCP `mx://docs/commands` resource teach parses as a real top-level or nested subcommand (`mx <a> [<b>] --help` exits 0), so a documented command cannot outlive its implementation again (bd:mech-crate-vxq) |

**bd:mech-crate-066** (`mx infra link/unlink` stubs, two link mechanisms) is the
fifth lane row retired, and the second retired by *replacing* its test. The lane
test asserted that `link` creates the Rust resolver's `.env.linked` marker, but
the fix chose the other mechanism: `MX_INFRA_USE_GLOBAL=true` inside the project
file, the one the bash scripts, the old bash `mx` and `INFRA_CONFIG.md` already
used and the only one a `-include` makes visible to `cloudflare.mk`. A test
pinning the losing marker would have failed against a correct fix, so it was
deleted and its subject moved to a suite of its own. Lane 10 → 9, gate
458 → 485 (15 in `infra_link.rs`, 10 unit tests on the resolver and the flag
writer in `mx-lib`, 2 on the inspect renderer), workspace total 468 → 494.

| Suite | Holds |
|---|---|
| `infra_link.rs` | `mx infra link/unlink` write and remove the in-file flag persistently and idempotently, restoring the project file exactly, with honest errors outside a project, for an unknown provider, and without a global config; `mx infra inspect` prints the resolved set with each key's source (environment / project / global, deprecated alias tagged), secrets masked, and reflects the link; a stray `.env.linked` marker is warned about and ignored; and the Rust resolver, `make cf-vars` and `cf-init-app.sh` agree on the account id and its scope before link, after link and after unlink (bd:mech-crate-066) |

**bd:mech-crate-rnj** (the two Unyform clients disagreeing on the org path
segment) is the sixth lane row retired, and the first since `gjl` retired the
intended way: `kb_both_unyform_clients_use_the_same_org_path_segment` lost its
`#[ignore]` and joined the gate, with one added assertion pinning *which*
segment is right, the org **slug** (`/v1/orgs/acme/recipes`), so the test
cannot pass by both clients agreeing on the id. Ground truth came from the
unyform.ai gateway: its JWT-authenticated org routes match `{org}` against the
caller's org slugs (`AuthenticatedUser::is_member_of` compares `o.slug`) and its
API-key org routes ignore the segment, so the slug works on both and the id on
neither. Both clients now build the path with `mx_lib::unyform::org_url` and
resolve the slug with `mx_lib::unyform::resolve_org_slug`; `login` records
`org_slug` in `credentials.json`, and the CLI recovers it once from
`/v1/auth/me` for credentials that predate it. The `9be` lane fixture gained an
`org_slug` so its arrange half still succeeds. Lane 9 → 8, gate 485 → 492 (the
graduated test, `cli_client_recovers_the_org_slug_for_id_only_credentials`
beside it, and 5 unit tests on the two helpers), workspace total 494 → 500.

## Notes on placement deviations

- **rnj** (graduated) lives in `mx-mcp-server` rather than `mx-lib`: that is
  the only crate where *both* Unyform clients are in scope, and the defect was
  precisely their disagreement.
- **dpi** exercises the real `make` layer rather than a stub-bin `make`. The CLI
  already forwards `platform=<p>` correctly; the drop happens in
  `templates/make/build.mk`'s `_build`, so a stub `make` test would pass today
  and prove nothing.
- **4jw** is DB-backed. Its env gate (`MX_RAG_TEST_DATABASE_URL`) lives *inside*
  the ignored test, so a run without a database returns early and reports as
  passing. `make test-known-broken` supplies the URL (container `mx-rag-test`
  on 55433); read a green here as "no DB" unless the container was up.
