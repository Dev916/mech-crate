---
title: "understudy: live meeting copilot that drafts what you would say (Repo Profile)"
category: repos
languages: [python, swift, rust, markdown]
complexity: intermediate
use_cases:
  - "understanding what understudy does and where its code lives"
  - "finding understudy's CLI, event-bus and provider surface before extending it"
  - "answering 'which repo listens to a live meeting and drafts replies'"
  - "resuming work on understudy in a fresh session"
summary: "understudy is a macOS meeting copilot: a forked Swift ScreenCaptureKit helper captures the microphone and system audio as two separate 16 kHz channels and frames them onto stdout, a Python asyncio pipeline runs each channel through its own Deepgram streaming websocket (mic = Mike, system = everyone else, so speaker attribution is free and needs no diarization), and a Claude engine drafts replies in Mike's voice from an hq briefing pack plus optional MCP context sources. A Rust ratatui console attaches over a Unix-socket JSONL event bus to show the rolling transcript, the briefing and up to three drafts, with `c` to copy one. Phase 1 listens only — there is no TTS into a call, no virtual mic, and `--voice` is refused outside the rehearsal sandbox. Drafts run on the Claude Code subscription by default (`claude -p` headless) rather than a metered API key. 29 commits since 2026-08-27, no CI, no license, 236 offline tests; the four-phase plan through voice cloning, push-to-approve speech and solo attendance lives in docs/SPEC.md and is not built."
provenance: researched
researched: 2026-09-02
publish: false
repo: https://github.com/Dev916/understudy
local_path: ~/dev/understudy
status: active
visibility: private
owner: PriceLove LLC (Dev916)
sources:
  - "README.md, docs/SPEC.md, docs/superpowers/plans/2026-08-26-phase-1-copilot.md, docs/checkpoint-01.md (target repo)"
  - "pyproject.toml, console/Cargo.toml (target repo)"
  - "understudy/: cli.py config.py capture.py frames.py spawn.py stt.py transcript.py bus.py live.py console.py suggest.py claude_cli.py briefing.py context.py meetnotes.py screen.py voice.py watch.py doctor.py filecapture.py (target repo)"
  - "bin/understudy-capture.swift, bin/build.sh, bin/Understudy.entitlements (target repo)"
  - "console/src/main.rs, console/src/app.rs, console/src/ui.rs (target repo)"
  - "scripts/voice_harvest.py, scripts/voice_harvest2.py (target repo)"
  - "docs/development/appendix-concurrency-time.md, rust-async-cancellation-graceful-shutdown.md, llm-token-cache-efficiency.md (mech-crate)"
---

# understudy

> understudy sits in on a meeting and drafts what Mike would say. A Swift helper
> forked from meetnotes captures the microphone and the system output as two
> separate channels and streams them as framed PCM on stdout; a Python pipeline
> runs each channel through its own Deepgram streaming socket, so the mic is Mike
> and the system is everyone else with no diarization anywhere; when the far end
> asks him a question, a Claude engine drafts a reply in his voice from an hq
> briefing pack. A Rust ratatui console attached over a Unix socket shows the
> transcript, the briefing and the drafts, and `c` copies one. **Phase 1 listens
> only** — nothing on the call can hear the agent, and the code refuses `--voice`
> outside the rehearsal sandbox. Six days old at the time of profiling, past its
> written plan and still moving daily.

## Identity
| Field | Value |
|---|---|
| Repository | `Dev916/understudy` (private) — default branch `main`, created 2026-08-27 |
| Local path | `~/dev/understudy` (directory name matches the repo name) |
| Owner / org | PriceLove LLC (Dev916) — **no hq project registered** for it (the live hq project list carries `meetnotes` and `hq`, not `understudy`) |
| Status | active — last commit 2026-09-02, 29 commits, profiled at `590b6aa` |
| Languages (by tracked file count) | Python 44 · Rust 5 · Markdown 3 · TOML 2 · lockfile 2 · JSONL 2 · Swift 1 · Shell 1 · entitlements 1 — 62 tracked files |
| Languages (by line count) | Python ~8,190 · Markdown ~2,820 · Swift 833 (one file) · Rust 382 |
| Build system | three: `pyproject.toml` (hatchling, driven by `uv`) · `console/Cargo.toml` (cargo, crate `understudy-console`) · `bin/build.sh` (swiftc + codesign into an `.app` bundle) |
| Runtime deps | macOS 13+ (ScreenCaptureKit), Xcode CLT (`swiftc`), Python ≥3.11 via `uv`, Rust stable; `anthropic`, `websockets>=14`, `httpx`; Deepgram account; Claude Code CLI (default) or an Anthropic API key; optional hq on loopback `:7717`; optional `meetnotes` CLI; macOS `screencapture`/`sips`/`afplay`/`say`; `ffmpeg` for the voice-harvest scripts only |
| License | none declared — no `LICENSE` file, and the GitHub API reports no license |
| CI / release | none — no `.github/` directory, no tags, no releases. Install is `uv sync` + `bash bin/build.sh` + `cargo build --release` plus two symlinks into `~/.local/bin` (`README.md`) |

## What It Does

The problem is the gap between what Mike knows and what he can recall in the four
seconds after a client asks him something. Everything needed is already written
down — prior meeting notes, open work, decisions, pricing history — in hq and in
the meetnotes store, and none of it is reachable mid-sentence.

