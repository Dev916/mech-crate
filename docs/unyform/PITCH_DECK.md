# unyform.ai Pitch Deck

## Investor Presentation

**Version:** 1.0  
**Date:** January 2025  
**Presenters:** Michael Price, Matt Vitebsky

---

## Slide 1: Title

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                        ┌───────────────────────┐                            │
│                        │                       │                            │
│                        │      unyform.ai       │                            │
│                        │                       │                            │
│                        └───────────────────────┘                            │
│                                                                             │
│                                                                             │
│                                                                             │
│              AI Infrastructure Governance for                               │
│                    Engineering Teams                                        │
│                                                                             │
│                                                                             │
│                                                                             │
│                        Seed Round | Q1 2025                                 │
│                                                                             │
│                                                                             │
│              Michael Price, CEO    |    Matt Vitebsky, CTO                  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "unyform.ai makes AI work the way your team works—enforcing your standards, learning your patterns, and keeping your code consistent and secure."

---

## Slide 2: The Problem

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                  AI Coding Tools Are Everywhere                             │
│                  But They're Creating New Problems                          │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                                                                      │  │
│   │  😰 Context Fragmentation                                            │  │
│   │     AI doesn't understand your codebase, internal libraries,         │  │
│   │     or architectural decisions                                       │  │
│   │                                                                      │  │
│   │  🔓 Trust Gap                                                        │  │
│   │     Security and compliance teams can't see or control               │  │
│   │     what AI is generating                                            │  │
│   │                                                                      │  │
│   │  🔀 Consistency Drift                                                │  │
│   │     Every developer + every AI session = different patterns          │  │
│   │     and styles, increasing tech debt                                 │  │
│   │                                                                      │  │
│   │  ⏱️ Cold Start Tax                                                   │  │
│   │     Teams spend weeks building custom prompts and guardrails         │  │
│   │     before AI becomes useful                                         │  │
│   │                                                                      │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "92% of developers now use AI coding tools. But enterprises are discovering that AI without governance is creating new problems: inconsistent code, security risks, and compliance gaps."

---

## Slide 3: The Cost of Inconsistency

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│               The Hidden Cost of Uncontrolled AI                            │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│        ┌────────────────────────────────────────────────────────┐           │
│        │                                                        │           │
│        │              $2-8M / year                              │           │
│        │                                                        │           │
│        │        per 100-developer team                          │           │
│        │                                                        │           │
│        └────────────────────────────────────────────────────────┘           │
│                                                                             │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │                 │  │                 │  │                 │            │
│   │    60%          │  │    2-3x         │  │    $4.5M        │            │
│   │    of AI        │  │    longer       │  │    avg cost     │            │
│   │    suggestions  │  │    code         │  │    of a         │            │
│   │    need rework  │  │    reviews      │  │    security     │            │
│   │                 │  │                 │  │    incident     │            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Our research with early customers shows that uncontrolled AI actually increases costs. Developers spend more time reworking AI output than they save. Code reviews take longer because patterns are inconsistent. And security teams are flying blind."

---

## Slide 4: What Exists Today

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    The Market is Fragmented                                 │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│   ┌────────────────────────────────────────────────────────────────────┐   │
│   │                                                                    │   │
│   │  Today's Solutions Solve Parts of the Problem                      │   │
│   │                                                                    │   │
│   │                                                                    │   │
│   │  ┌─────────────────┐     ┌─────────────────┐                      │   │
│   │  │ Code Assistants │     │ Context Tools   │                      │   │
│   │  │   (Copilot)     │────▶│  (Codeium, etc) │                      │   │
│   │  │ Just suggestions│     │ Repo search     │                      │   │
│   │  └─────────────────┘     └─────────────────┘                      │   │
│   │          │                       │                                │   │
│   │          │    ╔═══════════════════════════╗                       │   │
│   │          │    ║                           ║                       │   │
│   │          └───▶║    MISSING: Unified       ║◀──────┐               │   │
│   │               ║    Trust + Consistency    ║       │               │   │
│   │               ║    + Personalization      ║       │               │   │
│   │               ╚═══════════════════════════╝       │               │   │
│   │                                                   │               │   │
│   │  ┌─────────────────┐     ┌─────────────────┐     │               │   │
│   │  │ Security Tools  │     │ Workflow Tools  │─────┘               │   │
│   │  │   (Snyk, etc)   │────▶│  (PR review)    │                      │   │
│   │  │ After the fact  │     │ Too late        │                      │   │
│   │  └─────────────────┘     └─────────────────┘                      │   │
│   │                                                                    │   │
│   └────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Existing tools address pieces of this problem. Copilot gives suggestions. Codeium adds context. Snyk scans for vulnerabilities. But no one provides the unified layer that connects trust, consistency, and personalization at the point of generation."

