# RAG quick start with SQLite vectors using Turso SQL

## Purpose

Add small and fast RAG to any Node or TypeScript codebase with zero extra services. Works with a local `rag.db` file or a Turso database URL. Exposes an MCP tool for Cursor and other MCP aware clients.

## Install

```bash
pnpm add @libsql/client openai zod
# or
npm i @libsql/client openai zod
```

## Project Blueprint (repeatable across repos)

1. **Create a dedicated package** (e.g. `brain/rag`) so RAG code, dependencies, and scripts stay isolated from the primary app.
2. **Install shared deps** inside that package: `@libsql/client`, `openai`, `zod`, `simple-git`, `globby`, `yargs`, `dotenv`, plus `tsx` for TypeScript scripts.
3. **Layer environment variables** with a small helper that loads `.env` files from the package directory, its parent (e.g. `brain/.env`), and the repo root. Respect `TURSO_URL`, `TURSO_AUTH_TOKEN`, `OPENAI_API_KEY`, and optional `OPENAI_EMBED_MODEL`, falling back to `file:rag.db` only when unset.
4. **Centralize the `RAG` class** so it:
   - Lazily instantiates OpenAI and writes embeddings as `F32_BLOB(1536)` while also accepting comma-separated or JSON string embeddings when reading back from Turso.
   - Detects available libSQL vector functions (`distance`, `vector_distance`, `vector_distance_l2`, etc.) and gracefully falls back to client-side L2 scoring when the extension is missing.
   - Provides helpers for pins, document upserts, deletions, and retrieval with optional always-include sections.
5. **Ship repeatable scripts** (invoked via `yarn --cwd brain/rag <cmd>`):
   - `ingest` – glob project files, chunk, and upsert docs.
   - `pins` – seed project overview, coding standards, and culture snippets.
   - `query` – fetch context from the CLI with optional pins and `k`.
   - `git-index` – walk commit history using `simple-git`, chunk each file, and store snapshots under `git://<sha>/<path>` sources.
   - `migrate` – copy a local `rag.db` into Turso (supports `--dryRun`).
   - `mcp` / `mcp:self-test` – expose RAG tools to MCP-aware assistants.
6. **Alias scripts at the repo root** (e.g. `yarn rag:ingest`, `yarn rag:git-index`, `yarn rag:migrate`) so team members never cd into the package manually.
7. **Document the workflow** (install → pins → ingest → git-index → migrate) in the main README so every project boots the same way.

## Schema

This schema supports documents, chunked text, and a simple pin system for always include context.

```sql
-- create tables
CREATE TABLE IF NOT EXISTS docs (
  id INTEGER PRIMARY KEY,
  source TEXT NOT NULL,
  title TEXT,
  created_at INTEGER DEFAULT (unixepoch()),
  updated_at INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS chunks (
  id INTEGER PRIMARY KEY,
  doc_id INTEGER NOT NULL REFERENCES docs(id) ON DELETE CASCADE,
  ord INTEGER NOT NULL,
  text TEXT NOT NULL,
  embedding F32_BLOB(1536),
  created_at INTEGER DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS pins (
  id INTEGER PRIMARY KEY,
  key TEXT UNIQUE NOT NULL,
  text TEXT NOT NULL,
  embedding F32_BLOB(1536),
  created_at INTEGER DEFAULT (unixepoch())
);

-- add indexes
CREATE INDEX IF NOT EXISTS idx_chunks_doc_ord ON chunks(doc_id, ord);
CREATE INDEX IF NOT EXISTS idx_chunks_embedding ON chunks(embedding);
CREATE INDEX IF NOT EXISTS idx_pins_embedding ON pins(embedding);
```

## Minimal Node utility

Single file utility that you can import anywhere. Saves embeddings, retrieves top chunks, and merges pinned context. Uses OpenAI by default but you can swap the embeddings call.

