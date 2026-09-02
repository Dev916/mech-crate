---
title: "meetnotes: terminal-native macOS meeting capture that records, transcribes locally, mines notes with Claude, and files them under the right ~/dev project (Repo Profile)"
category: repos
languages: [python, swift, markdown]
complexity: intermediate
use_cases:
  - "understanding what meetnotes does and where its code lives"
  - "finding meetnotes' CLI, MCP and skill surface before extending it"
  - "answering 'which repo records and transcribes our meetings and files the notes'"
  - "resuming work on meetnotes in a fresh session"
summary: "meetnotes is a macOS-only, stdlib-only Python CLI (~52 .py) plus a 3-file Swift/ScreenCaptureKit helper that records a meeting's microphone and system audio (optionally the screen), transcribes it with local whisper.cpp by default (Groq or WhisperX+pyannote optional), mines a fixed seven-section notes.md with headless `claude -p`, routes the finished session to a project key through a nine-rung resolution ladder, then auto-pushes the note and each **[YOU]** action item into the local hq command-center over JSON-RPC. It also runs the other way: hq's triage engine calls meetnotes' `mine` in text mode to turn a Slack thread into hq's MineResult JSON. Surfaces are an 18-subcommand CLI (`meetnotes`), a 9-tool MCP server, a Claude Code skill, a launchd mic-watch daemon that auto-detects meetings, and a menu-bar app. Active and spec-driven (5 design specs, 11 plans, beads), 109 commits since 2026-07-12 — but local `main` is 49 commits ahead of the private GitHub remote, which has not been pushed since 2026-07-27."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/Dev916/meetnotes
local_path: ~/dev/meetnotes
status: active
visibility: private
owner: PriceLove LLC (Dev916)
hq_project: meetnotes
sources:
  - README.md, SKILL.md, AGENTS.md, CLAUDE.md (target repo)
  - Makefile, pyproject.toml, meetnotes_mcp.py (target repo)
  - meetnotes/{cli,config,routing,transcribe,mine,hq_push}.py (target repo)
  - meetnotes/{supervisor,capture,spawn,procs,state,doctor}.py (target repo)
  - meetnotes/{watch,menubar,dashboard,gcal,screen_select,notify}.py (target repo)
  - bin/build.sh, bin/meetnotes-capture.swift, bin/MeetNotes.entitlements (target repo)
  - docs/superpowers/specs/2026-07-11-meetnotes-design.md (target repo)
  - .beads/issues.jsonl (target repo)
---

# meetnotes

> **Record the meeting, keep the audio on the Mac, hand the decisions back later.**
> meetnotes is a terminal-native meeting recorder for macOS. `meetnotes start`
> puts mic and system audio (and optionally the screen) into a background
> recording; `meetnotes stop` mixes them, transcribes locally with whisper.cpp,
> mines a fixed seven-section `notes.md` with a headless `claude -p`, files the
> session under the `~/dev` project it belongs to, and pushes the note plus every
> `**[YOU]**` action item into hq. Afterwards `meetnotes context` — from the
> project directory or an agent's MCP call — prints that project's recent notes as
> prompt context. The interesting engineering is not the pipeline but the
> defensive layer around it: almost every comment in the codebase names a specific
> meeting that was lost or misfiled and the rule that now prevents it.

## Identity
| Field | Value |
|---|---|
| Repository | `Dev916/meetnotes` (private) — default branch `main`, created 2026-07-12 |
| Local path | `~/dev/meetnotes` (directory name matches the repo name). The session store lives elsewhere, at `~/dev/ops/meetings/data` — the project's original home per `docs/superpowers/specs/2026-07-11-meetnotes-design.md` |
| Owner / org | PriceLove LLC (Dev916) · hq project `meetnotes` ("meetnotes (meeting capture)", client `pricelove`) |
| Status | active — last commit 2026-09-02, 109 commits, profiled at `b511ee0`. **Local `main` is 49 commits ahead of `origin/main`**, which is stale at `e2e554f` (2026-07-27) |
| Languages (by tracked file count) | Python 52 · Markdown 21 · Swift 3 · JSON 2 · one each of Makefile, `.sh`, `.toml`, `.yaml`, `.jsonl`, `.entitlements`, `.env` — 93 tracked files; ~4,300 lines of package Python, ~1,130 lines of Swift, ~7,800 lines of tests |
| Build system | `setuptools` via `pyproject.toml` (console script `meetnotes = meetnotes.cli:main`, **zero pip dependencies**) plus a `Makefile` that drives `swiftc` and `pipx` |
| Runtime deps | macOS 14.4+ on Apple Silicon; `swiftc` (Xcode CLT); `ffmpeg`; `whisper-cli` (whisper.cpp) + a GGML model; the `claude` CLI; Python 3.11+. Optional: `whisperx` + pyannote (diarization), Groq (cloud transcription), a running hq server on loopback, Google Calendar credentials |
| License | none declared — no `LICENSE` file in the tree, and GitHub reports `license: null` |
| CI / release | none — no `.github/` directory, no workflows, no tags, no releases. Distribution is `make install` on the one machine |

