---
title: "cupcake: Textual TUI + Cloudflare service that downloads, converts and tags media (Repo Profile)"
category: repos
languages: [python, typescript, markdown, shell, vue]
complexity: intermediate
use_cases:
  - "understanding what cupcake does and where its code lives"
  - "finding cupcake's CLI, TUI, worker-API and web-BFF surfaces before extending them"
  - "answering 'which repo downloads media from a URL and manages the resulting library'"
  - "resuming work on cupcake in a fresh session"
summary: "cupcake is a personal media-acquisition system with three faces over one Python core: a Textual terminal UI (apps/cli, package `cupcake`) with Download / Library / Search / Playlists / Settings tabs; a Cloudflare Workers API (infra/cloudflare/api) that queues fetch jobs onto a Durable-Object container running the same Python pipeline and stores results in R2; and an Astro 5 SSR backend-for-frontend (apps/web, deployed as its own Cloudflare container) that gives the same thing a browser and a Slack slash command. The domain is video/audio downloading: paste or search a URL, an engine (yt-dlp by default, youtube-dl or any binary speaking the JSONL Cupcake Engine Protocol as alternatives) fetches it, ffmpeg optionally converts to mp3, eyeD3 writes ID3 tags, and the file lands in a scanned local library or an R2 bucket with m3u playlists and signed streaming URLs. It is an mx monorepo (Makefile + make/*.mk + docker/), agent-built against 25 design specs under docs/superpowers and tracked in beads, with 305 commits and 308 pytest tests; both public endpoints were live at the time of profiling."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/nyvorin/cupcake
local_path: ~/dev/dev916/cupcake
status: active
visibility: private
owner: Personal (nyvorin)
sources:
  - README.md, apps/web/README.md, infra/cloudflare/README.md (target repo)
  - CLAUDE.md, AGENTS.md, .claude/settings.json, .beads/issues.jsonl (target repo)
  - apps/cli/setup.py, apps/cli/cupcake/main.py (target repo)
  - apps/cli/cupcake/ui/{App,LibraryTab,PlaylistsTab}.py (target repo)
  - apps/cli/cupcake/engines/{protocol,registry,config,preflight}.py (target repo)
  - apps/cli/cupcake/threads/DownloadThread.py (target repo)
  - apps/cli/cupcake/{jobs,library,settings,playlist,search,convert}.py (target repo)
  - apps/cli/cupcake/{net_guard,egress_proxy,engine_update,challenge_resolver}.py (target repo)
  - infra/cloudflare/api/worker/src/{index,jobs}.ts, slack/router.ts, wrangler.toml (target repo)
  - infra/cloudflare/api/container/server.py, infra/cloudflare/api/Dockerfile.cloud (target repo)
  - infra/cloudflare/web/worker/src/index.ts, wrangler.toml (target repo)
  - apps/web/src/lib/worker.ts, src/pages/api/login.ts, astro.config.mjs, package.json (target repo)
  - Makefile, make/{cli,cloudflare,common,build,release}.mk (target repo)
  - docker/compose/web.yml, web.dev.yml, docker/.config/.env.* (target repo)
  - scripts/test.sh, scripts/dev.sh, .github/workflows/ci.yml (target repo)
  - docs/superpowers/specs/2026-06-22-mx-monorepo-migration-design.md (target repo)
  - docs/development/{mx-app-playbook,mx-cloudflare-deploy,appendix-astro-scaffold}.md (mech-crate)
---

# cupcake

> **"🧁 make life sweet 🧁."** cupcake is a personal media downloader that grew
> into a three-surface system around one Python pipeline: **download → convert →
> tag**. You give it a URL — typed into a terminal UI, searched from YouTube,
> POSTed to an HTTP API, or typed into Slack — a pluggable engine (yt-dlp by
> default) fetches the media, ffmpeg optionally converts video to 320k mp3,
> eyeD3 writes ID3 tags, and the result lands either in a local directory the app
> can browse, filter, play and build `.m3u` playlists from, or in a Cloudflare R2
> bucket streamed back over Range requests and signed, expiring URLs VLC can
> open. The repository is an mx monorepo — `apps/cli` (the Textual TUI),
> `apps/web` (an Astro 5 SSR backend-for-frontend) and `infra/cloudflare` (the
> worker + container API) — built almost entirely by agents against 25 design
> specs, tracked in beads, and actively pushed to as of 2026-09-01.

