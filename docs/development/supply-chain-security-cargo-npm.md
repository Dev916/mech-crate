---
title: "Supply-Chain Security for cargo and npm: Where Dependency Code Runs, What the Registries Enforce, and the Gates That Catch Things"
category: security
languages: [rust, javascript, typescript]
complexity: advanced
use_cases:
  - deciding which install-time and build-time hooks a Rust or Node project lets its dependencies run
  - choosing lockfile, pinning and cooldown settings for cargo, npm, pnpm, yarn and bun
  - configuring how a crate or npm package is published so a stolen token or a phished maintainer cannot ship a release
  - picking CI gates (cargo deny, cargo vet, cargo audit, npm audit signatures, OSV-Scanner, dependency review) and knowing what each is blind to
  - responding to a dependency compromise in a Rust or Node repo, and knowing where the advisories come from
summary: "State of practice as of 2026-09 for Rust and JavaScript dependency security, built from the 2025-2026 incident ledger: npm 12 and pnpm 11 block dependency scripts by default while cargo cannot, cooldowns now default on in pnpm and Dependabot, npm has staged 2FA-gated publishing and crates.io has per-crate Trusted Publishing enforcement but still no registry 2FA, and valid provenance was attached to malicious releases twice in 2026."
provenance: researched
researched: 2026-09-25
sources:
  - https://blog.rust-lang.org/releases/
  - https://github.com/npm/cli/releases/tag/v12.0.0
  - https://pnpm.io/blog/releases/11.0
  - https://github.com/yarnpkg/berry/pull/7089
  - https://yarnpkg.com/configuration/yarnrc
  - https://bun.com/blog/bun-v1.3
  - https://nodejs.org/api/permissions.html
  - https://www.stepsecurity.io/blog/harden-runner-detection-tj-actions-changed-files-action-is-compromised
  - https://github.com/advisories/ghsa-mrrh-fwg8-r2c3
  - https://blog.yossarian.net/2025/11/21/We-should-all-be-using-dependency-cooldowns
  - https://nx.dev/blog/s1ngularity-postmortem
  - https://www.aikido.dev/blog/npm-debug-and-chalk-packages-compromised
  - https://blog.rust-lang.org/2025/09/12/crates-io-phishing-campaign/
  - https://github.blog/security/supply-chain-security/our-plan-for-a-more-secure-npm-supply-chain/
  - https://www.kb.cert.org/vuls/id/534320
  - https://www.wiz.io/blog/shai-hulud-npm-supply-chain-attack
  - https://blog.rust-lang.org/2025/09/24/crates.io-malicious-crates-fasterlog-and-asyncprintln/
  - https://securitylabs.datadoghq.com/articles/shai-hulud-2.0-npm-worm/
  - https://socket.dev/blog/5-malicious-rust-crates-posed-as-time-utilities-to-exfiltrate-env-files
  - https://rustsec.org/advisories/RUSTSEC-2026-0036.html
  - https://github.com/axios/axios/issues/10636
  - https://tanstack.com/blog/npm-supply-chain-compromise-postmortem
  - https://safedep.io/mini-shai-hulud-strikes-again-314-npm-packages-compromised/
  - https://www.aikido.dev/blog/keyv-and-friends-compromised-in-npm-supply-chain-attack
  - https://snyk.io/blog/inside-keyv-npm-compromise-preinstall-malware-trusted-provenance-ide-hooks/
  - https://blog.rust-lang.org/2026/08/20/supply-chain-attack-on-arrayref/
  - https://socket.dev/blog/popular-rust-crates-compromised
  - https://rustsec.org/advisories/
  - https://socket.dev/blog/happy-birthday-shai-hulud
  - https://news.ycombinator.com/item?id=48100706
  - https://blog.rust-lang.org/2026/07/13/crates-io-development-update/
  - https://blog.rust-lang.org/2026/03/21/cve-2026-33056
  - https://blog.rust-lang.org/2026/05/25/cve-2026-5222/
  - https://blog.rust-lang.org/2026/05/25/cve-2026-5223/
  - https://arxiv.org/abs/2406.10279
  - https://snyk.io/blog/peacenotwar-malicious-npm-node-ipc-package-vulnerability/
  - https://docs.npmjs.com/cli/v11/using-npm/scripts
  - https://docs.npmjs.com/cli/v11/using-npm/config
  - https://github.com/orgs/community/discussions/198547
  - https://docs.npmjs.com/cli/v12/commands/npm-approve-scripts
  - https://github.blog/changelog/2026-06-09-upcoming-breaking-changes-for-npm-v12/
  - https://docs.npmjs.com/cli/v12/using-npm/config
  - https://github.com/pnpm/pnpm/releases/tag/v10.0.0
  - https://pnpm.io/settings/build
  - https://pnpm.io/settings/dependency-resolution
  - https://bun.com/docs/pm/lifecycle
  - https://bun.com/docs/runtime/bunfig
  - https://doc.rust-lang.org/cargo/reference/build-scripts.html
  - https://goals.rust-lang.org/2024h2/sandboxed-build-script.html
  - https://blog.rust-lang.org/2025/01/23/Project-Goals-Dec-Update/
  - https://blog.rust-lang.org/2026/05/18/project-goals-2026-04/
  - https://rust-analyzer.github.io/book/security.html
  - https://rust-analyzer.github.io/book/configuration.html
  - https://github.com/rust-secure-code/cargo-auditable
  - https://blog.rust-lang.org/2023/08/29/committing-lockfiles/
  - https://doc.rust-lang.org/cargo/faq.html
  - https://doc.rust-lang.org/cargo/commands/cargo-build.html
  - https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html
  - https://doc.rust-lang.org/cargo/commands/cargo-vendor.html
  - https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json
  - https://github.com/lirantal/lockfile-lint
  - https://docs.npmjs.com/cli/v11/commands/npm-ci
  - https://docs.npmjs.com/cli/v11/configuring-npm/package-json
  - https://securitylabs.datadoghq.com/articles/dependency-cooldowns/
  - https://pnpm.io/blog/releases/10.16
  - https://docs.github.com/en/code-security/dependabot/working-with-dependabot/dependabot-options-reference
  - https://github.blog/changelog/2025-07-01-dependabot-supports-configuration-of-a-minimum-package-age/
  - https://github.blog/changelog/2026-07-14-dependabot-version-updates-introduce-default-package-cooldown/
  - https://github.com/npm/cli/releases/tag/v11.10.0
  - https://github.com/yarnpkg/berry/releases/tag/%40yarnpkg%2Fcli%2F4.10.0
  - https://docs.renovatebot.com/configuration-options/
  - https://calpaterson.com/deps.html
  - https://www.sonatype.com/blog/software-dependency-cooldowns-are-a-symptom-not-a-strategy
  - https://news.ycombinator.com/item?id=46005111
  - https://news.ycombinator.com/item?id=47773812
  - https://github.blog/changelog/2026-07-28-npm-publish-time-malware-scanning-and-dual-use-metadata/
  - https://github.blog/changelog/2026-05-22-staged-publishing-and-new-install-time-controls-for-npm/
  - https://github.blog/changelog/2025-07-31-npm-trusted-publishing-with-oidc-is-generally-available/
  - https://github.blog/changelog/2025-09-29-strengthening-npm-security-important-changes-to-authentication-and-token-management/
  - https://github.blog/changelog/2025-11-05-npm-security-update-classic-token-creation-disabled-and-granular-token-changes/
  - https://github.blog/changelog/2025-12-09-npm-classic-tokens-revoked-session-based-auth-and-cli-token-management-now-available/
  - https://socket.dev/blog/npm-to-implement-staged-publishing
  - https://github.blog/security/supply-chain-security/disrupting-supply-chain-attacks-on-npm-and-github-actions/
  - https://docs.npmjs.com/trusted-publishers/
  - https://docs.npmjs.com/about-two-factor-authentication/
  - https://github.blog/changelog/2026-09-03-multiple-trusted-publishing-configurations-for-npm/
  - https://github.blog/changelog/2026-09-18-stage-only-npm-tokens-for-safer-automation/
  - https://docs.npmjs.com/requiring-2fa-for-package-publishing-and-settings-modification/
  - https://docs.npmjs.com/generating-provenance-statements
  - https://blog.rust-lang.org/2023/06/23/improved-api-tokens-for-crates-io/
  - https://blog.rust-lang.org/2025/02/05/crates-io-development-update/
  - https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07
  - https://rust-lang.github.io/rfcs/3691-trusted-publishing-cratesio.html
  - https://blog.rust-lang.org/2026/01/21/crates-io-development-update
  - https://blog.rust-lang.org/2026/02/13/crates.io-malicious-crate-update
  - https://nesbitt.io/2026/08/18/two-factor-authentication-across-package-registries.html
  - https://news.ycombinator.com/item?id=48664733
  - https://rustsec.org/
  - https://tanstack.com/blog/incident-followup
  - https://embarkstudios.github.io/cargo-deny/checks/index.html
  - https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html
  - https://mozilla.github.io/cargo-vet/
  - https://mozilla.github.io/cargo-vet/built-in-criteria.html
  - https://github.com/mozilla/cargo-vet/blob/main/registry.toml
  - https://mozilla.github.io/cargo-vet/importing-audits.html
  - https://github.com/crev-dev/cargo-crev
  - https://github.com/rust-secure-code/cargo-supply-chain
  - https://docs.npmjs.com/cli/v11/commands/npm-audit
  - https://overreacted.io/npm-audit-broken-by-design/
  - https://google.github.io/osv-scanner/supported-languages-and-lockfiles/
  - https://github.com/actions/dependency-review-action
  - https://github.com/ossf/scorecard/blob/main/docs/checks.md
  - https://docs.deps.dev/
  - https://slsa.dev/spec/v1.0/levels
  - https://docs.github.com/en/actions/concepts/security/artifact-attestations
  - https://docs.sigstore.dev/about/overview/
  - https://docs.github.com/en/actions/reference/security/secure-use
  - https://github.blog/changelog/2025-08-15-github-actions-policy-now-supports-blocking-and-sha-pinning-actions/
  - https://github.blog/changelog/2025-10-28-immutable-releases-are-now-generally-available/
  - https://github.com/actions/checkout
  - https://docs.deno.com/runtime/fundamentals/security/
