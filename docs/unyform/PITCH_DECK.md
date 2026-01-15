# unyform.ai Pitch Deck

## Investor Presentation

**Version:** 1.0  
**Date:** January 2025  
**Presenters:** Michael Price, Matt Vitebsky

---

## Slide 1: Title

```mermaid
flowchart TB
    LOGO[unyform.ai]
    TAG[The Rippling for AI-Assisted Development]
    SUB[Security + Velocity for Every Stack]
    ROUND[Seed Round - Q1 2025]
    TEAM[Michael Price, CEO - Matt Vitebsky, CTO]
    
    LOGO --> TAG --> SUB --> ROUND --> TEAM
    
    style LOGO fill:#6366f1,color:#fff,font-weight:bold
    style TAG fill:#8b5cf6,color:#fff
    style SUB fill:#a855f7,color:#fff
    style ROUND fill:#374151,color:#fff
    style TEAM fill:#374151,color:#fff
```

**Speaker Notes:**
> "unyform.ai is the Rippling for AI-assisted development. One central hub that connects to your entire stack—tailored to fit each organization like a fine tuxedo. Security AND velocity, not security OR velocity."

---

## Slide 2: The Problem

```mermaid
flowchart TB
    subgraph problems ["AI Coding Tools Are Everywhere—But Creating New Problems"]
        P1["😰 <b>Context Fragmentation</b><br/>AI doesn't understand your codebase,<br/>internal libraries, or architecture"]
        P2["🔓 <b>Trust Gap</b><br/>Security teams can't see or control<br/>what AI is generating"]
        P3["🔀 <b>Consistency Drift</b><br/>Every developer × every AI session =<br/>different patterns, more tech debt"]
        P4["⏱️ <b>Cold Start Tax</b><br/>Weeks building custom prompts and<br/>guardrails before AI becomes useful"]
        P5["📊 <b>Visibility Blind Spot</b><br/>Leadership can't measure AI impact,<br/>ROI, or code quality"]
    end
    
    style problems fill:#ef4444,color:#fff
    style P1 fill:#dc2626,color:#fff
    style P2 fill:#dc2626,color:#fff
    style P3 fill:#dc2626,color:#fff
    style P4 fill:#dc2626,color:#fff
    style P5 fill:#dc2626,color:#fff
```

**Speaker Notes:**
> "92% of developers now use AI coding tools. But enterprises face a painful choice: block AI entirely and lose productivity, or allow it with zero visibility. CTOs are asking: 'How much of our code is AI-generated? Is it secure? Is it making us faster or creating tech debt?'"

---

## Slide 3: The Cost of Inconsistency

```mermaid
flowchart TB
    subgraph cost ["The Hidden Cost of Uncontrolled AI"]
        MAIN["$2-8M/year per 100-dev team"]
    end
    
    subgraph metrics [" "]
        M1["60% of AI suggestions need rework"]
        M2["2-3x longer code reviews"]
        M3["$4.5M avg security incident"]
        M4["0% visibility into AI origin"]
    end
    
    cost --> metrics
    
    style cost fill:#ef4444,color:#fff
    style MAIN fill:#dc2626,color:#fff
    style M1 fill:#f97316,color:#fff
    style M2 fill:#f97316,color:#fff
    style M3 fill:#f97316,color:#fff
    style M4 fill:#f97316,color:#fff
```

**Speaker Notes:**
> "Uncontrolled AI actually increases costs. 60% of AI output needs rework. Code reviews take 2-3x longer due to inconsistency. And here's the killer: leadership has ZERO visibility into how much code is AI-generated, whether it's secure, or what the ROI actually is."

---

## Slide 4: What Exists Today