## Identity
| Field | Value |
|---|---|
| Repository | `nyvorin/cupcake` (private) — default branch `main`. The local remote still reads `git@github.com:web-mech/cupcake.git`; GitHub redirects `web-mech/*` to `nyvorin/*`, and `gh api repos/web-mech/cupcake` returns `full_name: nyvorin/cupcake`. Both names are live for the same repository. |
| Local path | `~/dev/dev916/cupcake` (directory name matches the repo name). A second, stale clone of the *same* repository sits at `~/pgm/cupcake` — see Relationships. |
| Owner / org | Personal (nyvorin) — no hq project registered for it (checked against the hq project list) |
| Status | active — last commit 2026-09-01, 305 commits, profiled at `94f9c10`; GitHub `pushed_at` 2026-09-01 |
| Languages (by file count) | 312 tracked files: Python 133 · Markdown 63 · TypeScript 24 · Shell 20 · Make 14 · JSON 11 · Vue 4 · Textual CSS 4 · rest config. Real source is ~4,800 lines of Python under `apps/cli/cupcake` plus ~5,200 lines of tests; a naive `find -name '*.py'` reports ~6,200 files because it counts the untracked `venv/` |
| Build system | GNU Make (root `Makefile` auto-globbing `make/*.mk`, mx convention) · setuptools (`apps/cli/setup.py`) · npm (`apps/web`, both workers) · wrangler · Docker |
| Runtime deps | Python 3.12, ffmpeg (auto-installed via `ffmpeg-downloader`), yt-dlp (or youtube-dl), a terminal for Textual; the cloud path adds Docker, Node ≥ 22.12, wrangler and Cloudflare KV + R2 + Queues + Containers/Durable Objects |
| License | none declared — no `LICENSE` file, and the GitHub API reports `license: null` |
| CI / release | `.github/workflows/ci.yml`: on push to `main` and on every PR, install `./apps/cli[dev]` on Python 3.12 and run `pytest apps/cli/tests`. No tags, no releases, no lint/type job. Deploy is manual via `make cf-deploy` / `make cf-deploy-web` |

## What It Does

The problem it solves is mundane and personal: getting media off the internet,
into a normalised audio file with correct tags, and into a library you can search
later — repeatedly, from whichever device you are at, without breaking every time
a site changes its bot defences.

The core is a three-stage pipeline in `apps/cli/cupcake`: `run_download_chain`
walks the configured engine priority list until one engine succeeds
(`threads/DownloadThread.py`), `convert.py` optionally re-encodes video to
320 kbps mp3 through ffmpeg, and `threads/ID3TagThread.py` writes tags. Every
other surface in the repository is a different way to drive that same pipeline:

- **Terminal.** `cupcake` (or `make cli-run`) opens a Textual app with five tabs
  — Download, Library, Search, Playlists, Settings (`ui/App.py`). Media lands in
  `CUPCAKE_DOWNLOAD_DIRECTORY` (default `~/Music`) with a JSON metadata sidecar
  per video id under `<dir>/.cupcake/`.
- **HTTP.** `POST /cupcake/fetch?clip=<url>` enqueues a job; a queue consumer
  boots a Durable-Object container running the *same* Python package, streams
  the finished file into R2, and records status in KV
  (`infra/cloudflare/api/worker/src/index.ts`, `api/container/server.py`).
- **Browser and Slack.** An Astro SSR app proxies the API behind a password and
  a signed session cookie so the browser never sees the API token
  (`apps/web/src/lib/worker.ts`), and a signature-verified `/cupcake` Slack
  slash command routes to the same job path (`worker/src/slack/router.ts`).

"Done" is: the file exists, it is tagged, and it is playable — from the Library
tab, a browser `<audio>`/`<video>` element, or VLC opening a signed `.m3u`. Both
deployed endpoints answered on 2026-09-02 (`api.cup-cake.io/cupcake/health` →
`200 {"ok":true}`, `cup-cake.io/` → `200`).

## Capabilities

### CLI
- `cupcake` / `cupcake bake` — launch the Textual TUI (default subcommand) (`apps/cli/cupcake/main.py`)
- `cupcake update-engines [--force]` — opt-in pip update of the builtin engines, PyPI-checked and PEP440-normalised so padding-only version differences are not treated as an update; pins the exact target and verifies it landed (`apps/cli/cupcake/main.py`, `apps/cli/cupcake/engine_update.py`)
- `cupcake rollback-engines` — restore each builtin engine's last-known-good version, stored under an `engine_rollback` key in the settings file (`apps/cli/cupcake/engine_update.py`)

### TUI (Textual)
- **Download tab** — URL input with live `validators.url` checking, a 🎥/💿 video-vs-audio switch, per-item progress, and a "⟳ Resume interrupted" button bound to `r` (`ui/App.py`, `ui/DownloadItem.py`)
- **Library tab** — scans the download directory for known media extensions, joins each file to its `.cupcake/<id>.json` sidecar for title/channel/duration, and re-scans on tab activation and after each finished download; keys `F5` refresh, `/` focus filter, `esc` clear filter, `p` play, `r` reveal, `c` convert-all-to-audio, `n` new playlist (`ui/LibraryTab.py`, `library.py`)
- **Search tab** — YouTube search with no API key via `yt-dlp ytsearch` (capped at 25 results), each row downloadable through the normal pipeline with a rolled-up progress bar (`ui/SearchTab.py`, `search.py`)
- **Playlists tab** — `.m3u` files under `<download_dir>/playlists/`; keys `p` play, `v` view, `e` edit, `r` rename, `d` delete, `F5` refresh (`ui/PlaylistsTab.py`, `playlist.py`)
- **Settings tab** — download directory, player path, and a "delete source video after audio conversion" toggle persisted to a TOML settings file, plus an Engines section driving update/rollback (`ui/SettingsTab.py`, `settings.py`)
- **Log pane** — an in-app viewer over the rotating file log, with `l` revealing it in the OS file manager (`ui/LogViewer.py`, `ui/CupcakeLogHandler.py`, `logsetup.py`)
- Pause/resume of a running download, and a durable per-job record under `<download_dir>/.cupcake/jobs/<id>.json` that is deleted on success — so any surviving record means an interrupted job that "Resume interrupted" can re-drive from its recorded stage (`jobs.py`)