understudy's answer is to attend the meeting as a reader. `understudy live`
spawns the Swift capture helper, opens a Unix socket at
`~/.local/state/understudy/events.sock`, and starts two Deepgram websockets, one
per audio channel (`understudy/cli.py`, `understudy/stt.py`). Because the mic and
the system output never mix, speaker attribution is a property of the wire rather
than a model output: source 0 is Mike, source 1 is the far end
(`understudy/frames.py`). Deepgram's `is_final` segments are accumulated and only
flushed on `speech_final`/`UtteranceEnd`, so a long question is classified once
and whole rather than in fragments (`understudy/transcript.py`).

When a completed far-end utterance is interrogative *and* names Mike — or is
second-person after four seconds of his own silence — a draft fires
automatically; any other far-end question publishes a dimmed prompt instead and
waits for `d` (`understudy/suggest.py`, `understudy/live.py`). The draft is
written by Claude against a mandatory persona block loaded from the `mikeify`
skill plus a briefing pack assembled from hq (`understudy/briefing.py`).

Its user is one person at a keyboard; its consumers are a Rust TUI and a
clipboard. "Done" for a session looks like: the console shows a two-speaker
transcript within about a second of speech, drafts appear when someone asks Mike
something, `c` copies the one he wants, and when the session ends the transcript
is handed to meetnotes to file (`understudy/meetnotes.py`). Nothing on the call
ever hears the agent — `--voice` on a non-rehearsal `live` run exits 2 with
"voice output is sandbox-only until Phase 3" (`understudy/cli.py`).

## Capabilities

### CLI — `understudy` (`understudy/cli.py`)
- `live` — the session orchestrator; flags `--project`, `--fake`, `--duration`, `--no-stt`, `--rehearse`, `--screen`, `--voice`, `--console`, `--no-meetnotes` (`understudy/cli.py`, `understudy/live.py`)
- `rehearse` — one-command sandbox; expands to `live --rehearse --console` plus `--voice` unless `--no-voice` (`understudy/cli.py`, function `rehearse_argv`)
- `replay <session-dir>` — stream a recorded meetnotes session (`mic.wav`/`sys.wav`) through the real pipeline at meeting pace; `--speed`, `--max-minutes` (`understudy/filecapture.py`)
- `stop` — end the running session from any shell by writing `{"cmd":"stop"}` to the socket (`understudy/bus.py`, function `request_stop`)
- `doctor` — live-ping preflight: capture helper, Deepgram project list, the drafting provider, hq `/health`, writable state dir, per-source MCP lines, and a two-second mic-signal check (`understudy/doctor.py`)
- `brief --project X` — build and print the briefing pack; exits 3 when hq is unreachable (`understudy/briefing.py`)
- `context servers` / `context probe "<question>"` — list configured MCP sources with reachability and scope, or run one standalone draft-style lookup and report tools called and elapsed time (`understudy/cli.py`, `understudy/context.py`)
- `config show` — the merged config as JSON (`understudy/config.py`)
- `frames-decode <file>` — frame and byte counts per channel from a raw capture dump (`understudy/frames.py`)
- `watch install|uninstall|status|run [--dry FILE]` — the opt-in mic-watch LaunchAgent and a dry policy replay that prints one JSON decision per sensor line (`understudy/watch.py`)

### Capture helper — `understudy-capture` (`bin/understudy-capture.swift`)
- `--pcm-stdout` — run both recorders concurrently and frame their PCM onto stdout (the understudy addition; upstream is mic-or-system with an early exit)
- `--fake` — synthetic 440/880 Hz tones, needs no TCC grant at all
- `--no-mic` / `--no-sys` — drop a channel (rehearsal uses `--no-sys`, so it needs only the microphone grant)
- `--mic-device NAME`, `--list-audio-devices`, `--mic-status` — device pinning and enumeration
- `--watch-mic` — the sensor: one JSON line per change in the set of processes holding the microphone
- inherited from meetnotes, unused by the Python side: `--out`, `--mic-out`, `--video-out`, `--display`, `--window-id`, `--fps`, `--max-dimension`, `--list-windows`

### Console — `understudy-console` (`console/src/`)
- Four-region ratatui layout: transcript, an optional briefing pane at 35% width, a suggestions list, a three-line log strip, and a status line carrying state, the last warn kind and `speaking` (`console/src/ui.rs`)
- Keys `c` copy focused draft to the clipboard via `arboard`, `j`/`k` move focus, `x` dismiss, `d` request a draft, `b` toggle briefing, `q` and Ctrl-C quit (`console/src/main.rs`)
- Focus discipline: a new suggestion never steals focus once `j`/`k` has been pressed, and the list is capped at three (`console/src/app.rs`)
- `--replay FILE` renders a JSONL event file through a `TestBackend` and prints counts; `--once` prints the first event and exits — both are the headless test hooks (`console/src/main.rs`)

### Event bus (Unix socket, JSONL)
- Published downstream: `transcript`, `suggestion_delta`, `suggestion`, `metrics`, `status`, `briefing`, `prompt`, `screen`, `speaking`, `warn`, `log` (`understudy/live.py`, `understudy/bus.py`, `console/src/app.rs`)
- Accepted upstream: `{"cmd":"draft"}` and `{"cmd":"stop"}` (`understudy/live.py`, method `_command`)
- Late subscribers receive sticky `status`/`briefing` events plus a 200-event replay before live fan-out; each subscriber owns a 1,000-slot queue and a slow console drops events rather than stalling the pipeline (`understudy/bus.py`)
- One session owns the socket: a second `EventBus.start()` raises `SessionAlreadyRunning` (exit 2) instead of unlinking a socket a console is attached to; a stale socket file is not a session (`understudy/bus.py`)

