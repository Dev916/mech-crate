---
title: "System One Decision Models: Jev, Typed Questions, and Calibrated Routing Around LLMs"
category: architecture
languages: [typescript, python]
complexity: intermediate
use_cases:
  - deciding whether a classify/route/score step in a pipeline needs an LLM at all
  - putting a fast calibrated classifier in front of, beside, behind, or instead of an LLM
  - designing atomic typed questions and composing their answers in code
  - setting confidence thresholds that scale with the risk of the action they gate
summary: "What a System One model is, what Jev actually returns (typed answers plus calibrated probabilities, no generated text), the patterns its docs prescribe, where it fits around an LLM, and what the independent evidence three days after launch does and does not support."
provenance: researched
researched: 2026-09-18
sources:
  - https://youtu.be/QbYBRjOaGOo
  - https://typesafe.ai/blog/introducing-system-one-models-and-jev
  - https://docs.typesafe.ai/concepts/system-one
  - https://docs.typesafe.ai/introduction
  - https://docs.typesafe.ai/concepts/state
  - https://docs.typesafe.ai/primitives
  - https://docs.typesafe.ai/primitives/choice
  - https://docs.typesafe.ai/primitives/score
  - https://docs.typesafe.ai/primitives/noul
  - https://docs.typesafe.ai/primitives/advanced
  - https://docs.typesafe.ai/confidence
  - https://docs.typesafe.ai/models
  - https://docs.typesafe.ai/api
  - https://docs.typesafe.ai/sdk
  - https://docs.typesafe.ai/model-jaggedness/jev-1.13
  - https://docs.typesafe.ai/introduction/machine-learning-primer
  - https://docs.typesafe.ai/concepts/how-to-build-with-system-one
  - https://docs.typesafe.ai/patterns/fan-out
  - https://docs.typesafe.ai/patterns/confidence-routing
  - https://docs.typesafe.ai/patterns/composite-scoring
  - https://docs.typesafe.ai/patterns/intent-routing
  - https://docs.typesafe.ai/cookbooks/parallel_questions
  - https://docs.typesafe.ai/cookbooks/llm_guardrails
  - https://docs.typesafe.ai/cookbooks/sde_cascade
  - https://docs.typesafe.ai/cookbooks/citation_check
  - https://docs.typesafe.ai/cookbooks/classifying_rag_passages
  - https://docs.typesafe.ai/cookbooks/hierarchical_classification
  - https://docs.typesafe.ai/cookbooks/classification_using_confidence
  - https://docs.typesafe.ai/demos/smart-home
  - https://docs.typesafe.ai/agent-skill
  - https://vercel.com/changelog/typesafe-ai-jev-now-available-on-ai-gateway
  - https://github.com/w3cj/jev-chat
  - https://www.theregister.com/ai-and-ml/2026/09/16/typesafe-ai-debuts-model-for-machines-that-plays-doom/5296711
  - https://github.com/anisselbd/jev-phishing-bench
  - https://github.com/pydantic/pydantic-ai/pull/8486
  - https://www.seangoedecke.com/two-techniques-for-working-with-system-one-models/
  - https://backnotprop.com/blog/jev-poker/
  - https://lindfors.no/blog/a-first-look-at-typesafes-jev/
  - https://nearhere.events/blog/typesafe-jev-mistral-gemini-event-validation
  - https://news.ycombinator.com/item?id=49717558
  - https://en.wikipedia.org/wiki/Thinking,_Fast_and_Slow
  - https://github.com/getsentry/junior
---

# System One Decision Models: Jev, Typed Questions, and Calibrated Routing Around LLMs

State of practice as of 2026-09. The seed source is Syntax's 2026-09-17 video "wtf is jev?" by CJ [1], watched end to end with vidwatch; every claim it makes about the product was checked against TypeSafe's announcement of 2026-09-15 [2] and the vendor documentation [3] to [30]. Inline `[n]` cites key to `sources`; `[1 @mm:ss]` is a timestamp in the video. This is a three-day-old product in early access behind a waitlist [2][1 @17:25-17:31], so treat every performance number below as vendor-reported unless it is explicitly attributed to an independent measurement, and expect the API and the limits to move. Code is illustrative of the documented request and response shapes; no call was made against the live API while writing this.