### Engine framework (Cupcake Engine Protocol v1)
- A JSONL-over-stdio protocol: the host writes one `download` request line, the engine emits `capabilities` / `metadata` / `progress` / `log` / `completed` / `error` / `entries` events; anything malformed raises `ProtocolError` (`engines/protocol.py`)
- Two adapters — `stdio` for protocol-native engines, `cli` for wrapping an arbitrary command with a progress regex, an optional metadata command and three output-discovery strategies (`newest_file`, `parse_output`, `glob`) — plus three builtin shims run one fresh subprocess each: yt-dlp, youtube-dl, and a deterministic offline `mock` (`engines/adapters/{stdio,cli}.py`, `engines/shims/`)
- Priority-ordered probing with capability degradation and fallthrough: options the engine cannot honour (proxy, cookies, headers) are dropped with a warning, a retryable failure falls through to the next engine, and playlist URLs expand into one download per entry when the engine reports the capability (`engines/registry.py`, `threads/DownloadThread.py`)
- Optional pre-download cookie harvesting for challenge-protected sites, gated behind two env flags: a plain Playwright cookie grab, or an OpenAI Vision-driven challenge resolver covering Cloudflare / reCAPTCHA / hCaptcha / DataDome / PerimeterX (`engines/preflight.py`, `challenge_resolver.py`)

### HTTP API (Cloudflare worker, `cupcake-cloud`)
- `GET /cupcake/health` — the only unauthenticated route (`worker/src/index.ts`)
- `POST /cupcake/fetch?clip=&mode=` — enqueue a job behind a fixed-window rate limit of 20 per minute, returns `202 {id}` (`worker/src/index.ts`, `worker/src/jobs.ts`)
- `GET /cupcake/file?id=` — R2 stream with single-byte-range parsing and 206 responses; served inline with `X-Content-Type-Options: nosniff`, or as an attachment with `?download=1` (`worker/src/index.ts`)
- `GET /cupcake/check/:id` — job record from KV, or 404; `GET /cupcake/jobs` — the 100 most recent job summaries, skipping `rl:` rate-limit keys (`worker/src/index.ts`)
- `GET /cupcake/playlist.m3u` — an `#EXTM3U` of every `done` job, each line a signed `id`+`exp`+`sig` URL with a 6-hour TTL that `/cupcake/file` accepts *instead of* the header token (`worker/src/index.ts`)
- `POST /cupcake/slack` — Slack slash command, signature-verified, with a handler registry (`download`, `status`, `search`, `help`); a bare URL routes to download, anything unknown to help (`worker/src/slack/router.ts`, `worker/src/slack/verify.ts`)
- `GET /cupcake/search?q=&n=` — proxied with a 60 s timeout to the container-internal API, which is reachable only with a shared internal token: `POST /internal/process-job`, `GET /internal/file`, `GET /internal/search`, `GET /internal/health` (`worker/src/index.ts`, `api/container/server.py`)

### Web UI (Astro 5 SSR BFF, `cupcake-web`)
- Password login compared in constant time against a server-only value, then an HMAC-signed httpOnly session cookie (~30 d) gating every `/api/*` route (`apps/web/src/pages/api/login.ts`, `src/lib/{session,worker}.ts`)
- Same-origin BFF endpoints — `login`, `logout`, `me`, `fetch`, `jobs`, `check/[id]`, `file/[id]`, `search`, `playlist-m3u`, `health` — each proxying to the worker with a server-held token the browser never receives (`apps/web/src/pages/api/`)
- `/` download view and `/library` grid with inline `<audio>`/`<video>` players (`file/[id]` passes `Range` through so playback can seek), built as Vue 3 islands hydrated via `client:*`, Tailwind v4 through its Vite plugin, Node adapter in standalone mode (`apps/web/src/components/*.vue`, `astro.config.mjs`)

### Background jobs
- Cloudflare Queue consumer with `max_batch_size = 1`, `max_retries = 3` and a 110 s per-job timeout; failures are recorded as `status: error` in KV then acked deliberately, to avoid infinite redelivery in local dev. Container instances sleep after 10 minutes idle — job containers are keyed per job, search shares one (`worker/src/index.ts`, `worker/wrangler.toml`)

### Not (yet) implemented
- The generic mx service targets (`make dev|up|down|build|test|logs|restart|start|stop|sh|exec|run`) are scaffolding no cupcake service uses, and `infra/cloudflare/README.md`'s `make cf-setup` / `make cf-init` do not exist as targets at all — the backing `scripts/cf-setup.sh` and `scripts/cf-init-app.sh` are present but unwired (see State, Gaps and Drift).
- Open roadmap items carrying beads ids and no implementation: samba server (`cupcake-x4t`), IPFS remote storage (`cupcake-3e2`), a live-streaming "cupcake server" (`cupcake-960`), convert/tag progress output (`cupcake-7xh`) (`README.md`). No Postgres, Redis, Drizzle, Pinia or component library is used despite the recipe scaffolding shipping all of them — `apps/web/README.md` calls that out as deliberate.

## Architecture

