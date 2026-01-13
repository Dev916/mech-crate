# unyform.ai MVP Product Requirements Document

## Phase 1: Enterprise Policy Enforcement + Standard Context

**Version:** 1.0  
**Date:** January 2025  
**Owner:** Product Team  
**Status:** Draft

---

## 1. Executive Overview

### 1.1 MVP Vision

Deliver the first enterprise-ready AI governance solution that enforces organizational policies at generation time—not after—while providing canonical context for AI-assisted development.

### 1.2 MVP Scope

**In Scope:**
- LLM Gateway with policy enforcement (input/output scanning)
- GitHub repository connector (code pattern ingestion)
- Organization Instruction Packs (codified standards)
- Basic Audit Log (compliance trail)
- IDE Integration (VS Code extension)
- CLI tooling (mx command enhancements)

**Out of Scope (Deferred to Phase 2+):**
- Confluence connector
- AI Recipe Generator
- Developer style profiles
- Conformance rewriting (auto-fix)
- Self-hosted deployment
- Recipe marketplace
- Governance dashboard (web UI)

### 1.3 Success Criteria

| Metric | Target |
|--------|--------|
| Pilot customers | 5-10 teams |
| Developers per pilot | 10-50 |
| Policy violations prevented | 100+ per team/month |
| Developer adoption rate | >80% of pilot team |
| Time to first value | <1 week |
| Gateway latency overhead | <500ms |
| System uptime | 99.5% |

### 1.4 Timeline

| Milestone | Target Date |
|-----------|-------------|
| LLM Gateway MVP | February 2025 |
| GitHub Connector MVP | March 2025 |
| Policy Engine v1 | March 2025 |
| VS Code Extension | May 2025 |
| Audit Log | May 2025 |
| Pilot Launch | June 2025 |

---

## 2. User Personas

### 2.1 IC Developer (Primary User)

**Profile:**
- Software engineer (junior to senior)
- Uses AI coding assistants daily (Copilot, Claude, GPT)
- Works in regulated or standards-conscious organization
- Wants AI to help them code faster

**Goals:**
- Write code faster with AI assistance
- Not have to manually check AI output against standards
- Avoid security review delays due to non-compliant code

**Pain Points:**
- AI generates code that doesn't match team conventions
- Spends time rewriting AI output to fit standards
- Gets code review feedback about style/security issues

**MVP Jobs to Be Done:**
1. Use AI assistants with automatic policy enforcement
2. Get context-aware suggestions that know our codebase
3. Trust that AI output won't cause security issues

---

### 2.2 Security/Compliance Lead (Buyer)

**Profile:**
- Security engineer or compliance officer
- Responsible for preventing security incidents
- Needs audit trails for compliance reporting
- Budget authority for security tools

**Goals:**
- Prevent AI from generating insecure code
- Have audit trail of AI usage for compliance
- Reduce manual security review burden

**Pain Points:**
- Can't track what AI is generating
- AI bypasses security controls
- No way to enforce policies on AI output

**MVP Jobs to Be Done:**
1. Define and enforce policies on AI usage
2. Get audit logs of all AI-generated code
3. Report on AI compliance for audits

---

### 2.3 Platform Engineering Lead (Champion)

**Profile:**
- Senior engineer or engineering manager
- Responsible for developer experience and tooling
- Drives adoption of new development tools
- Influence over technology decisions

**Goals:**
- Standardize development practices across teams
- Improve developer productivity with AI
- Reduce onboarding time for new developers

**Pain Points:**
- Every team uses AI differently
- No consistent patterns in AI-generated code
- Hard to scale best practices

**MVP Jobs to Be Done:**
1. Set up organization-wide AI standards
2. Onboard teams to governed AI usage
3. Measure productivity and compliance improvements

---

### 2.4 Engineering Manager (Decision Maker)

**Profile:**
- Manages 5-20 engineers
- Budget responsibility for team tools
- Reports on team productivity and quality

**Goals:**
- Ship faster without sacrificing quality
- Reduce code review cycles
- Show ROI on AI investment

**Pain Points:**
- AI adoption increases inconsistency
- More time spent on code review
- Hard to measure AI impact