## 1. What a System One model is

**The class, then the product.** A System One model is "a class of AI models built to make fast, structured decisions that software can use directly", evaluating a `state` and returning typed answers and probabilities [3]. Jev is the first one and TypeSafe's flagship [3][4]. The current model is `jev-1.13.0` [12]. The distinction matters because the vendor is claiming a category, not just a model [2].

**The naming is borrowed, and the vendor says so.** The docs note that the name "comes from the concept Daniel Kahneman popularized in his book *Thinking, Fast and Slow*", where "System 1 thinking is fast and intuitive. System 2 is slower and more deliberate" [3]. Kahneman's own split is "System 1: Fast, automatic, frequent, emotional, stereotypic, unconscious" against "System 2: Slow, effortful, infrequent, logical, calculating, conscious" [41]. The blog adds a caveat the video does not: "System 1 thinking has also implied error-prone", and the vendor's position is that System One models "can be made more reliable than its alternatives" for reasons it has not yet published [2]. "Jev" is named after William Stanley Jevons, on the bet that cheaper intelligence increases rather than reduces demand [2].

**What it does not do.** It "does not write replies, produce code, or generate explanations of their reasoning" [3]. Jev "currently accepts text input only. It evaluates strings, JSON objects, and arrays of text. Images, audio, and video are not supported (yet)" [3][12]. English is "the primary training language and where accuracy is currently best" [12]. The jaggedness page is blunter still: `jev-1.13` "is not trained to generate text" and forcing it by chaining choices "will not work well and will be very slow" [15].

**How it is trained.** RLCD, "Reinforcement learning for calibrated decisions", is presented as a third post-training path alongside RLHF and RLVR, applied to pretrained language models [16][2]. The output contract it optimizes for: "The model does not generate text. It returns decisions and probabilities. Higher probability should correspond to a greater chance that the answer is correct" [16]. RLHF was "co-invented by Diogo Almeida", TypeSafe's founder and the author of the announcement [16][2].

**The calibration claim, stated precisely.** Outcomes assigned 0.2 "should occur about 20% of the time", 0.8 about 80% [16]. The critical qualifier appears on both the primer and the System One page: "Calibration is measured across groups of predictions; it does not guarantee that an individual answer is correct" [3][16].

## 2. How it differs from an LLM

The rows below are the announcement's own comparison table, quoted [2]. Every number is vendor-reported, with the vendor's own caveat attached.

| Axis | Existing LLMs [2] | System One and Jev [2] | Caveat the vendor attaches |
|---|---|---|---|
| Optimized with | RLHF / RLVR | RLCD | No paper or technical report on RLCD has been published [2][16] |
| Optimizes for | "Human preference"; "Verifiable rewards" | "Calibrated decisions: answers with epistemically honest probabilities on System One tasks" | Calibration is a group property, not a per-answer guarantee [3][16] |
| Inputs | Unstructured data "with an emphasis on sequential messages" | Unstructured data "with an emphasis on structured program state" | Text only; no image, audio or video [3][12] |
| Outputs | "Strings / generated text"; need parsing and validation | "Type-safe structured values... The model never makes type errors" | The blog calls a counter-example to the no-type-error claim "mathematically impossible"; the 0% figure in its hallucination plot is separately marked "not empirical" [2] |
| Sampling | "Sequential. Generates one token at a time" | "Parallel. Generates all outputs in a single query" | The side-by-side demo's state is "short, dense" and "paints our model in an advantageous light" [2] |
| Cost | "Input tokens: from $0.20 to $10 / MTok. Output tokens: ~5x more expensive than input tokens." | "Input tokens: $0.042 / MTok ($42 per billion tokens). Output tokens: FREE (too cheap to meter)." | "We can't prove it isn't subsidized" [2] |
| Speed | "3 to 329 seconds for frontier models" | "70ms-500ms for TypeSafe. This can range from 40x-200x faster" | Evals "are generally run from our laptops on the West Coast" [2] |
| Confidence | "models tend to be overconfident and inconsistent" | "Always communicates confidence and uncertainty with every output" | Noul answers carry no confidence at all [6][11] |
| Use cases | Human-in-the-loop, verifiable problems, demos | "AI-Powered Workflows / smart if-statements", map-reduce over big data, real-time, "Verify everything" | Early access, days old [2] |