```mermaid
flowchart TB
    subgraph existing ["Today's Solutions Solve Parts of the Problem"]
        CA["<b>Code Assistants</b><br/>(Copilot)<br/>Just suggestions"]
        CT["<b>Context Tools</b><br/>(Codeium, etc)<br/>Repo search only"]
        ST["<b>Security Tools</b><br/>(Snyk, etc)<br/>After the fact"]
        WT["<b>Workflow Tools</b><br/>(PR review)<br/>Too late"]
    end
    
    subgraph gap ["MISSING: The Central Hub"]
        G1["Unified trust + consistency"]
        G2["Enterprise stack integration"]
        G3["Leadership analytics"]
        G4["Zero-config developer experience"]
    end
    
    CA --> gap
    CT --> gap
    ST --> gap
    WT --> gap
    
    style existing fill:#374151,color:#fff
    style gap fill:#ef4444,color:#fff
```

**Speaker Notes:**
> "The market is fragmented. Copilot gives suggestions but no governance. Snyk scans after code is written—too late. And critically: no one provides a central hub that connects ALL the tools, tailored to each enterprise's stack. Everyone has to integrate 5-10 tools separately."

---

## Slide 5: The Gap

```mermaid
flowchart TB
    subgraph needs ["What Enterprises Need"]
        N1["Policy enforcement BEFORE code is written"]
        N2["A central hub connecting their entire stack"]
        N3["Zero-config onboarding for developers"]
        N4["Leadership dashboards with AI vs Human metrics"]
        N5["Tailored fit for THEIR specific tools"]
    end
    
    subgraph quote ["VP Engineering Quote"]
        Q["We're blocking AI or letting it run wild - neither works"]
    end
    
    needs --> quote
    
    style needs fill:#ef4444,color:#fff
    style quote fill:#374151,color:#fff
```

**Speaker Notes:**
> "Security teams tell us they're blocking AI or allowing it blind. But here's the real gap: leadership can't answer basic questions. How much code is AI-generated? Is it more or less buggy? Are we actually faster? No one can tell them—until now."

---

## Slide 6: The Solution

```mermaid
flowchart TB
    subgraph solution ["unyform.ai - The Rippling for AI-Assisted Development"]
        TAG["Security + Velocity, Tailored Like a Fine Tuxedo"]
    end
    
    subgraph features ["Key Features"]
        F1["Central Hub: One integration, entire stack"]
        F2["Zero-Config: Sign in, bot onboards"]
        F3["Generation-Time Security: Prevention not detection"]
        F4["Leadership Analytics: AI vs Human, ROI"]
    end
    
    subgraph works ["Works with Claude, GPT-4, and existing AI tools"]
        W1[" "]
    end
    
    solution --> features --> works
    
    style solution fill:#6366f1,color:#fff
    style TAG fill:#8b5cf6,color:#fff
    style features fill:#22c55e,color:#fff
    style works fill:#374151,color:#fff
```

**Speaker Notes:**
> "unyform.ai is the central hub that connects AI to your entire stack. Platform team sets it up once—connects GitHub, Confluence, Jira. Developers just sign in. An onboarding bot walks them through. And leadership finally gets the dashboards they've been begging for: AI vs Human code, conformance, velocity, actual ROI."

---

## Slide 7: How It Works — The Hub Model

```mermaid
flowchart TB
    subgraph admin ["1️⃣ ADMIN SETUP (Once)"]
        PA[Platform Admin]
    end
    
    subgraph hub ["2️⃣ THE HUB"]
        direction TB
        GW["LLM Gateway<br/>Policy Engine<br/>Context Service<br/>Analytics Engine"]
    end
    
    subgraph spokes ["3️⃣ AUTO-CONNECTED SPOKES"]
        GH[GitHub]
        CONF[Confluence]
        JIRA[Jira]
        SLACK[Slack]
    end
    
    subgraph devs ["4️⃣ ZERO-CONFIG DEVELOPERS"]
        DEV1["Dev signs in"]
        DEV2["Bot onboards"]
        DEV3["AI just works"]
    end
    
    subgraph leadership ["5️⃣ LEADERSHIP VISIBILITY"]
        L1["AI vs Human Code %"]
        L2["Conformance Score"]
        L3["Velocity Metrics"]
        L4["ROI Dashboard"]
    end
    
    PA --> hub
    hub <--> spokes
    devs --> hub
    hub --> leadership
    
    style admin fill:#22c55e,color:#fff
    style hub fill:#6366f1,color:#fff
    style spokes fill:#8b5cf6,color:#fff
    style devs fill:#374151,color:#fff
    style leadership fill:#f59e0b,color:#fff
```