---

## Slide 5: The Gap

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                 No One Owns the Trust Layer                                 │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│        What enterprises need:                                               │
│                                                                             │
│        ✗  Policy enforcement BEFORE code is written                         │
│           (Not just detection after the fact)                               │
│                                                                             │
│        ✗  A unified enterprise semantic context                             │
│           (One source of truth for architecture, libraries, and rules)      │
│                                                                             │
│        ✗  Per-engineer style alignment                                      │
│           (Without breaking enterprise constraints)                         │
│                                                                             │
│        ✗  Evidence and audit trails                                         │
│           (Proof of what AI produced, why it was allowed)                   │
│                                                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │    "We're either blocking AI entirely or letting it run wild.       │  │
│   │     Neither option works."                                          │  │
│   │                                                                     │  │
│   │                                   — VP Engineering, Fortune 500     │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Security teams tell us they're either blocking AI entirely—losing productivity—or allowing it with no visibility. Enterprises need a middle path: AI that's safe, consistent, and auditable."

---

## Slide 6: The Solution

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                           unyform.ai                                        │
│                                                                             │
│           The Enterprise AI Trust and Consistency Layer                     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│        AI that builds like your team builds                                 │
│                                                                             │
│                                                                             │
│        ✓  Enforce policies at generation time                               │
│           Block secrets, forbidden patterns, and security risks             │
│           BEFORE they reach the developer                                   │
│                                                                             │
│        ✓  Inject enterprise context automatically                           │
│           AI knows your codebase, libraries, and patterns                   │
│                                                                             │
│        ✓  Audit everything                                                  │
│           Complete trail of what AI generated and why                       │
│                                                                             │
│        ✓  Optional personalization                                          │
│           Match each developer's style within enterprise rules              │
│                                                                             │
│                                                                             │
│        ┌─────────────────────────────────────────────────────────────────┐ │
│        │                                                                 │ │
│        │   Works with Claude, GPT-4, and your existing AI tools          │ │
│        │                                                                 │ │
│        └─────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "unyform.ai sits between developers and AI models. We enforce your policies, inject your context, and log everything—without changing how developers work. They use their favorite AI tools, but now those tools work the way the enterprise needs."

---

## Slide 7: How It Works

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                        Four Pillars                                         │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐    │
│  │             │   │             │   │             │   │             │    │
│  │   INGEST    │   │    LEARN    │   │  GENERATE   │   │   GOVERN    │    │
│  │             │   │             │   │             │   │             │    │
│  │  Connect    │   │  Analyze    │   │  Produce    │   │  Enforce    │    │
│  │  GitHub     │   │  patterns   │   │  compliant  │   │  policies   │    │
│  │  repos,     │   │  and        │   │  code       │   │  and audit  │    │
│  │  docs,      │   │  standards  │   │  using      │   │  every      │    │
│  │  configs    │   │  from code  │   │  recipes    │   │  request    │    │
│  │             │   │             │   │             │   │             │    │
│  └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘    │
│         │                 │                 │                 │            │
│         │                 │                 │                 │            │
│         ▼                 ▼                 ▼                 ▼            │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                                                                     │  │
│  │                      LLM Gateway                                    │  │
│  │         Policy checks → Context injection → AI request →            │  │
│  │         Output validation → Audit log → Return to developer         │  │
│  │                                                                     │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "It starts with ingestion—we connect to your GitHub repos and analyze your codebase. Then we learn your patterns and standards. When a developer makes an AI request, we generate code using your recipes and context, and we govern every request through our policy engine with complete audit trails."

