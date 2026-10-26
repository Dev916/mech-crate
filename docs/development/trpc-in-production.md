---
title: "tRPC in Production: Architecture, Security Hardening, and Type Performance"
category: api-design
languages: [typescript]
complexity: advanced
use_cases:
  - deciding whether tRPC fits a product (TypeScript monorepo and first-party clients) or whether oRPC, ts-rest, Hono RPC, GraphQL, REST/OpenAPI or Server Actions fit better
  - structuring routers, context and base procedures so authorization is on by default and the router type stays cheap to check
  - "hardening a tRPC server: CSRF with cookie auth, body and batch limits, error-message leakage, the two 2025 CVEs, rate limiting"
  - choosing links (batch, batch-stream, SSE subscription, FormData) and cache headers without mixing private data into cached batches
  - diagnosing slow tsc and editor autocomplete on a large AppRouter
summary: "State of practice as of 2026-10 for tRPC 11 (npm latest 11.19.0). tRPC is the right tool for a TypeScript monorepo whose clients you ship yourself, and the wrong tool for a public or polyglot API. Its defaults are tuned for developer experience, not for a hostile network: raw error messages from non-TRPCError exceptions reach the client even with isDev false, request bodies and batch sizes are unlimited, and multipart or octet-stream POSTs (both CORS-simple) are routed to any mutation, so cookie-authenticated apps need SameSite plus an Origin or Sec-Fetch-Site check. The doc gives a verified hardening config, the output-validator-as-whitelist pattern, SSE subscription rules (signal, tracked(), subscribe-before-replay), the cache-only-all-public-batches rule, and the type-performance ladder (lean context, sub-router inference, lazy routers, built .d.ts, TypeScript 7)."
provenance: researched
researched: 2026-10-26
sources:
  - https://trpc.io/docs/server/validators
  - https://trpc.io/docs/server/error-handling
  - https://trpc.io/docs/server/error-formatting
  - https://trpc.io/docs/server/middlewares
  - https://trpc.io/docs/server/context
  - https://trpc.io/docs/server/merging-routers
  - https://trpc.io/docs/server/server-side-calls
  - https://trpc.io/docs/server/subscriptions
  - https://trpc.io/docs/server/caching
  - https://trpc.io/docs/server/non-json-content-types
  - https://trpc.io/docs/client/links/httpBatchLink
  - https://trpc.io/docs/client/links/httpBatchStreamLink
  - https://trpc.io/docs/client/links/httpSubscriptionLink
  - https://trpc.io/docs/client/tanstack-react-query/setup
  - https://trpc.io/docs/client/tanstack-react-query/server-components
  - https://trpc.io/docs/rpc
  - https://trpc.io/docs/migrate-from-v10-to-v11
  - https://trpc.io/docs/faq
  - https://trpc.io/blog/announcing-trpc-v11
  - https://trpc.io/blog/introducing-tanstack-react-query-client
  - https://trpc.io/blog/typescript-performance-lessons
  - https://github.com/trpc/trpc/security/advisories/GHSA-pj3v-9cm8-gvj8
  - https://github.com/trpc/trpc/security/advisories/GHSA-43p4-m455-4f4j
  - https://github.com/trpc/trpc/issues/5522
  - https://github.com/trpc/trpc/discussions/2448
  - https://github.com/trpc/trpc/discussions/5636
  - https://github.com/trpc/trpc/discussions/1860
  - https://github.com/trpc/trpc/blob/main/packages/server/src/unstable-core-do-not-import/http/contentType.ts
  - https://github.com/trpc/trpc/blob/main/packages/server/src/unstable-core-do-not-import/error/TRPCError.ts
  - https://github.com/trpc/trpc/blob/main/packages/server/src/adapters/node-http/incomingMessageToRequest.ts
  - https://github.com/trpc/trpc/blob/main/packages/server/src/unstable-core-do-not-import/http/resolveResponse.ts
  - https://github.com/trpc/trpc/blob/main/www/docs/client/openapi.md
  - https://www.npmjs.com/package/@trpc/server
  - https://cal.com/blog/cal-com-cold-start-resolution-blog
  - https://news.ycombinator.com/item?id=37098875
  - https://metaduck.com/trpc-versus-graphql/
  - https://orpc.dev/docs/comparison
  - https://hono.dev/docs/guides/rpc
  - https://devblogs.microsoft.com/typescript/typescript-native-port/
  - https://www.esveo.com/en/blog/wenn-moderne-technologien-nerven-eine-dx-odyssee-mit-trpc-drizzle-und-typescript/
  - https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html
  - https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS
  - https://github.com/codedeeply/cal.diy/pull/29
  - https://github.com/trpc/trpc/releases
  - https://blog.logrocket.com/trpc-vs-orpc-type-safe-rpc/
  - https://github.com/mcampa/trpc-to-openapi
  - https://trpc.io/docs/server/websockets
  - https://trpc.io/docs/client/cors
---

# tRPC in Production