**Speaker Notes:**
> "Think of it like Rippling for HR. Admin sets it up once—connects GitHub, Confluence, Jira. The hub handles everything. Developers just sign in, our bot onboards them, and AI works with their existing tools. Leadership finally gets the dashboards: AI vs Human code, conformance, velocity, ROI."

---

## Slide 8: Developer Experience

```mermaid
flowchart LR
    subgraph before ["BEFORE unyform.ai"]
        B1["Download extension"]
        B2["Configure settings"]
        B3["Set up API keys"]
        B4["Write custom prompts"]
        B5["Hope it works"]
    end
    
    subgraph after ["AFTER unyform.ai"]
        A1["Download extension"]
        A2["Sign in"]
        A3["Bot onboards you"]
        A4["AI just works™"]
    end
    
    before --> after
    
    style before fill:#ef4444,color:#fff
    style after fill:#22c55e,color:#fff
```

**Developer sees:**
- ✅ Policy Check Passed
- Context: `auth.ts`, `middleware.ts`, `jwt-utils.ts`
- Instruction Pack: `acme-standards-v2`
- Code Origin: This will be tracked as **AI-Generated**

```typescript
// Generated code uses YOUR internal libraries
export async function authMiddleware(req, res, next) {
  const token = req.headers.authorization?.split(' ')[1];
  if (!token) {
    return res.status(401).json({ error: 'Unauthorized' });
  }
  const user = await verifyJWT(token); // Uses YOUR internal lib
  req.user = user;
  next();
}
```

**Speaker Notes:**
> "Compare the experience. Before: download, configure, set up API keys, write custom prompts, hope it works. After: download, sign in, our bot connects everything, AI just works. And every line of code is tracked—leadership knows this is AI-generated and whether it conforms to standards."

---

## Slide 9: Key Differentiators

```mermaid
flowchart TB
    subgraph diff ["What Makes Us Different"]
        D1["🎯 <b>Central Hub Model</b><br/>Like Rippling—one integration,<br/>connects your entire stack"]
        D2["🚀 <b>Zero-Config Developers</b><br/>Sign in, bot onboards,<br/>no new tools to learn"]
        D3["📊 <b>Leadership Analytics</b><br/>AI vs Human code, conformance,<br/>velocity, ROI—ONLY we can track this"]
        D4["🛡️ <b>Generation-Time Security</b><br/>Prevention, not detection—<br/>before code is written"]
        D5["👔 <b>Tailored Fit</b><br/>Like a fine tuxedo—customized<br/>for YOUR stack and patterns"]
        D6["🔌 <b>Platform Agnostic</b><br/>Claude, GPT, any model—<br/>not locked to one vendor"]
    end
    
    style diff fill:#6366f1,color:#fff
    style D3 fill:#f59e0b,color:#000
```

**The Killer Insight:**

> We're the ONLY system that knows the difference between AI-generated and human-written code. That means we're the ONLY ones who can tell leadership:
> - What % of code is AI-generated
> - Whether AI code is more or less buggy
> - True velocity impact of AI adoption
> - Actual ROI on AI investment

**Speaker Notes:**
> "Six differentiators, but one killer insight: we're the only system that can track AI vs Human code. No one else can answer these questions for leadership. That's our moat."

---

## Slide 10: Technology