---

## Slide 8: Demo / Product Screenshot

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                         Product in Action                                   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐ │
│  │                                                                       │ │
│  │  ┌─────────────────────────────────────────────────────────────────┐ │ │
│  │  │ VS Code                                              [x] [ ] [-]│ │ │
│  │  ├─────────────────────────────────────────────────────────────────┤ │ │
│  │  │                                                                 │ │ │
│  │  │  // Generate authentication middleware                          │ │ │
│  │  │  // @unyform                                                    │ │ │
│  │  │                                                                 │ │ │
│  │  │  ┌─────────────────────────────────────────────────────────┐   │ │ │
│  │  │  │ ✅ Policy Check Passed                                  │   │ │ │
│  │  │  │                                                         │   │ │ │
│  │  │  │ Context: auth.ts, middleware.ts, jwt-utils.ts           │   │ │ │
│  │  │  │ Instruction Pack: acme-standards-v2                     │   │ │ │
│  │  │  │ Policies: 5 checked, 5 passed                           │   │ │ │
│  │  │  └─────────────────────────────────────────────────────────┘   │ │ │
│  │  │                                                                 │ │ │
│  │  │  export async function authMiddleware(req, res, next) {         │ │ │
│  │  │    const token = req.headers.authorization?.split(' ')[1];      │ │ │
│  │  │    if (!token) {                                                │ │ │
│  │  │      return res.status(401).json({ error: 'Unauthorized' });    │ │ │
│  │  │    }                                                            │ │ │
│  │  │    const user = await verifyJWT(token); // Uses internal lib    │ │ │
│  │  │    req.user = user;                                             │ │ │
│  │  │    next();                                                      │ │ │
│  │  │  }                                                              │ │ │
│  │  │                                                                 │ │ │
│  │  └─────────────────────────────────────────────────────────────────┘ │ │
│  │                                                                       │ │
│  │  Generated code uses internal libraries and matches team patterns     │ │
│  │                                                                       │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Here's what developers see. They use their normal AI workflow—in VS Code, same as before. But now the response shows them which policies were checked, what context was used, and they get code that uses their internal libraries and matches their team's patterns. No manual prompt engineering required."

---

## Slide 9: Key Differentiators

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                       What Makes Us Different                               │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   🛡️  Guardrails BEFORE code is written                             │  │
│   │       Not post-generation scanning—we prevent issues at the source   │  │
│   │                                                                     │  │
│   │   🧠  Policy + Architecture as first-class context                  │  │
│   │       Not just code embeddings—we understand your rules and patterns │  │
│   │                                                                     │  │
│   │   👤  Developer style profiles (optional)                           │  │
│   │       Personalization that doesn't break enterprise constraints      │  │
│   │                                                                     │  │
│   │   🔌  Platform agnostic                                             │  │
│   │       Works with Claude, GPT, any model—not locked to one vendor     │  │
│   │                                                                     │  │
│   │   📋  Evidence-centric design                                       │  │
│   │       Audit trails designed for security and compliance from day one │  │
│   │                                                                     │  │
│   │   🏗️  Infrastructure included                                       │  │
│   │       MechCrate recipes—not just governance, but scaffolding too     │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Six things set us apart. First, we prevent issues before they happen—at generation time, not after. Second, we treat your policies and architecture as first-class context. Third, we offer optional per-developer personalization. Fourth, we work with any AI model. Fifth, we're evidence-centric—audit trails are core to our design. And sixth, we include infrastructure scaffolding through MechCrate."

---