---

# Supply-Chain Security for cargo and npm: Where Dependency Code Runs, What the Registries Enforce, and the Gates That Catch Things

State of practice as of 2026-09-25. Baseline versions: Rust 1.98.1 (2026-09-03) [1]; npm 12.0.0 (2026-07-08) [2]; pnpm 11.0 (2026-04-28) [3]; Yarn 4.14 with `enableScripts: false` (2026-04) [4][5]; Bun 1.3 [6]; the Node permission model stable since v22.13.0 [7]. The Python side of the same problem (Trusted Publishing, PEP 740, hash-pinned installs) lives in `python-secure-coding-practices.md`; container build hygiene lives in `docker-assembly-guide.md`.

Everything outside `## Synthesis (inferred)` is a claim traceable to a cited page; inline `[n]` keys to `sources`. Config examples were parse-checked (JSON, YAML, TOML); `cargo build --locked` and `cargo audit` were run on a scratch crate with cargo 1.93.0; `cargo deny`, `cargo vet`, pnpm 11 and the npm 12 commands were not installed locally, so those examples are illustrative and nothing was run against a live registry (verification note in section 9).

The doc is organised around the three places a dependency ecosystem can be attacked: **publish** (who can push a version), **resolve** (which version your tool picks and from where), and **execute** (what runs when you install, build, open, or run). Section 1 is the incident ledger; sections 2 to 8 take the three boundaries in turn; section 9 is the config; section 10 corrects the folklore; Synthesis gives the defaults.

## 1. Threat model: what the 2025–2026 incident ledger actually shows

The useful threat model is not a taxonomy but the ledger of what happened, how the attacker got in, how long the bad version was installable, and who noticed. Every row has a primary write-up from the registry operator, the maintainers, or the original discloser.

| Date | Ecosystem | Package(s) | Entry | Execution | Live window | Found by |
|---|---|---|---|---|---|---|
| 2025-03-14 | GitHub Actions | `tj-actions/changed-files` (CVE-2025-30066) | A PAT "linked to the @tj-actions-bot bot account" [8]; "Attackers retroactively modified multiple version tags to reference a malicious commit" [9] | Payload "extracted secrets from the Runner Worker process memory and printed them in GitHub Actions logs" [9] | ~3 days [10] | StepSecurity [8] |
| 2025-08-26 | npm | `nx` | "PR title validation workflow with injection vulnerability" on `pull_request_target`, with "Workflow permissions set to read/write" [11] | `postinstall` that "scanned user systems for sensitive data, attempted to use local AI tools (like Claude and Gemini), and uploaded the results to a public GitHub repo" [11] | "active for 4 hours before being completely removed" [11] | Community; Nx post-mortem [11] |
| 2025-09-08 | npm | `chalk`, `debug` + 16 more, "2+ billion weekly downloads" | Maintainer phished from `support@npmjs.help`, a domain "registered ... on September 5, 2025" [12] | Browser payload that "rewrites payment destinations so that funds and approvals are redirected to attacker-controlled accounts" [12] | under 12 hours [10] | Aikido [12] |
| 2025-09-12 | crates.io | (accounts) | Phishing from `rustfoundation.dev` asking users to "authenticate to limit damage to their crates" [13] | n/a | n/a | crates.io team [13] |
| 2025-09-15 | npm | Shai-Hulud wave 1, "500+ compromised packages" [14] | Compromised maintainer accounts; "a self-replicating worm ... injecting malicious post-install scripts" [14] | "A `postinstall` script led to the execution of a malicious `bundle.js` file" that ran TruffleHog and "self-propagated using the stolen credentials to publish itself to other repositories and package registries" [15] | — | Wiz [16] and other vendors |
| 2025-09-24 | crates.io | `faster_log`, `async_println` | Typosquats "Published May 25, 2025"; 7,181 and 1,243 downloads [17] | Runtime: "they did not execute any malicious code at build time" [17] | Four months on the registry; deleted "at 15:34 UTC on September 24, 2025" [17] | Socket Threat Research [17] |
| 2025-11-24 | npm | Shai-Hulud 2.0: "796 unique npm packages" across "1,092 package versions" [18] | Stolen npm and GitHub tokens [18] | "a new preinstall script" running `setup_bun.js` and `bun_environment.js`; propagates by "bumping the patch version"; if propagation fails it "attempts to delete the user's home directory" [18] | — | Datadog [18] and other vendors |
| 2026-02/03 | crates.io | `chrono_anchor`, `dnp3times`, `time_calibrator`, `time_calibrators`, `time-sync` | Brandjacked "time utilities" that `curl -F file=@.env` to lookalike `timeapis[.]io` [19] | Runtime, inside "routine-looking parameter validation and optional sync helpers" [19] | Four removed within 3–50 minutes; `chrono_anchor` listed until reported, 66 downloads [19] | Socket [19]; RustSec advisory [20] |
| 2026-03-31 | npm | `axios` 1.14.1, 0.30.4 | "targeted social engineering campaign and RAT malware" on the lead maintainer's PC [21] | Injected dependency `plain-crypto-js@4.2.1` that "installed a remote access trojan on macOS, Windows, and Linux" [21] | ~3 hours; attacker deleted issues reporting it [21] | Community, escalated to npm at 01:38 UTC [21] |
| 2026-05-11 | npm | 42 `@tanstack/*` packages, 84 versions | `pull_request_target` running fork code + Actions cache poisoning + "an OIDC token from the GitHub Actions runner process" [22] | `optionalDependencies` pointing at an orphan commit; "a ~2.3 MB obfuscated router_init.js smuggled into the affected tarball" [22] | 19:20 UTC to ~22:13 UTC first tarball removal [22] | External researcher (StepSecurity), 20–26 minutes after publish; "No internal alerting" [22] |
| 2026-05-19 | npm | 317 packages, 637 versions via the `atool` account | Compromised maintainer token [23] | `"preinstall": "bun run index.js"` plus an `optionalDependencies` GitHub fallback in 630 of 637 versions [23] | Two publish waves, 01:39–02:06 UTC [23] | SafeDep and other scanners [23] |
| 2026-08-04 | npm | `keyv` 6.0.0, `cacheable`, `flat-cache`, `cache-manager` + 7 more, then 444 names / 1,381 versions by worm spread | Compromised GitHub account; malicious files pushed to `main`, then a normal release [24] | `"preinstall": "node setup.mjs"` dropper downloading Bun, then a 728 KB credential extractor [24] | Eleven releases published 09:35–10:28 UTC; eight still on `latest` at 11:16 UTC [25] | Aikido, Snyk [24][25] |
| 2026-08-20 | crates.io | `arrayref` 0.3.10, `internment` 0.8.7, `append-only-vec` 0.1.9 | Maintainer "computer or credentials are likely compromised" [26] | Build time: the injected `proc-macro1` (typosquat of `proc-macro2`) "had a build script that was downloading a malicious payload" [26] from `23[.]254[.]165[.]112:9089` [27] | 86, 90 and 107 minutes [26] | Nextron Systems [26] |