**MVP Jobs to Be Done:**
1. Approve AI governance tool for team
2. Get metrics on AI productivity and compliance
3. Justify investment to leadership

---

## 3. User Stories

### 3.1 P0 - Must Have (MVP)

#### LLM Gateway Stories

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| GW-01 | As a developer, I want my AI requests to go through a policy-checked gateway so that I don't accidentally generate non-compliant code | Gateway proxies requests to configured LLM providers; policies are evaluated on every request |
| GW-02 | As a security lead, I want to block AI requests that contain sensitive data so that we don't leak secrets | Input scanning detects and blocks/redacts patterns matching secret policies |
| GW-03 | As a security lead, I want to block AI output that contains hardcoded credentials so that generated code is secure | Output scanning detects and blocks code with embedded secrets |
| GW-04 | As a developer, I want the gateway to be fast so that my AI experience isn't degraded | Gateway adds <500ms latency to requests |
| GW-05 | As an admin, I want to configure which LLM providers are allowed so that we control costs and data exposure | Admin can configure allowed providers (Claude, GPT, etc.) |

#### GitHub Connector Stories

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| GH-01 | As an admin, I want to connect our GitHub org so that the system can learn our codebase | OAuth flow connects GitHub org; repos are listed |
| GH-02 | As an admin, I want to select which repos to index so that we control what's ingested | Repo selection UI; selected repos are queued for indexing |
| GH-03 | As a developer, I want my AI requests to include context from our repos so that suggestions are relevant | RAG retrieval includes relevant code snippets from indexed repos |
| GH-04 | As an admin, I want indexing to update when code changes so that context stays current | Webhook-triggered incremental updates on push events |
| GH-05 | As a developer, I want my permissions respected so that I only get context I'm allowed to see | Context retrieval respects GitHub repo permissions |

#### Organization Instruction Pack Stories

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| IP-01 | As a platform lead, I want to define coding standards in a config file so that AI follows our conventions | YAML/JSON instruction pack format; loaded by gateway |
| IP-02 | As a developer, I want instruction packs automatically applied so that I don't have to remember to add context | Gateway injects instruction pack into system prompt |
| IP-03 | As an admin, I want to define allowed/forbidden dependencies so that AI suggests approved libraries | Dependency rules in instruction pack; validated in output |
| IP-04 | As an admin, I want to define naming conventions so that generated code matches our style | Naming rules in instruction pack; examples injected |
| IP-05 | As a platform lead, I want to version instruction packs so that we can roll back changes | Version control for instruction packs; history viewable |

#### Audit Log Stories

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| AU-01 | As a security lead, I want all AI requests logged so that we have an audit trail | Every gateway request creates audit event |
| AU-02 | As a compliance officer, I want to see which policies were evaluated so that I can report on enforcement | Audit events include policy IDs and results |
| AU-03 | As a security lead, I want to export audit logs so that we can feed SIEM | JSON export API; CSV download |
| AU-04 | As an admin, I want to set retention periods so that we comply with data policies | Configurable retention; automatic cleanup |
| AU-05 | As a security lead, I want to search audit logs by user, time, or policy so that I can investigate incidents | Search/filter API and basic UI |

#### IDE Integration Stories (VS Code)

| ID | Story | Acceptance Criteria |
|----|-------|---------------------|
| IDE-01 | As a developer, I want a VS Code extension so that I can use unyform from my IDE | Extension available in marketplace |
| IDE-02 | As a developer, I want to authenticate once so that requests are automatically authorized | OAuth login flow; token persisted |
| IDE-03 | As a developer, I want to see policy violations inline so that I can fix issues | Inline warnings/errors for blocked content |
| IDE-04 | As a developer, I want to trigger AI requests from the IDE so that I don't context switch | Command palette integration; inline triggers |
| IDE-05 | As a developer, I want to see what context was used so that I understand suggestions | Context sources shown in response UI |

---

### 3.2 P1 - Should Have (MVP Stretch)