## What It Does

The problem it solves is not "transcribe a meeting" — plenty of tools do that. It
is that the transcript then lives somewhere nobody looks. meetnotes' premise,
stated in its design spec, is that notes captured in a meeting should be
**immediately usable as prompt context later**, from the terminal where the work
already happens (`docs/superpowers/specs/2026-07-11-meetnotes-design.md`).

So the pipeline ends in a filing decision, not a file. `stop` runs transcribe →
mine → route → push: the session directory is physically moved into
`<store_root>/<project-key>/`, a store-level `index.json` is updated, and the
mined note plus each `**[YOU]**` action item is posted into hq (`meetnotes/cli.py`,
`meetnotes/routing.py`, `meetnotes/hq_push.py`). From then on `meetnotes context`
resolves the project from the caller's working directory and prints that project's
recent notes; an agent gets the same thing through the MCP `context` tool
(`meetnotes_mcp.py`).

Local-first is a hard constraint, not a preference: the default transcription
engine is whisper.cpp on the machine, chosen in the spec so confidential and
government calls never leave the Mac; Groq is an explicit `--cloud` opt-in
(`meetnotes/transcribe.py`, `docs/superpowers/specs/2026-07-11-meetnotes-design.md`).

Its users are both human and agent — a human runs `start`/`stop`/`dashboard`, an
agent calls the MCP tools, and hq's triage engine calls `meetnotes mine --text -`
over stdio to summarise Slack threads. "Done" for a meeting is a session directory
under the right project key holding `transcript.md`, `notes.md` and a `meta.json`
with `hq_pushed: true`.

## Capabilities

### CLI (`meetnotes`, 18 subcommands)
- `start` — background record; `--no-mic` / `--no-system`, `--screen`, `--screen-window <q>`, `--screen-display <n>`, `--project` (`meetnotes/cli.py`)
- `stop` — finalize and run the pipeline; `--cloud`, `--diarize`, `--no-transcribe`, `--no-mine`, `--no-calendar`, `--project` (`meetnotes/cli.py`)
- `status` (live recordings, elapsed, sources, capture health, a loud breadcrumb for the last auto-stop, `--json` for the menu bar) · `ps` (every live recording found in the process table, not just the one in the state file) (`meetnotes/cli.py`, `meetnotes/procs.py`)
- `recover` — transcribe → mine → route a session left unprocessed, guarded by a `.recovering` lock file against a concurrent second pass · `backlog` lists what is still waiting (`meetnotes/cli.py`)
- `transcribe [target|latest]` — (re)transcribe a session or wav; `--cloud`, `--diarize` (`meetnotes/transcribe.py`)
- `mine [target|latest]` — (re)mine `notes.md`; also **text mode** `mine --text - --kind slack --context "…"`, reading a thread on stdin and emitting hq `MineResult` JSON on stdout (`meetnotes/mine.py`)
- `push [target|latest]` — (re)push a mined session into hq; `--force` overrides the already-pushed guard (`meetnotes/hq_push.py`)
- `context [project]` — recent `notes.md` for the project resolved from cwd or key · `search <query>` scans `notes.md` + `transcript.md`, newest first, one hit per session (`meetnotes/routing.py`)
- `dashboard` — one rendered snapshot: recording, pending, storage per project, recent sessions, settings; `--watch <secs>` (`meetnotes/dashboard.py`)
- `watch install|uninstall|status|run` (the mic-watch LaunchAgent) · `menubar install|uninstall|status` (menu-bar login item) (`meetnotes/watch.py`, `meetnotes/menubar.py`)
- `doctor` — does capture *actually* work along the launchd path the watcher uses · `devices` lists microphones and names the one that would record (`meetnotes/doctor.py`, `meetnotes/cli.py`)
- `clean` (purge raw audio and screen video past `limits.audio_retain_days`) · `config --show | --set key.path=value` (dotted-path get/set) (`meetnotes/cli.py`, `meetnotes/config.py`)