Volume matters too: RustSec lists roughly 35 advisories dated 2026 for crates "removed from crates.io" for malicious code [28], and Socket's one-year retrospective counts a "Third Coming" (April 2026), a "Mini Shai-Hulud" (late April), the worm's source code published on GitHub (May 2026), and one variant that "pushed more than 400 malicious versions across 172 packages in about five hours" [29]. Discussion of the TanStack post-mortem on Hacker News: [30].

Six things fall out of the ledger.

**The entry point is almost always a person or a pipeline, not the registry.** axios was a maintainer's laptop [21]. keyv was a GitHub account [24]. TanStack and Nx were workflow triggers that ran fork code [22][11]. chalk was an email [12]. arrayref was a maintainer's machine or credentials [26]. The crates.io phishing wave went after GitHub credentials, because a crates.io identity is a GitHub identity [13][31]. Every registry-side control in section 5 is a response to that fact.

**Valid provenance does not mean safe code.** keyv's poisoned versions were "published to npm with valid provenance signed by GitHub Actions" because the attacker committed to `main` and let the project's own release workflow build and attest the result [24]. Snyk's reading is the one to keep: "provenance can faithfully attest a build whose source or workflow context has already been compromised" [25]. TanStack's attacker went one step further and lifted the OIDC token out of runner memory, so the publish itself came through the trusted-publisher binding [22]. The May 2026 wave carried "Sigstore abuse for legitimate code signing via stolen OIDC tokens" [23].

**Install-time and build-time hooks are the execution surface, and the ecosystems differ.** The npm worms from Nx to keyv ran from `postinstall` or `preinstall` [11][15][18][23][24]. The arrayref campaign ran from a `build.rs` [26], while the two earlier crates.io campaigns waited for the victim to call the crate at runtime [17][19]. Section 2 is about what each package manager lets you do about that.

**The live window is hours, and the finder is usually a scanner, not a victim.** Three hours for axios and TanStack [21][22], four for Nx [11], under two for the arrayref set [26], minutes for four of the five time crates [19]. In every 2026 case but axios the report came from a security vendor or researcher, not from an installer noticing; axios was caught by community members within the hour and escalated by a collaborator [21]. This is the empirical basis for cooldowns (section 4). The counterexample is a dormant runtime payload: `faster_log` sat on crates.io for four months [17].

**The payload now targets your agents and your CI, not just your `.npmrc`.** The keyv worm wrote `.claude/settings.json` with a `SessionStart` command and `.vscode/tasks.json` with `runOn: "folderOpen"` into every reachable branch, up to 50 per repository, with commits authored as `claude@users.noreply.github[.]com` [24][25]. The May wave dumped GitHub Actions secrets through `toJSON(secrets)` artifact uploads and harvested AWS, GCP, Azure, Kubernetes, Vault, Stripe and password-manager material [23]. TanStack's payload read AWS IMDS, GCP metadata and Kubernetes service-account tokens [22]. Nx's tried to drive local Claude and Gemini CLIs [11]. A dependency install is now a credential-harvesting event by default, which is why section 8 treats the install step as untrusted code.

**The client itself is in scope.** Cargo shipped three advisories in 2026: CVE-2026-33056 let "a malicious crate to change the permissions on arbitrary directories on the filesystem when Cargo extracts it during a build", with crates.io blocking such uploads from 2026-03-13 and a fix in Rust 1.94.1 [32]; CVE-2026-5222, where Cargo "incorrectly normalized the URLs of third-party registries" so credentials could leak between registries, affecting "All versions of Cargo shipped between Rust 1.68 ... and 1.96" at "low" severity [33]; and CVE-2026-5223 on symlinks in third-party registry tarballs, where "Users of crates.io are **not affected**, as crates.io forbids uploading crates containing any symlink" [34]. Keep the toolchain current for the same reason you keep the dependencies current.

Two attack classes did not make the 2026 ledger but shape the controls. Slopsquatting has quantified evidence: across "576,000 code samples" from "16 popular LLMs", hallucinated package names appeared at "5.2% for commercial models and 21.7% for open-source models", yielding "205,474 unique examples of hallucinated package names" [35]. Protestware is the node-ipc precedent (CVE-2022-23812): versions "10.1.1, 10.1.2, 9.2.2, 11.0.0, and 11.1.0" overwrote files for hosts geolocated in Russia or Belarus, reached via `@vue/cli → @vue/cli-ui → node-ipc@^9.2.1` [36].

## 2. Where dependency code executes, and what each tool lets you refuse

### 2.1 npm: lifecycle scripts, and the npm 12 default flip

On `npm install`, npm runs `preinstall`, `install`, `postinstall`, `prepublish`, `preprepare`, `prepare` and `postprepare` [37]. npm's own guidance to package authors is "Don't use `install`. Use a `.gyp` file for compilation, and `prepare` for anything else" [37]. The classic consumer switch is `ignore-scripts`: "Default: false ... If true, npm does not run scripts specified in package.json files" [38]. It is global: the doc says npm "does not run scripts specified in package.json files", plural, so your own project's `prepare` is skipped too [38], and any dependency that compiles or downloads in an install hook is left unbuilt.

npm 12.0.0 (2026-07-08) changed the default: "Dependency lifecycle scripts are now blocked by default unless allowed by the root package's allowScripts policy" [2]. The npm team's justification: "Install-time lifecycle scripts are the single largest code-execution surface in the npm ecosystem ... Making script execution opt-in closes that path while keeping it one command away for the packages you trust" [39]. The mechanism:

- `allowScripts` is an object in `package.json` mapping a package name, or a pinned `name@version`, to a boolean [40].
- "By default, `npm approve-scripts <pkg>` pins the approval to the installed version (`pkg@1.2.3`)"; `--no-allow-scripts-pin` writes a name-only entry that permits any future version [39][40].
- `npm approve-scripts --allow-scripts-pending` is "read-only: lists every package whose scripts aren't yet covered" [39]; `npm deny-scripts` writes an explicit `false` and "always writes name-only entries" [40].
- Native modules are covered too: "these are blocked even if they have no explicit install script, because npm runs an implicit `node-gyp rebuild` for any package with a `binding.gyp`" [39].
- In npm 11.16.0 and later the policy runs in warning mode: "an install script you haven't approved gets skipped, you get a warning, and the install still succeeds" [39]; the recommended migration is to upgrade to 11.16, run the pending listing, approve, and commit `package.json` [41].
- Also in v12: "allow-git and allow-remote now default to 'none'; set them to 'all' (or 'root') to install git or user-supplied tarball-URL dependencies" [2], while `allow-file` and `allow-directory` keep the default `"all"` [42]; `npm-shrinkwrap.json` support is gone; and "root preinstall now runs before dependencies are installed" [2].