### Drafting providers (one contract, two engines)
- `claude-cli` (default) — shells out to `claude -p` on the Claude Code subscription with `--safe-mode --strict-mcp-config --tools "" --include-partial-messages`, stripping `ANTHROPIC_API_KEY`, `CLAUDECODE` and `CLAUDE_CODE_ENTRYPOINT` from the child environment (`understudy/claude_cli.py`)
- `api` — the metered `anthropic` SDK, streaming, with the briefing pack as a `cache_control` block at `ttl: "1h"` and a `metrics` event carrying `stop_reason` and cache token counts (`understudy/suggest.py`)
- Both yield the same event sequence — `suggestion_delta`* → `suggestion` → `metrics` (→ `warn`) — so `live`, `replay`, `--rehearse` and `--screen` are provider-blind (`understudy/live.py`)

### Context sources (MCP)
- Aliases resolve either to a server Claude Code already knows, read directly out of `~/.claude.json` and `~/.claude-revenium/.claude.json`, or to an inline http/stdio definition; the generated mcp-config is written mode 600 and never printed (`understudy/context.py`)
- `projects` globs scope a source so an out-of-scope session never launches the server or hands it its environment; `default: true` means every session (`understudy/context.py`)
- Two use points: a prefetch at briefing time whose facts land inside the cached pack, and draft-time lookups bounded by `lookup_max_turns` (`understudy/cli.py`, `understudy/context.py`)

### Sandbox surfaces
- `--screen` — a silent `screencapture -x` still of the main display every `interval_secs`, downscaled with `sips`, pruned to `keep_shots`; analysis only on a jpeg-size change, at most every `analyze_min_secs`, with a 60 s backstop (`understudy/screen.py`)
- `--voice` — spoken drafts through the speakers with three providers behind one seam (`say`, `elevenlabs`, `cartesia`), latest-draft-wins, and a synthesis failure falling back to `say` with a `speak_fallback` warn (`understudy/voice.py`)
- Echo gate — while a draft is playing and for 0.4 s after, far-end audio is dropped before it reaches the transcript store, with a rate-limited `echo_gated` warn so the silence never reads as a broken mic (`understudy/live.py`, `understudy/voice.py`)

### Filing bridge
- A real live session starts `meetnotes start --yes --project X` beside itself, adopts an already-running recording for the same or no project, and asks before replacing one belonging to a different project (terminal prompt, or a 30 s dialog defaulting to no) (`understudy/meetnotes.py`)
- At session end the Deepgram transcript is rendered into meetnotes' `transcript.md` shape (mic = Mike, system = Others) and `meetnotes stop --no-transcribe` runs detached; an orphan is kept under `~/.local/state/understudy/transcripts/` (`understudy/meetnotes.py`)

### Scripts
- `scripts/voice_harvest.py` — walk the meetnotes store, strip silence from each `mic.wav`, stitch a master and cut 25-minute MP3 segments for voice-clone upload
- `scripts/voice_harvest2.py` — the corrected version: gates a 200 ms mic window only when `sys.wav` is simultaneously quiet, with guard bands and a `--blind-run` drop for stretches where `sys.wav` recorded literal digital zeroes

### Not (yet) implemented
- Phases 2–4 of `docs/SPEC.md` — voice clone, push-to-approve speech through a virtual mic, and solo attendance — exist as design only; no BlackHole mixer, no `AudioHardwareCreateProcessTap`, no Recall.ai, no Pipecat dependency anywhere in the tree
- `understudy panic` (the SPEC §9 kill switch) is not a subcommand; unknown subcommands fall through to "not implemented yet", exit 2 (`understudy/cli.py`)
- SPEC §6 Tier 2 (a local in-memory embedding index for sub-50 ms mid-meeting lookups) was deferred in the plan's own self-review and never built; the shipped Tier 3 MCP sources cost the 5–7 s the tier-2 index existed to avoid
- The every-N-minutes summary (SPEC §7 trigger c) is absent; its config key `suggest.summary_minutes` is defined and never read (`understudy/config.py`)
- `suggest.max_visible` is likewise unread — the three-suggestion cap is hardcoded in the Rust console (`console/src/app.rs`)
- `model_fast` is a Phase 3 placeholder, documented as such and unused (`understudy/config.py`)
- The console's `x` key sends `{"cmd":"dismiss","id":...}`, which the session's command handler ignores; dismissal is client-side only (`console/src/main.rs`, `understudy/live.py`)

## Architecture

**Stack.** Python 3.11+ asyncio for the pipeline (`anthropic`, `websockets>=14`,
`httpx` — three runtime dependencies, deliberately), one Swift file compiled with
`swiftc -O` against ScreenCaptureKit/AVFoundation/CoreMedia/CoreAudio, and a Rust
console on ratatui 0.30 + crossterm 0.29 + tokio + clap + arboard. The language
split is argued from measurement in `docs/SPEC.md` §2: the media pipeline goes
where the SDKs are (Python), the TCC-hardened capture path stays where it already
works (Swift), and the console goes where a static binary with zero TCC grants
wins (Rust).

**Component map.** `bin/understudy-capture.swift` (capture + mic sensor) →
`understudy/capture.py` (spawn, async frame stream) → `understudy/frames.py`
(protocol) → `understudy/stt.py` (one Deepgram socket per channel) →
`understudy/transcript.py` (utterance aggregation) → `understudy/suggest.py` /
`understudy/claude_cli.py` (drafting) → `understudy/bus.py` (socket fan-out) →
`console/` (TUI). Around that core: `briefing.py` and `context.py` build the
prompt, `screen.py` adds vision, `voice.py` adds sandbox speech, `watch.py` adds
the daemon, `meetnotes.py` adds filing, `doctor.py` checks it all, and
`filecapture.py` substitutes files for the helper.