### MCP tools (9, stdio server `meetnotes`)
- `start`, `stop`, `status` — recording control; start/stop are deliberately **no-confirmation** (nothing leaves the machine, macOS shows a recording indicator, it is trivially stoppable) (`meetnotes_mcp.py`)
- `context`, `search` — both accept a `cwd` so the *agent's* working directory drives project resolution rather than the server's fixed launch directory (`meetnotes_mcp.py`)
- `transcribe`, `mine`, `push`, `dashboard` — the post-hoc pipeline stages; `mine` carries the dual session/text mode (`meetnotes_mcp.py`)
- A thin shell-out to the globally installed `meetnotes` binary (decoupled from the package's install location), with `--list-tools` introspecting FastMCP across SDK variants and a self-termination guard for orphaned stdio children (`meetnotes_mcp.py`)

### Claude Code skill
- `meetnotes` skill — when to record, how to pull project context, the text-mode `MineResult` contract, the hq push semantics, the routing order, and the "pipeline never breaks" failure model (`SKILL.md`)
- Installed by `make install-agent` to `~/.claude/skills/meetnotes/` alongside the MCP server; the installed copy is **byte-identical** to the repo's (verified by `diff`) (`Makefile`)

### Swift capture helper (`bin/meetnotes-capture`)
- System-audio capture via ScreenCaptureKit plus screen video and window/display enumeration: `--out`, `--video-out`, `--display`, `--window-id`, `--list-windows`, `--fps`, `--max-dimension`; microphone capture and inspection: `--mic-out`, `--mic-device`, `--list-audio-devices`, `--mic-status` (`bin/meetnotes-capture.swift`)
- `--watch-mic` — the Core Audio sensor that emits one JSON line per mic-usage snapshot, the signal the watch daemon reasons over (`bin/meetnotes-capture.swift`, `meetnotes/watch.py`)
- Menu-bar client (`bin/meetnotes-menubar.swift`, `bin/meetnotes-icon.swift`) shipped as the **main executable** of the same `MeetNotes.app` bundle, so the consent a user grants the visible app is the consent the helper answers with (`bin/build.sh`)

### Background daemons
- **Supervisor** — the detached `python -m meetnotes.supervisor` that owns a recording: staggers the two ScreenCaptureKit streams, monitors capture health by bytes-on-disk, enforces the auto-stop cap, disk floor and sleep/lock stop, and kicks off a background `recover` on auto-stop (`meetnotes/supervisor.py`)
- **Mic-watch** — LaunchAgent `com.meetnotes.watch` running the sensor and a pure-policy `decide()`; prompts to record, or auto-records when a calendar event is live (`watch.auto_start = always|calendar|never`), with debounce, cooldown and an idle-silence auto-stop (`meetnotes/watch.py`)
- **Menu bar** — LaunchAgent `com.meetnotes.menubar`, launched through LaunchServices (`open`) rather than exec'd (a directly exec'd process is not registered as an app instance, so TCC will not attribute it to the bundle), with `KeepAlive` deliberately off (`meetnotes/menubar.py`)

### Integrations
- **hq (outbound)** — `hq_push_note` + one `hq_push_item` per `**[YOU]**` item over JSON-RPC 2.0 to a loopback MCP endpoint, stdlib `urllib` only, accepting both plain-JSON and SSE-framed replies (`meetnotes/hq_push.py`)
- **hq (inbound)** — `mine` text mode is the entry point hq's triage engine calls; the JSON contract is validated locally, retried once with a stricter prompt, then hard-errors (`meetnotes/mine.py`, `SKILL.md`)
- **Google Calendar** — reads refresh tokens from the google-workspace MCP's own credential directory and refreshes them itself; matches the live event's title, attendee emails and organizer against each project's `calendar_match`, first match winning (so a catch-all keyword belongs last) (`meetnotes/gcal.py`, `meetnotes/config.py`)

### Not (yet) implemented
- Real-time live captions during a meeting — explicitly a non-goal; transcription is batch, after `stop`. Any non-macOS support is likewise out: ScreenCaptureKit, TCC, `osascript`, `launchctl` and `ps -Ao lstart` are all assumed (`docs/superpowers/specs/2026-07-11-meetnotes-design.md`, `meetnotes/procs.py`)
- Speaker diarization is listed under "Not supported (yet)" in `README.md` — but a `WhisperXEngine` with `--diarize`, an HF-token path and speaker-label formatting is fully implemented and exposed on the CLI and MCP tools. The README is stale, not the code (`meetnotes/transcribe.py`, `README.md`)

## Architecture

**Stack.** Python 3.11+ standard library only — no pip dependencies at all, by
design (`pyproject.toml`). Everything heavy is an external binary invoked as a
subprocess: `ffmpeg` for the mic, the Swift helper for system audio and screen,
`whisper-cli` for transcription, `claude` for mining, `osascript` for
notifications, `launchctl` for daemons. The one exception is the MCP server, which
needs `mcp<2` and gets its own venv at `~/.local/share/meetnotes/mcp-venv`
(`Makefile`). Capture, transcription and mining each sit behind a small interface
so any one can be swapped: `TranscriptionEngine` has `LocalWhisperEngine`,
`GroqEngine` and `WhisperXEngine` selected by `engine_for(cfg, cloud, diarize)`;
`MiningEngine` has one implementation, `ClaudeCliEngine` (`meetnotes/transcribe.py`,
`meetnotes/mine.py`).

**Data flow.**

```
start ─▶ supervisor (detached) ─┬─ ffmpeg ─▶ mic.wav
                                ├─ MeetNotes.app helper (SCK) ─▶ sys.wav [+ screen.mp4]
                                └─ health monitor: bytes-on-disk, stall + no-audio alerts
stop  ─────────────────────────────▶ mix ─▶ audio.wav (16 kHz mono)
   transcribe │ whisper.cpp local (default) · Groq --cloud · WhisperX --diarize ─▶ transcript.md
   mine       │ claude -p, seven fixed ## sections                              ─▶ notes.md
   route      │ the nine-rung ladder ─▶ mv session into data/<key>/ + meta.json + index.json
   push       │ hq_push_note + one hq_push_item per **[YOU]**  ─▶ meta.json{hq_pushed:true}

later:  meetnotes context / search / dashboard   ·   MCP context(cwd=…)
        hq triage ──▶ meetnotes mine --text -  ──▶  MineResult JSON
```

