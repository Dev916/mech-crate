---
title: "Local-First Sync Engines and CRDTs in Practice: Which Architecture, Which Engine, and What Bites in Production"
category: architecture
languages: [typescript, rust, swift, sql]
complexity: advanced
use_cases:
  - deciding whether an app needs a CRDT, a server-authoritative sync engine, Postgres-to-client replication, an event log, or only optimistic UI with a request queue
  - choosing between Automerge, Yjs/yrs, Loro, PowerSync, Electric, Zero, LiveStore and Turso Sync for a web, CLI and iOS product
  - designing the write path, conflict policy and rejected-write handling for offline edits
  - embedding a text CRDT in one field of an otherwise server-authoritative app
  - avoiding the production traps of client-side storage (history growth, eviction, multi-tab corruption, schema drift, authorization)
summary: "State of practice as of 2026-10 for offline-capable, multi-device sync. With a central server, CRDTs are an optimization rather than a requirement; the engines that reached GA (Electric 1.0, Zero 1.0, PowerSync) are all server-authoritative and differ mainly on the write path; CRDT libraries (Automerge 3, Yjs 13, Loro 1.x) fixed memory and load time in 2024 to 2025 and are best embedded per field; and several once-popular options (Replicache, Triplit, InstantDB cloud, cr-sqlite, YSwift) are in maintenance, sunset or stale."
provenance: researched
researched: 2026-10-05
sources:
  - https://mattweidner.com/2024/06/04/server-architectures.html
  - https://mattweidner.com/2025/05/21/text-without-crdts.html
  - https://github.com/loro-dev/loro-docs/blob/main/pages/docs/concepts/when_not_crdt.mdx
  - https://bricolage.io/some-notes-on-local-first-development/
  - https://stack.convex.dev/a-map-of-sync
  - https://adamwiggins.com/posts/why-sync/
  - https://www.powersync.com/blog/local-first-conf-2025-reflections
  - https://www.inkandswitch.com/essay/local-first/
  - https://zero.rocicorp.dev/docs/when-to-use
  - https://zero.rocicorp.dev/docs/offline
  - https://zero.rocicorp.dev/docs/mutators
  - https://zero.rocicorp.dev/docs/auth
  - https://zero.rocicorp.dev/docs/status
  - https://docs.livestore.dev/evaluation/when-livestore/
  - https://docs.livestore.dev/reference/syncing/
  - https://automerge.org/blog/automerge-3/
  - https://github.com/automerge/automerge/releases
  - https://registry.npmjs.org/@automerge/automerge-repo
  - https://www.inkandswitch.com/patchwork/notebook/2024-version-control/08/
  - https://automerge.org/blog/2026-july/
  - https://github.com/yjs/yjs/releases
  - https://github.com/yjs/yjs/blob/main/INTERNALS.md
  - https://docs.yjs.dev/api/document-updates
  - https://discuss.yjs.dev/t/garbage-collection-and-version-snapshotting/1839
  - https://crates.io/crates/yrs
  - https://github.com/loro-dev/loro
  - https://github.com/loro-dev/loro-docs/blob/main/pages/blog/v1.0.mdx
  - https://github.com/loro-dev/loro-docs/blob/main/pages/docs/concepts/shallow_snapshots.mdx
  - https://github.com/josephg/diamond-types
  - https://github.com/vlcn-io/cr-sqlite
  - https://github.com/superfly/corrosion
  - https://github.com/garden-co/jazz
  - https://www.evolu.dev/docs
  - https://supabase.com/blog/triplit-joins-supabase
  - https://www.instantdb.com/essays/instant_team_joins_openai
  - https://tinybase.org/
  - https://electric.ax/blog/2025/03/17/electricsql-1.0-released
  - https://electric.ax/docs/guides/writes
  - https://electric.ax/docs/guides/shapes
  - https://electric.ax/docs/guides/auth
  - https://github.com/electric-sql/electric
  - https://pglite.dev/docs/sync
  - https://docs.powersync.com/installation/app-backend-setup/writing-client-changes
  - https://docs.powersync.com/usage/lifecycle-maintenance/handling-update-conflicts
  - https://docs.powersync.com/architecture/consistency
  - https://docs.powersync.com/usage/sync-rules
  - https://docs.powersync.com/resources/feature-status
  - https://github.com/powersync-ja/powersync-service/blob/main/LICENSE
  - https://replicache.dev/
  - https://docs.turso.tech/sync
  - https://docs.turso.tech/features/embedded-replicas/introduction
  - https://turso.tech/blog/turso-offline-sync-public-beta
  - https://docs.convex.dev/client/react/optimistic-updates
  - https://sqlite.org/wasm/doc/trunk/persistence.md
  - https://github.com/rhashimoto/wa-sqlite
  - https://github.com/TanStack/db
  - https://arxiv.org/abs/2305.00583
  - https://www.inkandswitch.com/peritext/
  - https://arxiv.org/abs/2409.14252
  - https://josephg.com/blog/crdts-go-brrr/
  - https://github.com/dmonad/crdt-benchmarks
  - https://github.com/loro-dev/loro-docs/blob/main/pages/docs/performance/native.mdx
  - https://www.inkandswitch.com/keyhive/notebook/
  - https://archive.jlongster.com/using-crdts-in-the-wild
  - https://github.com/actualbudget/actual
  - https://www.inkandswitch.com/cambria/
  - https://github.com/loro-dev/loro-docs/blob/main/pages/docs/advanced/undo.mdx
  - https://www.figma.com/blog/how-figmas-multiplayer-technology-works/
  - https://www.figma.com/blog/realtime-editing-of-ordered-sequences/
  - https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/
  - https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria
  - https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite
  - https://www.notion.com/blog/how-we-made-notion-available-offline
  - https://github.com/wzhudev/reverse-linear-sync-engine
  - https://linear.app/now/scaling-the-linear-sync-engine
  - https://crates.io/crates/automerge
  - https://crates.io/crates/samod
  - https://github.com/powersync-ja/powersync-swift
  - https://github.com/automerge/automerge-swift
  - https://github.com/loro-dev/loro-swift
  - https://github.com/y-crdt/yswift
  - https://news.ycombinator.com/item?id=45333021
  - https://marcobambini.substack.com/p/why-local-first-apps-havent-become
  - https://antoine.fi/sqlite-sync-engine-with-reactivity
  - https://github.com/tursodatabase/turso
  - https://crates.io/crates/loro
  - https://crates.io/crates/diamond-types
  - https://github.com/livestorejs/livestore
  - https://github.com/automerge/automerge-repo-swift
