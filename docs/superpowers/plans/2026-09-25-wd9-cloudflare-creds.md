# wd9 — Cloudflare credential resolution (CLOUDFLARE_ACCOUNT_ID vs CF_ACCOUNT_ID, global fallback)

**Branch:** `fix/wd9-cloudflare-creds` (from main @ dfc8834, post wave-3 merge)
**Tracking:** T1 = bd mech-crate-wd9; T2 gets a new issue at dispatch.
**Toolkit:** cli. TDD red first; conformance/unit nets extend existing suites; never weaken a test.
**Compatible with:** devloop skill v0.1+

Conventions: one commit per task in repo style; `make test` green after every task (baseline expected 427, verify fresh); bd is the orchestrator's; bd pre-commit hook sweeps .beads/issues.jsonl (commit --no-verify with explicit pathspecs, restore churn); no em dashes in authored copy; no .env* commits.

### Task 1: unify the credential variable and add global-fallback resolution (bd mech-crate-wd9)

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- `mx infra setup cloudflare` (repo-built mx, scratch HOME/config) writes credentials under the SAME variable names the deploy toolchain reads; no CLOUDFLARE_ACCOUNT_ID vs CF_ACCOUNT_ID split remains anywhere in crates/, templates/make/cloudflare.mk, or templates/scripts (grep proves zero mismatched pairs; a back-compat read of the old name is allowed and tested if implemented).
- cloudflare.mk resolves credentials with project-over-global precedence: with ONLY the global file populated, a scaffolded project's `make -n deploy` (or the mk's own var-dump/doctor target) sees the account id; with BOTH files populated, the project value wins. Proven live in scratch with printed values (dummy ids, never real secrets).
- With NEITHER file populated, the toolchain fails with a clear actionable message naming `mx infra setup cloudflare` (no silent empty var).
- Unit/integration tests cover the writer (infra.rs) and the resolution order; `make test` green.

- [x] Decide the canonical name (align with what the wider toolchain and cf-init-app.sh already read; record the call and any back-compat shim in the commit message).
- [x] Red-first tests: writer writes canonical names; mk include chain has global fallback with project override.
- [x] Live E2E per criteria in scratch (dummy credentials only).

### Task 2: docs rider + ship

**Acceptance Criteria (cli-observable):**
**Verify via:** cli
- docs/development/mx-cloudflare-deploy.md section 6 items 2+4 updated from "known gap" to the shipped behavior; any other doc naming the old variable (grep) updated.
- Gates green: `make test`, `make check`; site `npm test` + `npx astro build` (only if site content touched, else state untouched); `mx rag ingest --dry-run` 0 warnings.
- PR open from `fix/wd9-cloudflare-creds` to main, ci.yml + site.yml green. Do NOT merge.

- [ ] PR title `fix: cloudflare credential resolution — one variable name, global fallback`; body: summary + design call + doc updates; repo trailer convention.