**The 193.6x and 444.6x figures belong to the workflow evals, not to the table.** The announcement says "This is where the claims of 193.6x faster, 444.6x cheaper on our home page comes from, and we expect that these are on the higher end of real world gains" [2]. Vercel repeats them with attribution intact: "TypeSafe reports Jev was up to 193.6x faster and 444.6x cheaper than LLMs on its workflow evaluations" [31]. The eval method is agreement-based, not correctness-based: "we assume there is a correct compute graph... and use the predictions of the largest, smartest, and most expensive external models as reference probabilities", using "the average of GPT-6 Astra and Fable 5.1 as the reference answer, which biases answers towards OpenAI and Anthropic's models" [2]. The workflows "were made by individuals on our model capabilities team, so some bias could exist" [2].

**An independent reading of the hallucination claim.** The Register's Thomas Claburn wrote on 2026-09-16 that "TypeSafe claims that Jev is hallucination-free, which really isn't a fair comparison as its output is not natural language", and that "Jev instead returns structured responses with probabilities, and that does not preclude the possibility of being incorrect" [33]. That is the same distinction the announcement itself draws when it grounds the claim in schema matching rather than correctness [2].

## 3. The API surface

**One endpoint, one state, many questions.** `POST https://api.typesafe.ai/v1/systemone` with a bearer token; the body is `state`, `model`, and a `questions` map whose keys you choose and whose answers return under the same keys [13]. The question ID "is not sent to the underlying model and is not used in inference" [13]. State is "a string, JSON object, or array of text values" [5]; questions can name a part of it by path, including the backticks, as in ``Does `ticket.messages[0].text` request a refund?`` [6].

**Three primitives.**

| Type | Required fields [13] | Returns [6][13] | Notes |
|---|---|---|---|
| `choice` | `instructions`, `criteria` as option-to-description map | `choice`, `probabilities` (sum to 1), `confidence` | "A Choice question accepts up to 255 options" [7]; add an `other` option when the list may not cover every input [6] |
| `score` | `instructions`, `criteria` as ordered level array | `score`, `legend`, `probabilities`, `confidence` | "Needs at least two levels and takes up to 10" [8]; `score` "can land between two levels" [13] |
| `noul` | `instructions`; optional `criteria` with `true`/`false` | `noul` only | A Noul "asks the model to evaluate a yes/no question and return the probability that the answer is yes" [9]; "The yes/no answer on a scale from 0 (no) to 1 (yes)" [13]; "Noul has no separate `confidence`" [6] |

**Probabilities versus confidence.** `confidence` "is a statistic computed from the probability distribution the answer already gives you"; "a flatter distribution means lower confidence" [11]. The docs are explicit that you are not locked into their definition, which is why the full `probabilities` come back [11]. Use `probabilities` when you want your own uncertainty measure, `confidence` when you want a threshold [11].

**Model ids, limits, and price.** `jev-1.13.0` at "$42 / $0.042" per Btok and per Mtok, charged on input tokens only [12]. Rate limits: "250,000 tokens per second / 1,200 requests per minute", with a standing warning that "Rate limits are adjusting dynamically... the limits above can change without notice" [12]. Context: "64k tokens per request; 32k tokens for `state` plus the longest question" [12]. Aliases `jev-latest` (the SDK default) and `jev-preview`, both currently pointing at `jev-1.13.0`; pin the versioned id if you have tuned thresholds, because "an alias moves when a new release ships" [12]. There is no fine-tuning: "the same weights serve every account", and you adapt via `state`, `instructions` and `criteria` instead [12].

**Clients.** First-party Python and JavaScript SDKs with automatic retry and backoff [14][12], plus an agent skill for Claude Code and other agents [30]. Third-party access is through Vercel AI Gateway: install "AI SDK 7.0.105 onwards", call `experimental_evaluate` with model id `typesafe-ai/jev`, and the third primitive is renamed there, so the Vercel example uses `type: 'boolean'` where the native API uses `noul` [31][13]. Gateway calls support "Zero Data Retention and No Training", set per request, and confidence arrives under `result.providerMetadata.typesafe.confidence` [31]. Vercel's own advice is the right one: "Calibrate probabilities and confidence against labeled examples from your workflow" [31].

