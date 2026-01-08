# **Appendix: API Design & Structure Playbook**

An opinionated, battle-tested appendix for designing HTTP APIs that stay evolvable, predictable, and kind to clients. Pulls from HATEOAS patterns, "Your API is Bad", and other industry scars.

---

## **Goals and scope**

* Make resource models and URLs obvious and stable.
* Keep verbs, status codes, and errors semantically correct.
* Build in evolution: linking, pagination, versioning, and compatibility rules.

---

## **Core principles**

* **Nouns over verbs**: Model resources, not RPC actions. Keep verbs for rare domain commands.
* **Stable identifiers**: Immutable IDs, never positional indexes. Use opaque cursor tokens for pagination.
* **One level of nesting, max**: Show ownership/containment, but avoid deeply coupled URLs.
* **Self-descriptive messages**: Use media types and link relations (HATEOAS) so clients can follow contracts.
* **Explicit contracts**: Content types, schemas, error shapes, and version policies are part of the API, not tribal knowledge.

---

## **Resource modeling & URLs**

* **Plural, lowercased nouns**: `/users`, `/cars`. Avoid verbs like `/createUser`.
* **Canonical resource first**: `/cars/{car_id}` is the anchor; nested access is a convenience, not the only path.
* **Controlled nesting (1 level)**: Show ownership or context, but keep a top-level escape hatch.
  * Good: `/users/{user_id}/cars` (list owned cars), `/users/{user_id}/cars/{car_id}` (scoped fetch).
  * Also provide: `/cars?owner_id={user_id}` (top-level lookup without deep coupling).
  * Avoid: `/orgs/{org_id}/users/{user_id}/cars/{car_id}/tires/{tire_id}` (fragile, over-coupled).
* **Decision rules for nesting vs query**:
  * Nest when the parent is the only meaningful context (e.g., `/users/{id}/sessions`).
  * Keep top-level when the child has a life of its own or cross-cutting queries exist (`/cars/{id}` even if owned by a user).
  * Expose filters to avoid path explosion for queries (`/cars?owner_id=...&status=...` beats `/users/{id}/cars/status/...`).
* **Actions as sub-resources**: When you must model verbs, make them nouns:
  * Good: `POST /payments/{id}/capture`, `POST /users/{id}/email-verification`.
  * Avoid: `POST /capturePayment` or `GET /doDeleteUser?id=123`.
* **Field selection & expansions**: Use query params like `?fields=id,name` or `?include=owner` instead of bespoke endpoints.
* **Domain language**: Prefer stable business terms over implementation words (`/invoices`, not `/billingRecords`).
* **Multitenancy shape**: Pick one and stick with it: path prefix (`/tenants/{tid}/users`), header (`X-Tenant-Id`), or dedicated host (`{tid}.api.example.com`). Do not mix without a migration plan.

---

## **HTTP methods mapped to intent**

* **GET**: Safe, idempotent reads only. No mutations, no side effects.
* **POST**: Create new subordinate resources or invoke domain actions that are not pure replacements. Use idempotency keys for retries.
* **PUT**: Full replacement of a resource. Idempotent.
* **PATCH**: Partial updates (RFC 7386 merge-patch or JSON Patch). Idempotent in effect.
* **DELETE**: Idempotent removal; responding 404 for repeated deletes is acceptable but consider 204 for already-deleted to aid retries.
* **HEAD/OPTIONS**: Support `HEAD` where `GET` exists; `OPTIONS` must advertise allowed methods and CORS.
* **Safe vs unsafe**: Do not overload GET for search that executes expensive side effects or mutations; use POST for complex searches that accept large bodies.

---

## **Representations, media types, and links**

* **JSON by default** with explicit `Content-Type: application/json; charset=utf-8`.
* **Link affordances (HATEOAS)**:
  * Include `self` and key transitions: `next`, `prev`, `related`, `collection`, `canonical`.
  * Use standard relation names where possible (IANA link relations) or namespaced custom rels.
