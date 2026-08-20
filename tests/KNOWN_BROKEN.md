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
| mech-crate-z5i | `upgrade::tests::upgrade_discovery_works_against_real_templates_layout` | `crates/mx-lib/src/upgrade/mod.rs` | `discover_upgrades()` succeeds against the shipped `templates/` layout (no phantom `templates/project/`) | unit |
| mech-crate-wd9 | `kb_cloudflare_account_id_var_is_one_contract` | `crates/mx-cli/tests/known_broken.rs` | The `*_ACCOUNT_ID` name `mx infra setup cloudflare` writes is one `templates/make/cloudflare.mk` consumes from the credentials file (greps both sides, asserts equality) | integration |
| mech-crate-066 | `kb_infra_link_writes_marker_and_inspect_resolves_global` | `crates/mx-cli/tests/known_broken.rs` | `mx infra link cloudflare` creates the resolver's `.env.linked` marker, after which `mx infra inspect` resolves to the global credentials | integration |
| mech-crate-vxq | `kb_mx_cf_subcommand_exists` | `crates/mx-cli/tests/known_broken.rs` | The documented `mx cf` subcommand exists (`mx cf --help` exits 0). *Retire this test if the issue is instead closed by purging the doc references.* | integration |
| mech-crate-9be | `kb_recipes_apply_fix_changes_behavior` | `crates/mx-cli/tests/known_broken.rs` | `mx recipes apply <r> --fix` output differs from a plain apply — i.e. `--fix` performs the advertised dependency-drift comparison (wiremock Unyform stub) | integration |
| mech-crate-rnj | `kb_both_unyform_clients_use_the_same_org_path_segment` | `crates/mx-mcp-server/tests/known_broken_unyform.rs` | The CLI client (`mx_lib::unyform`) and the MCP client request the same `/v1/orgs/<seg>/recipes` path against a recording wiremock (today: org **id** vs org **slug**) | integration |
| mech-crate-dpi | `kb_build_platform_reaches_the_build_script` | `crates/mx-cli/tests/known_broken.rs` | `mx build svc --platform linux/arm64` reaches `scripts/build.sh` through the real `make _build` (recording build.sh stub) | integration |
| mech-crate-fr6 | `kb_make_rebuild_is_a_real_target` | `crates/mx-cli/tests/known_broken.rs` | `make rebuild` is a real target: `make -n rebuild` prints at least one command instead of nothing | integration |
| mech-crate-ten | `kb_nuxt_recipe_ships_the_release_scripts_make_invokes` | `crates/mx-lib/tests/recipes_conformance.rs` | The nuxt recipe's app `package.json` declares every `yarn release*` script `templates/make/release.mk` invokes. *Retire if the issue is closed by making `make release` degrade gracefully instead.* | integration |
| mech-crate-gjl | `commands::self_update::tests::kb_self_update_finds_the_source_root_recorded_by_init` | `crates/mx-cli/src/commands/self_update.rs` | `find_source_dir()` resolves the root recorded through `paths::save_source_root` (one marker path across the init ↔ self-update seam) | unit |
| mech-crate-3jt | `kb_recipes_info_shows_the_recipe_version` | `crates/mx-cli/tests/known_broken.rs` | `mx recipes info nuxt` surfaces `recipe.json`'s `version` (read from the recipe so it cannot go stale) | integration |
| mech-crate-o5f | `kb_router_inspect_lists_connected_services` | `crates/mx-cli/tests/known_broken.rs` | `mx router inspect` lists the containers attached to `devmesh-traefik` (stub-bin `docker` returns a 2-container network inspect) | integration |
| mech-crate-dqw | `kb_doctor_reports_router_status` | `crates/mx-cli/tests/known_broken.rs` | `mx doctor` reports on the router (network / container / port 80) alongside its structure and docker checks | integration |
| mech-crate-4jw | `corpus::store::tests::kb_lexical_arm_separates_relevant_from_irrelevant` | `crates/mx-lib/src/corpus/store.rs` | With the vector arm held equal (identical embeddings, orthogonal to the query), the lexical arm separates a relevant from an irrelevant ~1.2KB chunk by ≥5× and ≥0.05 of final score. Measured today: **2.18× / 0.0062** | integration (DB) |

14 lane tests, 13 named `kb_*`; the `z5i` test predates the naming convention
(written in the upgrade task) and is left as-is rather than churned.

**Scoreboard** (`make test-known-broken`): `14 tests run: 0 passed, 14 failed,
189 skipped` — 14 rows above, 14 red, zero bookkeeping debt. The gate suite
(`make test`) in the same tree: `189 passed, 14 skipped`. Those two numbers
partition the workspace; if they stop summing to 203, either a lane test lost
its `#[ignore]` or a gate test grew one.

The lane held at 14 across the first-touch-killer fixes (bd:mech-crate-bj4,
bd:mech-crate-0tj, bd:mech-crate-290, bd:mech-crate-ads): none of those defects
had a lane test, so they were fixed against fresh red tests that joined the gate
directly, taking it from 162 to 189.

## Notes on placement deviations

- **rnj** lives in `mx-mcp-server` rather than `mx-lib`: that is the only crate
  where *both* Unyform clients are in scope, and the defect is precisely their
  disagreement.
- **dpi** exercises the real `make` layer rather than a stub-bin `make`. The CLI
  already forwards `platform=<p>` correctly; the drop happens in
  `templates/make/build.mk`'s `_build`, so a stub `make` test would pass today
  and prove nothing.
- **4jw** is DB-backed. Its env gate (`MX_RAG_TEST_DATABASE_URL`) lives *inside*
  the ignored test, so a run without a database returns early and reports as
  passing. `make test-known-broken` supplies the URL (container `mx-rag-test`
  on 55433); read a green here as "no DB" unless the container was up.