**The routing ladder.** This is the load-bearing decision and it is nine rungs
deep, each one added after a specific misfile (`meetnotes/routing.py`):

| # | Rung | Basis |
|---|---|---|
| 1 | `override` | explicit `--project` |
| 2 | `calendar` | a live Google Calendar event matching a project's `calendar_match` |
| 3 | `cwd` | deepest configured `dev_path` / `dev_paths` that is an ancestor of the recording's cwd |
| 4 | `prompt` | interactive numbered pick |
| 5 | `existing` | the filing the session already has — a decision made at record time with context recovery no longer has |
| 6 | `keyword` | most hits of the project's own `calendar_match` terms in the transcript, word-aware, ties to config order |
| 7 | `hint` | the watcher's pre-meeting calendar guess |
| 8 | `catch-all` | configured `routing.catch_all`; when set it **replaces** the LLM guess entirely |
| 9 | `infer` | `claude -p` constrained to the configured key list, else `unsorted` |

The ordering is explicitly a trust ranking, and the comments name the incidents: a
Revenium standup filed under `fenzi` because `recover` ran from `/tmp` and
inference overwrote a correct decision; a CA-DMV call left `unsorted` while its
transcript said "cadmv"; a private conversation inferred into `ghostnn` and then
auto-pushed to hq under a work project; a session moved out of `revenium` because
the watcher's pre-meeting hint was passed as `--project`.

**Storage.** Four places, all outside the repo. Sessions:
`~/dev/ops/meetings/data/<project-key>/<YYYY-MM-DD-HHMM>-session/` holding
`audio.wav`, `mic.wav`, `sys.wav`, `screen.mp4`, `transcript.md`, `notes.md`,
`meta.json` and `capture.log`, plus a store-level `index.json`
(`meetnotes/notify.py`, `meetnotes/routing.py`). Runtime state:
`~/.local/state/meetnotes/{current.json, last_stop.json, watch.json, recover.log}`
(`meetnotes/state.py`). Installed artifacts:
`~/.local/share/meetnotes/{MeetNotes.app, bin, models, mcp-venv}` (`Makefile`).
Config: `~/.config/meetnotes/config.json` (`meetnotes/config.py`).

**Process model.** Detached and multi-recording-aware. Each recording is its own
supervisor process with its own child capture engines; liveness is derived from the
**process table**, never from the state file, because that file lags a just-started
recording, lies after a crash, and cannot represent a second concurrent recording
at all (`meetnotes/procs.py`). Capture health is attached per recording rather than
smeared across all of them, so a healthy recording cannot vouch for a silent one
(`meetnotes/cli.py`).

**Security and permissions model.** Most of the repo's complexity lives here, and
it is all macOS TCC. Three decisions matter. (1) The capture helper ships inside a
**signed `.app` bundle**, because macOS keys a Screen Recording grant for a bare
executable to its cdhash and every rebuild silently revoked it (`bin/build.sh`,
`meetnotes/capture.py`). (2) Capture children are spawned **disclaimed** via
`responsibility_spawnattrs_setdisclaim`, because macOS evaluates the request
against the *responsible* ancestor — under launchd that was the pipx interpreter,
which holds no grant, so every watcher-started recording lost system audio while
hand-run ones worked (`meetnotes/spawn.py`). (3) The bundle is signed with the
hardened runtime and a `com.apple.security.device.audio-input` entitlement,
without which the microphone request is denied instantly with no prompt and no
TCC entry at all (`bin/build.sh`, `bin/MeetNotes.entitlements`). Secrets are
files, not config values — a Groq key and a HuggingFace token read from
`~/.config/meetnotes/` or the environment — and `ANTHROPIC_API_KEY` is
deliberately *stripped* before invoking `claude`, so a stray key cannot override
the claude.ai login (`meetnotes/transcribe.py`, `meetnotes/mine.py`).

## Repository Layout