The May 2026 `atool` wave is the reason the git/remote defaults matter: 630 of its 637 versions carried an `optionalDependencies` entry resolving to an orphan commit on GitHub whose `prepare` script re-ran the payload, "bypassing `preinstall` blocks" [23]. Blocking scripts without blocking git sources leaves that door open.

### 2.2 pnpm, Yarn and Bun

| Manager | Dependency scripts | Allowlist | Exotic sources | Cooldown default |
|---|---|---|---|---|
| pnpm 11 | Blocked since 10.0.0 (2025-01-07): "Lifecycle scripts of dependencies are not executed during installation by default!" [43]; `strictDepBuilds` (default true in v11) makes "the installation ... exit with a non-zero exit code if any dependencies have unreviewed build scripts" [44] | `allowBuilds`, "a map from package name patterns to booleans" in `pnpm-workspace.yaml`; `onlyBuiltDependencies` "removed in v11" [3]; `pnpm approve-builds` "accepts positional arguments for non-interactive use; prefix a name with `!` to deny it" [3] | `blockExoticSubdeps` (v10.26.0, default true): "Only direct dependencies may use exotic sources (like git repositories or direct tarball URLs). All transitive dependencies must be resolved from a trusted source" [45] | `minimumReleaseAge: 1440` minutes [3] |
| Yarn 4.14+ | `enableScripts`: "If false (the default), Yarn will not execute the `postinstall` scripts from third-party packages" [5]; flipped by PR #7089 merged 2026-03-31, with an auto-migration writing `enableScripts: true` into existing projects [4] | opt back in per project with `enableScripts: true`, which the 4.14 migration writes for existing projects [4] | n/a | `npmMinimalAgeGate` default `"1w"` [5] |
| Bun 1.3 | "Bun is 'default-secure': it only runs lifecycle scripts for packages on an allow list" [46] | `trustedDependencies` "**replaces** the default list rather than extending it" [46] | n/a | `minimumReleaseAge` in seconds, "Default `null` (disabled)" [47] |
| npm 12 | Blocked by default [2] | `allowScripts` [40] | `allow-git`/`allow-remote` default `none` [2] | `min-release-age` "Default: null" [42] |

The `dangerouslyAllowAllBuilds` escape hatch exists in pnpm (v10.9.0, default false) [44]; the name is the documentation.

### 2.3 Cargo: build scripts and proc-macros run at build time, and there is no switch

"Placing a file named `build.rs` in the root of a package will cause Cargo to compile that script and execute it just before building the package" [48]. The Rust project's own goal text is blunter: "Build scripts in Cargo can do literally anything from network requests to executing arbitrary binaries" and "an unsandboxed build script is effectively an enormous `unsafe` block" [49]. There is no consumer-side setting to disable a dependency's build script; the `package.build` key only lets a package author disable their own [48]. Proc-macros are compiled and executed by the compiler in the same way.

Sandboxing was explored and shelved. The 2024H2 project goal was accepted [49], but the December 2024 update states that "Alternatives to sandboxed build scripts are going to be investigated instead of continuing this project goal into 2025h1", naming system-deps, Cackle with Bubblewrap, and Docker or Nix as the alternatives [50]. The April 2026 goals update contains nothing on the subject [51]. Treat Cargo as having no execution boundary between your shell and your dependency tree, and compensate in sections 6 and 8.

Opening the repo is also execution. rust-analyzer "assumes that all code is trusted" and "proc macros and build scripts are executed by default" [52]; `rust-analyzer.cargo.buildScripts.enable` defaults to `true`, and `rust-analyzer.procMacro.enable` defaults to `true` and "implies `#rust-analyzer.cargo.buildScripts.enable#`" [53]. The arrayref payload therefore fired the moment an editor indexed a workspace that depended on it [26].

The 2025 and early-2026 crates.io campaigns, by contrast, waited for runtime [17][19]. That is the case build-time controls cannot touch and only review (cargo vet, section 6) or runtime sandboxing (section 8) can.

One Rust-specific asset for after the fact: `cargo auditable` embeds the dependency list in the binary; "The JSON is Zlib-compressed and placed in a linker section named `.dep-v0`", "cargo audit v0.17.3+ can detect this data in binaries", and it is used by "Alpine Linux, NixOS, openSUSE, Void Linux, Chimera Linux and Wolfi OS", with "Ubuntu 26.04 uses it for select packages" [54]. If you ship Rust binaries, this is how you answer "which builds contain `arrayref` 0.3.10" a year later.

## 3. Lockfiles, pinning and reproducibility

### 3.1 Cargo

Commit `Cargo.lock`. The 2023 change to the guidance: the old rule was to "commit their `Cargo.lock` file for packages with binaries but not libraries"; the new one is to "do what is best for their project" with committing as the default, and `cargo new` stopped ignoring the lockfile for libraries "as of nightly-2023-08-24" [55]. The lockfile is local to your build: "`Cargo.lock` does not affect the consumers of your package, only `Cargo.toml` does that" [56].

Make CI assert it. `--locked` "Asserts that the exact same dependencies and versions are used as when the existing `Cargo.lock` file was originally generated" and errors if "The lock file is missing" or "Cargo attempted to change the lock file"; `--frozen` is "Equivalent to specifying both `--locked` and `--offline`" [57]. `cargo install` "will select the latest dependencies unless `--locked` is passed" [56], so a tool installed in CI without `--locked` resolves fresh every time.

Overrides and vendoring: "`[patch]` applies _transitively_ but can only be defined at the _top level_"; "Cargo only looks at the patch settings in the `Cargo.toml` manifest at the root of the workspace"; "`[replace]` is deprecated" [58]. `cargo vendor` "will vendor all crates.io and git dependencies for a project into the specified directory", prints the source-replacement stanza to add to `.cargo/config.toml`, and `--versioned-dirs` "causes all directories in the "vendor" directory to be versioned" [59]. Vendoring turns the registry into a reviewed diff in your own repo, which is the strongest form of pinning and the one that survives a registry outage.

### 3.2 npm

`package-lock.json` `lockfileVersion` 1 is "npm v5 and v6", 2 is "npm v7 and v8", 3 is "npm v9 and above"; each entry's `integrity` is "A `sha512` or `sha1` Standard Subresource Integrity string for the artifact that was unpacked" and `resolved` is "a url to a tarball" [60]. What the hash proves is narrow: the tarball is the one recorded when the lock was written. It does not prove who published it or that it is benign, and because `resolved` is a URL, a lockfile edit can point a name at any host, which is lockfile-lint's premise: "What happens when someone creates a Pull Request and sneaks a malicious resource package that replaces a real library?" [61]. pnpm is immune to that particular edit because "pnpm doesn't maintain the tarball source ... there's no way to inject an attacker-controlled malicious source file in `pnpm-lock.yaml`" [61].

`npm ci` is the CI install: "The project **must** have an existing `package-lock.json`"; "If dependencies in the package lock do not match those in `package.json`, `npm ci` will exit with an error, instead of updating the package lock"; "If a `node_modules` is already present, it will be automatically removed"; "It will never write to `package.json` or any of the package-locks: installs are essentially frozen" [62]. Pair it with `--ignore-scripts` on npm 11 or the `allowScripts` policy on npm 12.

Two lesser-known knobs. `--before` "will rebuild the npm tree such that only versions that were available **on or before** the given date are installed", and "If `before` and `min-release-age` are both set in the same source, `before` wins" [38]. `overrides` in the root `package.json` let you force a transitive version; "Overrides are only considered in the root `package.json` file for a project" [63].

## 4. Cooldowns: the one control the ledger directly supports, and the argument against it

The claim, from Woodruff's November 2025 post that started the practice's adoption: "dependency cooldowns are a free, easy, and **incredibly effective** way to mitigate the _large majority_ of open source supply chain attacks" [10]. His evidence is the window between publish and public detection across 2025 incidents: Ultralytics phase 2 and rspack one hour, web3.js five hours, chalk and num2words under 12 hours, Nx four hours, tj-actions three days, the Kong ingress controller about ten days, xz-utils about five weeks; "8 out of 10 attacks had windows under one week" [10]. He recommends seven days as blocking "the vast majority" and 14 as blocking everything but xz, and is explicit that cooldowns are "obviously, **not a panacea**" [10]. Datadog's analysis of axios adds that the "malicious versions were discovered roughly three hours after release" and "a cooldown of 12 hours would have blocked the spread entirely" [64]. The 2026 ledger in section 1 is consistent: every install-time worm was detected and removed within hours.