## Slide 10: Technology

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                       Built for Performance                                 │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                                                                      │  │
│  │  LLM Gateway + Policy Engine                 [ Rust ]                │  │
│  │  High-performance proxy with <500ms overhead                         │  │
│  │                                                                      │  │
│  │  Enterprise Context Service                  [ Weaviate + RAG ]      │  │
│  │  Semantic search over your codebase                                  │  │
│  │                                                                      │  │
│  │  MCP Server (Model Context Protocol)         [ 44 Tools ]            │  │
│  │  LLM-facing interface for operations                                 │  │
│  │                                                                      │  │
│  │  MechCrate Recipes                           [ 7 Stacks ]            │  │
│  │  Laravel, Nuxt, Astro, Rust-API, Rust-Leptos, Zola                   │  │
│  │                                                                      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│                                                                             │
│     ┌───────────────────────────────────────────────────────────────────┐  │
│     │                                                                   │  │
│     │   "MechCrate is the foundation—it already works.                  │  │
│     │    unyform.ai is the enterprise layer on top."                    │  │
│     │                                                                   │  │
│     └───────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Our stack is built for enterprise scale. The gateway and policy engine are written in Rust for performance—under 500ms overhead. We use Weaviate for semantic search. Our MCP server already has 44 tools for LLM integration. And MechCrate provides 7 production-ready recipes for common stacks."

---

## Slide 11: Market Opportunity

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                       Massive Market Opportunity                            │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│          ┌─────────────────────────────────────────────────────────┐       │
│          │                                                         │       │
│          │                    $51B                                 │       │
│          │                                                         │       │
│          │           Platform Engineering Market                   │       │
│          │                  by 2027                                │       │
│          │                                                         │       │
│          │              (Gartner)                                  │       │
│          │                                                         │       │
│          └─────────────────────────────────────────────────────────┘       │
│                                                                             │
│                                                                             │
│    ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐        │
│    │                 │   │                 │   │                 │        │
│    │     $15B        │   │      92%        │   │      60%        │        │
│    │                 │   │                 │   │                 │        │
│    │   AI coding     │   │  Developer      │   │  Enterprises    │        │
│    │   tools by      │   │  AI adoption    │   │  cite security  │        │
│    │   2028          │   │  rate           │   │  as #1 barrier  │        │
│    │                 │   │                 │   │                 │        │
│    └─────────────────┘   └─────────────────┘   └─────────────────┘        │
│                                                                             │
│                                                                             │
│         Our target: $500M segment (AI governance for dev tools)             │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "The platform engineering market is projected to hit $51 billion by 2027. AI coding tools are a $15 billion market growing rapidly. 92% of developers use AI tools, but 60% of enterprises cite security as their top barrier to adoption. We're targeting a $500 million segment at the intersection: AI governance for development tools."

---

## Slide 12: Competitive Landscape

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                     Competitive Positioning                                 │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│                        Enterprise Features                                  │
│                              ↑                                              │
│                              │                                              │
│                              │        ★ unyform.ai                          │
│                  Nexos.ai    │           (Generation-time                   │
│                  (DLP focus) │            + Infrastructure)                 │
│                              │                                              │
│                              │                                              │
│    ──────────────────────────┼──────────────────────────────────►          │
│    Developer                 │                      Enterprise              │
│    Focus                     │                      Focus                   │
│                              │                                              │
│        Lando                 │          Zenity                              │
│        (Local dev)           │          (GenAI governance)                  │
│                              │                                              │
│              GitHub Copilot  │                                              │
│              (Just suggestions)                                             │
│                              ↓                                              │
│                        Basic Features                                       │
│                                                                             │
│                                                                             │
│   unyform.ai differentiators:                                               │
│   • Generation-time enforcement (not post-hoc)                              │
│   • Infrastructure scaffolding included                                     │
│   • Open core (MechCrate)                                                   │
│   • 30-50% lower cost than pure enterprise                                  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "We compete at the intersection of developer experience and enterprise governance. Copilot gives suggestions but no governance. Nexos is enterprise-heavy, DLP-focused. Zenity focuses on GenAI risk. Lando is local-only. We uniquely combine generation-time enforcement with infrastructure scaffolding and an open-core foundation."