```ts
// file: rag.ts
import { createClient, Client } from "@libsql/client";
import OpenAI from "openai";

type RAGOpts = {
  url?: string;                 // "file:rag.db" for local or Turso URL
  authToken?: string;           // TURSO auth token if remote
  model?: string;               // embedding model id
};

export class RAG {
  private db: Client;
  private openai: OpenAI;
  private model: string;

  constructor(opts: RAGOpts = {}) {
    this.db = createClient({ url: opts.url ?? "file:rag.db", authToken: opts.authToken });
    this.openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
    this.model = opts.model ?? "text-embedding-3-small";
  }

  async init() {
    await this.db.execute(`
      CREATE TABLE IF NOT EXISTS docs(
        id INTEGER PRIMARY KEY,
        source TEXT NOT NULL,
        title TEXT,
        created_at INTEGER DEFAULT (unixepoch()),
        updated_at INTEGER DEFAULT (unixepoch())
      );
    `);
    await this.db.execute(`
      CREATE TABLE IF NOT EXISTS chunks(
        id INTEGER PRIMARY KEY,
        doc_id INTEGER NOT NULL REFERENCES docs(id) ON DELETE CASCADE,
        ord INTEGER NOT NULL,
        text TEXT NOT NULL,
        embedding F32_BLOB(1536),
        created_at INTEGER DEFAULT (unixepoch())
      );
    `);
    await this.db.execute(`CREATE INDEX IF NOT EXISTS idx_chunks_doc_ord ON chunks(doc_id, ord);`);
    await this.db.execute(`CREATE INDEX IF NOT EXISTS idx_chunks_embedding ON chunks(embedding);`);
    await this.db.execute(`
      CREATE TABLE IF NOT EXISTS pins(
        id INTEGER PRIMARY KEY,
        key TEXT UNIQUE NOT NULL,
        text TEXT NOT NULL,
        embedding F32_BLOB(1536),
        created_at INTEGER DEFAULT (unixepoch())
      );
    `);
    await this.db.execute(`CREATE INDEX IF NOT EXISTS idx_pins_embedding ON pins(embedding);`);
  }

  private async embed(input: string | string[]) {
    const { data } = await this.openai.embeddings.create({
      model: this.model,
      input
    });
    if (Array.isArray(input)) {
      return data.map(d => new Float32Array(d.embedding));
    }
    return new Float32Array(data[0].embedding);
  }

  async upsertDoc(source: string, title: string | null, chunks: string[]) {
    const doc = await this.db.execute({
      sql: "INSERT INTO docs(source, title) VALUES(?, ?) RETURNING id",
      args: [source, title]
    });
    const docId = Number(doc.rows[0].id);

    const embs = await this.embed(chunks);

    for (let i = 0; i < chunks.length; i++) {
      await this.db.execute({
        sql: "INSERT INTO chunks(doc_id, ord, text, embedding) VALUES(?, ?, ?, ?)",
        args: [docId, i, chunks[i], embs[i]]
      });
    }
    return docId;
  }

  async pin(key: string, text: string) {
    const emb = await this.embed(text);
    await this.db.execute({
      sql: `
        INSERT INTO pins(key, text, embedding) VALUES(?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET text=excluded.text, embedding=excluded.embedding
      `,
    args: [key, text, emb]
    });
  }

  async search(query: string, k = 6) {
    const qe = await this.embed(query);
    const { rows } = await this.db.execute({
      sql: `
        SELECT text, doc_id, ord
        FROM chunks
        ORDER BY distance(embedding, ?)
        LIMIT ?
      `,
      args: [qe, k]
    });
    return rows.map(r => ({
      text: String(r.text),
      docId: Number(r.doc_id),
      ord: Number(r.ord)
    }));
  }

  async retrieveWithPins(query: string, k = 6, pinKeys: string[] = []) {
    const hits = await this.search(query, k);
    let pinText = "";
    for (const key of pinKeys) {
      const res = await this.db.execute({ sql: "SELECT text FROM pins WHERE key = ?", args: [key] });
      if (res.rows.length) pinText += `\n\n[PIN ${key}]\n` + String(res.rows[0].text);
    }
    const context = hits.map(h => h.text).join("\n\n");
    return `${pinText}\n\n${context}`.trim();
  }
}
```

### Usage

```ts
import { RAG } from "./rag";

const rag = new RAG({ url: process.env.TURSO_URL ?? "file:rag.db", authToken: process.env.TURSO_AUTH_TOKEN });
await rag.init();

await rag.upsertDoc("docs/sys-handbook.md", "System Handbook", [
  "Section A purpose and scope ...",
  "Section B deployment notes ...",
  "Section C on call runbook ..."
]);

await rag.pin("system", "Environment map prod and staging. Secrets live in AWS SSM. Queue uses SQS. Horizon manages workers.");

const ctx = await rag.retrieveWithPins("how do I restart failed workers", 6, ["system"]);
```

## MCP adapter

This adapter exposes your RAG utility as MCP tools. It supports add document, pin, and retrieve.

```ts
// file: mcp-rag.ts
import { RAG } from "./rag";
import { z } from "zod";
import { Server, Tool } from "@modelcontextprotocol/sdk/server";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/transports/stdio";

const rag = new RAG({ url: process.env.TURSO_URL ?? "file:rag.db", authToken: process.env.TURSO_AUTH_TOKEN });
await rag.init();

const server = new Server(
  {
    name: "rag mcp",
    version: "1.0.0",
    description: "Tiny RAG over SQLite vectors"
  },
  new StdioServerTransport()
);

server.tool(
  new Tool({
    name: "rag_add_doc",
    description: "Add document with text chunks",
    inputSchema: z.object({
      source: z.string(),
      title: z.string().nullable().optional(),
      chunks: z.array(z.string()).min(1)
    })
  }),
  async ({ source, title, chunks }) => {
    const id = await rag.upsertDoc(source, title ?? null, chunks);
    return { content: [{ type: "text", text: `doc ${id} added` }] };
  }
);

server.tool(
  new Tool({
    name: "rag_pin",
    description: "Pin a small always include text snippet under a key",
    inputSchema: z.object({
      key: z.string(),
      text: z.string().min(1)
    })
  }),
  async ({ key, text }) => {
    await rag.pin(key, text);
    return { content: [{ type: "text", text: `pin ${key} stored` }] };
  }
);

server.tool(
  new Tool({
    name: "rag_retrieve",
    description: "Retrieve relevant context for a query",
    inputSchema: z.object({
      query: z.string(),
      k: z.number().int().min(1).max(20).default(6),
      pins: z.array(z.string()).optional()
    })
  }),
  async ({ query, k, pins }) => {
    const text = await rag.retrieveWithPins(query, k, pins ?? []);
    return { content: [{ type: "text", text }] };
  }
);

server.start();
```

### Package file example

```json
{
  "name": "rag-mcp",
  "private": true,
  "type": "module",
  "scripts": {
    "mcp": "tsx mcp-rag.ts"
  },
  "dependencies": {
    "@libsql/client": "^0.11.0",
    "@modelcontextprotocol/sdk": "^1.2.0",
    "openai": "^4.55.0",
    "tsx": "^4.19.0",
    "zod": "^3.23.8"
  }
}
```

## Cursor MCP config

Open Cursor settings and add this to your MCP servers section. Adjust the command path for your project.

```json
{
  "mcpServers": {
    "rag": {
      "command": "pnpm",
      "args": ["run", "mcp"],
      "env": {
        "OPENAI_API_KEY": "sk replace me",
        "TURSO_URL": "file:rag.db",
        "TURSO_AUTH_TOKEN": ""
      }
    }
  }
}
```

### Tool calls

```
mcp call rag rag_add_doc {"source":"docs/runbook.md","title":"Runbook","chunks":["step one ...","step two ..."]}
mcp call rag rag_pin {"key":"system","text":"Queues use SQS. Horizon manages workers. Alarms page on Slack."}
mcp call rag rag_retrieve {"query":"how to drain workers safely","k":6,"pins":["system"]}
```

## Suggested chunking

Keep chunks between about 500 and 1200 tokens. Preserve section boundaries. A quick splitter is fine to begin and you can evolve it later.

```ts
export function simpleSplit(text: string, maxChars = 1500) {
  const parts: string[] = [];
  let buf = "";
  for (const para of text.split(/\n{2,}/)) {
    if ((buf + "\n\n" + para).length > maxChars) {
      if (buf) parts.push(buf.trim());
      buf = para;
    } else {
      buf = buf ? buf + "\n\n" + para : para;
    }
  }
  if (buf) parts.push(buf.trim());
  return parts;
}
```

## Vector distance compatibility

- Probe libSQL for `distance`, `vector_distance`, `vector_distance_l2`, etc., caching the first helper that succeeds.
- Normalize embeddings read from Turso whether they arrive as blobs, JSON arrays, or comma-delimited strings.
- When no native distance helper exists, compute L2 distance in memory, sort ascending, and return the top `k` so retrieval continues to work on plain SQLite builds.

## Local → Turso migration pattern

- Add a CLI (`yarn rag:migrate --dryRun`) that connects to the local file DB and remote Turso, ensures schema parity, upserts docs by `source`, rewrites chunks, and syncs pins.
- Run the migration after ingesting or git-indexing locally to keep the remote context lake current.

## Embedding provider notes

You can swap OpenAI for any embedding client. The only contract is a function that returns `Float32Array`. For local only workflows consider `nomic embed` or `sentence transformers` and cast to `Float32Array`.

## Pinning strategy

Use pins for small and stable truths such as environment map, escalation rules, coding ground rules, and glossary. Keep each pin under one thousand tokens and keep content updated to avoid stale guidance.

## Safety and privacy notes

Store only text you are comfortable placing in the local project database. For Turso cloud, set a scoped auth token and restrict access. If you log queries, avoid sensitive values and redact secrets before storage.

### Redaction recipe (copy/paste into new repos)

Every implementation should ship a redaction helper that:

1. **Skips sensitive files** outright (`.env*`, `*.pem`, `*.key`, `id_rsa`, `secrets.*`, `credentials.*`, etc.) so they are never read.
2. **Redacts inline secrets** before chunking (OpenAI `sk-...`, GitHub `ghp_...`, AWS access/secret keys, Turso tokens, bearer tokens, PEM blocks, etc.).

Drop this alongside your RAG CLI and call it from both file indexing and git indexing flows:

```ts
// redactor.ts
const skipPatterns = [
  /\.env(\..+)?$/i,
  /\.pem$/i,
  /\.key$/i,
  /\.pfx$/i,
  /\.p12$/i,
  /\.crt$/i,
  /secrets?\./i,
  /credentials?\./i,
  /id_rsa/i
];

const redactions = [
  { regex: /(sk-[A-Za-z0-9]{20,})/g, replacement: "[REDACTED_OPENAI_KEY]" },
  { regex: /(ghp_[A-Za-z0-9]{36})/g, replacement: "[REDACTED_GITHUB_TOKEN]" },
  { regex: /(TURSO_AUTH_TOKEN\s*[:=]\s*)([A-Za-z0-9._-]+)/gi, replacement: "$1[REDACTED_TURSO_TOKEN]" },
  { regex: /((?:AWS)?[_-]?SECRET[_-]?ACCESS[_-]?KEY\s*[:=]\s*)([A-Za-z0-9/+=]{40})/gi, replacement: "$1[REDACTED_AWS_SECRET]" },
  { regex: /((?:AWS)?[_-]?ACCESS[_-]?KEY[_-]?ID\s*[:=]\s*)([A-Z0-9]{20})/gi, replacement: "$1[REDACTED_AWS_KEY]" },
  { regex: /(Bearer\s+)[A-Za-z0-9\-_\.]{20,}/g, replacement: "$1[REDACTED_BEARER]" },
  {
    regex: /(-----BEGIN [A-Z ]+ PRIVATE KEY-----)([\s\S]+?)(-----END [A-Z ]+ PRIVATE KEY-----)/g,
    replacement: "$1\n[REDACTED_PRIVATE_KEY]\n$3"
  }
];

export function shouldSkip(path: string) {
  return skipPatterns.some(pattern => pattern.test(path));
}

export function redact(text: string) {
  return redactions.reduce((acc, { regex, replacement }) => acc.replace(regex, replacement), text);
}
```

**Usage:**

```ts
if (shouldSkipPath(file)) return;
const raw = fs.readFileSync(file, "utf8");
const sanitized = redact(raw);
const chunks = simpleSplit(sanitized, maxChars);
```

Do the same inside your git indexer before chunking commit blobs. This way secrets never enter the RAG lake even if they accidentally live in the repo.

---

## Optional: make `docs.source` unique (helps dedupe)

Add a unique index so repeated indexing of the same logical source (e.g., a specific git blob or file path) overwrites instead of duplicating.

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_source ON docs(source);
```

> If you add this, update insert logic to use `INSERT ... ON CONFLICT(source) DO UPDATE`.

### Helper: upsert by source (drop-in replacement)

```ts
// add inside RAG class
async upsertDocBySource(source: string, title: string | null, chunks: string[]) {
  const embs = await this.embed(chunks);
  // create or fetch doc id by source
  const docRes = await this.db.execute({
    sql: `
      INSERT INTO docs(source, title) VALUES(?, ?)
      ON CONFLICT(source) DO UPDATE SET title=excluded.title, updated_at=unixepoch()
      RETURNING id
    `,
    args: [source, title]
  });
  const docId = Number(docRes.rows[0].id);
  // clear old chunks for this doc
  await this.db.execute({ sql: "DELETE FROM chunks WHERE doc_id = ?", args: [docId] });
  // insert new chunks
  for (let i = 0; i < chunks.length; i++) {
    await this.db.execute({
      sql: "INSERT INTO chunks(doc_id, ord, text, embedding) VALUES(?, ?, ?, ?)",
      args: [docId, i, chunks[i], embs[i]]
    });
  }
  return docId;
}
```

---

## Tiny CLI: `index`, `pin`, `query`

A minimal CLI to index files, pin small truths, and query. Save as `rag-cli.ts` at your project root.

```ts
#!/usr/bin/env tsx
import fs from "node:fs";
import path from "node:path";
import { globby } from "globby";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { RAG } from "./rag";
import { simpleSplit } from "./simpleSplit"; // or inline the function from this doc

async function loadRag() {
  const rag = new RAG({ url: process.env.TURSO_URL ?? "file:rag.db", authToken: process.env.TURSO_AUTH_TOKEN });
  await rag.init();
  return rag;
}

yargs(hideBin(process.argv))
  .command(
    "index <patterns..>",
    "Index files by glob patterns",
    y => y
      .positional("patterns", { type: "string", array: true, describe: "glob(s) to index" })
      .option("title", { type: "string", default: null })
      .option("prefix", { type: "string", default: "file://" }),
    async argv => {
      const rag = await loadRag();
      const files = await globby(argv.patterns as string[], { gitignore: true, absolute: true });
      for (const f of files) {
        const raw = fs.readFileSync(f, "utf8");
        const chunks = simpleSplit(raw, 2000);
        const src = `${argv.prefix}${path.relative(process.cwd(), f)}`;
        await rag.upsertDocBySource ?
          rag.upsertDocBySource(src, argv.title, chunks) :
          rag.upsertDoc(src, argv.title, chunks);
        console.log("indexed", src);
      }
    }
  )
  .command(
    "pin <key> <textOrFile>",
    "Pin a small always-include snippet (pass a path or raw text)",
    y => y.positional("key", { type: "string" }).positional("textOrFile", { type: "string" }),
    async argv => {
      const rag = await loadRag();
      let text = String(argv.textOrFile);
      if (fs.existsSync(text)) text = fs.readFileSync(text, "utf8");
      await rag.pin(String(argv.key), text);
      console.log("pinned", argv.key);
    }
  )
  .command(
    "query <q>",
    "Retrieve relevant context",
    y => y
      .positional("q", { type: "string" })
      .option("k", { type: "number", default: 6 })
      .option("pins", { type: "array", default: [] }),
    async argv => {
      const rag = await loadRag();
      const pins = (argv.pins as string[]) ?? [];
      const out = await rag.retrieveWithPins(String(argv.q), Number(argv.k), pins);
      console.log(out);
    }
  )
  .demandCommand(1)
  .help()
  .parse();
```

**Add scripts & deps** to your package.json (merge with the earlier example):

```json
{
  "scripts": {
    "mcp": "tsx mcp-rag.ts",
    "rag:index": "tsx rag-cli.ts index",
    "rag:pin": "tsx rag-cli.ts pin",
    "rag:query": "tsx rag-cli.ts query",
    "rag:git-index": "tsx rag-git-index.ts"
  },
  "dependencies": {
    "globby": "^14.0.2",
    "yargs": "^17.7.2"
  }
}
```

> If you prefer `pnpm dlx`, you can run `pnpm tsx rag-cli.ts ...` directly.

---

## Git history indexing (commits → chunks)

Index your repository history so the LLM can answer questions like “when did we introduce X?” or “what changed in this file around March?”

### Strategy

- Treat each *git blob at a commit* as a stable source: `git://<sha>/<path>`.
- Dedupe by making `docs.source` unique (see above).
- Filter to relevant extensions (ts, tsx, js, py, php, md, yaml, json, sql, etc.).
- Optionally skip large files and vendored paths.

### Git indexer script: `rag-git-index.ts`

```ts
#!/usr/bin/env tsx
import { simpleGit } from "simple-git";
import path from "node:path";
import { RAG } from "./rag";
import { simpleSplit } from "./simpleSplit";

const exts = new Set([".ts", ".tsx", ".js", ".jsx", ".py", ".php", ".md", ".yaml", ".yml", ".json", ".sql"]);
const skipDirs = ["node_modules/", "dist/", "build/", "vendor/", ".next/", ".output/"];

function shouldIndex(p: string) {
  if (skipDirs.some(d => p.includes(d))) return false;
  return exts.has(path.extname(p).toLowerCase());
}

async function main() {
  const since = process.env.SINCE || "3 months ago"; // accepts git date formats
  const limit = Number(process.env.MAX_COMMITS || 500);
  const git = simpleGit();
  const rag = new RAG({ url: process.env.TURSO_URL ?? "file:rag.db", authToken: process.env.TURSO_AUTH_TOKEN });
  await rag.init();

  const log = await git.log({ since, maxCount: limit });
  for (const c of log.all) {
    const sha = c.hash;
    // list files changed in this commit
    const nameStatus = await git.raw(["show", "--name-status", "--pretty=", sha]);
    const lines = nameStatus.split(/\r?\n/).filter(Boolean);
    for (const line of lines) {
      const parts = line.split(/\s+/);
      const status = parts[0];
      const file = parts[parts.length - 1];
      if (!shouldIndex(file)) continue;
      if (status === "D") continue; // skip deletes
      // read file content at this commit
      try {
        const content = await git.raw(["show", `${sha}:${file}`]);
        if (!content.trim()) continue;
        const chunks = simpleSplit(content, 2000);
        const source = `git://${sha}/${file}`;
        const title = `${path.basename(file)} @ ${sha.slice(0,7)}`;
        if ((rag as any).upsertDocBySource) {
          await (rag as any).upsertDocBySource(source, title, chunks);
        } else {
          await rag.upsertDoc(source, title, chunks);
        }
        console.log("indexed", source);
      } catch (e) {
        // file may be binary or too large; ignore
      }
    }
  }
}