* **Uniform envelopes**: Represent lists as objects with `data`, `links`, `meta`, not raw arrays, so pagination and metadata stay stable.
* **Consistent timestamps and IDs**: ISO-8601 UTC (`2024-06-01T12:00:00Z`), opaque IDs (UUID/KSUID/Snowflake).

Example list response with HATEOAS-style links:

```json
{
  "data": [
    {
      "id": "car_123",
      "make": "Subaru",
      "model": "Forester",
      "links": {
        "self": "/cars/car_123",
        "owner": "/users/user_9"
      }
    }
  ],
  "links": {
    "self": "/users/user_9/cars",
    "next": "/users/user_9/cars?cursor=abc",
    "canonical": "/cars?owner_id=user_9"
  },
  "meta": {
    "count": 1
  }
}
```

---

## **Responses, status codes, and transitions**

* **201 + Location** for creations; body returns the new resource.
* **202 Accepted** for long-running work; include `Location` of an operation/status resource and retry-after guidance.
* **204 No Content** when the body would be empty (DELETE success, idempotent completion).
* **304 Not Modified** with `ETag`/`If-None-Match` to save bandwidth.
* **409 Conflict** for state conflicts (duplicate names, version mismatch, business rules).
* **412 Precondition Failed** when `If-Match` / `If-Unmodified-Since` guard fails.
* **503 + Retry-After** for maintenance; prefer shedding load with explicit backpressure signals.
* **State transitions**: Document allowed transitions and their codes (e.g., `POST /payments/{id}/capture` returns 202 until settled, 409 if already settled, 422 if invalid state).

---

## **Errors that help instead of hurt**

* **Never 200 for failures**. Use accurate status codes: `400` for validation, `401/403` for auth, `404` for missing, `409` for conflicts, `422` for semantic validation, `429` for throttling, `500`/`503` for server/maintenance.
* **Structured errors**: Prefer RFC 7807 Problem Details:

```json
{
  "type": "https://api.example.com/errors/validation",
  "title": "Invalid request",
  "status": 422,
  "detail": "car_id is required",
  "instance": "req-49ad",
  "errors": [
    { "field": "car_id", "message": "missing" }
  ]
}
```

* **Correlation**: Return a request ID header (`X-Request-Id`) echoed in error bodies.
* **No stack traces or debug**: Never leak internals. Provide machine-safe codes and human-readable detail.
* **Error catalog**: Maintain stable error codes (`code: "duplicate_car"`, `code: "forbidden_scope"`) with docs for remediation.
* **Partial failures**: For bulk operations, return per-item statuses inside a multi-status envelope; avoid all-or-nothing 207 unless justified.
* **Validation discipline**: Return all field errors at once with JSON pointers/paths so clients can fix multiple issues.

---

## **Pagination, filtering, sorting, and searching**

* **Pagination**: Cursor-based preferred (`?cursor=` + `links.next`), with page size limits. Offset is acceptable for small, stable datasets; avoid when items can shift under you (it causes skips/dupes).
* **Filtering**: Prefix with fields (`?owner_id=...`, `?status=pending`). Avoid ad-hoc `filter` blobs unless you version them.
* **Sorting**: `?sort=created_at` or `?sort=-created_at` (descending).
* **Sparse fieldsets**: `?fields=make,model,year` to reduce payloads.
* **Search**: Use explicit endpoints (`/cars/search`) or query params (`?q=...`) with documented semantics.
* **Deterministic ordering**: Always define a tie-breaker (e.g., `created_at desc, id desc`) to keep pagination stable under inserts.
* **Total counts**: Expensive counts should be optional (`?include_count=true`) or eventually consistent; avoid blocking lists for exact totals.

---

## **Concurrency, caching, and retries**