---

# Local-First Sync Engines and CRDTs in Practice

Companion to `appendix-consistency-models.md`, which covers the theory (state-based and operation-based CRDTs, G-Counter, PN-Counter, LWW register, OR-Set, strong eventual consistency). This doc is about the engineering decision: what to build an offline-capable, multi-device app on in late 2026, and what goes wrong once it ships. Bracketed numbers refer to the Sources section at the end. Version numbers and project statuses were checked on 2026-10-05 and will drift; the closing paragraph of the Synthesis section lists what to re-check.

## 1. The first question is who orders the writes

**With a central server, a CRDT is optional.** Matthew Weidner's survey of server architectures concludes that in the centralized model "CRDTs and OT are merely optimizations over server reconciliation", and recommends application-level mutations with server reconciliation: on receiving remote changes the client undoes its pending mutations, applies the server's, then redoes its own [1]. The same holds for text: with a server ordering operations, per-character IDs, "insert after" operations and tombstones are enough, and only the decentralized variant with Lamport timestamps turns into an RGA-style CRDT [2].

**CRDTs merge; they do not reject.** Loro's own documentation lists the cases where a CRDT is the wrong tool: hard invariants such as a non-negative balance, exclusive ownership such as room booking, uniqueness constraints, authorization decisions that must be enforced at write time, and referential integrity. Its recommended alternatives are a transactional authority, consensus, or a hybrid where the CRDT holds drafts and the server holds the authoritative booking [3]. Kyle Mathews put the same concern as "CRDTs converge, but to where?" [4].

**"Sync engine" and "local-first" are now separate terms.** The 2019 Ink & Switch essay set seven ideals (no spinners, work not trapped on one device, network optional, seamless collaboration, the long now, security and privacy by default, user ownership) and already flagged history growth, schema evolution and access control as open problems [8]. By 2025 Adam Wiggins, a co-author, was describing local-first as "a set of principles to give users ownership of their work" and a superset of, or overlap with, sync [6]. A PowerSync recap of Local-First Conf 2025 reports a separate SyncConf track for sync engines as a UX tool [7]. Zero's documentation states the split outright: "Zero is not local-first. It's a client-server system with an authoritative server." [9]