**Stack.** Python 3.12 + Textual, click, yt-dlp, ffmpeg (via `ffmpeg-python` /
`ffmpeg-downloader`), eyeD3, `tomli_w`/`tomllib`, `packaging`, `validators`;
TypeScript on Cloudflare Workers with `@cloudflare/containers`; Astro 5 + Vue 3 +
Tailwind 4 on Node ≥ 22.12. Everything is glued by GNU Make.

**Component map.** `apps/cli/cupcake/` is the only place domain logic lives —
pipeline, engine framework, settings, library, playlists, jobs — with
`cupcake/ui/` a thin Textual renderer over it. `api/worker/` does routing, auth,
rate limiting, queue produce+consume, R2 streaming and Slack; `api/container/` is
a stdlib `ThreadingHTTPServer` importing that same `cupcake` package; `apps/web/`
is the Astro SSR session + same-origin proxy behind a 44-line pass-through worker
with a www→apex redirect. `Makefile`, `make/`, `scripts/` and `docker/` are mx
monorepo conventions inherited from `mx new`.

**Data flow.**

```
LOCAL  URL ─▶ Textual UI ─▶ run_download_chain ─▶ convert ─▶ ID3 tag ─▶ ~/Music
                             │ engine priority chain     └─▶ .cupcake/<id>.json sidecar
                             └───────────────────────────▶ .cupcake/jobs/<id>.json
                                                            (deleted on success)
CLOUD  browser ─▶ Astro BFF /api/* ─(X-Cupcake-Token)─┐
       Slack   ─▶ worker /cupcake/slack ──────────────┴─▶ /cupcake/fetch ─▶ KV status
                                                       Queue (batch 1) ─▶ Durable-Object
                                                       container (same Python pipeline,
                                                       behind the egress proxy) ─▶ R2
       R2 ─▶ /cupcake/file  200 | 206 Range · header token OR signed exp+sig URL ─▶ VLC
```

**Storage.** Four stores, no database anywhere: local media plus JSON metadata
sidecars and job records under the download directory's `.cupcake/`; settings
and engine config as TOML under the config directory; a rotating log file
(2 MB × 3 backups) beside them; and in the cloud, job records in Workers KV plus
media objects in R2 keyed `jobs/<job_id>/<file_name>`. **Integrations:** YouTube
(through yt-dlp only — no Google API key anywhere), PyPI (update checks),
Cloudflare (Workers, KV, R2, Queues, Containers/Durable Objects, custom
domains), Slack, and optionally OpenAI (the Vision challenge resolver).

**Process / concurrency model.** Every download runs as a fresh JSONL-speaking
subprocess, so a wedged engine cannot poison the app — the README credits this
rework with likely fixing the historical "stops downloading after a few days"
bug. In the TUI, Textual workers drive each `DownloadItem`, blocking work goes
off the event loop via `asyncio.to_thread`, and the engine registry is a
process-wide lazy singleton behind a lock, probed once per session. In the cloud
concurrency is the queue's: one message per batch, up to three containers.

**Security model.** Layered, and unusually thorough for a personal project. The
worker fails **closed** — with the shared token unset, every route but `/health`
is denied, and comparison is constant-time (`worker/src/crypto.ts`).
Worker→container calls carry a separate internal token compared with
`hmac.compare_digest`, and job ids pass a strict allowlist regex before touching
any filesystem path (`api/container/server.py`). SSRF defence is two-layer —
`assert_safe_url` on the initial URL plus a validating in-container egress proxy
covering every later hop (`net_guard.py`, `egress_proxy.py`; detailed under
Notable Techniques). Metadata is trimmed before storage because the raw yt-dlp
info dict embeds time-limited signed media URLs that must not be persisted
(`cloud_meta.py`), and R2 objects are served with `nosniff` plus a real media
content type so stored bytes cannot execute as HTML. Secrets are named, never
committed: worker values come from `wrangler secret put`, the BFF reads
server-only variables the browser never sees,
`docker/.config/.env.secrets.template` is key-only, and the Cloudflare API token
is read at make time from a key file outside the repo and never printed.

## Repository Layout

```
Makefile · make/            auto-globbed modules; only cli.mk + cloudflare.mk are real
scripts/                    shell helpers behind the generic (mx-scaffold) make targets
docker/                     compose/web{,.dev}.yml (Traefik, unfilled — see Drift) ·
                            dockerfiles/web/app{,.prod}, build, rebuild · .config/.env.*
apps/cli/setup.py           package "Cupcake", console script `cupcake` (entry point)
apps/cli/cupcake/
  main.py                   click group — bake / update-engines / rollback-engines (entry point)
  ui/App.py                 Textual App, the five tabs (entry point); dev.py = hot reload
  engines/                  protocol · registry · config · runner · preflight ·
                            adapters/{stdio,cli} · shims/{ytdlp,youtubedl,mock}
  threads/                  DownloadThread (run_download_chain) · ConvertThread · ID3TagThread
  {jobs,library,settings,playlist,search,convert}.py   pure, Textual-free modules
  {net_guard,egress_proxy,cloud_meta}.py               cloud safety + metadata trimming
  {engine_update,logsetup,challenge_resolver}.py · mock/  (dev-harness mock UI + threads)
apps/cli/tests/             308 pytest tests (~5,200 lines), incl. tests/engines/
apps/web/                   Astro 5 SSR BFF — astro.config.mjs (server, node standalone, vue, tailwind)
  src/pages/{index,library}.astro (entry points) · src/components/*.vue (Login Main Library JobCard)
  src/pages/api/            login logout me fetch jobs check/[id] file/[id] search playlist-m3u health
  src/lib/{worker,session}.ts   proxy + signed session cookie
infra/cloudflare/api/worker/src/index.ts   the API worker (entry point) + jobs, crypto, slack/
infra/cloudflare/api/container/server.py   container HTTP server (entry point);
                            Dockerfile.cloud = python:3.12-slim + ffmpeg + the cupcake package
infra/cloudflare/web/worker/src/index.ts   pass-through worker for the Astro container (entry point)
docs/superpowers/{specs,plans}/  25 design specs + 25 implementation plans
.beads/ · .github/workflows/ci.yml   issue database (+ 5 git hooks) · pytest on 3.12
```