The defaults moved in 2026:

| Tool | Setting | Default | Since | Exemptions |
|---|---|---|---|---|
| pnpm | `minimumReleaseAge` (minutes), introduced in 10.16.0 because "In most cases, malicious releases are discovered and removed from the registry within an hour" [65] | `1440`: "Newly published packages won't be resolved until they're at least 1 day old. To opt out, set `minimumReleaseAge: 0`" [3] | 2026-04-28 [3] | `pnpm audit --fix` "adds the minimum patched version for each advisory to `minimumReleaseAgeExclude`" so security fixes skip the wait [3] |
| Dependabot | `cooldown` with `default-days`, `semver-major-days`, `semver-minor-days`, `semver-patch-days`, `include`, `exclude` (up to 150 items each) [66]; launched opt-in 2025-07-01 [67] | "If not specified, Dependabot applies a default cooldown of 3 days" [66]; "waits until a new release has been available on its registry for at least three days" [68] | 2026-07-14 [68] | "Security updates still open immediately" [68]; Cargo and npm/Yarn both support the semver-specific keys [66] |
| npm | `min-release-age` (days), added in 11.10.0 (2026-02-11): "only versions that were available more than the given number of days ago will be installed" [69][42] | `null` in npm 11 and 12 [42] | opt-in | `before` wins if both set [38] |
| Yarn | `npmMinimalAgeGate`, introduced in 4.10.0 [70]: "If a package version is newer than the minimal age gate, it will not be considered for installation" [5] | `"1w"` [5] | 4.10 (2025-09) [70] | `npmPreapprovedPackages` [70] |
| Bun | `minimumReleaseAge` (seconds) [47] | `null` [47] | 1.3 (2025-10) [6] | `minimumReleaseAgeExcludes` [47] |
| Renovate | `minimumReleaseAge`, duration strings such as "3 days" [71] | `null` [71] | opt-in | |
| Cargo | none: no such setting exists in the Cargo reference; the only Rust-side cooldown is the Dependabot or Renovate one, and Dependabot's `default-days` applies to Cargo [66] | n/a | n/a | |

Paterson's April 2026 rebuttal is the strongest objection and deserves a fair statement: "dependency cooldowns work by free-riding on the pain and suffering of others", and if everyone adopts them, "any sufficiently widespread dependency cooldown becomes an ad-hoc, informally specified, hole-ridden, slow implementation of an upload queue" [72]. His alternative is to "separate package publication and package distribution" at the registry, with scanning and review in between, as Debian does [72]. Sonatype's June 2026 position is the practitioner version of the same point: "A package that is 24 hours old is not automatically safe. Neither is one that is 30 days old"; "time is a pretty weak proxy" [73]. Datadog concedes the adaptive risk: "As cooldowns become widespread, patient attackers may delay malware execution to survive the waiting period" [64]. The ledger already holds one such case: `faster_log` was published on 2025-05-25 and removed on 2025-09-24 [17]. Hacker News discussion of both posts: [74], [75].

The two positions are closer than the titles suggest, and the registries have started to build the queue he asks for. npm's publish-time malware scan inserts "a short delay between publishing and availability, typically around five minutes" [76], and staged publishing puts a human 2FA approval between upload and availability [77]. Neither is a seven-day hold, so client-side cooldowns remain the only thing that covers the hours-long windows in section 1. See Synthesis for the policy this doc recommends and for where the free-rider objection actually bites.

## 5. What the registries now enforce

### 5.1 npm: from long-lived tokens to staged, scanned, 2FA-gated publishes

The sequence matters because each step closed the entry point used by the previous wave.

| When | Change | Source |
|---|---|---|
| 2025-07-31 | Trusted Publishing GA for "GitHub Actions (GitHub-hosted runners)" and "GitLab CI/CD (gitlab.com shared runners)"; "requires npm CLI v11.5.1 or later"; "npm CLI publishes provenance attestations by default. The `--provenance` flag is no longer needed"; "Self-hosted runners are not currently supported" | [78] |
| 2025-09-22 | GitHub's plan after Shai-Hulud: "Granular tokens which will have a limited lifetime of seven days", "Deprecate legacy classic tokens", "Deprecate time-based one-time password (TOTP) 2FA, migrating users to FIDO-based 2FA", "Remove the option to bypass 2FA for local package publishing", "Set publishing access to disallow tokens by default" | [14] |
| 2025-09-29 | Granular tokens get "A default expiration of seven days, reduced from 30 days" and "A maximum expiration of 90 days, which used to be unlimited"; "New TOTP ... setups for npm access will be permanently disabled" | [79] |
| 2025-11-05 | "New npm classic tokens can no longer be created"; write tokens "enforce 2FA by default"; existing write tokens "capped at 90-day maximum lifetime" | [80] |
| 2025-12-09 | "We've permanently revoked all existing npm classic tokens"; `npm login` now yields "a two-hour session token" | [81]; lifetime extended to 12 hours three days later after maintainer complaints [82] |
| 2026-02-11 | npm 11.10.0: `min-release-age` and bulk trusted-publishing configuration | [69] |
| 2026-04 | CircleCI added as a trusted publishing provider (no provenance for CircleCI publishes) | [83][84] |
| 2026-05-22 | Staged publishing GA: "the prebuilt tarball is uploaded to a stage queue where a maintainer must explicitly approve it before it becomes installable"; "A human maintainer with a 2FA challenge is required to approve"; needs "npm CLI 11.15.0 or newer" | [77] |
| 2026-05-22 | `--allow-file`, `--allow-remote`, `--allow-directory` join `--allow-git`; each "accepts all (the current default) or none"; `--allow-git` flips "from all to none in the next major version of the CLI (v12)" | [77] |
| 2026-06 | High-impact accounts enter read-only mode for 72 hours after an email change or a 2FA recovery-code login; `actions/checkout` default changed to "prevent the checkout of untrusted code from forks in commonly exploited triggers"; Actions cache made read-only for untrusted triggers | [83] |
| 2026-07-08 | npm v12.0.0: dependency scripts blocked by default; `allow-git`/`allow-remote` default `none`; `npm-shrinkwrap.json` support removed; supports "node ^22.22.2 \|\| ^24.15.0 \|\| >=26.0.0" | [2] |
| 2026-07-28 | Publish-time malware scanning: "Newly published packages will be automatically scanned before they become available for install", a delay "typically around five minutes" and "up to 15 minutes or more, at peak times"; blocked publishers "may receive a notification with the option to appeal" | [76] |
| 2026-07-28 | Dual-use metadata: a `contentPolicy` field in `package.json` plus a `DISCLOSURE` file, used by npm's Trust & Safety team when reviewing packages whose legitimate behaviour "can resemble malware to automated scanning" | [76] |
| 2026-08 | "Starting August 2026, account-identity and account-governance actions cannot be performed with a bypass-2FA token" | [85] |
| 2026-09-03 | A package may have "more than one trusted publishing (OIDC) configuration"; "Direct publishing is opt-in per configuration, while staging is enabled by default"; the approve button "is now disabled while a package is still being scanned" | [86] |
| 2026-09-18 | Stage-only granular tokens: can run `npm stage publish` but "npm rejects direct `npm publish` attempts with that token, even if you've configured it to bypass 2FA" | [87] |

Where 2FA actually stands as of September 2026: publishing "requires either: Two-factor authentication (2FA) enabled on your account, OR A granular access token with bypass 2FA enabled" [85]. The package-level default is "Require two-factor authentication or a granular access token with bypass 2fa enabled"; the recommended setting is "Require two-factor authentication and disallow tokens" [88]. So the September 2025 promise to remove bypass-2FA for local publishing is delivered only for account-governance actions; the rest is a per-package setting you must flip yourself.