**Data flow.**

```
 mic  ─ AVCaptureSession ─┐
                          ├─▶ understudy-capture (Swift, signed bundle)
 sys  ─ SCStream audio  ──┘        ring buffer + writer thread
                                   0x55 | src | u32 len | s16 LE 16 kHz
                                          │ stdout pipe
                          ┌───────────────▼───────────────────────────┐
                          │ CaptureClient ─▶ per-channel PCM queues   │
                          │   src 0 ─▶ Deepgram WS  ─┐                │
                          │   src 1 ─▶ Deepgram WS  ─┴▶ TranscriptStore
                          │        speech_final flush ─▶ classify()    │
                          │            auto | arm | none               │
                          │        ─▶ draft(window, question)          │
                          │            claude -p  ―or―  anthropic SDK  │
                          │            (persona + hq pack + MCP tools) │
                          └───────────────┬───────────────────────────┘
                                          │ events.sock (JSONL, sticky + 200 replay)
                          ┌───────────────▼───────────────────────────┐
                          │ understudy-console (Rust, ratatui)        │
                          │  transcript │ briefing │ drafts │ log     │
                          │  c copy · d draft · x dismiss · q quit    │
                          └───────────────────────────────────────────┘
   session end ─▶ transcript.md ─▶ meetnotes stop --no-transcribe (detached)
```

**Storage.** Almost none, by design. The transcript lives in memory and on the
bus; filing is delegated to meetnotes (`understudy/meetnotes.py`). What is on
disk: `~/.cache/understudy/brief-<hash>.md` (briefing packs, TTL
`brief_ttl_hours`, cached only when every hq leg succeeded),
`~/.local/state/understudy/` (the socket, `live.log`, `meetnotes.log`, `screens/`,
orphaned `transcripts/`), `~/.config/understudy/config.json` plus per-key files,
and a mode-600 temporary mcp-config per drafting child.

**External integrations.** Deepgram streaming STT (`nova-3`,
`utterance_end_ms=1000`, interim results); Anthropic, reached either through the
Claude Code CLI on a subscription or through the SDK on a key; hq over loopback
HTTP `:7717`, GET routes only (`/api/agenda`, `/api/corpus/search`, `/api/notes`,
`/api/work`, `/health`); ElevenLabs and Cartesia REST for cloned playback; the
`meetnotes` CLI and its state file; arbitrary MCP servers by way of Claude Code's
own registries.

**Process and concurrency model.** One session process owning one socket, one
Swift child, and one asyncio task per concern (frame pump, per-channel STT,
utterance consumer, manual-draft consumer, screen watcher). Backpressure is
explicit and asymmetric: PCM queues are bounded at 200 frames and drop the oldest
with a rate-limited `pcm_drop` warn (`understudy/live.py`); subscriber queues drop
newest without stalling the publisher (`understudy/bus.py`); the Swift side has a
256-frame ring drained by a dedicated writer thread with `SIGPIPE` ignored,
because blocking an audio callback on a full pipe makes SCStream drop samples
silently (`bin/understudy-capture.swift`). Shutdown is bounded everywhere:
`CaptureClient.stop` terminates, waits with a deadline, then escalates to
`SIGKILL`; `EventBus.stop` closes clients before `wait_closed()` because the
reverse order hangs on Python ≥3.12.

**Security model.** Phase 1 has no inbound network surface — the only listener is
an `AF_UNIX` socket in the user's own state directory. Secrets are read from the
environment or from single-purpose files under `~/.config/understudy/`, never
from the config JSON, by name: `DEEPGRAM_API_KEY`, `ANTHROPIC_API_KEY`,
`ELEVENLABS_API_KEY`, `CARTESIA_API_KEY` (`understudy/config.py`, function
`resolve_key`). The drafting child runs with `--tools ""` so it cannot touch the
filesystem, and a resolved MCP server's environment travels into a mode-600
temporary file whose contents are never printed (`understudy/context.py`). macOS
TCC grants key to the bundle id `co.pricelove.understudy.capture` rather than to
a code hash, so a rebuild does not revoke them (`bin/build.sh`). The hard rules in
the persona are a security control of a different kind: never commit to scope,
dates, pricing or contracts; never claim work not in the briefing; defer rather
than improvise (`understudy/suggest.py`).

## Repository Layout

```
README.md                       413 lines: install, usage, every subsystem, TCC notes
pyproject.toml                  hatchling package, console script `understudy`
uv.lock                         resolved Python environment
docs/
  SPEC.md                       the four-phase design, latency budget, cost model
  superpowers/plans/
    2026-08-26-phase-1-copilot.md   2,185-line TDD plan, 11 tasks, all checked
  checkpoint-01.md              untracked hand-off note to Mike (sandbox recipe)
understudy/                     the pipeline package (entry point: cli.py:main)
  cli.py                        argparse surface and every command's wiring
  config.py                     DEFAULTS, deep merge, key resolution
  capture.py frames.py spawn.py capture child, frame protocol, disclaimed spawn
  stt.py transcript.py          Deepgram sockets, utterance aggregation
  briefing.py context.py        hq pack builder, MCP context sources
  suggest.py claude_cli.py      the two drafting engines and the trigger policy
  live.py bus.py console.py     session orchestrator, event bus, one-terminal mode
  screen.py voice.py            screenshare vision, sandbox speech
  watch.py                      mic-watch policy, dialog, LaunchAgent
  meetnotes.py                  filing bridge and ownership arbitration
  filecapture.py                replay a recorded session as if it were the helper
  doctor.py                     live-ping preflight
bin/
  understudy-capture.swift      833 lines, forked from meetnotes-capture.swift
  build.sh                      swiftc + Info.plist + codesign into Understudy.app
  Understudy.entitlements       com.apple.security.device.audio-input
console/                        Rust crate `understudy-console`
  src/main.rs                   entry point: socket client, key handling, replay mode
  src/app.rs                    the event enum and all view state
  src/ui.rs                     ratatui layout
  tests/render.rs               render smoke test
scripts/                        voice_harvest.py, voice_harvest2.py (Phase 2 prep)
tests/                          20 files, 236 test functions, all offline
```