---

## Slide 13: Business Model

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                        SaaS Business Model                                  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│   ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐         │
│   │   COMMUNITY     │   │      TEAM       │   │   ENTERPRISE    │         │
│   ├─────────────────┤   ├─────────────────┤   ├─────────────────┤         │
│   │                 │   │                 │   │                 │         │
│   │     FREE        │   │     $49         │   │    Custom       │         │
│   │                 │   │   /seat/mo      │   │                 │         │
│   │                 │   │                 │   │                 │         │
│   │  • MechCrate    │   │  • LLM Gateway  │   │  • Unlimited    │         │
│   │  • CLI          │   │  • Policies     │   │  • SSO/SAML     │         │
│   │  • Recipes      │   │  • GitHub       │   │  • Self-hosted  │         │
│   │  • Community    │   │  • Audit logs   │   │  • SLA          │         │
│   │                 │   │  • VS Code      │   │  • Support      │         │
│   │                 │   │                 │   │                 │         │
│   └─────────────────┘   └─────────────────┘   └─────────────────┘         │
│                                                                             │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │   Target Unit Economics                                           │    │
│   │                                                                   │    │
│   │   • Gross Margin: >70%                                            │    │
│   │   • CAC Payback: <12 months                                       │    │
│   │   • LTV/CAC: >3x                                                  │    │
│   │   • Net Retention: >110%                                          │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "We have a classic SaaS model with three tiers. Community is free and drives awareness through MechCrate. Team tier at $49/seat covers most use cases. Enterprise is custom pricing for large organizations with SSO, self-hosted, and SLA requirements. We're targeting 70%+ gross margins and 110%+ net retention through expansion."

---

## Slide 14: Traction

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                        Current Traction                                     │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │                     What We've Built                              │    │
│   │                                                                   │    │
│   │   ✅ MechCrate open source (foundation complete)                  │    │
│   │   ✅ 7 production-ready recipes                                   │    │
│   │   ✅ MCP Server with 44 tools                                     │    │
│   │   ✅ RAG documentation search (Weaviate)                          │    │
│   │   ✅ Cloudflare infrastructure templates                          │    │
│   │   ✅ Docker + Traefik scaffolding                                 │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│                                                                             │
│   ┌───────────────────────────────────────────────────────────────────┐    │
│   │                                                                   │    │
│   │                     What's Next (MVP)                             │    │
│   │                                                                   │    │
│   │   🔄 LLM Gateway with policy enforcement                          │    │
│   │   🔄 GitHub connector for repo ingestion                          │    │
│   │   🔄 Organization instruction packs                               │    │
│   │   🔄 Audit log with compliance export                             │    │
│   │   🔄 VS Code extension                                            │    │
│   │                                                                   │    │
│   └───────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│                                                                             │
│              MVP target: June 2025 | First pilots: Q1 2025                  │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "We're not starting from scratch. MechCrate—our open-source foundation—is already working. We have 7 recipes, 44 MCP tools, RAG search, and Cloudflare templates. The MVP adds the LLM Gateway, policy engine, GitHub connector, and VS Code extension. We're targeting first pilots in Q1 and full MVP by June 2025."

---