* **Optimistic concurrency**: `ETag` on responses; require `If-Match` on mutating requests to avoid lost updates.
* **Idempotency keys**: For POST/command endpoints, accept `Idempotency-Key` header; respond with the same status/body for retries.
* **Caching**: Use `Cache-Control`, `ETag`, and `Last-Modified`; never cache unsafe methods. Document which endpoints are cacheable.
* **Rate limits and backpressure**: Return `429` with `Retry-After` and `X-RateLimit-*` headers. Keep timeouts documented so clients set proper deadlines.
* **Replay safety**: Treat network retries as normal; design handlers to be idempotent or key-protected.
* **Conditional GET**: Encourage `If-None-Match` to reduce bandwidth; pair with strong ETags to prevent stale reads in high-churn areas.

---

## **Async and long-running operations**

* **Pattern**: `POST /reports` → `202 Accepted` with `Location: /operations/{op_id}` → `GET /operations/{op_id}` returns `status: pending/succeeded/failed`, `result` link when done.
* **Do not block**: Avoid holding connections for long tasks; push to queues/workers.
* **Callbacks/webhooks**: If pushing, sign webhooks, include timestamps and replay protection; expose `GET /webhook-deliveries/{id}` for troubleshooting.
* **Cancellation**: Provide `DELETE /operations/{op_id}` when feasible; respond with `409` if not cancellable.

---

## **Bulk, partial, and batch operations**

* **Batch create/update**: Accept arrays with per-item results; include `id`, `status`, and `errors` per element.
* **Atomicity contract**: Declare whether the batch is best-effort or all-or-nothing; default to best-effort and communicate failures clearly.
* **Pagination for batch outputs**: Large results should paginate or stream; avoid megabyte responses that exhaust clients.

---

## **Versioning and evolution**

* **Bias toward additive changes**: New fields, new link rels, and new resources should not break clients.
* **Deprecations are explicit**: `Deprecation` and `Sunset` headers with dates, plus docs.
* **Version strategies**: Pick one and stay consistent.
  * URI versioning (`/v1/cars`) keeps caches straightforward.
  * Header versioning (`Accept: application/vnd.example.cars+json;version=1`) keeps URLs clean and plays well with content negotiation.
* **Contracts as schemas**: Machine-readable (OpenAPI/JSON Schema). Validate in CI and at runtime for boundary enforcement.
* **Compatibility rules**:
  * Safe: adding optional fields, adding link relations, widening enums with default handling, adding endpoints.
  * Breaking: removing/renaming fields, changing types, tightening validation, changing error codes without migration.
* **Sunset playbook**: Announce deprecations via headers + changelog, offer dual-write or shim periods, provide migration guides and timelines.

---

## **Security and trust**

* **Auth**: OAuth2/OIDC or signed tokens. Reject missing/expired tokens with `401`; reject unauthorized scopes/roles with `403`.
* **Transport**: Always HTTPS; HSTS enabled.
* **Input hygiene**: Size limits, timeouts, and structured validation. Normalize encodings and forbid unexpected fields if you cannot safely ignore them.
* **Auditability**: Emit audit events for access to sensitive resources; include actor, target, action, and correlation IDs.
* **Least privilege**: Scope tokens to minimal permissions; prefer fine-grained scopes over role strings baked into endpoints.
* **PII handling**: Redact sensitive fields in logs; tokenize where possible; document data retention.

---

## **Testing, docs, and DX**

* **Contract tests**: Consumer-driven contracts for critical clients; schema checks for every deployment.
* **Golden examples**: Provide runnable snippets and fixtures for each endpoint, including error cases.
* **Mockable defaults**: Offer sandbox keys and deterministic fixtures so clients can test without production data.
* **Observability**: Log request IDs, auth principal, latency buckets, and status codes; expose health/readiness endpoints.
* **Drift detection**: Monitor schema/object drift between prod responses and specs; alert on undocumented fields or missing fields.
* **Load/latency SLOs**: Publish SLOs and error budgets; test under those budgets with chaos for dependency failures.
* **Docs shape**: Each endpoint needs purpose, auth, request schema, response schema, examples, error codes, and pagination semantics in one place.