```
README.md                     human-facing overview, install, pipeline, config
SKILL.md                      the Claude Code skill (entry point for agents)
AGENTS.md / CLAUDE.md         agent instructions — beads workflow, shell-safety rules
Makefile                      build · test · install · whisper · fetch-model · install-agent
pyproject.toml                setuptools; console script meetnotes = meetnotes.cli:main
meetnotes_mcp.py              stdio MCP server, 9 tools shelling out to the CLI (entry point)
.env                          tracked; one HuggingFace token key (see State/Gaps)
meetnotes/                    the package — 20 modules, ~4,300 lines
  cli.py                      argparse surface + pipeline orchestration (18 subcommands)
  config.py / state.py        JSON config with defaults+deep-merge; current/last_stop files
  procs.py / spawn.py         process-table recording discovery; disclaimed posix_spawn (TCC)
  capture.py / supervisor.py  helper+ffmpeg wiring; detached owner: health, caps, auto-stop
  transcribe.py               LocalWhisper / Groq / WhisperX engines + transcript formatting
  mine.py                     notes prompt, thread prompt, MineResult validation, timeouts
  routing.py                  the nine-rung ladder, relocation, index, context, search
  hq_push.py / gcal.py        JSON-RPC push into hq; Calendar OAuth + event→project match
  watch.py / menubar.py       mic-watch policy + daemon; LaunchAgent installers
  dashboard.py                pure collection + text rendering of the snapshot
  doctor.py                   launchd-path capture probe judged on bytes, not exit codes
  screen_select.py            pure window fuzzy-match / picker / disk preflight logic
  notify.py                   osascript notifications, free-space, session dir naming
bin/
  build.sh                    swiftc both binaries, assemble + codesign MeetNotes.app
  meetnotes-capture.swift     SCK system audio + screen video + mic + --watch-mic sensor
  meetnotes-menubar.swift     menu-bar client (bundle main executable) + -icon.swift
  MeetNotes.entitlements      com.apple.security.device.audio-input
tests/                        32 pytest files, ~7,800 lines
docs/superpowers/specs/       5 design specs (core, hq bridge, screen, mic-watch, menubar)
docs/superpowers/plans/       11 implementation plans (phases 1–7 + features)
.superpowers/sdd/             a full subagent-driven-development run for mic-watch
.beads/                       beads issue tracker (20 issues, 2 open)
```

Entry points: `meetnotes/cli.py:main` (the console script), `meetnotes_mcp.py`
(the MCP server), `SKILL.md` (the agent skill), `meetnotes/supervisor.py` (spawned
detached, never invoked by hand), `bin/meetnotes-capture.swift` (the helper).

## How It Was Built

**Toolchain.** Python 3.11+ with no third-party runtime dependencies; `swiftc`
from the Xcode Command Line Tools with ScreenCaptureKit, AVFoundation, CoreMedia,
SwiftUI and AppKit; `pytest`; `pipx`; `make` as the front door. `uv.lock` exists
but pins only the editable package itself. **Build / run / test, as they really are:**

| Target | What it does |
|---|---|
| `make build` | `bash bin/build.sh` — compiles both Swift binaries, assembles `MeetNotes.app`, codesigns it (real identity if one is found, ad-hoc with a loud warning otherwise) |
| `make install` | build, copy the helper and bundle into `~/.local/share/meetnotes`, then `pipx install --force .` (falling back to `pip install --user`) |
| `make test` | `python3 -m pytest tests -v` |
| `make whisper` / `make whisperx` / `make fetch-model` | install `whisper-cpp`, install `whisperx` pinned to Python 3.12, download the ~550 MB GGML model |
| `make install-agent` | copy `SKILL.md` + `meetnotes_mcp.py` into `~/.claude/skills/meetnotes/`, build the dedicated `mcp-venv`, and re-register the user-scoped MCP with `claude mcp add`. `make uninstall` reverses `install` |

**Dev loop.** Run from the repo with `python3 -m meetnotes.cli <cmd>` and iterate
against the real machine — most of the hard behaviour (TCC grants, launchd
spawning, Core Audio) cannot be unit-tested, which is why `meetnotes doctor`
exists as a production-path probe rather than a test. The suite is nonetheless
large (32 files, ~7,800 lines), which it achieves by keeping the tricky logic
pure. **CI/CD and deploy path: none** — no `.github/`, no workflow, no tag, no
release; "deploy" is `make install` on the developer's own Mac.

**Configuration.** `~/.config/meetnotes/config.json`, auto-created from `DEFAULTS`
and deep-merged over them so a partial hand-edited file is always valid
(`meetnotes/config.py`). Key groups by name: `capture_engine`; `sources`
(`mic`/`system`/`screen`); `mic_device` (a device *name*, never an index, because
AVFoundation indices shift); `transcription` (`engine`, `model`, `diarize`,
`diarize_model`); `mining` (`engine`, `model`, `user_names`); `limits`
(`max_minutes`, `disk_floor_gb`, `audio_retain_days`, `screen_disk_warn_gb`);
`screen` (`fps`, `max_dimension`); `reminders_minutes`; `auto_process`;
`store_root`; `projects` (`key`, `dev_path`/`dev_paths`, `hq_slug`,
`calendar_match`); `routing.catch_all`; `calendar` (`enabled`, `accounts`); `hq`
(`url`, `push`, `timeout`); `watch` (`apps`, `browsers`, `meeting_domains`,
`auto_start`, `idle_stop_minutes`, `cooldown_minutes`, `debounce_seconds`).

**Environment variable names** (purpose only): `MEETNOTES_HELPER` (capture-helper
path), `MEETNOTES_WHISPER` / `MEETNOTES_WHISPERX` / `MEETNOTES_CLAUDE` (binary
discovery overrides), `MEETNOTES_BUNDLE_ID` / `MEETNOTES_SIGN_ID` (build-time
bundle and signing identity), `GROQ_API_KEY` (cloud transcription), `HF_TOKEN` /
`HUGGING_FACE_TOKEN` / `HUGGING_FACE_HUB_TOKEN` (diarization), and
`ANTHROPIC_API_KEY` — explicitly removed from the environment handed to `claude`.