Three details are easy to get wrong. Staged publishing is opt-in per workflow: you must "Update CI/CD workflows to use `npm stage publish` instead of `npm publish`" [77], and a trusted-publishing configuration can be "limited to stage-only, which means npm publish from that workflow will be rejected and only npm stage publish is accepted" [77]. The 2FA guarantee sits on the promotion step, not the upload: "granular access tokens with bypass-2FA may still be used to publish **to staging**" [76], which is the intended design and why the stage-only token exists [87]. And provenance has preconditions: "Ensure you are on `9.5.0+`", supported from "GitHub Actions and GitLab CI/CD", verified with `npm audit signatures`, and it "remains unavailable when publishing from private source repositories" [89]; trusted publishing also requires "Node version 22.14.0 or higher" [84].

The migration off classic tokens was rough. Trusted publishing at the time "cannot be used to publish new packages", required "manual configuration through the npm website on a per-package basis", and "lacks an API for bulk setup"; the rapid succession of policy changes caused "unexpected authentication failures" for maintainers running many packages from personal accounts [82]. Read that as the cost of the change, not a reason to keep long-lived tokens.

### 5.2 crates.io: trusted publishing, enforced per crate, and still no registry-level 2FA

| When | Change | Source |
|---|---|---|
| 2023-06 | API tokens gain "endpoint scopes, crate scopes and expiration dates" | [90] |
| 2025-02 | "API tokens created on crates.io now expire after 90 days by default. It is still possible to disable the expiry" | [91] |
| 2025-07-11 | Trusted Publishing announced, GitHub Actions first, per RFC 3691; "A _Trusted Publisher Configuration_ can only be created after an initial manual publishing of a crate"; provenance is explicitly "Out of Scope" | [92][93] |
| 2026-01-21 | A "Security" tab on crate pages "displays security advisories from the RustSec database" | [94] |
| 2026-01-21 | Trusted Publishing "now supports GitLab CI/CD in addition to GitHub Actions"; "this currently only works with GitLab.com" | [94] |
| 2026-01-21 | "Crate owners can now enforce Trusted Publishing for their crates": once on, "traditional API token-based publishing is disabled" | [94] |
| 2026-01-21 | "The `pull_request_target` and `workflow_run` GitHub Actions triggers are now blocked from Trusted Publishing" because they "have been responsible for multiple security incidents" | [94] |
| 2026-01-21 | GitHub OAuth access tokens "are now encrypted at rest in the database" | [94] |
| 2026-02-13 | Per-crate malware blog posts stop, because they covered "crates that have no evidence of real world usage"; crates.io "will always publish a RustSec advisory when a crate is removed for containing malware", and crates with real usage still get both | [95] |
| 2026-03-13 | Uploads exploiting CVE-2026-33056 blocked at the registry, ahead of the Cargo fix | [32] |
| 2026-05 | RFC 3946 accepted: native crates.io usernames, the prerequisite for login providers other than GitHub | [96][31] |
| 2026-08-20 | arrayref response: malicious versions deleted, previously yanked benign versions unyanked, the maintainer account locked | [26] |

Two structural facts distinguish crates.io from npm in 2026. Identity is still GitHub identity: "signing in means 'Log in with GitHub', and your crates.io identity is your GitHub username" [31], which is why the September 2025 phishing wave targeted GitHub credentials [13] and why the June 2026 "GitHub shouldn't be a dependency for publishing Rust" thread landed [97]. And 2FA is not enforced by the registry: Nesbitt's August 2026 survey records crates.io as requiring 2FA for nothing at the registry level, delegating it to GitHub, with the refuse-logins-without-2FA idea discussed "but it has not been implemented" [96]. No provenance or attestation support exists on crates.io either; the July 2026 update adds only a "Code" tab for browsing "the exact files that `cargo` downloads" [31]. The per-crate Trusted Publishing enforcement is therefore the strongest publisher control a crate owner has today [94].

The notification policy change has a practical consequence: the signal for malicious crates is the RustSec advisory feed, not the Rust blog, so a `cargo audit` or `cargo deny` run in CI is the mechanism by which you hear about a removal [95]. RustSec is "Maintained by the Rust Secure Code Working Group", exports "to Open Source Vulnerabilities in real time", and "The Github Advisory Database imports our advisories" [98], so OSV-Scanner and Dependabot alerts see the same removals. The March 2026 `time-sync` advisory is what one looks like: category "malicious", available as OSV JSON, with the note that the crate had "no evidence of actual downloads" [20].

### 5.3 What the incidents say about publisher hygiene

The maintainers who wrote post-mortems converged on the same list.

TanStack, after the May 2026 compromise: "Removed all uses of pull_request_target from our CI"; "Removed all caches from GitHub Actions" and "Disabled the pnpm cache in our release pipeline"; "Pinned every action in the org to a commit SHA"; "Enforced non-SMS 2FA across npm and GitHub"; kept OIDC trusted publishing with short-lived credentials; added zizmor as a required check on workflow files; "Upgraded every repo to pnpm 11" for the cooldown default [99]. Their post-mortem's own gap statement is that "OIDC trusted-publisher binding has no per-publish review" [22], which is precisely what staged publishing added ten days later [77].

axios, after March 2026: "Complete wipe of all lead maintainer devices as well as resetting of all credentials", "Proper adoption of OIDC flow for publishing", an "Immutable release setup", and the admission that "Publishing directly from a personal account was a risk that could have been avoided" [21].

Nx, after August 2025: switched to "Trusted Publisher using OIDC authentication" and "manual 2FA for publishing", having found the root cause in a `pull_request_target` workflow with write permissions [11].

crates.io, in January 2026, made two of those lessons registry policy: `pull_request_target` and `workflow_run` cannot be trusted-publishing triggers, and a crate can refuse token publishes altogether [94]. GitHub made a third one a platform default in June 2026 by changing `actions/checkout` and restricting the Actions cache under untrusted triggers [83].

What none of these controls would have stopped is keyv, where the attacker held the GitHub account and simply committed to `main` [24]. The control for that case is human: phishing-resistant 2FA on the source-hosting account and branch protection that requires a second reviewer on `main`, which the ledger shows is the remaining hole.

## 6. Consumer-side gates: what each catches and what it is blind to