---

## **Hypermedia representation examples**

* **HAL-style**

```json
{
  "_links": {
    "self": { "href": "/cars/car_123" },
    "owner": { "href": "/users/user_9" },
    "collection": { "href": "/cars" }
  },
  "id": "car_123",
  "make": "Subaru",
  "model": "Forester"
}
```

* **JSON:API-style collection with pagination**

```json
{
  "data": [
    {
      "type": "cars",
      "id": "car_123",
      "attributes": { "make": "Subaru", "model": "Forester" },
      "relationships": {
        "owner": { "links": { "related": "/users/user_9" }, "data": { "type": "users", "id": "user_9" } }
      },
      "links": { "self": "/cars/car_123" }
    }
  ],
  "links": {
    "self": "/cars?page[limit]=1&page[cursor]=abc",
    "next": "/cars?page[limit]=1&page[cursor]=def"
  },
  "meta": { "count": 1 }
}
```

* **Async operation pattern**

```json
// POST /reports
// 202 Accepted, Location: /operations/op_789
{
  "status": "pending",
  "links": {
    "self": "/operations/op_789",
    "result": null
  },
  "meta": { "poll_after_seconds": 3 }
}

// GET /operations/op_789 (later)
{
  "status": "succeeded",
  "links": {
    "self": "/operations/op_789",
    "result": "/reports/rpt_456"
  }
}
```

---

## **Payment hypermedia examples (intent → authorize → capture → refund)**

* **JSON:API-style payment**

```json
{
  "data": {
    "type": "payments",
    "id": "pay_123",
    "attributes": {
      "status": "authorized",
      "amount": 4200,
      "currency": "USD",
      "capture_method": "manual",
      "authorized_at": "2024-06-01T12:00:00Z",
      "capturable_amount": 4200,
      "refundable_amount": 0
    },
    "relationships": {
      "customer": {
        "data": { "type": "customers", "id": "cus_9" },
        "links": { "related": "/customers/cus_9" }
      },
      "refunds": {
        "links": { "related": "/refunds?payment_id=pay_123" }
      }
    },
    "links": { "self": "/payments/pay_123" }
  },
  "links": {
    "capture": "/payments/pay_123/capture",
    "void": "/payments/pay_123/void"
  }
}
```

* **HAL-style payment with operations and state**

```json
{
  "_links": {
    "self": { "href": "/payments/pay_123" },
    "customer": { "href": "/customers/cus_9" },
    "refunds": { "href": "/refunds?payment_id=pay_123" },
    "capture": { "href": "/payments/pay_123/capture", "method": "POST" },
    "void": { "href": "/payments/pay_123/void", "method": "POST" },
    "operations": { "href": "/operations?subject=pay_123" }
  },
  "id": "pay_123",
  "status": "authorized",
  "amount": 4200,
  "currency": "USD",
  "authorized_at": "2024-06-01T12:00:00Z",
  "capturable_amount": 4200,
  "refundable_amount": 0
}
```

* **Capture flow with async operation**

```
POST /payments/pay_123/capture
Headers: Idempotency-Key: a1b2
Body: { "amount": 3000 }
→ 202 Accepted
Location: /operations/op_555

GET /operations/op_555
{
  "status": "succeeded",
  "links": {
    "self": "/operations/op_555",
    "result": "/payments/pay_123"
  },
  "meta": {
    "captured_amount": 3000,
    "remaining_capturable": 1200
  }
}
```

* **Refund example**

```json
{
  "id": "ref_777",
  "payment_id": "pay_123",
  "amount": 1200,
  "currency": "USD",
  "status": "succeeded",
  "created_at": "2024-06-01T12:10:00Z",
  "links": {
    "self": "/refunds/ref_777",
    "payment": "/payments/pay_123"
  }
}
```

---

## **Common footguns to avoid**