Confidence thresholds scaled by the stakes of the action, following the docs' worked example [11][19]:

```python
from typesafe_sdk import Choice, TypeSafeClient


def show_balance(account_id: str) -> None: ...
def confirm_then_execute(account_id: str) -> None: ...
def ask_user_to_confirm(account_id: str) -> None: ...
def route_to_human(message: str) -> None: ...


# The SDK reads TYPESAFE_API_KEY from the environment.
client = TypeSafeClient()


def handle(account_id: str, user_message: str) -> None:
    response = client.system_one(
        state=user_message,
        questions={
            "action": Choice(
                instructions="What is the user trying to do?",
                criteria={
                    "check_balance": "View account balance",
                    "approve_transfer": "Approve the pending withdrawal request",
                    "support": "Get help with an issue",
                },
            ),
        },
    )
    action = response.answers["action"]

    # Below the floor the model is telling you it does not know. Do not guess.
    if action.confidence < 0.5:
        route_to_human(user_message)
    elif action.choice == "check_balance":
        # Low stakes: showing the wrong screen is recoverable.
        show_balance(account_id)
    elif action.choice == "approve_transfer":
        # High stakes: the bar for acting unattended is higher.
        if action.confidence > 0.9:
            confirm_then_execute(account_id)
        else:
            ask_user_to_confirm(account_id)
```

The answer shapes, typed, so code can branch on them without parsing prose [13][6]:

```typescript
type ChoiceAnswer<K extends string> = {
  type: "choice";
  choice: K;
  probabilities: Record<K, number>;
  confidence: number;
};

type ScoreAnswer = {
  type: "score";
  score: number;
  legend: Record<string, string>;
  probabilities: Record<string, number>;
  confidence: number;
};

// A Noul carries no confidence: the probability itself is the signal.
type NoulAnswer = { type: "noul"; noul: number };

type Intent = "order_status" | "product_question" | "complaint";

type Triage = {
  intent: ChoiceAnswer<Intent>;
  complexity: ScoreAnswer;
  refundRequested: NoulAnswer;
};

type Route = "code" | "llm" | "human";

export function route(a: Triage): Route {
  if (a.intent.confidence < 0.5) return "human";
  if (a.intent.choice === "order_status") return "code";
  if (a.intent.choice === "complaint") {
    const unsure = a.complexity.confidence < 0.5;
    return a.complexity.score > 1 || unsure ? "human" : "llm";
  }
  return "llm";
}
```

## 4. The patterns the docs prescribe

The governing rule is stated three times in three places: keep the workflow in code and give the model narrow, constrained decisions [17][4][6]. The docs contrast three architectures: traditional code as "a complex decision tree made from simple software primitives"; LLM agents where "every loop introduces another opportunity to go off the rails"; and AI-powered software where "code handles deterministic work and owns the control flow" and the model "appears only where the system needs programmable common sense" [17].

| Pattern | The rule | Why it works | Source |
|---|---|---|---|
| Atomic questions composed in code | Ask "a judgment a knowledgeable person makes in a second"; decompose anything needing multiple factors and weight them yourself | "When priorities shift, change a coefficient in your code rather than rewriting a prompt" | [6][4] |
| Speculative fan-out | Ask every question you might need in one call, including irrelevant ones, and let code discard the rest | "There is no speed cost for additional questions"; the smart-home demo names the sequential alternative "the wrong way" | [18][29] |
| Confidence-gated routing | One floor below which nothing is automated, then per-action thresholds above it | "The answer tells you what; confidence tells you whether to act" | [19][11] |
| Composite scoring | Score each dimension separately, normalize, apply weights in code | "It gives you visibility into how exactly the final score is being calculated" | [20] |
| Intent routing | Classify first, then send each class to code, a specialist LLM, or a human | "The expensive resources only get invoked for the requests that actually need them" | [21] |
| Hierarchical classification | One Choice per level, walking the tree in code, with beam search when probabilities are close | Lets the model "see what lives under a branch before committing to it" | [10][27] |
| Self-consistency | Repeat the same evaluation and compare the answers before trusting a tuned threshold | The docs state the model "is designed to return stable answers across repeated evaluations"; an independent audit found a borderline answer moving between runs | [17][35] |