## Slide 15: Roadmap

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                          Product Roadmap                                    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│   2025                           2026                                       │
│   ─────────────────────────────  ─────────────────────────────────         │
│                                                                             │
│   Q1        Q2        Q3        Q4        Q1        Q2                     │
│   │         │         │         │         │         │                      │
│   │         │         │         │         │         │                      │
│   ▼         ▼         ▼         ▼         ▼         ▼                      │
│                                                                             │
│   ┌─────────────────────────────┐                                          │
│   │   PHASE 1: MVP              │                                          │
│   │   LLM Gateway               │                                          │
│   │   Policy Engine             │                                          │
│   │   GitHub Connector          │                                          │
│   │   Audit Log                 │                                          │
│   │   VS Code Extension         │                                          │
│   └─────────────────────────────┘                                          │
│             ┌─────────────────────────────┐                                │
│             │   PHASE 2: Scale            │                                │
│             │   Cross-repo context        │                                │
│             │   Conformance rewriting     │                                │
│             │   Web dashboard             │                                │
│             │   JetBrains plugin          │                                │
│             └─────────────────────────────┘                                │
│                       ┌─────────────────────────────┐                      │
│                       │   PHASE 3: Personalize      │                      │
│                       │   Developer style profiles  │                      │
│                       │   Recipe generator          │                      │
│                       │   Evaluation suite          │                      │
│                       └─────────────────────────────┘                      │
│                                 ┌─────────────────────────────┐            │
│                                 │   PHASE 4: Platform         │            │
│                                 │   Recipe marketplace        │            │
│                                 │   Partner ecosystem         │            │
│                                 │   Self-hosted Enterprise    │            │
│                                 └─────────────────────────────┘            │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Our roadmap has four phases. Phase 1—the MVP—delivers the core LLM Gateway, policy engine, and GitHub integration by mid-2025. Phase 2 scales with cross-repo context and a web dashboard. Phase 3 adds developer personalization. Phase 4 builds the platform ecosystem with a marketplace and partner integrations."

---

## Slide 16: Team

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                           The Team                                          │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│        ┌─────────────────────────────────────────────────────────┐         │
│        │                                                         │         │
│        │   ┌─────────┐                    ┌─────────┐            │         │
│        │   │         │                    │         │            │         │
│        │   │  [pic]  │                    │  [pic]  │            │         │
│        │   │         │                    │         │            │         │
│        │   └─────────┘                    └─────────┘            │         │
│        │                                                         │         │
│        │   Michael Price                  Matt Vitebsky          │         │
│        │   CEO & Co-founder               CTO & Co-founder       │         │
│        │                                                         │         │
│        │   • 15+ years engineering        • 15+ years systems    │         │
│        │   • Founded 2 companies          • Ex-Amazon, Ex-Google │         │
│        │   • Enterprise SaaS focus        • Platform engineering │         │
│        │   • M&A experience               • Open source leader   │         │
│        │                                                         │         │
│        └─────────────────────────────────────────────────────────┘         │
│                                                                             │
│                                                                             │
│        ┌─────────────────────────────────────────────────────────┐         │
│        │                                                         │         │
│        │   "We've built infrastructure platforms before.         │         │
│        │    We know what enterprise teams need."                 │         │
│        │                                                         │         │
│        └─────────────────────────────────────────────────────────┘         │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Michael brings 15 years of enterprise SaaS experience, having founded two companies and led M&A transactions. Matt is a systems engineer with experience at Amazon and Google, specializing in platform engineering and open-source projects. We've built infrastructure platforms before—we know what enterprise teams need."

---

## Slide 17: The Ask

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                            The Ask                                          │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│           ┌───────────────────────────────────────────────┐                │
│           │                                               │                │
│           │              $2M Seed Round                   │                │
│           │                                               │                │
│           └───────────────────────────────────────────────┘                │
│                                                                             │
│                                                                             │
│           Use of Funds                                                      │
│           ─────────────                                                     │
│                                                                             │
│           ┌────────────────────────────────────────────────────────────┐   │
│           │                                                            │   │
│           │   Engineering (50%)          $1,000,000                    │   │
│           │   ├── 3 engineers (Gateway, Policy, Frontend)              │   │
│           │   └── Infrastructure and tooling                           │   │
│           │                                                            │   │
│           │   Go-to-Market (30%)         $600,000                      │   │
│           │   ├── Head of Sales                                        │   │
│           │   ├── Developer Advocate                                   │   │
│           │   └── Marketing programs                                   │   │
│           │                                                            │   │
│           │   Operations (20%)           $400,000                      │   │
│           │   ├── Legal and compliance                                 │   │
│           │   ├── Security audit (SOC2)                                │   │
│           │   └── Runway buffer                                        │   │
│           │                                                            │   │
│           └────────────────────────────────────────────────────────────┘   │
│                                                                             │
│                                                                             │
│           18-month runway to $1M ARR and Series A                           │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "We're raising a $2M seed round. 50% goes to engineering—hiring 3 engineers to build out the gateway, policy engine, and IDE integrations. 30% goes to go-to-market—head of sales, developer advocate, and marketing programs. 20% covers operations including SOC2 compliance. This gives us 18 months of runway to hit $1M ARR and position for Series A."