Entry points: `understudy/cli.py` (`main`, exposed as the `understudy` console
script), `console/src/main.rs` (the `understudy-console` binary), and
`bin/understudy-capture.swift` (top-level Swift, argument switch near line 620).

## How It Was Built

**Toolchain.** `uv` for Python (3.11+, must also pass on 3.13 — the machine
default), `cargo` for the console with ratatui pinned to 0.30 and crossterm to
0.29 (the plan's constraints note that mismatched versions silently compile two
crossterms), and `swiftc -O` for the helper. No formatter or linter is configured
anywhere in the tree.

**Build / run / test — as they really are.** `uv sync`; `bash bin/build.sh`;
`cd console && cargo build --release`; `uv run pytest -q` (236 tests, entirely
offline — the capture tests skip themselves when `bin/understudy-capture` has not
been built); `cd console && cargo test`. Anything touching Deepgram, Anthropic or
hq for real is a manual smoke, never part of the suite (`README.md`,
`docs/superpowers/plans/2026-08-26-phase-1-copilot.md`).

**Dev loop.** `uv run understudy rehearse --project X` boots the pipeline and the
console in one terminal, with the pipeline's stderr parked in
`{state_dir}/live.log` so a Deepgram reconnect notice never prints across the TUI
frame (`understudy/console.py`). `understudy replay <meetnotes-session-dir>` is
the higher-fidelity loop: real recorded colleagues, real Deepgram, real Claude,
paced like the meeting happened. `--fake` needs no TCC grant at all.

**CI/CD and deploy path.** None. There is no `.github/` directory, no tag and no
release; "deploy" is two symlinks into `~/.local/bin` plus the LaunchAgent that
`understudy watch install` writes (label `co.pricelove.understudy.watch`).

**Configuration and environment variable names** (names and purpose only).
Config is `~/.config/understudy/config.json`, deep-merged over `DEFAULTS`, with
`UNDERSTUDY_CONFIG` pointing elsewhere for experiments. Notable keys:
`llm.provider` / `cli_bin` / `cli_model` / `cli_screen_model`, `model`,
`model_fast`, `hq_url`, `socket_path`, `helper_path`, `state_dir`, `cache_dir`,
`brief_ttl_hours`, `stt.dg_model`, `suggest.*`, `voice.provider` and its per-
provider voice-id and model keys, `screen.interval_secs` /`analyze_min_secs` /
`keep_shots` / `max_tokens`, `context.mcp_servers` / `lookup_max_turns` /
`prefetch` / `prefetch_queries`, `mic_device`, `meetnotes.enabled` / `bin` /
`handoff_transcript`, and `watch.apps` / `debounce_secs` / `cooldown_secs` /
`prompt_timeout_secs` / `open_console`. API keys are never read from that file:
they come from the environment (`DEEPGRAM_API_KEY`, `ANTHROPIC_API_KEY`,
`ELEVENLABS_API_KEY`, `CARTESIA_API_KEY`) or from same-named lowercase files
beside the config, which is how a LaunchAgent-started session sees them at all.

**Provenance.** Spec-first and agent-built. `docs/SPEC.md` (2026-08-26) records a
six-agent research fan-out over the meetnotes codebase, the hq API surface, the
Rust and Python voice ecosystems, macOS audio wiring and voice-clone providers,
and every architectural choice in it cites a finding. The implementation plan is
a `writing-devloop-plans` artifact — it carries `**Compatible with:** devloop
skill v0.1+`, eleven `**Verify via:** cli` task blocks and a five-step
failing-test-first rhythm — and its revision note records an adversarial review by
three verification agents that compiled the Rust, executed the Python blocks, ran
the real meetnotes binary and probed live hq. Tasks 1–11 map onto the first
eleven commits almost one-to-one. Everything after 2026-08-27 (the watch daemon,
the sandbox modes, screen vision, the subscription provider, spoken drafts, MCP
context sources, `rehearse`, the meetnotes bridge) is post-plan work with no plan
document behind it. All 29 commits are by web-mech.

## Relationships

**meetnotes (`Dev916/meetnotes`, `~/dev/meetnotes`) — parent, sibling and
dependency, all three.** See `docs/development/repos/meetnotes.md`.