| ID | Story | Priority | Notes |
|----|-------|----------|-------|
| CF-01 | As a developer, I want blocked content automatically fixed so that I don't have to manually edit | P1 | Conformance rewriting - stretch |
| CI-01 | As an admin, I want a CI check for AI-generated code so that PRs are validated | P1 | CI integration - stretch |
| DB-01 | As an admin, I want a web dashboard to configure policies so that I don't need CLI access | P1 | Dashboard MVP - stretch |

---

### 3.3 P2 - Nice to Have (Post-MVP)

| ID | Story | Priority | Notes |
|----|-------|----------|-------|
| CF-02 | Confluence connector | P2 | Phase 2 |
| DS-01 | Developer style profiles | P2 | Phase 3 |
| RM-01 | Recipe marketplace | P2 | Phase 4 |

---

## 4. Functional Requirements

### 4.1 LLM Gateway

#### 4.1.1 Request Handling

```
Request Flow:
1. Client sends request to gateway
2. Gateway authenticates client (API key or OAuth token)
3. Gateway evaluates INPUT policies on prompt
4. Gateway retrieves relevant context from RAG
5. Gateway injects instruction pack into system prompt
6. Gateway forwards request to LLM provider
7. Gateway evaluates OUTPUT policies on response
8. Gateway logs audit event
9. Gateway returns response to client
```

**Requirements:**

| ID | Requirement | Priority |
|----|-------------|----------|
| GW-R01 | Support Claude API (Anthropic) | P0 |
| GW-R02 | Support OpenAI API | P0 |
| GW-R03 | Support Azure OpenAI | P1 |
| GW-R04 | API key authentication | P0 |
| GW-R05 | OAuth token authentication | P1 |
| GW-R06 | Rate limiting per user/org | P0 |
| GW-R07 | Request/response logging | P0 |
| GW-R08 | Streaming response support | P0 |
| GW-R09 | Request timeout configuration | P0 |
| GW-R10 | Health check endpoint | P0 |

#### 4.1.2 Policy Evaluation

**Input Policy Actions:**
- `ALLOW` - Continue processing
- `BLOCK` - Reject request with error
- `REDACT` - Remove matched content and continue
- `REQUIRE_APPROVAL` - Queue for manual review (P1)

**Output Policy Actions:**
- `ALLOW` - Return to client
- `BLOCK` - Return error, do not return content
- `REDACT` - Remove matched content from response
- `REWRITE` - Transform content (P1)

---

### 4.2 Policy Engine

#### 4.2.1 MVP Policy Types

| Policy Type | Input | Output | Description |
|-------------|-------|--------|-------------|
| **secrets** | ✅ | ✅ | Detect credentials, API keys, tokens |
| **pii** | ✅ | ✅ | Detect personal identifiable information |
| **dependencies** | ❌ | ✅ | Validate allowed/forbidden packages |
| **patterns** | ❌ | ✅ | Match forbidden code patterns |
| **custom_regex** | ✅ | ✅ | User-defined regex rules |

#### 4.2.2 Policy Configuration Format

```yaml
# unyform-policy.yml
apiVersion: unyform.ai/v1
kind: PolicySet
metadata:
  name: acme-corp-mvp
  version: "1.0"

policies:
  # Secrets detection
  - name: no-secrets
    type: secrets
    severity: critical
    scope: [input, output]
    action: block
    patterns:
      - type: api_key
        regex: "(api[_-]?key|apikey)\\s*[:=]\\s*['\"][^'\"]{20,}['\"]"
      - type: password
        regex: "(password|passwd|pwd)\\s*[:=]\\s*['\"][^'\"]+['\"]"
      - type: jwt
        regex: "eyJ[A-Za-z0-9-_]+\\.eyJ[A-Za-z0-9-_]+\\.[A-Za-z0-9-_]+"
        
  # Forbidden dependencies
  - name: allowed-deps
    type: dependencies
    severity: error
    scope: [output]
    action: block
    rules:
      forbidden:
        - package: "moment"
          reason: "Use date-fns instead"
        - package: "request"
          reason: "Use fetch or axios"
          
  # Forbidden patterns
  - name: no-unsafe-patterns
    type: patterns
    severity: error
    scope: [output]
    action: block
    patterns:
      - pattern: "eval\\s*\\("
        message: "eval() is forbidden"
      - pattern: "innerHTML\\s*="
        message: "Use textContent or sanitization"
```