## How It Was Built

**Toolchain.** Python 3.12 (CI-pinned), Node ≥ 22.12, Docker, wrangler, GNU
Make. There is no lockfile on the Python side — `setup.py` pins nothing but the
`yt-dlp[default]` extra, justified in-code as required for the EJS challenge
solver YouTube now needs.

**Build / run / test — as they really are.** Install with
`python -m venv venv && pip install -e ./apps/cli` (or `make cli-install`); run
the TUI with `make cli-run`, or `make cli-dev` for hot reload via `cupcake/dev.py`.
Tests are `python -m pytest apps/cli/tests`, which is what CI runs — `make test`
does **not** run them (see Drift). `make cf-dev` starts `wrangler dev` on port
8787 with the `mock` engine, `make cf-down` stops it, and `make cf-deploy` /
`make cf-deploy-web` deploy, each with a `-dry` variant running
`wrangler deploy --dry-run`. Nothing was executed against the repo for this
profile — the above is read from `make/{cli,cloudflare}.mk` and the CI workflow.

**Dev loop.** Spec → plan → beads issue → branch → PR. `docs/superpowers/` holds
25 matched, dated spec/plan pairs and the README annotates nearly every roadmap
item with its beads id; `.claude/settings.json` wires `bd prime` into both
`SessionStart` and `PreCompact` hooks so an agent session starts and re-hydrates
with the tracker loaded, and `.beads/` ships five git hooks.

**As an mx app.** The repo follows the mx monorepo shape described in
`docs/development/mx-app-playbook.md`: a root Makefile including `make/*.mk`,
`docker/compose/<service>.yml` plus a `.dev.yml` override, `docker/dockerfiles/`,
and `apps/<service>/` for recipe-scaffolded services. `docker/compose/web.yml`
declares the playbook's router contract — `traefik.enable=true`, a
`traefik.http.routers.web.rule` Host label, `loadbalancer.server.port=4321`,
`traefik.docker.network=devmesh-traefik`, and `devmesh-traefik` as an external
network — with the dev override exposing only Vite's HMR port rather than the
app port, exactly as the playbook prescribes. It is, however, **template output
that was never filled in** (see Drift), and in practice `apps/web` never runs
through the router: it is built into a Cloudflare container image and deployed,
so the mx compose path is dormant scaffolding, not the live dev loop.

**CI/CD and deploy path.** CI is tests only; deploy is a human running a make
target. Wrangler builds each container image from a repo-root build context (both
`wrangler.toml` files set `image_build_context` four levels up, with a comment
explaining why), pushes it to the Cloudflare registry, and deploys the worker —
each keeping `workers_dev = true` so the `*.workers.dev` hostname survives as a
fallback beside the custom domain.

**Configuration and env-var names (names and purpose only).**