- **Forked code, one direction.** `bin/understudy-capture.swift` still opens with meetnotes' own header comment, `// meetings/bin/meetnotes-capture.swift`. Diffed against `~/dev/meetnotes/bin/meetnotes-capture.swift`: 684 lines → 833, an additive fork in which the `Recorder` (SCStream), `MicRecorder` (AVCaptureSession), device enumeration, TCC run-loop pumping and format conversion survive essentially verbatim. understudy adds three things: the `PCMStdoutWriter` ring and writer thread, optional output paths so the helper can stream without writing files, and a dispatch block running both recorders concurrently behind one stop semaphore (upstream is mic-**or**-system with an early `exit(0)`). The bundle ids differ — `co.pricelove.understudy.capture` against `com.meetnotes.app` — so the two hold independent TCC grants and can record the same meeting at once.
- **Copied module.** `understudy/spawn.py` opens with `# meetings/meetnotes/spawn.py` and keeps meetnotes' measurement table for the disclaimed-spawn fix, extended with a `stdout_fd` parameter and a `spawn_disclaimed_piped` wrapper.
- **Copied build strategy.** `bin/build.sh` cites `meetnotes bin/build.sh` in its second line for the bundle-not-bare-binary rule.
- **Runtime dependency.** `understudy/meetnotes.py` shells out to the `meetnotes` CLI and reads its state file at `~/.local/state/meetnotes/current.json`, with adopt/ask/start arbitration so the two tools never cut each other's meetings in half. Two recorders run on purpose: meetnotes' supervisor is detached and survives an understudy crash.
- **Data dependency.** `understudy replay` reads meetnotes session directories, and both voice-harvest scripts walk the meetnotes store. Both paths are read-only, and the plan's global constraints forbid modifying anything under `~/dev/meetnotes` or `~/dev/hq`.
- **Nothing flows back.** A recursive search for "understudy" across the meetnotes tree returns no files: meetnotes does not know understudy exists.
- **The contrast is the point.** meetnotes is a batch archivist — stdlib-only Python, WAVs on disk, whisper.cpp after the fact, a SwiftUI menu bar. understudy is a live participant — asyncio, streaming Deepgram per channel, nothing on disk, a Rust TUI. understudy defers all filing back rather than duplicating it.

**hq (`~/dev/hq`, hq project slug `hq`).** See `docs/development/repos/hq.md`. The
briefing pack is assembled from four hq GET routes in parallel plus a
vocabulary-seeded corpus query (`understudy/briefing.py`); `doctor` pings
`/health`; the watch daemon resolves a project slug from `/api/agenda`
(`understudy/watch.py`). hq is optional throughout — its absence costs the pack,
not the session. `docs/SPEC.md` §6 records that hq is loopback-bound with no auth
and that corpus chunks currently carry `project_slug = null`, which is why corpus
pulls go unscoped with the project's vocabulary in the query text.

**devloop (`nyvorin/devloop`).** See `docs/development/repos/devloop.md`. Not a
code dependency — a methodology one: understudy's implementation plan is a
devloop-compatible plan, and the CLI verification toolkit is what its acceptance
criteria are written against.

**Claude Code.** A hard runtime dependency of the default configuration, in three
distinct roles: the drafting engine (`claude -p`), the screenshare analyzer, and
the MCP registry that `understudy/context.py` reads to resolve source aliases.

**The `mikeify` skill** (`~/.claude/skills/mikeify/SKILL.md`) is loaded at runtime
for the persona block, frontmatter stripped, with an embedded fallback and a
sanity check on a known phrase so the skill and the agent cannot drift apart
silently (`understudy/suggest.py`).

**Third-party services:** Deepgram (STT), Anthropic, ElevenLabs and Cartesia
(playback providers), Recall.ai (Phase 4, design only).

## Notable Techniques

- **Channel separation as diarization.** Keeping mic and system audio on separate wires all the way to two independent STT sockets makes speaker attribution a property of the transport. It costs one extra websocket and removes an entire model class from the pipeline. `docs/SPEC.md` §3 names the alternative it replaces: meetnotes' WhisperX diarization, which is minutes-latency batch.
- **Never block an audio callback.** The Swift ring buffer plus dedicated writer thread, `SIGPIPE` ignored and a dropped-frame counter, exists because a full 64 KB pipe is about two seconds of audio and blocking there makes SCStream drop samples with no error. The same discipline reappears in Python as bounded queues that drop rather than stall — the one-writer, bounded-fan-out shape analysed in `docs/development/appendix-concurrency-time.md`, which `understudy/bus.py` cites directly.
- **Bounded, ordered shutdown.** `understudy/capture.py` cites `docs/development/rust-async-cancellation-graceful-shutdown.md` for terminate → wait with deadline → `SIGKILL`; `understudy/bus.py` records that on Python ≥3.12 `wait_closed()` waits for handler coroutines, so clients must be closed first or `stop()` hangs on an idle subscriber.
- **A subscription CLI as an LLM provider.** `understudy/claude_cli.py` is a worked example of treating `claude -p` as a metered-API substitute: strip three environment variables or the child blocks on an interactive prompt with nothing to answer it; pass `--safe-mode --strict-mcp-config` or startup loads the whole MCP fleet (measured 7.85 s against 2.45 s); read text from the partial-message deltas *or* the final result, never both. Related corpus reading: `docs/development/llm-token-cache-efficiency.md`.
- **Cache TTL chosen against the interaction cadence.** Drafts in a meeting are often more than five minutes apart, so a default five-minute prompt cache never gets a read and costs about 25% more than no caching; the one-hour TTL is one double-priced write per meeting and 0.1× reads thereafter, and every draft emits cache token counts so the assumption stays observable (`docs/SPEC.md` §7, `understudy/suggest.py`).
- **Two-tier triggering over completed utterances.** Classifying on `speech_final` rather than on `is_final` avoids firing on fragments; requiring the name or a second-person question after measured silence avoids the bare `you`-regex that fires on most questions in a group call and would evict real drafts with noise (`understudy/suggest.py`, `understudy/transcript.py`).
- **Gate the detector, not the recording.** `scripts/voice_harvest2.py` keeps a mic window only when the *other* channel is simultaneously quiet, and drops stretches where that channel recorded literal zeroes — a blind detector, not a quiet room. The README records what this was worth: 61 minutes of corpus down to 40, and the worst session from three apparent speakers to one.
- **Preflight by live ping, not by string presence.** `understudy/doctor.py` replaced "is the key non-empty" with a real Deepgram project list, a real `claude -p` that must answer `pong`, and a two-second capture that fails on a silent default input device — with offline as a third state (`warn`, not `fail`) so it does not cry wolf.
- **Backlog candidates** (listed here per the profiling procedure; `RESEARCH_BACKLOG.md` is not edited by this task): (1) *real-time streaming voice-agent pipelines* — STT/LLM/TTS staging, turn-taking, endpointing and barge-in, which is exactly the Phase 3 work `docs/SPEC.md` §4 and §8b budget for; the profiling task described this as already queued in `RESEARCH_BACKLOG.md` with an added-date of 2026-08-26, and no such entry appears in that file on `main` or on any branch searched — both facts recorded, neither asserted. (2) *macOS capture under TCC* — responsible-process disclaiming, bundle-id-keyed grants, and why launchd-started capture fails silently. (3) *headless coding-CLI as an LLM provider* — flag sets, environment hygiene, streaming contracts and the latency they buy or cost.