main().catch(e => { console.error(e); process.exit(1); });
```

**Add deps**:

```bash
pnpm add simple-git globby yargs
```

### Usage

```bash
# index last 3 months of commits (default), up to 500 commits
pnpm rag:git-index

# index aggressively
SINCE="1 year ago" MAX_COMMITS=2000 pnpm rag:git-index

# query with git context now in the DB
pnpm rag:query "when did we switch to SQS for queues?" --pins system
```

### Tips

- To keep DB small, run git indexing on a CI job that targets `main` only.
- Re-run nightly with `SINCE="14 days ago"` to keep fresh.
- Consider also indexing `git tag`s for release snapshots; source like `git://v1.2.0/path`.
- For huge repos, add a size guard: skip files over ~200 KB or chunk more aggressively.

---

## Cursor MCP quick hooks for git

Expose a tool to index recent commits straight from the LLM:

```ts
// add to mcp-rag.ts
server.tool(
  new Tool({
    name: "rag_git_index_recent",
    description: "Index git commits since a date (e.g., '30 days ago')",
    inputSchema: z.object({
      since: z.string().default("30 days ago"),
      maxCommits: z.number().int().min(1).max(5000).default(500)
    })
  }),
  async ({ since, maxCommits }) => {
    process.env.SINCE = since;
    process.env.MAX_COMMITS = String(maxCommits);
    // naive: spawn the script so MCP stays responsive
    const { execa } = await import("execa");
    await execa("pnpm", ["rag:git-index"], { stdio: "inherit" });
    return { content: [{ type: "text", text: `git indexed since '${since}' (up to ${maxCommits})` }] };
  }
);
```

Now you can do in Cursor:

```
mcp call rag rag_git_index_recent {"since":"90 days ago","maxCommits":1000}
```

---

## Security & PII guardrails (recommended)

- Before indexing, run a redactor to drop secrets and tokens (AWS keys, DB URLs). Consider `dotenv` parsing + regexes for key formats.
- Store the DB in project-local path (default `file:rag.db`) and Git-ignore it.
- For Turso remote, use a scoped token and restrict access by org/project.


```python
pins = ["llm_core_codex", "llm_effects_optics", "llm_concurrency_time"]
```