---

### 4.3 GitHub Connector

#### 4.3.1 Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| GH-R01 | GitHub App authentication | P0 |
| GH-R02 | Organization-level installation | P0 |
| GH-R03 | Repository selection (allow/deny list) | P0 |
| GH-R04 | Code file indexing (.js, .ts, .py, .go, .rs, .php, etc.) | P0 |
| GH-R05 | Webhook for push events | P0 |
| GH-R06 | Incremental indexing | P0 |
| GH-R07 | Permission-aware retrieval | P1 |
| GH-R08 | Branch selection (default branch only for MVP) | P0 |
| GH-R09 | File size limits (skip files >1MB) | P0 |
| GH-R10 | Rate limit handling | P0 |

#### 4.3.2 Indexing Pipeline

```
1. GitHub webhook triggers on push
2. Fetch changed files via GitHub API
3. Filter by file type and size
4. Parse files (AST for code, markdown for docs)
5. Chunk content into segments
6. Generate embeddings via sentence-transformers
7. Store in Weaviate with metadata
8. Update index timestamp
```

---

### 4.4 Audit Event Schema

```json
{
  "id": "uuid",
  "timestamp": "2025-01-15T14:32:17Z",
  "organization_id": "org_123",
  "user": {
    "id": "user_456",
    "email": "dev@company.com",
    "name": "Developer Name"
  },
  "request": {
    "id": "req_789",
    "method": "POST",
    "path": "/v1/chat/completions",
    "model": "claude-3-sonnet",
    "prompt_hash": "sha256:abc123...",
    "prompt_length": 1500,
    "context_sources": [
      {"type": "repo", "ref": "acme/api:main:src/auth.ts"},
      {"type": "instruction_pack", "ref": "acme-standards:v1.2"}
    ]
  },
  "response": {
    "status": 200,
    "model": "claude-3-sonnet",
    "response_hash": "sha256:def456...",
    "response_length": 2500,
    "tokens_input": 500,
    "tokens_output": 800
  },
  "policy_evaluation": {
    "policies_checked": 5,
    "policies_passed": 4,
    "policies_failed": 1,
    "action_taken": "block",
    "violations": [
      {
        "policy_id": "no-secrets",
        "policy_name": "No Hardcoded Secrets",
        "severity": "critical",
        "match": "api_key = 'sk-...'",
        "action": "block"
      }
    ]
  },
  "duration_ms": 1250,
  "gateway_version": "0.1.0"
}
```

---

## 5. Non-Functional Requirements

### 5.1 Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| Gateway latency overhead | <500ms p95 | Time added by gateway processing |
| Policy evaluation time | <100ms p95 | Time for all policy checks |
| Context retrieval time | <200ms p95 | Time for RAG retrieval |
| Indexing throughput | >1000 files/min | GitHub connector indexing rate |
| Concurrent requests | 100/org | Simultaneous gateway requests |

### 5.2 Reliability

| Metric | Target |
|--------|--------|
| Uptime | 99.5% |
| Error rate | <0.1% |
| Recovery time | <5 min |
| Data durability | 99.99% |

### 5.3 Security

| Requirement | Implementation |
|-------------|----------------|
| Authentication | API keys + OAuth 2.0 |
| Authorization | Org-scoped access control |
| Encryption in transit | TLS 1.3 |
| Encryption at rest | AES-256 |
| Audit logging | All operations logged |
| Secret handling | Never log or store secrets |
| Compliance | SOC2 Type 1 (target) |

### 5.4 Scalability

| Dimension | MVP Target | Growth Path |
|-----------|------------|-------------|
| Organizations | 50 | Horizontal scaling |
| Users per org | 100 | Already supported |
| Repositories per org | 50 | Incremental indexing |
| Requests per day | 100K total | Rate limiting + scaling |
| Audit events retained | 90 days | Configurable |

---

## 6. Out of Scope (Deferred)