State of practice as of 2026-10-26. Inline `[n]` cites key to the `sources` list in the frontmatter. Every code block in this doc was type-checked as one project with `@trpc/server` / `@trpc/client` / `@trpc/tanstack-react-query` 11.19.0, `@tanstack/react-query` 5.105, `zod` 4.6, `superjson` 2.2 and TypeScript 7.0.2 under `strict`; the server-side caller and the content-type and error-leak experiments in sections 4, 5 and 10 were run, not just compiled. Where a claim rests only on that run it is marked "verified here". Hacker News comment claims are marked as such and are not stated as fact.

## 1. Fit: when tRPC is the right tool

tRPC gives a TypeScript client the server's procedure types without a schema or codegen step: the client imports `type AppRouter` and every call, input and output is checked at compile time. The FAQ calls it "designed for TypeScript monorepos"; the maintainer's answer to "Monorepo mandatory?" is that you either share the repo or share the types some other way, such as publishing the server's types as a private npm package [18][27]. Adoption is large and still growing: v11 shipped on 2025-03-21 with 35k+ GitHub stars and 700k+ weekly downloads [19]; on 2026-10-26 the npm registry reports about 3.9M weekly downloads for `@trpc/server`, versus 1.3M for `@orpc/server`, 0.5M for `@ts-rest/core` and 50M for `hono` (which is a web framework, not only an RPC layer) [33].

| Situation | Verdict | Why |
|---|---|---|
| TypeScript monorepo, first-party web/mobile/CLI clients, one team ships both ends | **Use tRPC** | This is the design centre: no contract to maintain, refactors break the client at compile time [18][19] |
| Public or partner API, non-TypeScript consumers | **Do not** make tRPC the surface | The wire format is tRPC-specific (JSON in a query param, comma-joined batch paths); the official `@trpc/openapi` is alpha (`11.19.0-alpha` on npm) and excludes subscriptions; the community `trpc-to-openapi` fork targets `@trpc/server ^11.1.0` [16][32][46]. oRPC or ts-rest are contract-first alternatives [37] |
| Frontend and backend in separate repos or separate teams | Works with extra plumbing | Publish built `.d.ts` of the router, or accept a weaker contract [27][25] |
| Next.js App Router, mostly reads plus a few form mutations | Maybe not | tRPC's own RSC guide says React Server Components "solve a lot of the same problems tRPC was designed to solve, so you may not need tRPC at all"; its Server Actions caller is still `experimental_` [15][23] |
| Clients need to shape responses per screen, many teams share one graph | GraphQL | The one argument that survives for GraphQL over tRPC is client-defined data requirements; the 2025 metaduck essay concedes tRPC wins on iteration speed and learning curve [36] |
| Edge runtime (Workers, Deno, Bun) | Fine via the fetch adapter | Only streaming needs platform flags (Cloudflare `streams_enable_constructors`, Lambda response streaming) [12] |