```mermaid
flowchart TB
    subgraph stack ["Built for Enterprise Scale"]
        GW["<b>LLM Gateway + Policy Engine</b><br/>Rust • <500ms overhead"]
        CTX["<b>Enterprise Context Service</b><br/>Weaviate + RAG • Semantic search"]
        AN["<b>Analytics Engine</b><br/>TimescaleDB • AI vs Human tracking"]
        MCP["<b>MCP Server</b><br/>44 Tools • Model Context Protocol"]
        MC["<b>MechCrate Recipes</b><br/>7 Stacks • Laravel, Nuxt, Rust, etc."]
    end
    
    subgraph deploy ["Deployment Models"]
        D1["P0: IDE Plugin<br/>(VS Code, JetBrains)"]
        D2["P1: Cloud Codex<br/>(Online Environment)"]
        D3["P2: Hub IDE<br/>(Browser-Based)"]
        D4["P3: On-Prem VM<br/>(Air-Gapped)"]
    end
    
    stack --> deploy
    
    style stack fill:#6366f1,color:#fff
    style AN fill:#f59e0b,color:#000
    style deploy fill:#374151,color:#fff
```

> "MechCrate is the foundation—it already works. unyform.ai is the enterprise hub on top. Multiple deployment paths, same governance."

**Speaker Notes:**
> "Rust for performance, Weaviate for semantic search, TimescaleDB for time-series analytics. We support multiple deployment models because every enterprise is different—IDE plugins, cloud environments, browser-based, or on-prem. Same hub, tailored delivery."

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

```mermaid
quadrantChart
    title Competitive Positioning
    x-axis Developer Focus --> Enterprise Focus
    y-axis Basic Features --> Enterprise Features
    quadrant-1 "Enterprise Leaders"
    quadrant-2 "Governance Only"
    quadrant-3 "Developer Tools"
    quadrant-4 "Full Platform"
    "GitHub Copilot": [0.2, 0.3]
    "Lando": [0.15, 0.2]
    "Nexos.ai": [0.7, 0.75]
    "Zenity": [0.8, 0.6]
    "unyform.ai": [0.65, 0.85]
```

**unyform.ai differentiators:**
| Competitor | What They Do | What We Add |
|------------|--------------|-------------|
| Copilot | Suggestions | Governance + Analytics |
| Nexos.ai | DLP focus | Central hub + Scaffolding |
| Zenity | Risk mgmt | Developer experience |
| Lando | Local dev | Enterprise scale |

**Our Unique Position:**
- ✅ Central hub model (like Rippling)
- ✅ AI vs Human code tracking (ONLY us)
- ✅ Generation-time enforcement
- ✅ Zero-config developer experience
- ✅ 30-50% lower cost than pure enterprise

**Speaker Notes:**
> "We're the only ones combining enterprise governance with great developer experience. And we're the only ones who can track AI vs Human code—that's data no competitor can provide."

---

## Slide 13: Business Model

```mermaid
flowchart TB
    subgraph tiers ["SaaS Tiers"]
        subgraph community ["COMMUNITY"]
            C1["<b>FREE</b>"]
            C2["MechCrate CLI"]
            C3["7 Recipes"]
            C4["Community Support"]
        end
        
        subgraph team ["TEAM"]
            T1["<b>$49/seat/mo</b>"]
            T2["LLM Gateway + Policies"]
            T3["GitHub + Analytics"]
            T4["VS Code Extension"]
            T5["Onboarding Bot"]
        end
        
        subgraph enterprise ["ENTERPRISE"]
            E1["<b>Custom</b>"]
            E2["Unlimited Everything"]
            E3["SSO/SAML"]
            E4["Leadership Dashboard"]
            E5["Deployment Choice"]
        end
    end
    
    community --> team --> enterprise
    
    style community fill:#374151,color:#fff
    style team fill:#6366f1,color:#fff
    style enterprise fill:#8b5cf6,color:#fff
```

**Target Unit Economics:**
| Metric | Target |
|--------|--------|
| Gross Margin | >70% |
| CAC Payback | <12 months |
| LTV/CAC | >3x |
| Net Retention | >110% |