| Feature | Reason | Target Phase |
|---------|--------|--------------|
| Confluence connector | Focus on GitHub first | Phase 2 |
| Developer style profiles | Requires more data | Phase 3 |
| Conformance rewriting | Complex, needs validation | Phase 2 |
| Recipe generator | Depends on pattern learning | Phase 2 |
| Self-hosted deployment | Enterprise feature | Phase 4 |
| Recipe marketplace | Ecosystem feature | Phase 4 |
| Team dashboard (full) | CLI first, web later | Phase 2 |
| JetBrains plugin | VS Code first | Phase 2 |
| Approval workflows | Enterprise feature | Phase 3 |

---

## 7. Success Metrics

### 7.1 Pilot Success Criteria

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to first policy enforcement | <1 day | From signup to first blocked request |
| Developer adoption | >80% of team | Active users / team size |
| Policy violations prevented | 100+/team/month | Blocked requests |
| False positive rate | <5% | User-reported false blocks |
| Developer satisfaction | >7/10 NPS | Survey |
| Renewal intent | >80% | End-of-pilot survey |

### 7.2 Product Metrics

| Metric | Definition |
|--------|------------|
| WAU | Weekly active users (made ≥1 request) |
| Requests per user | Avg requests per active user per week |
| Policy coverage | % of requests evaluated against policies |
| Violation rate | Violations / total requests |
| Latency p50/p95/p99 | Gateway overhead percentiles |
| Context utilization | % of requests using RAG context |

---

## 8. Dependencies & Risks

### 8.1 Technical Dependencies

| Dependency | Risk | Mitigation |
|------------|------|------------|
| Weaviate | Vector store availability | Managed service or redundancy |
| GitHub API | Rate limits, availability | Caching, incremental updates |
| LLM APIs | Cost, availability, changes | Multi-provider support |
| Cloudflare | Infrastructure dependency | Multi-cloud capability |

### 8.2 Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LLM API changes break gateway | Medium | High | Abstract provider layer, monitor changes |
| Policy false positives frustrate devs | High | Medium | Easy override, tuning, good defaults |
| GitHub rate limits slow indexing | Medium | Medium | Incremental sync, caching |
| Latency impacts developer experience | Medium | High | Aggressive optimization, caching |
| Security vulnerability in gateway | Low | Critical | Security audit, pen testing |

---

## 9. Open Questions

| Question | Owner | Due Date |
|----------|-------|----------|
| Which LLM provider to prioritize? (Claude vs GPT) | Product | Jan 20 |
| Pricing for pilot customers? (free vs paid pilot) | Business | Jan 25 |
| VS Code extension distribution? (marketplace vs direct) | Engineering | Feb 1 |
| On-prem option timeline for regulated customers? | Product | Feb 15 |

---

## 10. Appendix

### 10.1 API Endpoints (MVP)

```
# Gateway API
POST /v1/chat/completions      # Proxy to LLM with policy enforcement
GET  /v1/health                # Health check

# Management API
GET  /v1/policies              # List policies
POST /v1/policies              # Create policy
PUT  /v1/policies/:id          # Update policy
DELETE /v1/policies/:id        # Delete policy

GET  /v1/repos                 # List connected repos
POST /v1/repos/connect         # Connect GitHub org
POST /v1/repos/:id/index       # Trigger re-index

GET  /v1/audit                 # Query audit logs
GET  /v1/audit/export          # Export audit logs

GET  /v1/instruction-packs     # List instruction packs
POST /v1/instruction-packs     # Create instruction pack
PUT  /v1/instruction-packs/:id # Update instruction pack
```

### 10.2 Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Gateway | Rust | Performance, safety |
| Policy Engine | Rust | Performance, regex support |
| API Server | Rust (Axum) | Consistent stack |
| Vector Store | Weaviate | Existing integration |
| Database | PostgreSQL | Reliable, familiar |
| Queue | Redis | Simple, fast |
| IDE Extension | TypeScript | VS Code standard |
| CLI | Bash + Rust | Existing mx CLI |

### 10.3 Wireframes

*(To be added: VS Code extension UI, policy configuration UI)*

---

**Document History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | Jan 2025 | Product Team | Initial draft |

---

*Building AI that respects how your team builds.*