**Provenance.** 109 commits since 2026-07-12, all authored by `web-mech`, and
visibly agent-built: five design specs and eleven plans under `docs/superpowers/`,
a beads tracker (20 issues, 18 closed), and a complete
`.superpowers/sdd/2026-07-20-mic-watch-autostart/` run preserving six task briefs,
six task reports, a safety-net report, a feature report and seventeen review
diffs. The plan sequence matches the shipped system: capture core → transcription
→ mining → routing core → calendar auto → agent MCP → dashboard → hq bridge →
diarization → screen channel → mic-watch, then a menubar client spec.

## Relationships

- **Depends on / used by (ours): hq** (`Dev916/hq`, `~/dev/hq`), in both
  directions. Outbound, hq is the push target at a loopback JSON-RPC MCP endpoint
  and the reason `hq_slug` exists on each project entry. Inbound, hq's triage
  config names `"meetnotes"` as a valid summarizer engine, reached over
  `meetnotes_transport` `"http"` (POST to `meetnotes_url`) or `"stdio"` (spawn
  `meetnotes_cmd`, whose default argv points at
  `~/.claude/skills/meetnotes/meetnotes_mcp.py`), both falling back to hq's native
  LLM engine on transport failure (`~/dev/hq/crates/hq-core/src/config.rs`,
  `~/dev/hq/crates/hq-corpus/src/mine_stdio.rs`). See `docs/development/repos/hq.md`.
- **The hq arrow is asymmetric.** meetnotes knows hq's *write* tools by name
  (`hq_push_note`, `hq_push_item`) and hq's `MineResult` shape. It does **not**
  know `hq_schedule_meeting` — the string "schedule" appears nowhere in the
  meetnotes tree. meetnotes only *emits* a `meeting_intent` object inside
  `MineResult`; hq is the side that reads it and drafts a calendar create
  (`~/dev/hq/crates/hq-channels/src/triage.rs`). Scheduling is entirely hq's.
- **Depends on (third-party / adjacent):** the `claude` CLI as the mining engine;
  whisper.cpp and optionally WhisperX + pyannote; Groq's transcription API; and —
  notably — the **google-workspace MCP server's** credential directory, which
  `meetnotes/gcal.py` reads and refreshes directly rather than calling that server.
- **Shares patterns with:** `docs/development/repos/devloop.md` (both
  spec-and-plan-driven, beads-tracked, agent-built repos whose real product is a
  contract at a process boundary), `docs/development/repos/claude-skills.md` (skill
  + MCP as the agent surface), and `docs/development/repos/understudy.md`.
- **Canonical copies — clean, unusually.** `~/.claude/skills/meetnotes/SKILL.md`
  and `meetnotes_mcp.py` are **byte-identical** to the repo's copies (verified by
  `diff`); `make install-agent` copies rather than symlinks, so drift is possible
  but has not happened, and there is no third copy.
- **One orphan.** `~/dev/ops/meetings/` is the project's original home per the
  design spec. It now holds the **live session store** (`data/`) plus pre-move
  leftovers: a stale `Makefile`, a stale `bin/meetnotes-capture.swift` (6.5 KB
  versus the current 32.5 KB), a compiled binary, and empty `__pycache__` dirs.
  Not a git checkout and not a usable second copy — but the live data lives inside
  it, so it cannot simply be deleted.

## Notable Techniques

- **Liveness from the process table, not the state file.** A state file lags, lies
  after a crash, and is structurally singular. `meetnotes/procs.py` parses `ps -Ao
  pid=,lstart=,command=`, filters out processes that merely *mention* the marker
  (greps, shells), and demotes the state file to enrichment trusted only when its
  `supervisor_pid` matches.
- **Judge capture on bytes, not on exit codes.** A denied macOS capture exits
  cleanly and `ffmpeg` runs happily with a dead input device, so
  `meetnotes/supervisor.py` monitors *file growth* — a grace window, a per-source
  minimum, a stall timeout, a bad-tick streak before alerting, and a renotify
  cooldown. `meetnotes/doctor.py` applies the same rule to its probe.
- **Probe the production path, not a convenient one.** `doctor` originally ran its
  probe from the user's shell, inherited the terminal's TCC grant, and reported
  all-green for weeks while every watcher-started recording silently lost system
  audio. It now spawns through `spawn_disclaimed`, the exact path production uses.
  The sharpest reusable lesson in the repo.
- **macOS TCC as an architecture constraint.** Three findings, each documented with
  measurements: a grant for a bare executable dies on rebuild (hence the signed
  bundle); the *responsible* ancestor is what gets evaluated (hence
  `responsibility_spawnattrs_setdisclaim`, with a measured 0-bytes-versus-377 KB
  comparison in `meetnotes/spawn.py`); the microphone needs a hardened-runtime
  entitlement or it is refused with no prompt and no settings entry.