**Speaker Notes:**
> "Land with Team tier—$49/seat gets the hub, gateway, GitHub integration, and the onboarding bot. Expand to Enterprise for leadership dashboards, deployment flexibility, and unlimited analytics. Analytics drives the upsell: once leadership sees the AI vs Human data, they want the full dashboard."

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

```mermaid
gantt
    title Product Roadmap
    dateFormat YYYY-MM
    section Phase 1: MVP
        LLM Gateway + Hub Core       :2025-01, 3M
        VS Code + Onboarding Bot     :2025-02, 3M
        Analytics Foundation         :2025-03, 3M
        Pilot Launch                 :milestone, 2025-06, 0d
    section Phase 2: Scale
        Leadership Dashboard         :2025-06, 4M
        Cloud Codex Deployment       :2025-07, 3M
        JetBrains Plugin             :2025-08, 2M
    section Phase 3: Expand
        Developer Style Profiles     :2025-10, 3M
        Hub IDE (Browser)            :2025-11, 4M
        Recipe Generator             :2025-12, 3M
    section Phase 4: Platform
        On-Prem VM Images            :2026-02, 3M
        Recipe Marketplace           :2026-03, 4M
        Partner Ecosystem            :2026-04, 6M
```

**Key Milestones:**

| Phase | Deliverable | Target |
|-------|-------------|--------|
| **Phase 1** | Hub MVP + IDE Plugin + Analytics | Q2 2025 |
| **Phase 2** | Leadership Dashboard + Cloud Codex | Q3-Q4 2025 |
| **Phase 3** | Hub IDE + Personalization | Q1 2026 |
| **Phase 4** | On-Prem + Marketplace | Q2 2026 |

**Speaker Notes:**
> "Phase 1 delivers the hub, IDE plugin with onboarding bot, and analytics foundation. Phase 2 adds the leadership dashboard they'll pay for and Cloud Codex as an alternative deployment. Phase 3 expands to browser-based IDE. Phase 4 adds on-prem for regulated industries and the marketplace ecosystem."

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

```mermaid
flowchart TB
    subgraph vision ["The Vision"]
        V1["Every engineering team will use AI"]
        V2["Winners: those who trust their AI"]
        V3["unyform.ai: the central hub"]
    end
    
    subgraph tagline ["The Tagline"]
        T1["The Rippling for AI-Assisted Development"]
        T2["Security + Velocity"]
        T3["One Hub - Zero Friction - Total Visibility"]
    end
    
    vision --> tagline
    
    style vision fill:#1e1b4b,color:#fff
    style tagline fill:#6366f1,color:#fff
```

**From every angle:**
- **Developers:** Zero new tools, just sign in and AI works better
- **Platform Teams:** One integration, connects entire stack
- **Security:** Generation-time enforcement, not after-the-fact scanning  
- **Leadership:** Finally see AI vs Human code, conformance, velocity, ROI

**Speaker Notes:**
> "Every engineering team will use AI. The winners will be those who can trust their AI. unyform.ai is the central hub that makes that possible—like Rippling made HR tools work together. Security plus velocity, tailored like a fine tuxedo for each enterprise's unique stack."

---

## Slide 19: Contact

```mermaid
flowchart TB
    subgraph contact ["Contact Us"]
        LOGO["unyform.ai"]
        TAG["The Rippling for AI-Assisted Development"]
        SUB["Security + Velocity - Tailored Like a Fine Tuxedo"]
    end
    
    LOGO --> TAG --> SUB
    
    style contact fill:#6366f1,color:#fff
```

**hello@unyform.ai**

**unyform.ai** | **github.com/unyform**

---

| Michael Price | Matt Vitebsky |
|---------------|---------------|
| CEO & Co-founder | CTO & Co-founder |
| michael@unyform.ai | matt@unyform.ai |

---

**Speaker Notes:**
> "Thank you for your time. We're building the central hub for AI-assisted development—like Rippling did for HR. Security plus velocity, tailored for each enterprise. Let's talk about how we can make AI work the way your portfolio companies build."

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