**The fan-out number, vendor-measured.** A 13-question regulatory briefing over the GDPR Wikipedia article: "batching every question into one TypeSafe call is 12.2x cheaper and 10.0x faster with no change in answers" [22]. That figure is about batching questions against one state, not about Jev versus an LLM, and it is the honest version of the "adding questions barely changes the response time" claim [4][22].

**When two requests are justified.** "Two requests are the exception, not the rule" [6]. The dependency is real only when code cannot build the second request until it has the first answer: it needs the answer to fetch more data, to decide what the state is made of, or to pick the next question's options [6].

**Guardrail and verification recipes.** One request screens messages into and out of an LLM app by "describing possible hazards ('is this a jailbreak attempt?') and scoring severity" [23]. A Choice decides "whether the quote's context supports the claim, and its confidence can flag the citation for human review" [25]. Retrieved passages are scored so code can "keep and flag ones that contradict the question, and drop ones carrying a hidden instruction or prompt injection" [26]. The SDE cascade runs "mini to verify to reasoning" so that a cheap extraction is checked before an expensive model is called [24]. Confidence can also pick the granularity of the answer: classify into 75 industry groups, then "read the answer's own confidence to decide whether to report that group or the broader division above it" [28].

## 5. Where it sits in an LLM stack

**In front of the LLM.** Classify the request, then spend the expensive resource only where it is needed [21][1 @04:48-05:29]. The video's worked case is a model router: Sentry's Junior Slack bot has "this entire turn router here with a prompt that describes what types of requests should go to what models", which "can basically be turned into two questions for Jev: first, how much reasoning is required? And then which profile does it fit?" [1 @11:26-12:11]. Note the tense: that is a proposal in the video, not a shipped integration [1 @11:56-11:58].

**Beside the LLM.** Real-time paths where an LLM round trip is too slow: classifying text as it is typed [1 @07:21-08:02], a browser extension filtering a feed in place [1 @08:22-08:34], browser use driven from "snapshots of the browser" [1 @05:36-06:11]. The docs' own framing is "Most queries complete in about 100 ms. System One is fast enough for real-time request paths and user interfaces" [17].

**Behind the LLM.** Verification after generation: run the LLM's extracted claims back through Jev "and make sure that it's truthful and didn't hallucinate anything" [1 @11:03-11:26]; check citations against the source [25]; score a diff or a file for risk in code review [1 @09:45-10:08].

**Instead of the LLM.** The strongest architectural demonstration is CJ's jev-chat, "a chat-shaped command bar that calls real tools, without an LLM writing anything" [32][1 @12:10-12:17]. Every turn, "a classifier picks: what was asked, which tool to call, which value goes in each argument, whether to confirm first, and what kind of reply to give"; because "no model ever writes the text, the assistant cannot invent a fact: every value on screen was either typed by the user or returned by a tool" [32]. The mechanism that makes this safe is a pool: "Jev can only pick from these pools, so they are the only values that can end up in a text argument" [32]. Its inspector, shown on screen at [1 @16:40], reports one turn as 280 ms in Jev plus 82 ms in the tool for 363 ms total, with 5 of 41 questions used [1 @16:40]. That is speculative fan-out at work: the README describes the same mechanism as asking "every tool's questions in one round trip, before it knows which tool will be used" [32][18].

The README is franker about the limits than the video is [32]:

- "It can still pick the wrong tool, the wrong span or the wrong line, and a tool can return wrong data."
- "Routing is a model's judgment, not a rule", so anything that changes state shows a confirm card first.
- "Jev makes each judgment in one pass. Anything that needs working out goes to code or a tool."
- Compound requests such as "turn the light green, then red after 5 seconds" are "not yet" supported.
- "Tool results are untrusted input... Jev can't be talked into writing a tool call, but keep the policy checks in code."