| Gate | Catches | Blind to |
|---|---|---|
| `cargo audit` on `Cargo.lock` (and on binaries built with `cargo auditable`) [54] | Crates with a RustSec advisory, including malicious-crate removals [98][95] | Anything not yet in RustSec; novel malicious code |
| `cargo deny check` [100] | `advisories`: "crates with security vulnerabilities, or that have been marked as `Unmaintained`, or which have been yanked"; `bans`: "specific crates in your graph, as well as duplicates"; `licenses`; `sources`: "the source location for each crate", where `allow-registry` defaults to crates.io and `unknown-registry`/`unknown-git` default to `warn` [100][101] | Code content; only policy and known advisories |
| `cargo vet` [102] | Whether every dependency version has an audit for a criterion, from you or an imported trusted set; `safe-to-run` means "can be compiled, run, and tested on a local workstation or in controlled automation without surprising consequences", `safe-to-deploy` means "will not introduce a serious security vulnerability to production software exposed to untrusted input" and "implies safe-to-run" [103]; importable audit sets include actix, ariel-os, bytecode-alliance, embark-studios, fermyon, google, isrg, mozilla and zcash [104] | Anything covered only by an exemption; imports are "not transitive" [105], and exemptions must be "ratcheted down over time" [102] |
| `cargo crev` [106] | Cryptographically signed code reviews from a web of trust; "a work in progress" | Crates nobody in your trust graph has reviewed |
| `cargo supply-chain` [107] | "author, contributor and publisher data on crates in your dependency graph": how many people can publish into your tree | Nothing about code; it is a blast-radius inventory |
| `npm audit` [108] | Known advisories in the resolved tree; `--audit-level` sets "The minimum level of vulnerability for `npm audit` to exit with a non-zero exit code" | Exploitability in context; Abramov's 2021 critique still stands: it reports build-time transitive issues and "I need a way to mark for my users that a certain vulnerability can't possibly affect them" [109] |
| `npm audit signatures` [108] | That tarballs carry valid registry signatures and, where present, provenance attestations [89] | That the attested build was of benign source (keyv, section 1) |
| OSV-Scanner [110] | Advisories for `Cargo.lock`, `bun.lock`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock` without resolving | Anything absent from OSV |
| GitHub dependency review action [111] | "scans your pull requests for dependency changes, and will raise an error if any vulnerabilities or invalid licenses are being introduced"; `fail-on-severity`, `deny-licenses`, `allow-ghsas`, `deny-packages` | Vulnerabilities already present before the PR |
| OpenSSF Scorecard [112] | Upstream hygiene: Pinned-Dependencies ("explicitly set to a specific hash"), Token-Permissions, Dangerous-Workflow, Signed-Releases, Maintained, Branch-Protection, Code-Review | Whether a given release is malicious |
| deps.dev [113] | Cross-ecosystem graph and OSV data for "the Cargo, Go, Maven, npm, NuGet, PyPI, and RubyGems package ecosystems" via HTTP and gRPC | Same as above |
| Bun security scanner API [6] | "Bun will scan all packages before installation, display security warnings, and cancel installation if critical advisories are found" | Bun-only |

Provenance levels, so the words mean something. SLSA Build L1: "Provenance exists"; L2: "Build platform runs on dedicated infrastructure ... provenance is tied to that infrastructure through a digital signature"; L3 adds that the platform must "prevent runs from influencing one another, even within the same project" and "prevent secret material used to sign the provenance from being accessible to the user-defined build steps" [114]. GitHub's artifact attestations "by itself provides SLSA v1.0 Build Level 2", reusable workflows can "meet SLSA v1.0 Build Level 3", and "Artifact attestations are *not* a guarantee that an artifact is secure" [115]. Under the hood Sigstore's Fulcio "issues a short-lived certificate bound to the provided identity and public key" and Rekor is "an immutable, append-only ledger" [116]. TanStack's OIDC-token extraction from runner memory is an L3 failure of exactly the "secret material accessible to user-defined build steps" kind [22].

## 7. Publisher-side hygiene

The controls, each tied to the incident it answers.

- **Publish only through Trusted Publishing, and disallow tokens.** npm: set the package to "Require two-factor authentication and disallow tokens" [88] and route CI through OIDC [78]; crates.io: enable "Trusted Publishing only" so "traditional API token-based publishing is disabled" [94]. Answers chalk, Shai-Hulud, the `atool` wave [12][14][23].
- **Stage every npm release and approve it with 2FA.** `npm stage publish` from CI, human promotion with a 2FA challenge [77]; give CI a stage-only token if it cannot use OIDC [87]. Answers keyv and TanStack, where the CI publish itself was the attack [24][22].
- **Phishing-resistant 2FA on the account that matters.** npm no longer allows new TOTP setups [79] and does not support SMS [96]; for crates.io the account that matters is your GitHub account, because the registry enforces nothing itself [96]. TanStack "Enforced non-SMS 2FA across npm and GitHub" [99].
- **Never use `pull_request_target` or `workflow_run` with a checkout of untrusted code.** GitHub's own guidance says these "expose the repository to security compromises" [117]; Nx and TanStack are the case studies [11][22]; crates.io blocks both triggers from Trusted Publishing [94].
- **Pin actions to full commit SHAs and let the org enforce it.** "Pinning an action to a full-length commit SHA is currently the only way to use an action as an immutable release" [117]; the August 2025 org policy means "any workflow that attempts to use an action that isn't pinned will fail" [118]. Answers tj-actions, where tags were repointed [9]. For your own releases, immutable releases mean "Tags for new immutable releases are protected and can't be deleted or moved" and "receive signed attestations" [119].
- **Least privilege in the workflow.** "It's good security practice to set the default permission for the `GITHUB_TOKEN` to read access only for repository contents" [117]; `actions/checkout` `persist-credentials` defaults to `true` and should be `false` unless a later step pushes [120]. Nx's root cause included "Workflow permissions set to read/write" [11].
- **No caches across trust boundaries in the release path.** TanStack "Removed all caches from GitHub Actions" after cache poisoning bridged a fork PR into the release job [99][22]; GitHub made the cache read-only for untrusted triggers in June 2026 [83].
- **Lint the workflows.** TanStack added zizmor as a required check [99]; Scorecard's Dangerous-Workflow and Token-Permissions checks cover the same ground for public repos [112].
- **Short-lived, scoped tokens where a token is unavoidable.** npm granular tokens default to seven days with a 90-day maximum [79]; crates.io tokens carry endpoint and crate scopes [90] and expire after 90 days by default [91].

## 8. Runtime and least privilege: treat install and build as untrusted code

CERT's guidance after Shai-Hulud is the baseline: "Use `npm install --ignore-scripts` where feasible", "Set up an internal npm registry ... and centrally approve packages", and "isolate build environments" [15]. The ledger explains why isolation is not optional: install and build scripts run as your user, with your environment, and the 2026 payloads read cloud metadata endpoints, Kubernetes tokens, Vault, `~/.npmrc`, SSH keys and password-manager vaults [22][23][24]. An install step that has a cloud credential in its environment is an install step that donates it.

Node's permission model is now stable and useful for code you run, not for code you install: "v23.5.0, v22.13.0 - This feature is no longer experimental"; the flags are `--permission` with `--allow-fs-read`, `--allow-fs-write`, `--allow-child-process`, `--allow-worker`, `--allow-addons`, `--allow-wasi`, `--allow-ffi`, `--allow-net`, `--allow-openssl-store` and `--permission-audit` [7]. Its own caveats bound what it is for: "It does not provide security guarantees in the presence of malicious code"; "Using existing file descriptors via the `node:fs` module bypasses the Permission Model"; "The model does not inherit to a worker thread" [7]. Deno's default is the inverse: "Deno is secure by default. Unless you specifically enable it, a program run with Deno has no access to sensitive APIs", though `--allow-run` and `--allow-ffi` are treated "as equivalent to `--allow-all`" [121]. Neither sandboxes `npm install`; for that you need an OS boundary (a container or VM with no secrets mounted, or the rootless namespace sandboxes queued in the research backlog).

Rust has one more edit-time surface. Because rust-analyzer runs build scripts and proc-macros on open [52], an untrusted checkout should be opened with `rust-analyzer.cargo.buildScripts.enable` and `rust-analyzer.procMacro.enable` set to `false` [53], or inside a container. The same applies to any editor extension that indexes by building.

Finally, the new persistence targets. The keyv worm's `.claude/settings.json` `SessionStart` hook and `.vscode/tasks.json` `folderOpen` task [25], and the May wave's Claude Code, VS Code and Codex targeting plus systemd and LaunchAgent persistence [23], mean a post-incident sweep must include agent and editor configuration in every branch, not only `node_modules` and the lockfile. SafeDep's checklist is the concrete one: rotate everything reachable from the build environment, review OIDC token-exchange logs, and look for injected `.claude/settings.json`, `.vscode/tasks.json`, systemd units and LaunchAgents [23].

## 9. Configuration that implements the above

Version numbers in the examples are illustrative. Verification: the JSON, YAML and TOML blocks parse; `cargo build --locked --all-targets` and `cargo audit` ran clean on a scratch crate with cargo 1.93.0 and cargo-audit 0.22.2; `deny.toml` parses as TOML but was not executed because cargo-deny was not installed; `cargo vet`, pnpm 11 and npm 12's `approve-scripts` were likewise unavailable, so those lines are illustrative, and nothing was run against a live registry.

**`.github/dependabot.yml`: seven-day cooldown for both ecosystems, patch releases after three** (keys per [66]; security updates ignore the cooldown [68]):

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    cooldown:
      default-days: 7
      semver-patch-days: 3
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
    cooldown:
      default-days: 7
      exclude:
        - "@our-org/*"
```

**`package.json` on npm 12: pinned script approvals** (shape per [40]; write it with `npm approve-scripts esbuild sharp`, not by hand):