* **Deep nesting that encodes workflow**: Couples clients to your org chart and blocks refactors.
* **Verb-heavy endpoints**: `/doThing`, `/getUserCars`—hard to reason about safety and caching.
* **200 with error objects**: Breaks intermediaries and client heuristics.
* **Silent breaking changes**: Renaming fields or changing types without version bumps.
* **Leaky validation**: Returning first error only; omit field paths; hide the cause. Provide actionable validation errors.
* **Inconsistent pagination**: Mixing offset and cursor in the same surface; omitting `next`/`prev` links.
* **Enum surprises**: Treat enums as open; clients must ignore unknown values, servers must not reject future-safe inputs without version gates.
* **File uploads ad hoc**: Use `POST /uploads` with signed URLs or multipart; avoid base64 blobs in JSON.
* **Implicit timezones**: Always explicit UTC; never local server time.

---

## **Domain pattern: Payments (auth/capture/refund)**

* **Resources**
  * `/payments` (canonical payment resource: auth/capture/refund status)
  * `/payment-intents` (if you separate intent from execution)
  * `/refunds` (top-level, linked to payments)
* **States (example)**: `requires_payment_method` → `authorized` → `captured` → `partially_refunded`/`refunded` → `voided` → `failed`.
* **URLs and actions**
  * `POST /payments` (create + optionally authorize) → `201` with `status`.
  * `POST /payments/{id}/capture` (capture authorized funds; supports `amount`) → `200` or `202` with operation link.
  * `POST /payments/{id}/void` (cancel uncaptured auth) → `200`; `409` if already captured.
  * `POST /refunds` with `{ payment_id, amount }` → `201`; links to payment.
  * `GET /payments/{id}` (includes links to captures/refunds).
  * `GET /payments?customer_id=...&status=authorized` (query instead of deep nesting).
* **Idempotency and concurrency**
  * Require `Idempotency-Key` on `POST /payments`, `capture`, and `refunds` to survive retries.
  * Use `ETag` + `If-Match` on `PATCH /payments/{id}` if you support mutable fields (e.g., metadata).
* **Partial operations**
  * Capture: allow partial capture with remaining capturable amount tracked; subsequent captures allowed until fully captured or expired.
  * Refund: allow multiple partial refunds until total refunded equals captured amount. Return `409` if refund exceeds remaining refundable balance.
* **Errors to standardize**
  * `422` with codes like `card_declined`, `insufficient_funds`, `expired_auth`, `already_captured`, `refund_exceeds_balance`.
  * `409` for state conflicts (capture after void, refund before capture).
* **Async considerations**
  * Network-dependent processors: return `202` with operation resource for captures/refunds; include `status`, `failure_reason`, `links.result`.
  * Webhooks: sign them; include `event_id`, `created_at`, `type` (`payment.captured`, `payment.failed`), and a `payment` resource snapshot link.
* **Security & PCI boundaries**
  * Do not accept raw PANs unless in a compliant vault; prefer tokens. Mask sensitive fields in logs and responses.
* **Reconciliation**
  * Expose `GET /payouts`/`/settlements` or export endpoints for finance teams; include stable IDs and `created_at`/`available_on` timestamps.

---

## **Shipping checklist**

* Resource map with canonical URLs and one-level nesting rules documented.
* OpenAPI/JSON Schema validated; request/response examples (happy + error) checked in.
* Status code and error contract reviewed against RFC 7807 shape.
* Idempotency and concurrency story confirmed (ETag/If-Match or idempotency keys).
* Pagination/filtering/sorting documented with limits and examples.
* Deprecation/version policy stated; backwards-compatible changes verified.
* Observability: request ID, metrics, and rate limit headers present; health/readiness endpoints covered.
* Async: long-running operations return 202 + operation resource, with polling/webhook contract documented.
* Security: scopes/roles mapped to endpoints; sensitive fields redacted in logs; rate limits and abuse controls defined.

---

*End of document — API Design & Structure Playbook*