The smart-home demo shows the hybrid that most systems will actually want: Jev decides, and an LLM is called only to split a compound request or to answer a general-knowledge question, where "the initial TypeSafe response is so fast compared to the LLM response that it adds negligible latency to the overall system" [29].

## 6. Limits and open questions

**The vendor's own jaggedness page is the most useful document in the set** [15]. Its nine failure modes, with the fix it prescribes: literal reading (write the exact condition); math and numbers (keep arithmetic in code; "`jev-1.13` does not count reliably"); date and time comparison ("reads dates as text, not as ordered quantities", so extract components and compare in code); indirection; large state full of irrelevant detail ("Jev suffers from context rot, so unrelated material in the `state` costs you accuracy"); adversarial content ("State is data, and `jev-1.13` does not treat it as hostile by default"); contradictory instructions and criteria; common-sense structural invariants; and generation [15].

**The structural-invariant warning deserves its own line.** The same question asked as a Noul and as a yes/no Choice on one ticket returned `noul` 0.22 against Choice `yes` 0.01 at confidence 0.97; a question and its negation as two Nouls summed to 1.19 [15]. The guidance follows: "Don't carry a threshold tuned on a Noul over to a Choice, and don't hold the model to arithmetic identities between separate questions" [15].

**Independent evidence exists, and it is more mixed than the launch material.** Three days in, this is what non-vendor testing shows.

| Source | Setup | Result |
|---|---|---|
| Phishing decision benchmark, 2,000 emails, seeded and reproducible [34] | `jev-1.13.0` versus Claude Haiku 4.5 on the PhishNChips v5.2 set | "Jev's own verdict loses clearly on accuracy (McNemar p < 0.0001) and wins on speed and cost": 62.6% versus 81.3% accuracy, ECE 0.154 versus 0.097, p50 239 ms versus 687 ms, $0.038 versus $0.462 per 1,000 emails. With the decision decomposed into five signal Nouls the gap closes, at "about 27 times cheaper and 5 times faster than Haiku for signals of comparable quality, on a dataset that a regex already separates at 91.8%" |
| Norwegian government hearing responses, 24 documents [38] | Stance, classification and scoring against "DeepSeek V4.1 Flash through OpenRouter" | "Median latency 0.32 s" against 2.7 s (reasoning off) or 26 s (on); "Cost per 1,000 documents $0.22" against $1.31 or $3.08; judgments in the 0.7 to 0.9 bin agreed with the reference labels 97% of the time, where those labels are a frontier model's (Claude Fable 5.1 labelling all 24 documents twice) rather than ground truth; verbose instructions degraded performance |
| Event validation, 50-case test set [39] | Jev versus Mistral Small 4 and Gemini 3.5 Flash-Lite | 96% versus 84% and 86% accuracy; 0.59 s versus 2.90 s and 3.40 s; $0.043 versus $0.370 and $2.496 per 1,000 decisions. On a separate 21-listing sample run after prompt selection, Jev scored 19 of 21 against Gemini's 20 of 21, and the authors caution that "The additional sample was too small and too narrow to establish a general accuracy advantage" |
| Poker trap spot, 16 runs [37] | A check-or-shove decision where "the answer does not depend on sizing" | "Jev shoved in sixteen runs out of sixteen", the opposite of correct play; the author warns about "naive deployments" before published evals exist |
| Pydantic AI documentation audit, 2026-09-18 [35] | Its own TypeSafe integration docs checked against live `jev-1.13.0` | "Ten claims were wrong or overstated." Among them: "Jev does not revise an answer" is false (on four tickets with two rejections each, "one moved from `billing` to `bug` and back"); a documented confidence value of 0.95 measured "0.80 to 0.86 across 5 runs"; and a `float` field "has no `confidence` entry by design", so a low-confidence fallback built on one silently never fires |
| Practitioner writeup on working with System One models [36] | Selecting among many options with Choice | Tournament sampling: "fed a hundred links at a time into each choice, then did a second pass with the chosen links", because of his own layer's ceiling, "While it technically would scale out to more choices, it stopped working well after a hundred or so." Of the vendor's two-stage score-then-choose approach for high-cardinality options he says only "This did not work very well for me at all." The author's placement: "a meaningful alternative to tool calls for realtime scenarios or use-cases where you need predictable inference timing" |