## State, Gaps and Drift

**Maturity.** Six days old, 29 commits, no CI, no tags, no license, no `.beads/`
directory. The test suite is unusually strong for the age — 236 functions across
20 files, all offline, with a fake console, a sensor-line fixture and a frame
fixture generator — and every module carries a docstring that explains why it is
shaped the way it is. Zero literal `TODO`, `FIXME` or `HACK` markers appear in
the Python, Rust or Swift sources; deferrals are written as prose and as phase
labels instead.

**Plan-vs-code drift is forward, not stale.** All eleven plan tasks are checked
and the code has since moved well past the plan, which was never revised — the
watch daemon, the sandbox modes, screenshare vision, the subscription provider,
the voice seam, MCP context sources and the meetnotes bridge are all
undocumented in the plan and fully documented in `README.md` and `docs/SPEC.md`,
both of which were updated as those landed (`docs/SPEC.md` §5 marks Phase 1.5 as
implemented and §6b as shipped 2026-09-02).

**README-vs-code drift.**
- `README.md` states "Python spawns the helper disclaimed, so macOS attributes the capture to the signed bundle instead of to your terminal or to Python." In the live path it does not: `CaptureClient.__init__` defaults `disclaimed=False` and `understudy/cli.py` never overrides it, so the helper is started with a plain `asyncio.create_subprocess_exec`. The disclaimed path exists and works, and at runtime is used for exactly one thing — the `--watch-mic` sensor (`understudy/watch.py`, function `_start_sensor`). The session the daemon starts is itself a plain `subprocess.Popen(start_new_session=True)`.
- The README's prerequisites list `ANTHROPIC_API_KEY` as needed "only if you switch to the metered API path", which the code agrees with; but `understudy/watch.py` still writes both `DEEPGRAM_API_KEY` and `ANTHROPIC_API_KEY` key files on `watch install` regardless of provider, so a subscription-only user gets a file they do not need.
- `understudy doctor` is documented as printing a `meetnotes (filing)` line; that line is produced by `understudy/doctor.py`, verified present.

**Config keys with no reader.** `suggest.max_visible`, `suggest.summary_minutes`
and `model_fast` are defined in `DEFAULTS` and read nowhere. The first is
enforced as a literal `3` in `console/src/app.rs`, so editing the config silently
does nothing; the second corresponds to a SPEC feature never built; the third is
an intentional Phase 3 placeholder.

**Seam mismatch.** The console emits a `dismiss` command that the session ignores
(`console/src/main.rs`, `understudy/live.py`). Nothing breaks — dismissal is a
local list edit — but the wire has a verb with no listener.

**Risks.**
1. *Silent capture failure under launchd.* `understudy/spawn.py`'s own measurement table shows that a capture child whose responsible process is a launchd-started interpreter loses system audio with no visible error. The daemon path today spawns an undisclaimed session, which spawns an undisclaimed helper. Nothing in the suite could catch it, because the whole suite is offline.
2. *Single-vendor STT.* `understudy/stt.py` hardcodes Deepgram in the URL, in the auth header shape and in the message parsing, and `_stt_stack` builds `DeepgramSTT` unconditionally, while `config.py` advertises a `stt.provider` key. There is no second implementation behind that key.
3. *Two recorders on one microphone.* Deliberate and measured once (2026-09-02, both helpers reading comparable RMS), but it is an assumption about macOS device sharing that no test can hold in place.
4. *No license and no CI* on a repo that captures meeting audio and drafts speech attributed to a named person.

### Synthesis (inferred)

understudy reads as meetnotes' second act rather than its replacement, and the
split is cleaner than most sibling pairs manage: meetnotes owns the past tense
(record, transcribe, mine, file) and understudy owns the present tense (hear,
attribute, draft). The bridge in `understudy/meetnotes.py` is the thesis stated
in code — understudy could easily have written its own transcripts to disk, and
chose instead to hand them to the tool that already knows where meetings go. The
fork is where the discipline slips slightly: the Swift helper and `spawn.py` were
copied rather than shared, so meetnotes' hardest-won asset now exists twice and
will drift. That is the right call at day six and the wrong one at month six; the
cheap correction is to extract the capture helper into something both repos build
from, before either file changes again.