---

## Slide 18: Vision

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                          The Vision                                         │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                                                             │
│                                                                             │
│        ┌───────────────────────────────────────────────────────────┐       │
│        │                                                           │       │
│        │                                                           │       │
│        │     Every engineering team will use AI.                   │       │
│        │                                                           │       │
│        │     The winners will be those who can                     │       │
│        │     trust their AI to build like they build.              │       │
│        │                                                           │       │
│        │                                                           │       │
│        │     unyform.ai is the trust layer that                    │       │
│        │     makes that possible.                                  │       │
│        │                                                           │       │
│        │                                                           │       │
│        └───────────────────────────────────────────────────────────┘       │
│                                                                             │
│                                                                             │
│                                                                             │
│             From local development to production.                           │
│             From individual developer to enterprise scale.                  │
│             AI that builds like your team builds.                           │
│                                                                             │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Every engineering team will use AI—that's inevitable. The winners will be those who can trust their AI to build like they build. unyform.ai is the trust layer that makes that possible. From local development to production, from individual developer to enterprise scale—AI that builds like your team builds."

---

## Slide 19: Contact

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│                        ┌───────────────────────┐                            │
│                        │                       │                            │
│                        │      unyform.ai       │                            │
│                        │                       │                            │
│                        └───────────────────────┘                            │
│                                                                             │
│                                                                             │
│                      AI that builds like your team builds                   │
│                                                                             │
│                                                                             │
│                                                                             │
│                         hello@unyform.ai                                    │
│                                                                             │
│                         unyform.ai                                          │
│                                                                             │
│                         github.com/unyform                                  │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
│              Michael Price               Matt Vitebsky                      │
│              michael@unyform.ai          matt@unyform.ai                    │
│              @michaelprice               @mattvitebsky                      │
│                                                                             │
│                                                                             │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Speaker Notes:**
> "Thank you for your time. We'd love to continue the conversation. You can reach us at hello@unyform.ai. Let's talk about how we can build the trust layer for AI-powered development together."

---

## Appendix Slides

### A1: Detailed Financial Projections

```
Year 1:
  Q1: $0 (pilots)
  Q2: $117K ARR (10 teams)
  Q3: $367K ARR (25 teams)
  Q4: $882K ARR (50 teams)

Year 2:
  Q1: $1.56M ARR
  Q2: $2.4M ARR
  Q3: $3.5M ARR
  Q4: $4.8M ARR (+ enterprise)
```

### A2: Customer Testimonials

*(To be added after pilot completion)*

### A3: Detailed Competitive Analysis

*(See COMPETITIVE_ANALYSIS.md)*

### A4: Technical Architecture Deep Dive

*(See TECHNICAL_ARCHITECTURE.md)*

---

**Presentation Notes:**

| Slide | Time | Key Point |
|-------|------|-----------|
| 1 | 30s | Hook with tagline |
| 2-5 | 3 min | Problem + gap |
| 6-8 | 4 min | Solution + demo |
| 9-10 | 2 min | Differentiation |
| 11-12 | 2 min | Market + competition |
| 13-14 | 2 min | Business model + traction |
| 15-16 | 2 min | Roadmap + team |
| 17-18 | 2 min | Ask + vision |
| 19 | 30s | Close |

**Total: ~18 minutes** (leaves room for Q&A in 30-minute meeting)

---

**Document History:**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | Jan 2025 | Michael Price | Initial draft |

---

*AI that builds like your team builds.*