- **A trust-ranked resolution ladder.** Nine rungs ordered by how much each source
  can be trusted, with a free deterministic keyword scan placed *above* a paid LLM
  guess and a configured catch-all that **replaces** inference rather than
  preceding it — a confidently wrong answer being worse than an unopinionated one
  (`meetnotes/routing.py`).
- **A pipeline stage that cannot break the pipeline.** `hq_push.py` returns a status
  dict and never raises; a failed push records `hq_pushed:false`, warns, notifies
  and exits 0, and a tool error mentioning the project retries once without it.
- **Adaptive LLM timeouts, typed retries, validated output.** The mining budget
  scales with transcript size (base 300 s, +12 s/KB, capped at 1800 s) and **only a
  timeout is retried** — a non-zero exit buys the same failure at twice the cost.
  On the JSON path, `thread_to_json` checks the contract field by field and
  re-prompts once with a "your previous output was invalid" preamble before
  hard-erroring (`meetnotes/mine.py`).
- **Purity for testability.** `watch.decide()` takes the clock as a parameter,
  `screen_select` performs no I/O and `dashboard.collect()` never mutates — how a
  repo this OS-entangled sustains ~7,800 lines of tests.
- **Backlog candidates** (not filed here, per the profiling procedure): *macOS TCC
  for background/agent-spawned processes* (bundle identity, responsible-process
  disclaiming, entitlements versus TCC-only permissions) — uncovered in the corpus
  and expensive to relearn; *health monitoring by observable side effects* (bytes
  on disk, grace windows, stall detection, alert debouncing) as the general form
  of "exit code 0 is not evidence"; *trust-ranked resolution ladders* for
  classification mixing deterministic and LLM signals; and *adaptive timeout and
  typed-retry policy for LLM subprocess calls*.

## State, Gaps and Drift

**Maturity.** Genuinely production-used and still moving: 109 commits over seven
weeks, six design phases plus four feature specs shipped, a real session store
holding recordings across six project keys, and a comment density that reads like
an incident log. Zero literal `TODO`/`FIXME`/`HACK`/`XXX` markers in the package,
Swift helper or MCP server. Beads: 20 issues, 18 closed, 2 open — both on
mic-watch, pending live verification against a real Zoom/Safari call.

**The biggest risk is distribution, not code.** Local `main` is **49 commits ahead
of `origin/main`**; GitHub's last push was 2026-07-27. Everything since — the
menu-bar client, the disclaimed-spawn TCC fix, the signed bundle, `doctor`,
`devices`, the capture-health monitor, the routing rungs added after each misfile
— exists only on one Mac. The repo is private with no CI, so nothing notices.

**Secret hygiene.** `.env` is **tracked in git** and carries a
`HUGGING_FACE_TOKEN` entry; `.gitignore` covers `.secrets/` but not `.env`.
Everything else is stored correctly as a file under `~/.config/meetnotes/` or read
from the environment. This one file should be untracked and the token rotated.

**README-vs-code drift** — the README (last touched 2026-07-27, the same day
`origin` went stale) is a phase behind in four places:
- It lists **speaker diarization** under "Not supported (yet)", but
  `WhisperXEngine` is implemented, `--diarize` exists on `transcribe`/`stop`, and
  the MCP `stop` and `transcribe` tools expose it.
- Its config example shows `limits.max_minutes: 120` and the safety rails say
  "default 2 h"; `config.py` defaults to **240** (4 h) and documents `0` to
  disable the cap — neither appears in the README.
- Its routing section describes **five** rungs; the code has **nine**, including
  the three (`existing`, `keyword`, `catch-all`) added specifically to stop the
  misfiles the comments describe. `SKILL.md` — the document *agents* read — is
  stale the same way.
- Six of eighteen subcommands are missing from the usage block: `ps`, `backlog`,
  `doctor`, `menubar` and `devices` never appear, and `recover` only in prose.

**Internal inconsistency.** `make whisperx` tells the user to accept terms for
`pyannote/speaker-diarization-3.1`, while `transcribe.py` pins
`DEFAULT_DIARIZE_MODEL = "pyannote/speaker-diarization-community-1"` and its error
message names that one — following the Makefile leaves diarization gated.

**Structural risks.** The session store sits at `~/dev/ops/meetings/data`, inside a
directory that also holds pre-move leftovers — a collision waiting for someone to
tidy up. There is no `LICENSE`, so the repo is all-rights-reserved by default.
`gcal.py` reads another tool's OAuth credential directory by hard-coded path and
`calendar_fn` swallows every exception, so a change on the google-workspace MCP
side breaks calendar routing silently. And mining is Claude-only in practice:
`MiningEngine` has exactly one implementation.

### Synthesis (inferred)

meetnotes is best read as an **evidence-collection system that happens to record
audio**. The pipeline itself — ffmpeg, whisper, a prompt, a `mv` — is maybe three
hundred lines. The other four thousand exist because the failure mode of a meeting
recorder is uniquely bad: you find out it did not work after the meeting is over
and unrepeatable. So almost every design choice is a refusal to trust a cheap
signal. Do not trust the state file — read the process table. Do not trust an exit
code — weigh the bytes. Do not trust a probe run from your terminal — run it
through launchd the way production does. Do not trust the model's filing guess
over a decision made at record time. Do not let the hq push, the newest and least
essential stage, be able to lose a recording.