**What that adds up to.** Independent measurements reproduce the direction of the speed and cost claims and land far below the headline multipliers [34][38][39]. Across the three benchmarks above the latency ratio against non-reasoning baselines runs from about 2.9x to about 8x, rising to about 80x only against a model with reasoning switched on, and the cost ratio runs from about 6x to about 58x [34][38][39]. Accuracy is task-dependent and can be worse than a cheap LLM on the same task [34]. Calibration held up in one study [38] and did not in another [34]. Nothing in the independent record reproduces the 40x-200x or the 193.6x and 444.6x figures [2][31], and neither the announcement nor the AI primer publishes a calibration curve, an expected-calibration-error figure, or a technical report on RLCD [2][16].

**Open questions, and what would change the assessment.** A published RLCD method with a calibration curve would move the reliability claim from assertion to evidence [2][16]. A stable rate-limit and price schedule would answer the sustainability caveat the vendor itself raises [2][12]. Vendor evals run against reference probabilities from other models measure agreement, not correctness, so a correctness-labelled benchmark from the vendor would matter [2]. The launch thread on Hacker News reached roughly 1,890 points and 495 comments on 2026-09-15 [40], so attention is not the constraint; independent, adversarial evaluation is.

## Overstated or unverified

- **The video's headline multipliers do not match the announcement.** CJ says Jev "is 20 to 200 times faster than traditional LLMs, and it's 40 to 400 times cheaper" [1 @00:33-00:40]. The announcement's table says "40x-200x faster" and gives per-token prices rather than a cost multiplier [2]. The video's figures match the founder's launch post as shown on screen [1 @00:15], not the blog table. Use the blog's numbers.
- **"It can't hallucinate" is a claim about types, not about correctness.** The announcement's own evidence section grounds it in schema matching being "mathematically impossible" to violate, and marks the figure as "not empirical" [2]; The Register calls the comparison unfair because "its output is not natural language" [33]; jev-chat's README says plainly that it "can still pick the wrong tool, the wrong span or the wrong line" [32].
- **The Sentry model router is a proposal in the video, not a shipped Jev integration.** CJ describes what the existing prompt-based turn router "can basically be turned into" [1 @11:56-11:58], and Junior's own README lists its runtime and eighteen plugin packages without naming TypeSafe or Jev [42].
- **"Jev does not revise an answer" is contradicted by live measurement.** The Pydantic AI audit found a borderline answer moving between categories across retries [35].
- **The token budget is documented two ways.** The Models page says "64k tokens per request; 32k tokens for `state` plus the longest question" [12]; the Primitives page says the budget "is around 32,000 tokens, roughly 150,000 characters of English text" shared by state and questions [6]. The rule this doc adopts, inferred rather than documented, is to assume the tighter figure until the two pages agree.
- **"Adding more questions does not create context-rot" [4] and "Jev suffers from context rot" [15] are both in the docs.** They are reconcilable, since the first is about questions and the second about state, but the second is the one that will bite you.
- **The video misstates Kahneman.** It describes System 2 as "slow, effortful, and frequent" [1 @01:08-01:14]; the source says infrequent [41].
- **Third-party demo numbers in the video are uncorroborated.** A thousand research papers classified for "eight cents" at "256 milliseconds per classification" [1 @06:20-06:34], a seven-second flight booking [1 @05:45-05:50], and "10 times cheaper" resume screening [1 @06:58-07:02] are all demos shown on screen with no published method [1 @05:36-07:02]. This doc treats them as existence proofs of the shape of the workload rather than as measurements, which is its own judgement, not the video's.

## Agreed vs folklore (compressed)