```json
{
  "name": "example-app",
  "private": true,
  "allowScripts": {
    "esbuild@0.27.3": true,
    "sharp@0.34.4": true,
    "left-pad-telemetry": false
  }
}
```

**`pnpm-workspace.yaml` on pnpm 11: seven-day cooldown, strict builds, no exotic subdeps** (keys per [3][44][45]):

```yaml
minimumReleaseAge: 10080
minimumReleaseAgeExclude:
  - "@our-org/*"
strictDepBuilds: true
blockExoticSubdeps: true
allowBuilds:
  esbuild: true
  sharp: true
```

**`deny.toml`: crates.io only, no unknown git, yanked and advisories fail the build** (check names and `sources` keys per [100][101]; illustrative):

```toml
[advisories]
version = 2
yanked = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]

[licenses]
version = 2
allow = ["MIT", "Apache-2.0"]
```

**Rust CI job** (flags per [57]; `cargo vet` exits non-zero when a dependency lacks an audit or exemption [102]; the first and third lines were run locally, the `cargo deny` and `cargo vet` lines are illustrative):

```yaml
- run: cargo build --locked --all-targets
- run: cargo deny check advisories bans sources licenses
- run: cargo audit
- run: cargo vet
```

**npm release workflow: OIDC trusted publishing, staged, no persisted credentials, actions pinned** (per [78][77][117][120]; the SHAs shown are placeholders to replace with the ones you verified):

```yaml
name: release
on:
  push:
    tags: ["v*"]
permissions:
  contents: read
  id-token: write
jobs:
  stage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@08c6903cd8c0fde910a37f88322edcfb5dd907a8 # v5.0.0, pin to a full SHA
        with:
          persist-credentials: false
      - uses: actions/setup-node@a0853c24544627f65ddf259abe73b1d18a591444 # v5.0.0, pin to a full SHA
        with:
          node-version: 24
          registry-url: https://registry.npmjs.org
      - run: npm ci --ignore-scripts
      - run: npm run build
      - run: npm stage publish
```

A maintainer then approves the staged version on npmjs.com with a 2FA challenge [77]. On npm 12 replace `--ignore-scripts` with a committed `allowScripts` policy so the build's own native dependencies still compile [2].

## 10. Common misconceptions

- *"The lockfile protects me."* It pins what you already resolved [57][62]; a compromised version you pinned to, or a `resolved` URL someone edited in a PR [61], is still yours. Pair it with `--locked`/`npm ci`, a cooldown, and lockfile linting.
- *"Provenance means the package is safe."* It means a specific workflow built it. keyv had valid provenance [24][25]; GitHub's own docs say attestations "are *not* a guarantee that an artifact is secure" [115].
- *"Trusted publishing removes the human from the loop."* It removes the long-lived token. TanStack's OIDC token was stolen from the runner [22]; npm added staged publishing precisely to put a human 2FA step back [77].
- *"npm 12 blocks install scripts, so I am done."* It blocks dependency scripts unless approved [2]; approvals pinned by name rather than version re-open the door on the next release [40], and git or remote sources needed their own flip [2][23].
- *"Cargo has the same protection."* It has none: `build.rs` and proc-macros run unsandboxed at build time [48][49], and the sandboxing goal was not continued [50]. rust-analyzer runs them on open [52].
- *"crates.io requires 2FA."* It does not; identity is delegated to GitHub and nothing is enforced at the registry [96]. Enforce it on the GitHub account and turn on Trusted-Publishing-only per crate [94].
- *"A one-day cooldown stops supply-chain attacks."* It stops the install-time worms whose detection took hours [10][64]; it did nothing for `faster_log`'s four months [17], and adversaries can wait [64][73].
- *"`npm audit` failing means I am vulnerable."* It means a known advisory is somewhere in the tree, exploitable or not [109]; use `--audit-level` and `--omit=dev` deliberately rather than muting it [108].
- *"The malicious crate would have been on the Rust blog."* Not since February 2026; the RustSec feed is the channel [95].

## Synthesis (inferred)

These are inferences drawn across the sources above; they are not themselves cited.

**The three-boundary model resolves the ecosystem differences.** At the publish boundary npm and crates.io have converged (OIDC trusted publishing, per-package refusal of tokens), except that npm has a human 2FA gate on release and registry-enforced 2FA on accounts, and crates.io has neither. At the resolve boundary the JavaScript tools have converged on cooldowns and exotic-source blocking, and Cargo has nothing native. At the execute boundary JavaScript now defaults to "no dependency scripts" across npm, pnpm, Yarn and Bun, and Cargo cannot. So a Rust project's security budget should go where Cargo is weakest: `cargo vet` with imported audits for the execute boundary, Dependabot's cooldown for the resolve boundary, and Trusted-Publishing-only plus GitHub-side WebAuthn for the publish boundary.

**Defaults for a Node repo, in the order they pay off.** (1) pnpm 11 or npm 12 with a committed script allowlist and every entry pinned to a version. (2) A seven-day cooldown in the package manager and in Dependabot, with security updates exempt, because every 2026 worm was gone in hours and seven days also covers tj-actions-style windows. (3) `npm ci` or the pnpm frozen install in CI, with no cloud credentials in the install job's environment. (4) Publish through OIDC, staged, promoted by a human with 2FA, package set to disallow tokens. (5) Actions pinned to SHAs, `contents: read`, `persist-credentials: false`, no `pull_request_target`, no caches in the release job, zizmor in CI. (6) OSV-Scanner and dependency review on PRs; `npm audit` at a severity threshold you chose.

**Defaults for a Rust repo.** (1) `Cargo.lock` committed and `--locked` everywhere, including `cargo install` in CI. (2) `cargo deny` with `unknown-git = "deny"` and crates.io as the only registry. (3) `cargo vet` initialised with the mozilla, google and bytecode-alliance imports and the exemption list treated as debt with an owner. (4) `cargo audit` in CI as the RustSec channel, and `cargo auditable` for anything you ship. (5) Dependabot with `cooldown.default-days: 7` for `cargo`. (6) Trusted-Publishing-only on every crate you own and WebAuthn on the GitHub account behind it. (7) A `.vscode`/editor policy that untrusted checkouts are opened with build scripts and proc-macros disabled, or inside a container.

**Where the free-rider objection actually bites.** Paterson is right that a universal cooldown would remove the population that discovers attacks by getting hit. The 2026 ledger shows that population is already mostly not installers: all but one 2026 detection came from a scanner vendor or a researcher within hours of publish, and npm's publish-time scan now runs before availability. The honest position is that cooldowns are a client-side stopgap for a registry-side queue that does not yet exist at a useful length, and that the queue is the right end state. Until a registry holds new versions for days with scanning in between, a cooldown is the correct default for a team that is not itself a scanner vendor.

**Only two gates read code.** Every gate in section 6 matches known-bad against your lockfile or checks process hygiene; only `cargo vet` and `cargo crev` are about a human having read the source, and only they can cover a runtime-only payload like `faster_log` before it runs. That is why vet sits third in the Rust defaults above, ahead of anything that merely scans.

**Provenance is an audit tool, not a gate.** Its value is retrospective: after keyv, provenance told responders exactly which workflow and commit produced each poisoned tarball. Treat `npm audit signatures` and SLSA levels as making incident response cheap, and treat staged publishing, cooldowns and script allowlists as the controls that prevent the install.

**The next attack surface is already in the ledger.** Agent hooks (`.claude/settings.json`, `.vscode/tasks.json`) turned a dependency compromise into a persistent foothold that survives `rm -rf node_modules`. Post-incident checklists, repo scanners and branch protection rules should treat those paths the way they treat `.github/workflows`: reviewed, owned, and diffed on every PR. The queued backlog entries on rootless sandboxes for coding agents and on secrets handling for headless agents are the natural continuation of this doc.

**What would change this doc.** Cargo shipping any consumer-side control over dependency build scripts; crates.io enforcing 2FA or adding provenance after RFC 3946 lands; npm removing bypass-2FA tokens for publishing outright; a registry-side hold measured in days. Re-check those four before treating the defaults above as current.