**A taxonomy that still holds.** Mathews' 2023 grouping was replicated data structures (Yjs, Automerge), replicated database tables (ElectricSQL, PowerSync) and replication as protocol (Replicache) [4]. Convex's "map of sync" adds nine axes to place a product on: data size, update rate, structure; input latency, offline support, concurrent clients; centralization, flexibility, consistency [5]. Updated for 2026 by this doc, the products below fall into five architectures (the grouping is this doc's own, extending [4] and [5]):

| Architecture | Who resolves conflicts | Examples | Offline writes |
|---|---|---|---|
| Optimistic UI plus request queue | Your existing API | hand-rolled; TanStack DB as client store [56] | Only what you queue |
| Read-path replication from Postgres | Your API (writes are yours) | Electric [38] | Yours to build |
| Server-authoritative sync with client mutation replay | Server re-runs mutators or accepts uploads | Zero [11], PowerSync [43], Replicache [49], Linear's engine [74] | PowerSync yes; Zero no [10] |
| Event log with client rebase | Global total order of events | LiveStore [15], Actual Budget's message log [64] | Yes |
| CRDT documents | The data type's merge function [1] | Automerge, Yjs, Loro [16][21][27] | Yes, including peer to peer [1] |

## 2. Sync engines: what each does on the write path

**Electric.** Rebuilt from scratch in 2024 and declared 1.0 GA on 2025-03-17 [37]. It syncs data out of Postgres only: "Electric does not do write-path sync." The docs describe four write patterns (online writes, optimistic state, shared persistent optimistic state, through-the-database sync) and leave merge logic and rollback of server-rejected offline writes to you [38]. A shape syncs "data from a single table" and shape definitions are immutable [39]. Authorization is at the HTTP layer: "shapes are just resources", guarded by a proxy or a gatekeeper that issues shape-scoped tokens [40]. The repository's tagline is now "The agent platform built on sync" and the project lives at electric.ax; the sync service is still released (1.8.1, 2026-09-07) [41]. PGlite's Electric sync plugin is still labelled alpha and does not sync local writes out or resolve conflicts [42]. One practitioner who tried Electric with PGlite for a single-player app in 2025 reported minute-long startups before compaction existed and memory leaks with PGlite live queries, and moved to wa-sqlite with timestamp polling [84].

**PowerSync.** Local SQLite on the client; writes go into an upload queue and your `uploadData()` sends them to your own backend [43]. The documented default is per-field last-write-wins by arrival order: "the last update (as received by the server) to each individual field wins", customizable in your backend [44]. The client does not advance to a new checkpoint while its own mutations are unacknowledged, and the docs call the model causal+ consistency [45]. Returning a 4xx from the upload endpoint "will block the PowerSync client's upload queue", so rejections have to be handled as data, not as HTTP errors [43]. Partial sync is configured with Sync Streams; the older Sync Rules are now labelled legacy in the docs [46]. Source databases: Postgres and MongoDB GA, MySQL and SQL Server beta [47]. The service is FSL-1.1-ALv2 (source-available, converting to Apache-2.0 after two years) [48]; the Swift SDK is Apache-2.0 and GA (1.16.3, 2026-10-02) [78][47].

**Zero (Rocicorp).** GA as of March 2026 [13]. A `zero-cache` service holds a Postgres replica and serves incrementally maintained queries; it is Postgres-only, supports "only TypeScript clients", and is recommended for datasets under 100GB [9]. Writes are custom mutators that run optimistically on the client and then again, authoritatively, at your server's push endpoint [11]; permissions are ordinary server code [12]. Offline reads work but "writes are rejected", for a stated reason: "Foreign keys and other constraints can pass while offline, but break when the user reconnects." [10] Its predecessor Replicache "is now in maintenance mode" and existing users are told to migrate to Zero [49].

**LiveStore.** Event-sourced: events are the source of truth and are materialized into SQLite [15]. Sync is Git-like: "Local pending events which haven't been pushed yet need to be rebased on top of the latest upstream events before they can be pushed", which yields a global total order [15]. The project says not to use it when an existing database is the source of truth, and that client data must fit in an in-memory SQLite database [14]. Version 0.4.0 (2026-06-02), Apache-2.0, documented as beta [88].

**Turso.** Two generations. libSQL embedded replicas read locally and send writes to the remote primary [51]. Turso Sync, on the Rust rewrite of the engine, accepts local writes with explicit `push()` and `pull()` and resolves conflicts as "last push wins" [50]. The March 2025 beta announcement said it was not recommended for production and had no durability guarantees [52], and the engine tagged `v0.8.2-pre.2` on 2026-10-02 and a non-pre-release `v0.8.2` on 2026-10-06 [85].

**Convex.** Server-authoritative; optimistic updates are rolled back when the mutation completes [53].

**Status changes to know before picking.**

- Triplit joined Supabase on 2025-10-08 with the statement "Our focus isn't to directly integrate Triplit into our platform" [34].
- InstantDB's team joined OpenAI in August 2026 and the hosted service is being sunset; the code remains open source for self-hosting [35].
- Jazz is mid-rewrite: the repository README announces a 2.0 alpha "with an entirely new API" [32].
- cr-sqlite has had no release since v0.16.3 (2024-01-17), though the repository still receives community build fixes [30]. Fly.io's Corrosion, which builds on it for server-side replication, reached 1.0 in May 2026 [31].

## 3. CRDT libraries: current state

| Library | Version checked | Core | Licence | Notes |
|---|---|---|---|---|
| Automerge | JS 3.5.0, crate 0.12.0 (2026-09-16) [17][76] | Rust | MIT | JSON-like documents, full history kept |
| Yjs | 13.6.33 stable; v14 at rc.28 [21] | JavaScript | MIT | Largest web editor ecosystem |
| yrs (y-crdt) | 0.28.0 (2026-09-17) [25] | Rust | MIT | Rust port of Yjs |
| Loro | crate 1.16.2, npm 1.16.4 [26][86] | Rust | MIT | Fugue text, rich text, movable list and tree, LWW map [27] |
| Diamond Types | crate 1.0.0 from 2022 [87] | Rust | ISC | "WIP", plain text only, published crate out of date [29] |

**Automerge 3** (July 2025) keeps the compressed columnar format in memory at runtime. The team's figures: memory "cut ... by over 10x", pasting Moby Dick consumed 700Mb in Automerge 2 and 1.3Mb in Automerge 3, and a document that had not loaded after 17 hours loaded in 9 seconds [16]. The file format is unchanged from v2 [16]. Automerge stores every change by design ("Automerge never deletes anything") [19]. The `latest` npm tag of `@automerge/automerge-repo` currently points at a 2.6.0 alpha, so pin the version deliberately [18]. A new sync system (Subduction) and end-to-end encryption via Keyhive are in progress [20].

**Yjs** discards the content of deleted items when garbage collection is on and keeps no record of when or by whom an item was deleted [22]; `Y.mergeUpdates` does not garbage-collect [23]. Snapshots for version history need GC disabled, and a production user reported that documents with GC off were "awful for performance, disk space, and network throughput" [24]. That is a forum report, not a measurement.

**Loro** reached 1.0 on 2024-10-23 with a stable encoding [27]. Shallow snapshots work "like Git's shallow clone": they keep current state and drop old history, with the limit that "Peers can only sync if they have versions after the shallow snapshot point" [28]. On a real 1,659,541-operation document, Loro reports snapshot import falling from 17.3ms to 1.15ms, and 375µs for a depth-1 shallow snapshot (M1 MacBook Pro, 1.0.0-beta.1 against 0.16.12) [27].

**Others.** Evolu is SQLite-based, TypeScript, end-to-end encrypted by default, and syncs through relays [33]. TinyBase's `MergeableStore` "acts as a native CRDT" with pluggable synchronizers [36].

## 4. Text and sequences

- **Interleaving.** When two users insert at the same position concurrently, some algorithms interleave the two passages character by character. The Fugue paper defines "maximal non-interleaving" and gives algorithms that satisfy it [57]; Loro's text type uses Fugue [27].
- **Rich text.** Peritext shows that rich text cannot be modelled as plain text with control characters or as a JSON tree without anomalies; formatting marks anchor to character IDs and each mark type has its own expand behaviour. Block structure (headings, lists, tables) was out of scope [58].
- **Eg-walker** (Gentle and Kleppmann, EuroSys 2025) stores the event graph of index-based operations and builds CRDT state only transiently when merging concurrent edits. The abstract's claim: "Compared to existing CRDTs, it consumes an order of magnitude less memory in the steady state, and loading a document from disk is orders of magnitude faster. Compared to OT, merging long-running branches is orders of magnitude faster." It covers plain text [59]. Loro says it "closely resembles Eg-walker in terms of algorithmic properties" while persisting more ID and integrity data [27].
- **Ordered lists without a CRDT.** Figma uses fractional indexing with the server assigning a unique position on collision [69].

**Benchmarks, with their conditions.** Joseph Gentle's 2021 run of one editing trace (author-reported: Diamond Types is his own library) (about 260k edits) took 291s in Automerge 1.0 preview, 0.97s in Yjs 13.5.5 and 0.056s in native Diamond Types; the point was that data structures, not the algorithm, dominated [60]. The crdt-benchmarks B4 table (Yjs 13.6.11, Loro 0.10.1, Automerge 2.1.10) gives parse times of 39ms, 13ms and 1,805ms [61]. Loro's native table (M2 Max, 2024-10-18) gives decode times of 0.189ms for Loro, 2.19ms for diamond-types, 3.82ms for yrs and 506.30ms for automerge, with Loro's own caveat that such numbers "serve as indicators of the absence of performance pitfalls rather than as measures of which project is superior" [62]. Every comparative table found predates Automerge 3 [16][61][62].

## 5. Production pitfalls

**History and tombstone growth.** Named as an open problem in 2019 [8]. Current answers differ: Automerge keeps everything, compressed [19][16]; Yjs discards deleted content unless you need versions [22][24]; Loro offers shallow snapshots at the cost of syncing with peers behind the cut [28]. Actual Budget's per-field message log grows forever, which its author lists as a downside [64].

**Invariants.** Uniqueness, balances and foreign keys need a coordinating authority [3]; this is Zero's stated reason for rejecting offline writes [10]. PowerSync's documented options for a write the server will not accept are to relax the constraint, block until resolved, persist the failure to a separate table, or discard it [45].

**Authorization and partial sync.** In server-authoritative engines the server decides what leaves the database: Electric at the shape request [40], PowerSync through stream parameters bound to the authenticated user [46], Zero in server-side mutators and query context [11][12]. With full replicas on every device, access control "must travel with the data itself"; Ink & Switch's Keyhive (capabilities plus group key agreement) addresses this and is marked "DO NOT use this release in production applications" [63].

**Clocks.** Actual Budget orders per-field messages with hybrid logical clocks (wall time, counter, node id), uses a Merkle tree to find where two replicas diverge, and allows one minute of clock drift, rejecting messages from a client whose clock differs by more [64]. It is still actively released [65].

**Schema evolution.** Old clients keep writing old shapes, which is the problem Cambria set out to solve [66]. Cambria proposed bidirectional lenses applied on read and remained a research project; the write-up reports that the team found its own prototype issue tracker "too unstable to be our only system of record during the project" [66]. With an event log, read models are rebuilt cheaply but every historical event must stay readable [14].

**Undo.** Undo should revert only the local user's operations, which is how Loro's `UndoManager` behaves [67]; Figma's rule is that undoing a lot, copying something, and redoing back to the present should leave the document unchanged [68].

**Browser storage.** Safari's tracking prevention deletes "all of a website's script-writable storage after seven days of Safari use without user interaction on the site", with installed home-screen web apps exempt [70]. Elsewhere, best-effort storage is evicted a whole origin at a time under pressure unless `navigator.storage.persist()` is granted [71]. For SQLite in the browser, the official `opfs` VFS needs COOP/COEP headers; `opfs-sahpool` does not and has "the highest OPFS performance" but "does not support multiple simultaneous connections" [54]. wa-sqlite is a maintained alternative [55].

**Multiple tabs.** Notion found rows "with the same ID but different content" when several tabs wrote to OPFS concurrently. The fix was a SharedWorker that routes all queries to one active tab, with Web Locks to detect when that tab closes. They report page navigation 20 percent faster after the move to WASM SQLite [72].

**End-to-end encryption.** Evolu encrypts by default and syncs through relays that cannot read the data [33]; Keyhive aims for the same with Automerge [63].

**Vendor-reported testing.** Electric cites Antithesis testing before its 1.0 [37].

## 6. What shipped products chose

- **Figma.** "Figma isn't using true CRDTs though." Server-authoritative last-write-wins per property per object; "simultaneous editing of the same text value doesn't work in Figma." [68]
- **Linear.** A custom engine in which the server assigns a monotonically increasing sync id giving a total order, with a client object graph persisted to IndexedDB and a transaction queue. This rests on a community reverse-engineering write-up [74]; Linear's own account is a recorded talk [75].
- **Notion.** Offline mode (December 2025) turned the SQLite cache into a durable store, tracks each reason a page is offline ("We should only remove a page from the offline set when the last reason disappears"), and "pages that are marked as available offline are dynamically migrated to our new CRDT data model for conflict-resolution" [73]. The post does not name the CRDT.
- **Actual Budget.** Per-field LWW messages in SQLite, hybrid logical clocks, a Merkle tree and a small sync server [64].

**What practitioners argue (opinion, not established fact).** A September 2025 essay by the founder of SQLite AI argued that local-first apps are rare because sync makes every app a distributed system, and proposed hybrid logical clocks plus CRDTs over SQLite [83]. In the 485-comment Hacker News thread, commenters replied that adopting a CRDT forces its data model onto the whole application, and that, as one commenter put it, "There is definitely no general solution but for some domains there may be acceptable solutions", with merges of code best left to the human user [82].

## 7. Rust and Swift

| Need | Rust | Swift |
|---|---|---|
| Automerge | `automerge` 0.12.0; `samod` 0.15.0 is published alongside it [76][77] | automerge-swift 0.7.2 (2025-12-20) [79]; automerge-repo-swift last pushed 2024-11 [89] |
| Yjs | `yrs` 0.28.0 [25] | YSwift 0.2.1, last pushed 2024-07 [81] |
| Loro | `loro` 1.16.2 [86] | loro-swift 1.16.2, README says "experimental" [80] |
| PowerSync | SDK listed as beta [47] | GA, 1.16.3 [78][47] |
| Zero | none; TypeScript clients only [9] | none [9] |

## 8. Common misconceptions

- *"Offline support means I need CRDTs."* With a server you need mutation replay and a conflict policy; CRDTs are an optimization on top [1].
- *"Electric syncs my writes."* It does not [38].
- *"Zero is local-first."* Its own docs say it is not, and it rejects offline writes [9][10].
- *"A CRDT will keep my constraints."* It merges and cannot reject [3].
- *"Automerge is too slow and memory-hungry."* That was true of 1.x and 2.x [60][61]; version 3 changed the in-memory representation [16], and the public comparison tables have not been rerun [61][62].
- *"Data in IndexedDB or OPFS is durable."* Safari can delete it after seven days without interaction, and other browsers evict under pressure unless persistence is granted [70][71].

## Synthesis (inferred)

These are inferences drawn across the sources above; they are not themselves cited.

**A decision ladder for a small team shipping web, CLI and iOS on Postgres.**

1. *Online-mostly, Postgres is the truth, you want instant UI.* Optimistic UI plus a request queue against the existing API. Add Electric shapes if live partial replicas on the read path are worth another service.
2. *Web-only, Linear-style, no offline writes.* Zero. A native iOS or CLI client rules it out.
3. *Real offline writes across web, iOS and Rust or CLI, Postgres as truth.* PowerSync is the only surveyed engine with a GA Swift SDK. You write the upload endpoint and the conflict policy. Budget design time for the rejected-write experience, and read the FSL licence.
4. *Per-user or per-workspace data with no existing source-of-truth database.* An event log materialized into SQLite. LiveStore if TypeScript-only and beta is acceptable; otherwise hand-roll the Actual Budget pattern, which ports to Rust and Swift because it is only SQLite, a clock and a message table.
5. *Rich-text or structured co-editing inside a record.* Embed a CRDT for that field alone, stored as a blob in a row and carried by whichever engine was chosen above. This is what Linear and Notion appear to do. Loro if Rust and Swift both matter; Yjs if the web editor ecosystem matters most; Automerge if full history is a feature.
6. *No trusted server, or end-to-end encryption as a requirement.* Only here are CRDTs needed end to end. Accept that authorization has no production-grade answer yet and that server-side invariants are unavailable.

**Avoid for new builds as of this writing:** InstantDB's hosted service, Triplit, Replicache, YSwift, cr-sqlite on clients, Jazz 2 until it leaves alpha, PGlite sync for anything with writes, and Turso Sync for data you cannot lose.

**The engines converged on the same shape.** Electric, PowerSync and Zero all reached GA as server-authoritative systems that differ mainly in how much of the write path they own: none, a queue, or the mutators. The interesting choice is therefore not CRDT against no CRDT but how a rejected offline write is shown to the user. Design that screen before choosing the engine.

**Per-field LWW covers more than it seems.** Figma, Linear, PowerSync's default and Actual all use last-write-wins on fields. The cases where it fails are concurrent edits to one long text value and ordered lists, and both have narrow fixes (a text CRDT in that column, fractional indexing).

**Bindings lag cores.** The Swift bindings for Automerge and Yjs trail their Rust cores by many releases, and Loro's are version-locked but still labelled experimental. For an iOS client, check that the binding can read documents written by the core version the other platforms use before committing.

**Browser checklist regardless of engine.** Request persistent storage; assume the local copy can vanish and make full rehydration from the server a tested path; run one writer per origin; keep uniqueness, balances and foreign keys on the server; make undo local; version the mutation or event schema from the first release.

**Connection to our own repos.** hq and meetnotes describe themselves as local-first in the single-machine sense (one laptop, local Postgres). None of the above applies until one of them needs a second device or a second user writing offline; at that point rung 3 or 4 is the likely fit.

**What would change this doc.** Zero supporting offline writes or native clients; Yjs 14 going stable; an independent benchmark that includes Automerge 3; Keyhive leaving pre-alpha; Turso documenting durability guarantees for Sync (the engine itself left pre-release with `v0.8.2` on 2026-10-06 [85]); Jazz 2 leaving alpha; Electric dropping or de-emphasizing the Postgres sync service.

## Sources

Numbers match the order of the `sources:` list in the frontmatter.

1. Matthew Weidner, "Architectures for Central Server Collaboration", 2024-06-04. https://mattweidner.com/2024/06/04/server-architectures.html
2. Matthew Weidner, "Collaborative Text Editing without CRDTs or OT", 2025-05-21. https://mattweidner.com/2025/05/21/text-without-crdts.html
3. Loro docs, "When not to use CRDTs". https://github.com/loro-dev/loro-docs/blob/main/pages/docs/concepts/when_not_crdt.mdx
4. Kyle Mathews, "Some notes on local-first development", 2023-09-08. https://bricolage.io/some-notes-on-local-first-development/
5. Convex, "A Map of Sync". https://stack.convex.dev/a-map-of-sync
6. Adam Wiggins, "Why sync", 2025-09. https://adamwiggins.com/posts/why-sync/
7. PowerSync, "Local-First Conf 2025 reflections", 2025-06-04 (vendor recap). https://www.powersync.com/blog/local-first-conf-2025-reflections
8. Ink & Switch, "Local-first software", 2019-04. https://www.inkandswitch.com/essay/local-first/
9. Zero docs, "When to use Zero". https://zero.rocicorp.dev/docs/when-to-use
10. Zero docs, "Offline". https://zero.rocicorp.dev/docs/offline
11. Zero docs, "Mutators". https://zero.rocicorp.dev/docs/mutators
12. Zero docs, "Auth". https://zero.rocicorp.dev/docs/auth
13. Zero docs, "Status". https://zero.rocicorp.dev/docs/status
14. LiveStore docs, "When to use LiveStore". https://docs.livestore.dev/evaluation/when-livestore/
15. LiveStore docs, "Syncing". https://docs.livestore.dev/reference/syncing/
16. Automerge, "Automerge 3.0", 2025-07. https://automerge.org/blog/automerge-3/
17. Automerge releases. https://github.com/automerge/automerge/releases
18. npm registry, `@automerge/automerge-repo`. https://registry.npmjs.org/@automerge/automerge-repo
19. Ink & Switch, Patchwork notebook, 2024-03-26. https://www.inkandswitch.com/patchwork/notebook/2024-version-control/08/
20. Automerge, "This Month in Automerge: July 2026". https://automerge.org/blog/2026-july/
21. Yjs releases. https://github.com/yjs/yjs/releases
22. Yjs INTERNALS.md. https://github.com/yjs/yjs/blob/main/INTERNALS.md
23. Yjs docs, "Document updates". https://docs.yjs.dev/api/document-updates
24. Yjs forum, "Garbage collection and version snapshotting" (user report). https://discuss.yjs.dev/t/garbage-collection-and-version-snapshotting/1839
25. crates.io, `yrs`. https://crates.io/crates/yrs
26. Loro repository. https://github.com/loro-dev/loro
27. Loro, "Loro 1.0", 2024-10-23. https://github.com/loro-dev/loro-docs/blob/main/pages/blog/v1.0.mdx
28. Loro docs, "Shallow snapshots". https://github.com/loro-dev/loro-docs/blob/main/pages/docs/concepts/shallow_snapshots.mdx
29. Diamond Types repository. https://github.com/josephg/diamond-types
30. cr-sqlite repository. https://github.com/vlcn-io/cr-sqlite
31. Corrosion repository. https://github.com/superfly/corrosion
32. Jazz repository. https://github.com/garden-co/jazz
33. Evolu docs. https://www.evolu.dev/docs
34. Supabase, "Triplit joins Supabase", 2025-10-08. https://supabase.com/blog/triplit-joins-supabase
35. Instant, "Instant team joins OpenAI", 2026-08. https://www.instantdb.com/essays/instant_team_joins_openai
36. TinyBase home page. https://tinybase.org/
37. Electric, "Electric 1.0 released", 2025-03-17. https://electric.ax/blog/2025/03/17/electricsql-1.0-released
38. Electric docs, "Writes". https://electric.ax/docs/guides/writes
39. Electric docs, "Shapes". https://electric.ax/docs/guides/shapes
40. Electric docs, "Auth". https://electric.ax/docs/guides/auth
41. Electric repository. https://github.com/electric-sql/electric
42. PGlite docs, "Sync". https://pglite.dev/docs/sync
43. PowerSync docs, "Writing client changes". https://docs.powersync.com/installation/app-backend-setup/writing-client-changes
44. PowerSync docs, "Handling update conflicts". https://docs.powersync.com/usage/lifecycle-maintenance/handling-update-conflicts
45. PowerSync docs, "Consistency". https://docs.powersync.com/architecture/consistency
46. PowerSync docs, "Sync Rules". https://docs.powersync.com/usage/sync-rules
47. PowerSync docs, "Feature status". https://docs.powersync.com/resources/feature-status
48. PowerSync service licence. https://github.com/powersync-ja/powersync-service/blob/main/LICENSE
49. Replicache home page (maintenance notice). https://replicache.dev/
50. Turso docs, "Sync". https://docs.turso.tech/sync
51. Turso docs, "Embedded replicas". https://docs.turso.tech/features/embedded-replicas/introduction
52. Turso, "Offline sync public beta", 2025-03-31. https://turso.tech/blog/turso-offline-sync-public-beta
53. Convex docs, "Optimistic updates". https://docs.convex.dev/client/react/optimistic-updates
54. SQLite, "Persistent storage options" (WASM). https://sqlite.org/wasm/doc/trunk/persistence.md
55. wa-sqlite repository. https://github.com/rhashimoto/wa-sqlite
56. TanStack DB repository. https://github.com/TanStack/db
57. Weidner and Kleppmann, "The Art of the Fugue". https://arxiv.org/abs/2305.00583
58. Litt, Lim, Kleppmann, van Hardenberg, "Peritext". https://www.inkandswitch.com/peritext/
59. Gentle and Kleppmann, "Collaborative Text Editing with Eg-walker: Better, Faster, Smaller", EuroSys 2025. https://arxiv.org/abs/2409.14252
60. Joseph Gentle, "5000x faster CRDTs", 2021-07-31. https://josephg.com/blog/crdts-go-brrr/
61. crdt-benchmarks. https://github.com/dmonad/crdt-benchmarks
62. Loro docs, native benchmarks. https://github.com/loro-dev/loro-docs/blob/main/pages/docs/performance/native.mdx
63. Ink & Switch, Keyhive notebook. https://www.inkandswitch.com/keyhive/notebook/
64. James Long, "Using CRDTs in the Wild", 2019. https://archive.jlongster.com/using-crdts-in-the-wild
65. Actual Budget repository. https://github.com/actualbudget/actual
66. Ink & Switch, "Project Cambria", 2020-10. https://www.inkandswitch.com/cambria/
67. Loro docs, "Undo". https://github.com/loro-dev/loro-docs/blob/main/pages/docs/advanced/undo.mdx
68. Evan Wallace, "How Figma's multiplayer technology works", 2019-10-16. https://www.figma.com/blog/how-figmas-multiplayer-technology-works/
69. Evan Wallace, "Realtime editing of ordered sequences". https://www.figma.com/blog/realtime-editing-of-ordered-sequences/
70. WebKit, "Full Third-Party Cookie Blocking and More", 2020-03-24. https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/
71. MDN, "Storage quotas and eviction criteria". https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria
72. Notion, "How we sped up Notion in the browser with WASM SQLite". https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite
73. Notion, "How we made Notion available offline", 2025-12-11. https://www.notion.com/blog/how-we-made-notion-available-offline
74. "Reverse engineering of Linear's sync engine" (community). https://github.com/wzhudev/reverse-linear-sync-engine ; HN discussion https://news.ycombinator.com/item?id=44123131
75. Linear, "Scaling the Linear Sync Engine" (video), 2023-06-29. https://linear.app/now/scaling-the-linear-sync-engine
76. crates.io, `automerge`. https://crates.io/crates/automerge
77. crates.io, `samod`. https://crates.io/crates/samod
78. PowerSync Swift SDK. https://github.com/powersync-ja/powersync-swift
79. automerge-swift. https://github.com/automerge/automerge-swift
80. loro-swift. https://github.com/loro-dev/loro-swift
81. YSwift. https://github.com/y-crdt/yswift
82. Hacker News, "Why haven't local-first apps become popular?" (505 points, 485 comments), 2025-09-22. https://news.ycombinator.com/item?id=45333021
83. Marco Bambini, "Why local-first apps haven't become popular", 2025-09-22. https://marcobambini.substack.com/p/why-local-first-apps-havent-become
84. Antoine, "Lessons learned from building a sync-engine and reactivity system with SQLite", 2025-08-15. https://antoine.fi/sqlite-sync-engine-with-reactivity ; HN discussion https://news.ycombinator.com/item?id=44929478
85. Turso Database repository. https://github.com/tursodatabase/turso
86. crates.io, `loro`. https://crates.io/crates/loro
87. crates.io, `diamond-types`. https://crates.io/crates/diamond-types
88. LiveStore repository. https://github.com/livestorejs/livestore
89. automerge-repo-swift. https://github.com/automerge/automerge-repo-swift