**Versions you should be on.** npm `latest` is 11.19.0 (published 2026-09-16); git tags v11.20.0 and v11.21.0 exist but are not published, so features merged after 11.19 (per-procedure `.errors()`, PR #7591) are not installable yet [33][44]. v11 requires TypeScript `>=5.7.2` as a peer dependency, Node 18+ and, for the React packages, React `>=18.2` and TanStack Query v5 [17]. The v10 line still receives security fixes (`v10` dist-tag 10.45.4) [23][33]. tRPC follows strict semver where exported-type changes count as major; `unstable_` APIs are production-used and only change in minors, `experimental_` APIs "might change during any bump" [18].

## 2. Architecture that stays cheap at scale

**One `initTRPC`, exported as base procedures.** Middleware cannot be attached to a router, only to procedures, so policy lives in a small set of base procedures (`publicProcedure`, `protectedProcedure`, `adminProcedure`) that every router imports [4][18]. A middleware that throws when `ctx.user` is null and then calls `opts.next({ ctx: { user: ctx.user } })` narrows the type for everything downstream, which is the documented authorization pattern [4]. Reusable plugins that work across tRPC instances use `.concat()`; `experimental_standaloneMiddleware` is deprecated in its favour [4]. Order matters: a middleware that rewrites a context key hides the original type from the next one [4].

```ts
// trpc.ts — one initTRPC per app; export only the pieces routers need.
import { initTRPC, TRPCError } from '@trpc/server';
import { ZodError } from 'zod';
import superjson from 'superjson';

// Inner context: request-independent, always present (also in tests and server-side callers).
export async function createContextInner(opts: { user: { id: string; role: 'user' | 'admin' } | null }) {
  return { user: opts.user };
}
export type Context = Awaited<ReturnType<typeof createContextInner>>;

const isDev = process.env.NODE_ENV !== 'production';

export const t = initTRPC.context<Context>().create({
  transformer: superjson,
  isDev, // controls whether error.data.stack is sent; set it explicitly
  errorFormatter({ shape, error }) {
    return {
      ...shape,
      // Non-TRPCError exceptions inherit cause.message; never forward that in production (section 5).
      message: !isDev && error.code === 'INTERNAL_SERVER_ERROR' ? 'Internal server error' : shape.message,
      data: {
        ...shape.data,
        zodError:
          error.code === 'BAD_REQUEST' && error.cause instanceof ZodError ? error.cause.flatten() : null,
      },
    };
  },
});

export const router = t.router;
export const publicProcedure = t.procedure;
export const createCallerFactory = t.createCallerFactory;

// Timing middleware: observe, never swallow.
const timed = t.middleware(async (opts) => {
  const start = Date.now();
  const result = await opts.next();
  const durationMs = Date.now() - start;
  if (!result.ok) console.error('trpc', opts.type, opts.path, durationMs, 'ms', result.error.code);
  return result;
});

// Authorization middleware: narrows ctx.user from `X | null` to `X` for everything downstream.
export const protectedProcedure = publicProcedure.use(timed).use(async (opts) => {
  if (!opts.ctx.user) throw new TRPCError({ code: 'UNAUTHORIZED' });
  return opts.next({ ctx: { user: opts.ctx.user } });
});

export const adminProcedure = protectedProcedure.use(async (opts) => {
  if (opts.ctx.user.role !== 'admin') throw new TRPCError({ code: 'FORBIDDEN' });
  return opts.next();
});
```

**Inner and outer context.** The docs split context into an inner part (database handle, session, nothing that needs `req`/`res`) and an outer part that adds the request, and they say to infer the `Context` type from the inner one because only it is "really always available" (tests and server-side callers have no request) [5]. `createContext` runs once per HTTP request, so every procedure in a batch shares one context [5]. Keep heavy generated client types (Prisma, Supabase) out of the context type: the docs and two long GitHub threads trace slow type-checking and the "inferred type of this node exceeds the maximum length the compiler will serialize" error to context bloat; one reporter's emitted declarations fell from 3,542 lines to 102 after moving Prisma out of context [5][25][26].

**Routers.** Nest by domain (`router({ user: userRouter, post: postRouter })`), flatten with `t.mergeRouters` when you want one namespace, and wrap sub-routers in `lazy(() => import(...))` to defer loading; the docs' stated purpose for `lazy` is reducing cold starts, and "there's no difference in how you use the router after it's been lazy loaded" [6]. v11 also accepts a plain object as an inline sub-router [19]. Cal.com's 2023 cold-start write-up is the field case: one `/api/trpc/[trpc]` function bundling about 20 sub-routers took 7 to 15 s (30 s worst case) to serve a first request; splitting routers into their own API routes brought the public booking page from 15 s to about 2 s, and the remaining routes load concurrently so a page calling five of them still waits about 3 s instead of the sum [34]. `lazy()` is the in-framework version of that split.

```ts
// routers/app.ts
import { lazy } from '@trpc/server';
import { router, publicProcedure } from '../trpc.js';

export const appRouter = router({
  health: publicProcedure.query(() => ({ ok: true })),
  // Lazy sub-router: the module is imported on first call, which trims serverless cold starts.
  post: lazy(() => import('./post.js').then((m) => m.postRouter)),
  // Shorthand sub-router (v11): a plain object instead of router({...}).
  publicStats: { count: publicProcedure.query(() => 1) },
});
export type AppRouter = typeof appRouter; // clients: `import type` only
```

**Server-side calls.** `createCallerFactory(appRouter)` is built once; each test or request creates a caller with its own context, and errors arrive as thrown `TRPCError`s rather than HTTP responses [7]. The docs warn against calling a caller from inside another procedure because it rebuilds context, reruns middleware and revalidates input; share plain functions instead [7]. In Next.js the server caller is deliberately detached from the React Query cache (section 9) [15].

```ts
// caller.ts (verified here: anonymous create -> UNAUTHORIZED, missing id -> NOT_FOUND, '' -> BAD_REQUEST)
import { createCallerFactory } from './trpc.js';
import { appRouter } from './routers/app.js';

const createCaller = createCallerFactory(appRouter);

export async function test() {
  const caller = createCaller({ user: { id: 'u1', role: 'user' } });
  const { id } = await caller.post.create({ title: 't', body: 'b' });
  const anon = createCaller({ user: null });
  await anon.post.create({ title: 't', body: 'b' }).catch((e) => e.code); // 'UNAUTHORIZED'
  return id;
}
```

## 3. Validation: input on everything, output as a whitelist

tRPC infers validators through the Standard Schema interface when the library implements it and through custom adapters otherwise; Zod, Valibot, ArkType, Effect, Yup, Superstruct, TypeBox, Typia and Sury are first-class, and a plain function works too [1]. Chained `.input()` calls merge object schemas, later keys winning; non-object inputs cannot be chained [1].

The docs give two reasons for `.output()`: "Checking that data returned from untrusted sources is correct" and "Ensure that you are not returning more data to the client than necessary"; a failed output parse answers `INTERNAL_SERVER_ERROR` [1]. The output middleware replaces the resolver's return value with the parser's result, so with a schema that strips unknown keys (Zod's default `z.object`) an ORM row that later gains a `passwordHash` or `internalScore` column never leaves the server. Verified here: a resolver returning `{ id, title, body, authorId, internalScore }` through `.output(z.object({ id, title, authorId }))` produced `{"id":"p1","title":"hello","authorId":"u1"}` on the wire.

```ts
// routers/post.ts
import { z } from 'zod';
import { TRPCError, tracked } from '@trpc/server';
import { on, EventEmitter } from 'node:events';
import { router, publicProcedure, protectedProcedure } from '../trpc.js';

type Post = { id: string; title: string; body: string; authorId: string; internalScore: number };
const db = { posts: [{ id: 'p1', title: 'hello', body: '…', authorId: 'u1', internalScore: 42 }] as Post[] };
const ee = new EventEmitter<{ add: [Post] }>();

// Output schema: a whitelist, so a widened DB row can never leak new columns.
const PublicPost = z.object({ id: z.string(), title: z.string(), authorId: z.string() });

export const postRouter = router({
  byId: publicProcedure
    .input(z.object({ id: z.string().min(1).max(64) }))
    .output(PublicPost)
    .query(({ input }) => {
      const post = db.posts.find((p) => p.id === input.id);
      if (!post) throw new TRPCError({ code: 'NOT_FOUND' });
      return post; // extra fields (body, internalScore) are stripped by .output()
    }),

  create: protectedProcedure
    .input(z.object({ title: z.string().min(1).max(200), body: z.string().max(20_000) }))
    .mutation(({ ctx, input }) => {
      const post: Post = { id: crypto.randomUUID(), ...input, authorId: ctx.user.id, internalScore: 0 };
      db.posts.push(post);
      ee.emit('add', post);
      return { id: post.id };
    }),

  // SSE subscription; `tracked(id, data)` lets the client resume from lastEventId after a reconnect.
  onAdd: protectedProcedure
    .input(z.object({ lastEventId: z.string().nullish() }))
    .subscription(async function* ({ input, signal }) {
      const live = on(ee, 'add', { signal }); // subscribe BEFORE replaying so nothing is missed
      if (input.lastEventId) {
        for (const p of db.posts.filter((p) => p.id > input.lastEventId!)) yield tracked(p.id, p);
      }
      for await (const [post] of live) yield tracked(post.id, post);
    }),
});
```

Put size bounds in every string and array schema: tRPC itself imposes no body limit by default (section 10), so the validator is the first place an oversized payload is rejected, and it runs after the whole body has been read.

## 4. Errors: codes, formatting, and what leaks by default

`TRPCError` codes map to HTTP statuses (BAD_REQUEST 400, UNAUTHORIZED 401, FORBIDDEN 403, NOT_FOUND 404, CONFLICT 409, PAYLOAD_TOO_LARGE 413, UNSUPPORTED_MEDIA_TYPE 415, UNPROCESSABLE_CONTENT 422, TOO_MANY_REQUESTS 429, CLIENT_CLOSED_REQUEST 499, INTERNAL_SERVER_ERROR 500) and `getHTTPStatusCodeFromError()` converts; a batch whose calls end with different statuses returns `207 Multi-Status` [2][16]. `error.data.stack` is sent only when `isDev` is true, and `isDev` defaults to `process.env.NODE_ENV !== 'production'`; the docs recommend setting it explicitly so behaviour does not depend on the runtime's environment handling [2].

**The default leaks messages, not just stacks.** `TRPCError`'s constructor sets `message = opts.message ?? cause.message ?? code`, and any non-`TRPCError` thrown in a resolver is wrapped as `INTERNAL_SERVER_ERROR` with that inherited message [29]. Verified here with `isDev: false`: a resolver throwing `new Error('connect ECONNREFUSED db.internal:5432 password=hunter2')` returned exactly that string in `error.message` to the client. The fix is the `errorFormatter` above: keep `shape` (so `data.code`, `httpStatus` and `path` stay typed on the client), replace the message for 500s in production, and expose only what you choose (the docs' own example adds `error.cause.flatten()` for Zod errors so forms get field-level messages with full type inference on `mutation.error.data.zodError`) [3]. `onError` sees every error with `type`, `path`, `input`, `ctx` and `req` and is the hook for reporting; the docs say it is for observability only and that the wire shape is set by the formatter [2].

Per-procedure typed errors (`.errors()`, chainable into a union of shapes) were merged on 2026-09-27 and are in the v11.20.0 tag but not on npm as of this writing [3][44]. Until then oRPC's "tRPC has one global error shape" row is accurate [37].

## 5. Transport, batching and links

**Wire format.** Queries are `GET` with the input JSON-stringified in `?input=`; mutations are `POST`; subscriptions are `GET` over SSE or a WebSocket. A batch joins procedure paths with commas in the pathname and sends `?batch=1&input={"0":…,"1":…}`; a procedure called `GET` on a mutation path answers 405 (verified here) [16][31]. Because queries are plain GETs they must be side-effect free: the OWASP CSRF cheat sheet's rule "Do not use GET requests for state changing operations" applies verbatim, and `SameSite=Lax` cookies ride along on top-level cross-site GET navigations [41].

**Batching defaults.** The server allows batching by default (`allowBatching ?? true`), has no `maxBatchSize` unless you set one (oversized batches then get 400, and the type doc says to set the client's `maxItems` to the same or a lower value), and the client's `maxURLLength` and `maxItems` default to `Infinity` [11][31]. The docs suggest `maxURLLength: 2083` to avoid `413`, `414` and proxy `404`s on long GET batches [11]. `allowMethodOverride` (default false) lets a client send queries as POST [31].

**`httpBatchStreamLink`.** It sends the same batch but streams each result as it resolves (`transfer-encoding: chunked`, `trpc-accept: application/jsonl` or, with `streamHeader: 'accept'`, the standard `Accept` header, which avoids a CORS preflight cross-origin) [12]. Headers cannot change once the stream starts, so procedures that set cookies (login, logout) must go through `httpBatchLink` or `httpLink` via `splitLink` [12]. React Native needs `TextDecoderStream` and Streams polyfills; AWS Lambda needs response streaming configured, otherwise the link degrades to plain batching; Cloudflare Workers need `streams_enable_constructors` [12].

**Non-JSON inputs.** v11 accepts `FormData`, `File`, `Blob` and octet streams, but only through `httpLink`; route them with `splitLink` and `isNonJsonSerializable`, validate with `z.instanceof(FormData)` or `zod-form-data`, and make sure no upstream body parser (`express.json()`) consumed the body first [10].

```ts
// client.ts
import {
  createTRPCClient, httpBatchStreamLink, httpLink, httpSubscriptionLink, splitLink, isNonJsonSerializable,
} from '@trpc/client';
import superjson from 'superjson';
import type { AppRouter } from './routers/app.js';

const url = 'http://localhost:3000';

export const client = createTRPCClient<AppRouter>({
  links: [
    splitLink({
      condition: (op) => op.type === 'subscription',
      true: httpSubscriptionLink({ url, transformer: superjson }),
      false: splitLink({
        // FormData / binary inputs cannot be batched: route them to a plain httpLink.
        condition: (op) => isNonJsonSerializable(op.input),
        true: httpLink({ url, transformer: superjson }),
        false: httpBatchStreamLink({
          url,
          transformer: superjson, // v11: the transformer lives on the link
          maxURLLength: 2083,     // split batches before proxies answer 414
          maxItems: 20,           // <= server maxBatchSize
          headers: () => ({ authorization: `Bearer ${'token'}` }),
        }),
      }),
    }),
  ],
});
```

If you use a transformer such as superjson, every client, including any generated from the OpenAPI document, has to use the same one or `Date`, `Map` and `Set` values "may be silently wrong" [32].

## 6. Subscriptions: SSE first, generators, `tracked()`

The docs recommend SSE "if you are unsure" because it needs no separate WebSocket server [8][19]. A subscription is an async generator: pass `opts.signal` to the event source so it stops when the client disconnects, and put cleanup in `try/finally` because tRPC calls the generator's `.return()` when the subscription stops [8]. Yielding `tracked(id, data)` makes the client reconnect with `lastEventId` (propagated through `.input()` for SSE, automatically for `wsLink`); when replaying a backlog, start the live listener before querying the backlog or "newly emitted events" can be missed [8]. A 5xx thrown inside the generator makes the client reconnect from the last tracked id; other errors end the subscription [8].

Operational settings live on `initTRPC.create({ sse: { ping: { enabled, intervalMs }, maxDurationMs, client: { reconnectAfterInactivityMs } } })` for SSE and `keepAlive: { pingMs, pongWaitMs }` on the WebSocket server [13][47]. Browser `EventSource` cannot send custom headers and does not re-run `eventSourceOptions()` on reconnect, so the docs use cookies, an EventSource ponyfill for header auth, `withCredentials: true` cross-domain, and `retryLink` to re-establish with a refreshed token; `connectionParams` go into the URL for SSE and into the first message for WebSockets [13][47]. Long-lived SSE on serverless platforms is bounded by the platform's request timeout; `maxDurationMs` lets you end and let the client resume rather than be cut mid-event [13].

Security note for WebSocket deployments: CVE-2025-43855 (GHSA-pj3v-9cm8-gvj8, high) let any unauthenticated client crash a v11.0.0 WebSocket server that had `createContext` set by sending malformed `connectionParams`; fixed in 11.1.1 [22].

## 7. Caching: only all-public, error-free query batches

Queries are GETs, so CDN and browser caching work through standard `Cache-Control` headers set in `responseMeta`, which receives the batch's `paths`, `errors` and `type` [9]. The documented rule, and the only safe one with batching on by default, is to emit a cache header only when every path in the batch is public, no procedure errored and the type is `query`; the docs also say to omit cache headers entirely when an auth token or cookie is present and to keep public and private calls in separate batches with `splitLink` [9]. `httpBatchStreamLink` cannot set headers after the first chunk, so cached responses need the non-streaming link [12].

```ts
// server.ts — standalone adapter; the same options exist on the fetch/Next/Express adapters.
import { createHTTPServer } from '@trpc/server/adapters/standalone';
import { appRouter } from './routers/app.js';
import { createContextInner } from './trpc.js';

createHTTPServer({
  router: appRouter,
  createContext: async ({ req }) => {
    const token = req.headers.authorization;
    const user = token ? { id: 'u1', role: 'user' as const } : null; // verify the token for real
    return createContextInner({ user });
  },
  allowBatching: true,
  maxBatchSize: 20,        // no default: unbounded batches otherwise
  maxBodySize: 1_000_000,  // bytes, no default on node adapters: unbounded bodies otherwise
  onError({ error, type, path }) {
    if (error.code === 'INTERNAL_SERVER_ERROR') console.error('report', type, path, error.cause);
  },
  responseMeta({ paths, errors, type }) {
    // Only cache when EVERY procedure in the batch is public, succeeded, and is a query.
    const allPublic = paths?.every((p) => p.startsWith('publicStats.')) ?? false;
    if (allPublic && errors.length === 0 && type === 'query') {
      return { headers: new Headers([['cache-control', 's-maxage=60, stale-while-revalidate=86400']]) };
    }
    return {};
  },
}).listen(3000);
```

## 8. Client integration: the TanStack-native client and server components

`@trpc/tanstack-react-query` (`createTRPCContext` for apps with a provider, `createTRPCOptionsProxy` for SPAs) exposes `queryOptions()`, `mutationOptions()`, `queryKey()`, `queryFilter()` and `pathFilter()` factories that plug into the plain TanStack hooks; the docs call it "simpler and more TanStack Query-native" and say "we think it's the future and recommend using this over the classic client" [14]. The announcement gives the reasons: the classic client's hook wrappers conflict with React hook rules and the React Compiler, and it is harder to keep in step with TanStack releases; the classic `@trpc/react-query` is not deprecated and will be maintained "for a long time", and the two share query keys so migration can be gradual [20].

```tsx
// react.tsx
import { useQuery, useMutation, useQueryClient, QueryClient } from '@tanstack/react-query';
import { createTRPCContext } from '@trpc/tanstack-react-query';
import type { AppRouter } from './routers/app.js';

export const { TRPCProvider, useTRPC, useTRPCClient } = createTRPCContext<AppRouter>();

export function Post({ id }: { id: string }) {
  const trpc = useTRPC();
  const qc = useQueryClient();
  const post = useQuery(trpc.post.byId.queryOptions({ id }));
  const create = useMutation(
    trpc.post.create.mutationOptions({
      onSuccess: () => qc.invalidateQueries(trpc.post.pathFilter()), // every post.* query
    }),
  );
  if (post.isPending) return null;
  return <button onClick={() => create.mutate({ title: 'x', body: 'y' })}>{post.data?.title}</button>;
}
export const makeQueryClient = () => new QueryClient({ defaultOptions: { queries: { staleTime: 30_000 } } });
```

For the Next.js App Router the documented shape is a `server-only` module that builds `createTRPCOptionsProxy({ ctx, router, queryClient })` with the QueryClient wrapped in React's `cache()` so one client serves the whole request, `prefetch(trpc.x.queryOptions())` in the server component, `HydrateClient` around the client subtree, and `useSuspenseQuery` in a `'use client'` component; `shouldDehydrateQuery` is widened so pending prefetches dehydrate too. A `createCaller` instance is kept deliberately separate from the query cache so server-read data and client-cache expectations never diverge [15].

## 9. Security hardening

**CSRF with cookie auth.** v11 added explicit content-type checks after issue #5522 described forged form posts: a POST body is accepted only as `application/json`, `multipart/form-data` or `application/octet-stream`; anything else gets 415 [24][28]. Verified here against 11.19.0 with a public no-input mutation:

| Cross-site POST to `/noInput` | Status | CORS-simple content type? [42] |
|---|---|---|
| `text/plain` body `{}` | 415 UNSUPPORTED_MEDIA_TYPE | yes |
| `application/x-www-form-urlencoded` | 415 | yes |
| `application/json` | 200, mutation ran | no (preflighted cross-origin) |
| `multipart/form-data` (`curl -F`) | 200, mutation ran | **yes** |
| `application/octet-stream` | 200, mutation ran | no in the fetch spec list, but any script can send it with `credentials: 'include'` only if CORS allows the origin |

So an HTML `<form enctype="multipart/form-data">` on an attacker's page can call any mutation whose input validator tolerates what arrives as form fields (a no-input mutation, or one whose schema is loose) with the victim's cookies if those cookies are sent cross-site. The multipart body is parsed into an object by field name, so a schema requiring `{ n: number }` rejected `-F n=5` (string) with BAD_REQUEST (verified here), which is why strict input schemas are part of the defence, not only a correctness tool. A September 2026 fix in the cal.diy fork is the field case: that app was pinned to a v11 beta that still parsed every content type as JSON, cross-site `text/plain` form posts triggered mutations, and the fix required `application/json` and rejected `Sec-Fetch-Site: cross-site` [43]. Defence in depth, per OWASP: `SameSite=Lax` or `Strict` on the session cookie (Lax still allows top-level GET navigations, so queries must stay side-effect free), an `Origin` or `Sec-Fetch-Site` check on every POST, and a CSRF token if you must run `SameSite=None`; a custom header is a defence only while CORS is not misconfigured for sibling subdomains [41]. If you accept uploads, allow multipart only on the specific upload procedures and keep JSON-only elsewhere (a sketch is under Synthesis).

**Resource limits.** The node-http adapters pass `maxBodySize ?? null` to the request converter, and `null` means no limit; an overflow answers `PAYLOAD_TOO_LARGE` [30]. Set it, or set the body-parser limit in the host framework if that parser runs first. Set `maxBatchSize` on the server and `maxItems` on the client; without them one request can fan out into an unbounded number of procedure calls sharing one context [11][31]. Bound every string, array and number in input schemas.

**Rate limiting** is not built in. Write it as middleware keyed by user id or IP, configure per procedure through `meta`, and throw `TOO_MANY_REQUESTS` (429) [2][4]; apply it on `publicProcedure` so every procedure inherits it.

**Known vulnerabilities.** Two high-severity advisories exist, both fixed: CVE-2025-43855 (WebSocket DoS via invalid `connectionParams`, `>=11.0.0 <11.1.1`) [22] and CVE-2025-68130 (prototype pollution in `formDataToObject`, used only by `experimental_caller` / `experimental_nextAppDirCaller`, `>=10.27.0 <10.45.3` and `>=11.0.0 <11.8.0`; the advisory's impact section shows `__proto__[isAdmin]=true` turning every `if (user.isAdmin)` check true) [23]. The second one is the reason to keep `experimental_` APIs off authorization-critical paths; the FAQ says they may change or vanish in any release [18].

**Other rules.** Do authorization in middleware or the resolver, never trust `input` for identity (take the user from `ctx`). Keep `isDev` false in production and redact 500 messages (section 4). Configure CORS at the adapter or proxy with an explicit origin allow-list when cookies cross origins; tRPC does not set CORS headers itself [48]. Do not let a procedure's output schema be the ORM type.

## 10. Performance

**Runtime.** Batching turns a screen's concurrent queries into one request; streaming returns the fast ones first; `lazy()` and per-domain routes shrink serverless cold starts (the Cal.com numbers in section 2) [11][12][6][34]. `createContext` runs once per request, so expensive per-request work (session lookup) is amortised across a batch, and anything per-procedure belongs in middleware [5]. Published throughput comparisons (oRPC's "about 4.2x as many requests as tRPC" [37], LogRocket's "comparable" [45]) give no method and should not drive the decision.

**Type-checking.** This is the cost that surprises teams. tRPC's own benchmark app has 3,500 procedures across 1,000 routers, profiled with `tsc --generateTrace`; a v10 refactor that removed an eager mapped type cut one app's check from 332 ms to 136 ms [21]. Hono's RPC guide states the shared truth for inferred RPC clients: "the more routes you have, the slower your IDE will become", with the same remedies [38]. The ladder, in the order teams report applying it:

1. Lean context types: no ORM client in `ctx`; import it where used [5][26].
2. `import type { AppRouter }` only; infer helper types from a sub-router (`inferRouterOutputs<typeof userRouter>`) rather than from the root [25].
3. Split by domain into sub-routers and `lazy()` them; keep Zod composition (`.extend/.merge/.pick`) shallow on hot schemas [6][25].
4. `skipLibCheck`, `incremental`, and the editor in watch mode (one report: 15 s to 3 s) [25].
5. Build the API package to `.d.ts` so consumers read declarations instead of re-inferring: project references with `composite` and `disableSourceOfProjectReferenceRedirect`, or a separate `tsc --emitDeclarationOnly` build; a 2025 two-million-line monorepo needed that plus a Zod-to-ArkType move before autocomplete became usable, and one reporter went from about 6 s to 0.5 s [25][40]. `declarationMap` keeps go-to-definition landing in source.
6. TypeScript 7 (the native port): Microsoft's launch post measured the tRPC repository itself at 5.5 s to 0.6 s for a full check, and tRPC builds with it since 11.19; the examples in this doc were checked under 7.0.2 [39][44].

## 11. What practitioners argue about

The 316-point Hacker News thread of 2023-08 is the clearest record of the objections [35]. These are comment claims, listed with the primary-source answer where one exists:

- **"Tight coupling; Prisma mapped straight through to the component; junior developers stop thinking about interfaces."** The framework does not force this, but its defaults make it easy. `.output()` schemas as whitelists (section 3) and inner-context design (section 2) are the in-framework counter; the coupling complaint is really about skipping a domain layer.
- **"What about version skew between deployed client and server?"** Static types assume one build; the thread's own answer is the usual field lifecycle (add optional, then required) and tolerant parsers. tRPC has no contract versioning of its own [16].
- **"What happens when one side is no longer TypeScript?"** The honest answer from the FAQ and the monorepo discussion: you lose the guarantee and fall back to OpenAPI generation or a published types package [18][27][32].
- **"RPC IDLs keep being reinvented."** The author of the WunderGraph BFF conceded in-thread that tRPC "pushed TypeScript only stacks to the limit in terms of DX" while noting the hype cycle was fast and REST/GraphQL did not go away; the 2025 GraphQL essay makes the same concession from the other side [35][36].

## 12. Synthesis (inferred)

The following are this doc's own conclusions, drawn from the cited material rather than stated by it.

**Treat tRPC as an internal bus with a hostile edge.** Everything above that is "on by default" (batching, unlimited bodies, multipart mutations, message passthrough) is a reasonable default for a localhost dev loop and an unreasonable one for a cookie-authenticated production origin. The minimum production config is the `server.ts` in section 7 plus the `errorFormatter` in section 2 plus a front-door check. A sketch for the fetch adapter, not a tRPC API:

```ts
// Front door for cookie-authenticated deployments (fetch adapter). Illustrative, not from the docs.
export function rejectCrossSite(req: Request, uploadPaths: Set<string>): Response | null {
  if (req.method !== 'POST') return null;
  const path = new URL(req.url).pathname.split('/').pop() ?? '';
  const ct = req.headers.get('content-type')?.split(';')[0]?.trim();
  const site = req.headers.get('sec-fetch-site');
  if (site === 'cross-site') return new Response('cross-site POST rejected', { status: 403 });
  if (ct !== 'application/json' && !uploadPaths.has(path)) return new Response('json only', { status: 415 });
  return null;
}
```

`Sec-Fetch-Site` is set by all current browsers; a missing header should be treated as same-origin only for non-browser clients you authenticate another way.

**Base procedures are your policy layer; name them by what they guarantee.** `protectedProcedure` guarantees a user, `adminProcedure` a role, `rateLimitedProcedure` a budget. Resource-level checks (ownership, tenancy) that need the input belong in a `.concat()` plugin that declares its own `.input()` or in the resolver, never on the client.

**Decide on the type-cost budget up front.** If the AppRouter will exceed a few hundred procedures, start with sub-router inference and a built `.d.ts` package rather than retrofitting them under pressure, and keep the ORM out of context from day one; the retrofit stories in the GitHub threads all describe weeks of degraded editor performance first.

**SSE is the default; WebSockets are for bidirectional or very chatty streams.** Every production caveat in section 6 (ping, max duration, reconnect, auth headers) is smaller with SSE, and the one DoS CVE so far was in the WebSocket adapter.

**If a non-TypeScript consumer is on the roadmap, pick the contract-first tool now.** Retrofitting OpenAPI onto tRPC means an alpha package or a community fork plus a transformer that those clients must replicate; oRPC's gradual-migration path exists, but so does the cost of running two RPC layers.

## 13. Checklist

- [ ] `@trpc/*` all on the same version, `>=11.1.1` if WebSockets, `>=11.8.0` (or `10.45.3`) if any `experimental_` caller; TypeScript `>=5.7.2`, `strict: true`.
- [ ] One `initTRPC` with `isDev` explicit, `errorFormatter` redacting 500 messages, `onError` reporting.
- [ ] Inner/outer context; no ORM client type in `ctx`; `Context` inferred from the inner function.
- [ ] `publicProcedure` / `protectedProcedure` / `adminProcedure`; authorization reads `ctx`, never `input`.
- [ ] Every procedure has `.input()` with size bounds; every procedure returning DB or third-party data has `.output()`.
- [ ] `maxBodySize`, `maxBatchSize` (server) and `maxItems`, `maxURLLength` (client) set.
- [ ] Session cookie `SameSite=Lax` or stricter; `Origin`/`Sec-Fetch-Site` check on POST; multipart only on upload procedures.
- [ ] Queries are side-effect free; cookie-setting procedures bypass the stream link; FormData goes to `httpLink`.
- [ ] Subscriptions: generator with `opts.signal`, `tracked()`, listener before backlog, ping and `maxDurationMs` configured.
- [ ] `responseMeta` caches only all-public, error-free query batches.
- [ ] `@trpc/tanstack-react-query` for new code; RSC prefetch plus `HydrateClient`; `createCallerFactory` for tests.
- [ ] Type budget: sub-router inference, `lazy()` by domain, built `.d.ts` once autocomplete slows, TypeScript 7 evaluated.