| Group | Names and purpose |
|---|---|
| Paths | `CUPCAKE_DOWNLOAD_DIRECTORY` (media + Library scan root, default the user's Music folder) · `CUPCAKE_PLAYER` (player binary for the Library `p` key, else the OS opener) · `CUPCAKE_SETTINGS` / `CUPCAKE_CONFIG` (settings and engines TOML locations) |
| Engine behaviour | `CUPCAKE_ENGINES` (priority override; `mock` in local cloud dev) · `CUPCAKE_DL_RETRIES` (default 3) · `CUPCAKE_PLAYLIST_LIMIT` (default 100) · `CUPCAKE_PROXY` · `CUPCAKE_YT_CLIENTS` · `CUPCAKE_USER_AGENTS` · `CUPCAKE_MOCK_SCENARIO` |
| Pipeline · challenge preflight | `CUPCAKE_CLEANUP_SOURCE` (delete source video after audio conversion) · `CUPCAKE_USE_PLAYWRIGHT` · `CUPCAKE_USE_CHALLENGE_RESOLVER` · `OPENAI_API_KEY` (consumed only by the resolver) |
| Container | `CUPCAKE_EGRESS_PROXY_PORT` · `CUPCAKE_INTERNAL_TOKEN` · `PORT` |
| Worker secrets | `CUPCAKE_TOKEN` · `INTERNAL_TOKEN` · `STREAM_SECRET` · `SLACK_SIGNING_SECRET` (all via `wrangler secret put`) |
| BFF (server-only) | `CUPCAKE_API_BASE` · `APP_PASSWORD` · `SESSION_SECRET` |

**Provenance.** 305 commits in two eras. 2024-03 through 2025-05 is 27 commits
of hand-written groundwork (Textual shell, youtube-dl, ffmpeg, ID3). Then a
near-total rewrite: 150 commits in 2026-06 and 105 in 2026-07 built the engine
framework, the Cloudflare service, the web UI, the Slack command, the SSRF
hardening, the tabs, the playlists, the durable logs and the safe engine
updates — each with a dated spec and plan in `docs/superpowers/`, the signature
of agent-run spec→plan→execute work. 2026-08 is nearly silent; 2026-09 resumes
with the library live-refresh feature. Authorship is `web-mech` (314 commits
across all branches) plus `Nyvorin` and `Michael Price` — one person, three
identities.

## Relationships

**Two local checkouts, one repository.** `~/pgm/cupcake` is *not* a fork — it is
a stale clone of the same repository, on `main` at `84d21ef` (2025-05-15, 27
commits) with the same `web-mech/cupcake` remote; that commit is a verified
ancestor of the live `HEAD`, which is 278 commits ahead. The old checkout keeps
the pre-monorepo shape: `cupcake/` at the repo root, a `YoutubeDLInstaller`,
`youtube-dl-nightly` as the only engine, `make dev` / `make run` instead of
`cli-dev` / `cli-run`, and a README whose roadmap is the same list, entirely
unchecked — reading the two READMEs side by side is the clearest available diff
of what the 2026 rewrite delivered. **`~/dev/dev916/cupcake` is the live copy**;
`~/pgm/cupcake` is a snapshot to delete or re-fetch, never to edit.

**Migration into the mx monorepo.** Documented rather than inferred:
`docs/superpowers/specs/2026-06-22-mx-monorepo-migration-design.md` specifies
converting the flat repo into an mx monorepo "with zero behavior change",
explicitly mirroring `~/dev/dev916/unyform.ai`'s conventions — root Makefile
including `make/*.mk`, `docker/compose/` + `docker/dockerfiles/`,
`infra/cloudflare/<svc>/worker/`, and `apps/` for recipe-scaffolded services. The
package name was deliberately held constant (`import cupcake`) while its
directory moved to `apps/cli/cupcake`; Phase 2 then added `apps/web`.

**User data directories (shapes only; no contents were opened).** `~/cupcake` is
a legacy download directory holding one media file plus a `.cupcake/` sidecar
directory — the ancestor of today's `CUPCAKE_DOWNLOAD_DIRECTORY` convention.
`~/.config/cupcake/` is the live config location: a `settings.toml` and a
`logs/` directory, exactly matching `settings.py` and `logsetup.py`. The engines
TOML `engines/config.py` looks for alongside them is absent — expected, since it
is optional, and its absence means the default `yt-dlp → youtube-dl` priority is
in force.

- **Depends on (ours):** the mx toolchain for its monorepo shape and (nominally)
  its router — see `docs/development/repos/mech-crate.md`. It borrows
  `unyform.ai`'s `make/cloudflare.mk` layout and reads the Cloudflare API token
  from a key file inside that project's tree, coupling cupcake's deploy to a
  sibling repository's checkout being present.
- **Depends on (third-party):** yt-dlp, ffmpeg, Textual, click, eyeD3,
  Astro/Vue/Tailwind, Cloudflare Workers + KV + R2 + Queues + Containers, beads
  (`bd`), and optionally Playwright and OpenAI.
- **Shares patterns with:** `docs/development/mx-app-playbook.md` (Traefik host
  labels, dev override exposing only HMR), `mx-cloudflare-deploy.md` (worker +
  container deploy, custom domains), `appendix-astro-scaffold.md` (Astro SSR +
  Vue islands + Tailwind v4).
- **Built with:** the same spec→plan→beads→PR loop mech-crate's agent tooling
  uses (`docs/development/repos/devloop.md` covers the executor side).
  **Supersedes / superseded by:** nothing.

## Notable Techniques

- **A JSONL subprocess protocol as a plugin boundary.** The Cupcake Engine
  Protocol turns "which downloader" into a config line: any binary emitting the
  seven event types is a first-class engine, and anything with a progress regex
  can be wrapped by the `cli` adapter without writing code. The pay-off is
  isolation — a fresh process per download means a leaking or wedged engine dies
  with its job — and the cost is that every failure is a parse problem rather
  than an exception.
- **Capability degradation instead of feature gating.** Engines advertise
  capabilities; the host drops request options an engine cannot honour, logs the
  drop rather than refusing to run, and shows an explicit `indeterminate` state
  for engines with no progress capability.
- **Two-layer SSRF defence with an in-process forward proxy.** A fail-fast URL
  check plus a validating proxy that re-resolves and pins every hop, applied to
  the *subprocess's* traffic (yt-dlp and its ffmpeg child) via `http_proxy`
  variables. The code documents what it does not cover — rtmp/srt and other
  non-http protocols, because a kernel firewall needs `NET_ADMIN`, which
  Cloudflare Containers do not grant. Refusing to download at all when the proxy
  is down is the part most implementations skip.
- **Signed, expiring URLs as an alternative auth path for dumb clients.** VLC
  cannot send a custom header, so `/cupcake/file` accepts an HMAC over `id:exp`
  with a 6-hour TTL *instead of* the token — one endpoint, two credentials, no
  long-lived secret handed to the player.
- **Delete-on-success as the recovery signal.** A job record's *existence* means
  interruption, so recovery needs no status scan and no background watcher — and
  because records live under the download directory they travel with the media.
- **Version-aware update gating.** Normalising both sides through PEP440 kills
  the phantom-update bug class, and re-checking the installed version afterwards
  catches a pip that silently did something else.
- **Backlog candidates** (not filed here, per the profiling procedure): *validating
  egress proxies for untrusted subprocess downloads*, which generalises to any
  agent or job runner that shells out to a network tool; and *signed-URL side
  doors for header-less clients* — TTL choice, HMAC input, revocation.

## State, Gaps and Drift

**Maturity.** Genuinely healthy for a personal project: 308 tests over ~5,200
lines against ~4,800 lines of source, zero literal TODO/FIXME/HACK markers in
`apps/cli/cupcake`, `apps/web/src` or `infra/`, and CI that runs the suite on
every PR. The pure/impure split is disciplined — `library.py`, `playlist.py`,
`settings.py`, `jobs.py`, `search.py` and `convert.py` each carry an explicit
"Textual-free" or "Pure" docstring, with the tabs as thin renderers over them.
Beads is live, not decorative: 180 issues (136 closed, 36 open, 3 in progress, 5
unparseable), cross-referenced by the README's roadmap item by item. At profiling
time `.beads/issues.jsonl` was modified-but-uncommitted alongside several
untracked scratch files.

**mx scaffolding that never got wired up.** The largest drift in the repo, and
it is uniform. `docker/compose/web.yml` still carries the literal double-brace
`SERVICE_NAME` placeholder in its Traefik host rule and app URL, and includes a
`db.yml` and `redis.yml` that do not exist (both `optional: true`, so the
include silently no-ops). `scripts/test.sh` — behind `make test` — iterates
`docker/compose/*.yml` and runs `npm test` in each service container; cupcake's
tests are pytest in no container, so `make test` reports success while running
nothing, and CI is the only real test path. `make/release.mk` exposes 17
standard-version release targets against a repo with no tags and a `setup.py`
that has said `version="0.1.0"` since the first commit.
`infra/cloudflare/README.md` is generic mx text documenting `make cf-setup` /
`make cf-init` (neither exists) over an `infra/cloudflare/apps/<name>/` layout
that does not match the real `{api,web}/worker/` structure — that directory
holds only a `.gitkeep`. `docker/.config/.env.web` and `.env.secrets.template`
list Postgres, Redis, JWT and Sentry names no cupcake code reads.

**Documentation gaps.** The root README documents `CUPCAKE_USE_PLAYWRIGHT` but
not `CUPCAKE_USE_CHALLENGE_RESOLVER`, and never mentions that the resolver calls
OpenAI's Vision API with an `OPENAI_API_KEY` — a network-egress and cost surface
only `apps/cli/cupcake/CHALLENGE_RESOLVER_README.md` describes, and the one the
repo root `.env` actually enables locally. Five further variables the code reads
(`CUPCAKE_DL_RETRIES`, `CUPCAKE_YT_CLIENTS`, `CUPCAKE_USER_AGENTS`,
`CUPCAKE_MOCK_SCENARIO`, `CUPCAKE_EGRESS_PROXY_PORT`) are absent from the
README's env-var section.

**Other risks.** No license file on a repository whose function is downloading
third-party media — the legal posture is simply unstated. `make/cloudflare.mk`
reads the deploy token from a path inside a *different* project's checkout, so
moving or renaming that project makes every deploy target silently deploy with
an empty token. `/cupcake/jobs` and `/cupcake/playlist.m3u` each list up to
1,000 KV keys then issue one `KV.get` per key — fine at personal scale, first
thing to break as the library grows — and the rate limiter is one global
fixed-window counter, so 20 requests a minute from anywhere exhausts it for
everyone. The Slack path bypasses the `X-Cupcake-Token` gate by design (it has
its own signature check), correct but leaving two independent auth paths into
the same job queue. `main.py` still registers the scaffold-era `greet` command,
and the README's one known-unfixed bug — downloads stopping after a few days,
believed fixed by the subprocess-per-download rework — is still open for
verification as `cupcake-yv9`.

**Undetermined.** Whether `~/pgm/cupcake` is still referenced by anything (no
tooling in either checkout points at it), and whether the mx compose path for
`apps/web` ever ran before the Cloudflare deploy replaced it — no plan or spec
describes running it and the placeholder was never substituted, suggesting not.

### Synthesis (inferred)

cupcake reads as a hobby project that was handed to agents and came back an
order of magnitude larger without losing its shape. The tell is the ratio: 27
hand-written commits over fourteen months, then 255 in two, each traceable to a
dated spec and a beads id. What is striking is that quality went *up* — the 2026
code is better factored than the 2024 code, with a real protocol boundary, a
pure/impure split enforced by docstring convention, more test code than source,
and security work (two-layer SSRF containment, constant-time comparisons,
metadata trimming so signed URLs are never persisted) well beyond what a
personal downloader needs. Specs first, one feature per branch, and a suite that
grows with the code appear to be what made that possible.

The architecture's real idea is **one pipeline, many mouths**.
`run_download_chain` is called identically by a Textual worker, a container HTTP
handler and (transitively) a Slack command; the Dockerfile literally installs the
same `apps/cli` package into the cloud container, and everything else — worker,
BFF, Slack router — is transport. That is why three surfaces could be added in
two months without the core changing much.

The drift all points one way: every unfinished thing is mx *scaffolding* —
compose placeholders, a `make test` that tests nothing, seventeen release
targets, a Cloudflare README describing a different layout — and nothing
hand-written is half-done. The reading is that `mx new` output was merged
wholesale during the migration and only the two modules actually needed
(`cli.mk`, `cloudflare.mk`) were adapted, because the deploy target turned out
to be Cloudflare containers rather than the local Traefik router the scaffolding
assumes. The cheapest fix is deleting what is unused; the most valuable is
`scripts/test.sh`, because a green `make test` that runs nothing is worse than
no target at all.

## Quick Reference
| Task | Command / path |
|---|---|
| Install (editable) | `python -m venv venv && pip install -e ./apps/cli` (or `make cli-install`) |
| Run the TUI | `make cli-run` · `make cli-dev` for hot reload (`apps/cli/cupcake/dev.py`) |
| Tests (the real ones) | `python -m pytest apps/cli/tests` — **not** `make test` |
| Update / roll back engines | `cupcake update-engines [--force]` · `cupcake rollback-engines` |
| Local cloud stack | `make cf-dev` (worker on :8787, `mock` engine) · `make cf-down` |
| Deploy | `make cf-deploy[-dry]` (API) · `make cf-deploy-web[-dry]` (web) · `make cf-status` |
| Live endpoints | `https://api.cup-cake.io` · `https://cup-cake.io` (both 200 on 2026-09-02) |
| Downloads + metadata | `CUPCAKE_DOWNLOAD_DIRECTORY` (default `~/Music`), sidecars and job records in `<dir>/.cupcake/` |
| Settings / engines / logs | `~/.config/cupcake/settings.toml` · `engines.toml` · `logs/cupcake.log` |
| Resume interrupted work | press `r` (or the ⟳ button) on the Download tab |
| Issue tracker · design history | `bd ready` / `bd show <id>` · `docs/superpowers/{specs,plans}/` (25 pairs) |
| Stale historical clone | `~/pgm/cupcake` — same repo, 278 commits behind; do not edit |

## Sources

- `README.md` — monorepo layout, engine config format, engine updates, pause/resume, the Cloudflare and Slack sections, the tab-by-tab TUI description, the env-var list, the known bug, the beads-annotated roadmap. `apps/web/README.md` — BFF architecture, auth flow, server-only variable table, the "intentionally unused" list, search/library/VLC behaviour. `infra/cloudflare/README.md` — the generic mx text that does not match the real layout.
- `CLAUDE.md`, `AGENTS.md`, `.claude/settings.json`, `.beads/issues.jsonl` — beads workflow, session-completion protocol, the `bd prime` hooks, issue counts by status.
- `apps/cli/setup.py`, `cupcake/main.py` — package name, dependencies, extras, entry point, the click command surface. `cupcake/ui/{App,LibraryTab,PlaylistsTab}.py` — tab composition, key bindings, resume action, refresh triggers.
- `cupcake/engines/{protocol,registry,config,preflight}.py` — CEP event types, priority probing and builtin factories, the TOML schema, cookie-preflight gating. `cupcake/threads/DownloadThread.py` — the engine chain, capability degradation, retry/fallthrough, output template, playlist expansion.
- `cupcake/{jobs,library,settings,playlist,search,convert}.py` — the pure domain modules and their storage conventions. `cupcake/{net_guard,egress_proxy,cloud_meta}.py` — SSRF policy, the validating proxy, metadata trimming. `cupcake/{engine_update,challenge_resolver}.py` + `CHALLENGE_RESOLVER_README.md` — PEP440 update gating, rollback storage, the OpenAI/Playwright path.
- `infra/cloudflare/api/worker/src/{index,jobs}.ts`, `slack/router.ts`, `wrangler.toml` — route table, auth and signed-URL verification, queue consumer, rate limiter, Slack registry, bindings and custom domains. `api/container/server.py` + `Dockerfile.cloud` — the internal API, job-id allowlist, egress-proxy enforcement, and the image that installs the same Python package. `web/worker/src/index.ts`, `wrangler.toml` — pass-through worker, www→apex redirect, container build context.
- `apps/web/src/lib/worker.ts`, `src/pages/api/login.ts`, `astro.config.mjs`, `package.json` — token containment, constant-time password check, cookie settings, SSR adapter, dependency versions.
- `Makefile`, `make/{cli,cloudflare,common,build,release}.mk`, `scripts/{test,dev}.sh` — the real target surface, the scaffolded remainder, and what the generic targets actually do. `docker/compose/web{,.dev}.yml`, `docker/.config/.env.*`, `docker/dockerfiles/web/app.prod` — the Traefik/devmesh-traefik contract, the unsubstituted placeholder, env-var names.
- `.github/workflows/ci.yml` — the only automated check. `docs/superpowers/specs/2026-06-22-mx-monorepo-migration-design.md` — the migration goal, the unyform conventions mirrored, the target layout.
- Lineage established with `git -C ~/pgm/cupcake log`/`remote`, `git merge-base --is-ancestor` and `git rev-list --count`; repository metadata via `gh api repos/web-mech/cupcake`; liveness via `curl` against the two custom domains; directory shapes of `~/cupcake` and `~/.config/cupcake` via `ls` only (no personal data files were opened).