The most transferable engineering here is not the meeting product at all — it is
the treatment of latency as a design input rather than a benchmark. Nearly every
non-obvious decision in the tree is a measured number written down beside the code
it justifies: 2.45 s against 7.85 s for the lean CLI invocation, 2.6 s for the
MCP-enabled variant, 5–7 s for a mid-draft lookup (which is the entire argument
for the briefing prefetch), 24.2 s and eight sourced facts for a pack build, a
1,000 ms Deepgram endpointing floor. That habit is why the Phase 3 latency budget
in `docs/SPEC.md` §4 is honest enough to admit its own p90 sits past the two-second
comfort line, and it is the part of this repo worth copying into unrelated work.

The gap between the built thing and the designed thing is wide and clearly
marked, which is the healthy version of this situation. Phase 1 is real,
exercised daily and hardened by failures that left their scars in comments (the
Bluetooth headset that delivered digital zeros, the stale API key that blocked a
child on an interactive prompt, the mic bleed that made a voice clone sound like a
client). Phases 2–4 are a research document with cost tables. The risk is not that
the plan is wrong; it is that the disclaimed-spawn gap and the offline-only test
suite mean the daemon path — the one that makes this tool automatic rather than
deliberate — is the least verified path in the repo.

## Quick Reference
| Task | Command / path |
|---|---|
| Install | `uv sync` · `bash bin/build.sh` · `cd console && cargo build --release` |
| Preflight | `uv run understudy doctor` |
| Run a meeting | `uv run understudy live --project <slug>` (add `--console` for one terminal) |
| Practice run | `uv run understudy rehearse --project <slug>` |
| Replay a recorded meeting | `uv run understudy replay <meetnotes-session-dir> --max-minutes 2` |
| End a session from anywhere | `uv run understudy stop` |
| No-permission smoke | `uv run understudy live --fake --duration 3 --no-stt` |
| Python tests | `uv run pytest -q` (236 tests, offline) |
| Console tests | `cd console && cargo test` |
| Attach a console | `understudy-console --socket ~/.local/state/understudy/events.sock` |
| Check MCP sources | `uv run understudy context servers --project <slug>` · `context probe "<question>"` |
| Config | `~/.config/understudy/config.json` (`understudy config show` prints the merge) |
| Logs | `~/.local/state/understudy/live.log`, `meetnotes.log` |
| Capture helper | `bin/Understudy.app/Contents/MacOS/understudy-capture` (bundle id `co.pricelove.understudy.capture`) |
| The design | `docs/SPEC.md` · plan: `docs/superpowers/plans/2026-08-26-phase-1-copilot.md` |

## Sources

- `README.md` — install, every user-visible subsystem, the measured CLI latency figures, TCC and signing notes, the voice-harvest story, and the listen-only guarantee.
- `docs/SPEC.md` — the four phases, the evidence-based language split, the capture frame protocol, the hq two-tier corpus design, the brain (providers, models, cache TTL, hard rules, trigger policy), the Phase 3 speaking-path mechanics, and the consent and disclosure policy.
- `docs/superpowers/plans/2026-08-26-phase-1-copilot.md` — the eleven tasks, the global constraints (audio format, socket path length, offline-test rule), the devloop compatibility header, and the self-review recording the two deliberate deferrals.
- `docs/checkpoint-01.md` (untracked) — the hand-off note establishing what had actually been exercised by 2026-08-28, including the 253-event replay of a real meeting.
- `understudy/cli.py`, `config.py` — the complete command surface, the guards that refuse `--voice` outside rehearsal and `--screen` without the LLM path, every configuration key, and the environment/key-file resolution rule.
- `understudy/capture.py`, `frames.py`, `spawn.py`, `stt.py`, `transcript.py` — child lifecycle, the frame protocol and resync, the disclaimed spawn (with its provenance header), the Deepgram socket with reconnect and fatal-error classification, and utterance aggregation.
- `understudy/suggest.py`, `claude_cli.py`, `briefing.py`, `context.py` — the persona loader, the two-tier trigger, the two drafting engines and their shared event contract, the hq pack builder and cache guard, and MCP source resolution, scoping and prefetch.
- `understudy/live.py`, `bus.py`, `console.py`, `meetnotes.py`, `screen.py`, `voice.py`, `watch.py`, `doctor.py`, `filecapture.py` — the session orchestrator, the socket bus with sticky events and replay, one-terminal mode, filing arbitration, screenshare vision, sandbox speech, the mic-watch policy, the live-ping preflight, and file replay.
- `bin/understudy-capture.swift`, `bin/build.sh`, `bin/Understudy.entitlements` — the fork's additions, the bundle build and signing fallback, and the audio-input entitlement.
- `console/src/main.rs`, `app.rs`, `ui.rs` — the socket client and keys, the event enum and focus rules, and the pane layout; `scripts/voice_harvest.py` and `voice_harvest2.py` — the Phase 2 corpus builders and the dual-channel gate.
- Fork divergence established by diffing `bin/understudy-capture.swift` against `~/dev/meetnotes/bin/meetnotes-capture.swift`; repo metadata via `git log`/`git status`/`git ls-files` and `gh api repos/Dev916/understudy`; the absence of an hq project entry via the live hq project list.
- `docs/development/appendix-concurrency-time.md`, `docs/development/rust-async-cancellation-graceful-shutdown.md`, `docs/development/llm-token-cache-efficiency.md` (mech-crate) — the corpus docs this repo cites in its own comments, plus the cache-economics reading for the provider design.