- **Agreed:** the model returns typed values constrained to the options you supply, so code never parses prose [6][13]; probabilities and confidence come with every Choice and Score, and Noul carries no confidence [6][11]; questions in one request are evaluated independently and batching them is much cheaper than one call each [4][22]; the documented failure modes are real and the vendor publishes them [15]; latency and cost are genuinely far below an LLM on the same decision, as measured independently [34][38][39].
- **"No text generation, so it cannot hallucinate."** Folklore as usually repeated. What is guaranteed is the absence of type errors and values outside your schema [2][13]. Wrong answers, wrong tool picks and wrong spans all remain possible [32][33][34].
- **"It replaces the LLM."** No. The vendor's own patterns route to "deterministic logic, a specialist LLM, or a human" [21], and its own demo calls an LLM to split compound requests and to answer open questions [29]. The one place replacement is real is a narrow tool-dispatch UI where every reply can be assembled from tool data [32].
- **"Confidence means accuracy."** Calibration is a property of groups of predictions and "does not guarantee that an individual answer is correct" [3][16]. One independent study found confidence tracking accuracy well [38]; another measured worse expected calibration error than the LLM baseline on its task [34].
- **"A small local classifier does the same thing."** Partly fair, and the gap is narrower than the launch material implies: in one benchmark a two-line regex rule beat Jev's best single signal on the same data [34]. What is actually new is the combination: calibrated probabilities as the product contract, many independent typed questions evaluated in parallel against one shared state in a single round trip, and an answer space you define per request rather than per trained model [2][4][12]. A fine-tuned classifier gives you one label space per model; this gives you an arbitrary one per call, at a latency the vendor docs put at "about 100 ms" [17].

## Synthesis (inferred)

**A decision rule for where to put a decision model.** Work through these in order and stop at the first match.

1. **Can code decide it?** A date comparison, a count, a threshold on a number you already have. Keep it in code. The vendor says this first too, and its jaggedness page exists largely because people ask the model these things anyway.
2. **Is the answer space closed and known at request time, and is one snap judgment enough?** Put a decision model there *instead of* an LLM. Tool dispatch, intent classification, severity scoring, relevance filtering.
3. **Is it a gate in front of an expensive or dangerous step?** Put it *in front*: classify, then spend. The economics only work if the classification is cheap enough that you can run it on every request, which is exactly the regime a sub-cent, sub-second call creates.
4. **Is it a check on something already generated?** Put it *behind*: citation checks, claim verification, guardrails on output. This is the highest-value placement in my reading, because it is the one place where the model being unable to write anything is a feature rather than a constraint.
5. **Does the path need generated text, open-ended reasoning, or multi-hop inference?** Keep the LLM and put the decision model *beside* it for the parts that are judgments rather than prose.
6. **Everything else stays with the LLM.** A decision model is not a reasoning model, and the vendor agrees.

**Run your own calibration check before trusting any threshold.** The vendor's calibration claim is about groups, and the independent record is split, so treat thresholds as a local measurement, not a constant. The cheap version: collect 200 to 500 labelled examples from your own traffic, run them through the exact questions you will ship, bucket the answers into ten confidence bins, and plot observed accuracy per bin against the bin centre. You want three numbers out of it: the expected calibration error, the confidence floor below which accuracy falls off a cliff, and the automation rate at your target accuracy, which is the fraction of decisions you can take unattended if you route everything under that floor to a human. Re-run it whenever you move off a pinned model id, because an alias moving silently invalidates every threshold you tuned. Use the full probability distribution rather than the scalar confidence if your options are not mutually exclusive, since a flat distribution over four plausible options and a flat distribution over one plausible and three nonsense options mean different things.

**One application note for this corpus's own tooling.** Two places in our stack have the shape this is for. First, tool selection: the mx MCP server publishes 47 tools in one flat list, which sits inside the band where model tool-selection accuracy is known to degrade, and the jev-chat design is a working demonstration of the alternative, where a Choice over every tool plus per-tool argument questions is answered in one speculative request and code owns the dispatch. That would be a rebuild rather than a tweak, but it is the clearest local instance of "classify, then act". Second, and cheaper to try: technique-research claim verification. Every doc this skill ships asserts that a quoted figure appears verbatim on a cited page, and that check is currently done by an expensive model reading the page. It is really a per-claim Noul against a fetched state, which is exactly a guardrail-behind-the-LLM placement, with confidence gating which claims a human needs to look at. Both are worth a spike, not a commitment, until the calibration evidence settles.