That makes the repo more valuable as a *source of techniques* than as a component
to depend on: it is macOS-only, single-machine, single-user, and its most
transferable knowledge — TCC responsibility and bundle identity, side-effect
health monitoring, trust-ranked resolution — is currently trapped in code comments
on one laptop. Extracting the TCC material into the corpus is the highest-value
follow-up; pushing the 49 commits is the most urgent.

Read in order, the routing ladder is a history of learning that *inference is the
last resort, not the fallback*. Its `catch-all` rung is the sharpest design move
here: for a project misfiling would embarrass, the system is configured to answer
"I don't know, put it here" rather than guess — exactly the property a triage
system feeding a shared command-center needs and rarely has.

## Quick Reference
| Task | Command / path |
|---|---|
| Build / install | `make build` (`bin/build.sh`) · `make install`, then `make whisper && make fetch-model` |
| Install the agent surface | `make install-agent` → `~/.claude/skills/meetnotes/` + user-scoped MCP |
| Tests / run from source | `make test` (`python3 -m pytest tests -v`) · `python3 -m meetnotes.cli <cmd>` |
| Record / stop | `meetnotes start` · `meetnotes stop` (add `--cloud`, `--diarize`, `--screen`) |
| Is it working? | `meetnotes status` · `meetnotes ps` · `meetnotes doctor` · `meetnotes dashboard --watch 5` |
| Context · fix an unprocessed meeting | `meetnotes context` from the project dir (or MCP `context(cwd=…)`) · `meetnotes recover` |
| Hands-free capture · thread → hq JSON | `meetnotes watch install`/`status`/`uninstall` · `echo "…" \| meetnotes mine --text - --kind slack` |
| Config | `~/.config/meetnotes/config.json` (`meetnotes config --show`) |
| Session store | `~/dev/ops/meetings/data/<project>/<YYYY-MM-DD-HHMM>-session/` |
| State · artifacts · CLI | `~/.local/state/meetnotes/` · `~/.local/share/meetnotes/` (`MeetNotes.app`, `models/`, `mcp-venv/`) · `~/.local/bin/meetnotes` |

## Sources

- `README.md` — pipeline overview, requirements, install path, usage block, hq-bridge section, config example, and the stale "Not supported (yet)" and routing claims.
- `SKILL.md` — the agent-facing contract: recording tools, context retrieval, the `MineResult` schema, hq push semantics, failure model.
- `meetnotes_mcp.py` — the nine tools and arguments, the shell-out design, dual-mode `mine`, tool introspection, the orphan-process guard.
- `meetnotes/cli.py` — the eighteen subcommands, the `stop` pipeline order, the recover lock, the status/health reporting rules.
- `meetnotes/routing.py` — the nine-rung ladder with its incident annotations, keyword scoring, relocation, index, `context` and `search`.
- `meetnotes/transcribe.py`, `meetnotes/mine.py` — the three transcription engines and selection logic; the seven-section notes prompt, the thread prompt, adaptive timeouts, timeout-only retry, `MineResult` validation.
- `meetnotes/hq_push.py`, `meetnotes/config.py` — the JSON-RPC contract, SSE-framed responses, `[YOU]` parsing, slug mapping, the never-raise failure model; defaults, deep-merge and every config key with its purpose.
- `meetnotes/{supervisor,capture,spawn,procs,state,doctor,menubar}.py` — capture-health, TCC, process-discovery and daemon material.
- `meetnotes/watch.py` — sensor protocol, pure `decide()`, daemon policy, LaunchAgent management, detached recover.
- `meetnotes/{gcal,dashboard,screen_select,notify}.py` — calendar matching, dashboard collection/render, window selection, notifications, session naming.
- `bin/build.sh`, `bin/meetnotes-capture.swift`, `bin/MeetNotes.entitlements`, `Makefile`, `pyproject.toml` — helper flags, bundle assembly, signing rationale, the microphone entitlement, make targets, install layout, the MCP venv rationale, the zero-dependency declaration.
- `docs/superpowers/specs/2026-07-11-meetnotes-design.md` — goals, non-goals, engine seams, and the original `~/dev/ops/meetings` home.
- `.beads/issues.jsonl`, `AGENTS.md`, `CLAUDE.md`, `.superpowers/sdd/` — provenance: beads counts, agent workflow rules, the preserved mic-watch SDD run.
- hq side, for the relationship only: `~/dev/hq/crates/hq-core/src/config.rs` (engine selection and the four `meetnotes_*` keys), `~/dev/hq/crates/hq-corpus/src/mine_stdio.rs` (stdio transport), `~/dev/hq/crates/hq-channels/src/triage.rs` (`meeting_intent` consumption).
- Metadata via `git log`/`status`/`ls-files` and `gh api repos/Dev916/meetnotes`; the hq slug via the `hq_projects` MCP tool; skill-copy equality via `diff` against `~/.claude/skills/meetnotes/`.
