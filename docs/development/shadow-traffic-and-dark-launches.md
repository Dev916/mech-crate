---
title: "Shadow Traffic and Dark Launches: Mirroring Production Requests Safely, Diffing Responses, and Knowing When Not To"
category: infra
languages: [yaml, python]
complexity: intermediate
use_cases:
  - proving a rewritten service or data-store read path returns the same answers as the old one under real production inputs before any user sees it
  - load-testing a new backend with real traffic shape before a launch, without exposing its responses
  - choosing between gateway mirroring, in-process experiments, recorded replay, canary and feature flags for a risky change
  - setting up a mirror on Envoy, Istio, Gateway API, NGINX, HAProxy or Traefik without the mirror slowing, breaking or double-charging production
summary: "Shadow traffic in 2026: what each mirror (Envoy, Istio, Gateway API, NGINX, HAProxy, Traefik, Scientist, GoReplay, SageMaker) does with responses, bodies and sampling; Diffy-style diffing with noise cancellation; side-effect, cost, privacy and observability rules; when to use canary instead."
provenance: researched
researched: 2026-10-10
sources:
  - https://cloud.google.com/blog/products/gcp/cre-life-lessons-what-is-a-dark-launch-and-what-does-it-do-for-me
  - https://cloud.google.com/blog/products/gcp/cre-life-lessons-practicalities-of-dark-launches
  - https://martinfowler.com/bliki/DarkLaunching.html
  - https://martinfowler.com/bliki/CanaryRelease.html
  - https://martinfowler.com/bliki/BlueGreenDeployment.html
  - https://launchdarkly.com/blog/guide-to-dark-launching/
  - https://www.envoyproxy.io/docs/envoy/latest/api-v3/config/route/v3/route_components.proto
  - https://www.envoyproxy.io/docs/envoy/latest/api-v3/config/listener/v3/listener.proto
  - https://istio.io/latest/docs/tasks/traffic-management/mirroring/
  - https://istio.io/latest/docs/reference/config/networking/virtual-service/
  - https://github.com/istio/api/blob/master/networking/v1alpha3/virtual_service.proto
  - https://gateway-api.sigs.k8s.io/guides/http-request-mirroring/
  - https://gateway-api.sigs.k8s.io/geps/gep-3171/
  - https://kubernetes.io/blog/2025/06/02/gateway-api-v1-3/
  - https://github.com/kubernetes-sigs/gateway-api/blob/v1.6.3/apis/v1/httproute_types.go
  - https://nginx.org/en/docs/http/ngx_http_mirror_module.html
  - https://mailman.nginx.org/pipermail/nginx/2017-October/055043.html
  - https://mailman.nginx.org/pipermail/nginx/2018-September/056765.html
  - https://dev.to/dzeban/nginx-mirroring-tips-and-tricks-17c2
  - https://www.haproxy.com/blog/haproxy-traffic-mirroring-for-real-world-testing
  - https://docs.aws.amazon.com/vpc/latest/mirroring/what-is-traffic-mirroring.html
  - https://docs.aws.amazon.com/vpc/latest/mirroring/traffic-mirroring-considerations.html
  - https://cloud.google.com/load-balancing/docs/https/traffic-management-global
  - https://developers.cloudflare.com/workers/runtime-apis/context/
  - https://github.com/github/scientist
  - https://github.blog/developer-skills/application-development/scientist/
  - https://github.blog/engineering/engineering-principles/move-fast/
  - https://github.com/probelabs/goreplay
  - https://github.com/probelabs/goreplay/wiki/Saving-and-Replaying-from-file
  - https://github.com/probelabs/goreplay/wiki/Middleware
  - https://github.com/probelabs/goreplay/wiki/Rate-limiting
  - https://github.com/probelabs/goreplay/blob/master/settings.go
  - https://docs.aws.amazon.com/sagemaker/latest/dg/shadow-tests.html
  - https://github.com/twitter/diffy
  - https://web.archive.org/web/2020/https://blog.twitter.com/engineering/en_us/a/2015/diffy-testing-services-without-writing-tests
  - https://web.archive.org/web/2024/https://netflixtechblog.com/migrating-critical-traffic-at-scale-with-no-downtime-part-1-ba1c7a1c7835
  - https://stripe.com/blog/online-migrations
  - https://web.archive.org/web/2021/https://tech.trivago.com/2020/06/10/cross-cluster-traffic-mirroring-with-istio/
  - https://doc.traefik.io/traefik/reference/routing-configuration/http/load-balancing/service/
  - https://github.com/traefik/traefik/blob/v3.7.14/pkg/server/service/loadbalancer/mirror/mirror.go
  - https://github.com/traefik/traefik/blob/v3.6.1/pkg/config/dynamic/http_config.go
  - https://docs.stripe.com/api/idempotent_requests
  - https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:32016R0679
  - https://shadowtraffic.io/
---

# Shadow Traffic and Dark Launches: Mirroring Production Requests Safely, Diffing Responses, and Knowing When Not To

State of practice as of 2026-10. The seed sources are Google's two-part CRE series on dark launches [1][2], the mirror semantics in the Envoy, Istio, Gateway API, NGINX and Traefik references [7][9][12][16][39], GitHub's Scientist library and its two launch posts [25][26][27], Twitter's Diffy [34][35] and Netflix's replay-testing write-up [36]. Every field name, default, version and quoted phrase below was read off the cited page or source file on 2026-10-10, not recalled. Inline `[n]` keys to `sources`. The Envoy config passed `envoy --mode validate` on Envoy 1.39.3, the NGINX config passed `nginx -t` on nginx 1.30.5, the Traefik file was loaded by a Traefik v3.6.1 container and reported `enabled` by its API, and the Python experiment was compiled, linted and run; the Gateway API manifest is illustrative (parsed as YAML only).

Scope note: this doc is about copying real requests to a path whose answers users never see. Exposure-controlled rollout with feature flags is a separate topic; the pattern-level summary of both lives in [appendix-pattern-playbook.md](appendix-pattern-playbook.md). For the dashboards and SLO alerts that a shadow experiment must not pollute, see [grafana-observability-practices.md](grafana-observability-practices.md); for building the candidate image, [docker-assembly-guide.md](docker-assembly-guide.md); for the Django side of a candidate service, [django-production-hardening.md](django-production-hardening.md); for event-stream replay, [appendix-streams.md](appendix-streams.md).

## 1. What shadow traffic is, and what it is not

**Definition.** Google's CRE team defines a dark launch as sending "a copy of real user-generated traffic to your new service" and discarding the new service's result before the user sees it [1]. Istio calls the same mechanism "Traffic mirroring, also called shadowing" and says the copy "happens out of band of the critical request path for the primary service" [9]. Netflix calls it replay traffic: production traffic "cloned and forked over to a different path in the service call graph" [36]. Google lists exactly two goals: verify the new service answers realistic queries the same way as the old one, and measure how it performs under realistic load [1].

**Vocabulary collision.** Feature-flag vendors use "dark launch" for something else: LaunchDarkly defines it as releasing features "to a small group of users while hiding them from the rest of the user base" [6]. Google notes that dark launches get called feature toggles but that this does not capture the hidden-traffic aspect [1]. Fowler's definition sits between the two: call the new back end from existing users "without the users being able to tell it's being called", switched by a feature flag, and optionally run old and new code in parallel with only one answer returned [3]. A product named ShadowTraffic is a generator that will "simulate production traffic to your backend", which is synthetic load, not mirroring [44].

| Term | Who uses it | What reaches the user | Question it answers |
|---|---|---|---|
| Shadowing, mirroring | Envoy [7], Istio [9], Gateway API [12], NGINX [16], HAProxy [20], Traefik [39] | only the primary's response | does the new path survive and agree under real requests? |
| Dark launch (Google sense) | Google CRE [1][2], Fowler [3] | only the old response | same, plus capacity at 100% of real load [1] |
| Dark launch (flag sense) | LaunchDarkly [6] | the new feature, for a subset | do some real users accept it? |
| Experiment, control and candidate | Scientist [25] | the control's value | does the new function return the same value in-process? |
| Replay | Netflix [36], GoReplay [28] | nothing (offline or async) | correctness and load from recorded or forked traffic |
| Shadow test, shadow variant | SageMaker [33] | the production variant's response | does a new model, container or instance perform? |
| Canary release | Fowler [4] | the new version, for a small subset of users | is the new version safe for real users? |
| Blue-green | Fowler [5] | all traffic switches environment at once | can we cut over and roll back fast? |

**The distinction that matters.** Canary and blue-green expose users to the new version and measure their outcomes [4][5]; a shadow never exposes users, so it can test correctness and load but never user-facing impact [1][33]. Diffing works best on reads; with mutations Google warns that "you can't sensibly apply the same mutation twice in parallel" [1].

## 2. Mechanisms and their exact semantics

**Envoy.** `request_mirror_policies` on a route or virtual host; Envoy describes the implementation as "fire and forget," and "will not wait for the shadow cluster to respond before returning the response from the primary cluster" [7]. The mirrored request gets `-shadow` appended to its host or authority header (`cluster1` becomes `cluster1-shadow`), which the docs say "is useful for logging", and `disable_shadow_host_suffix_append` turns that off [7]. `runtime_fraction` samples; "If not specified, all requests to the target cluster will be mirrored" [7]. `trace_sampled` defaults to inheriting the parent span's sampling decision [7]. Shadowing is skipped if the primary cluster does not exist and does not support HTTP CONNECT or upgrades [7]. Bodies are buffered for retries and shadowing; `per_request_buffer_limit_bytes` is deprecated in favour of `request_body_buffer_limit`, and with neither set the listener's `per_connection_buffer_limit_bytes` applies [7], whose unset default is "an implementation defined default is applied (1MiB)" [8]. `request_headers_mutations` edits only the mirrored copy [7]. Excerpt of a full file (listener and primary cluster omitted here) that passed `envoy --mode validate` on `envoyproxy/envoy:v1.39-latest` (1.39.3):

```yaml
routes:
- match: { prefix: "/api/search" }
  route:
    cluster: api_v1
    timeout: 2s
    request_mirror_policies:
    - cluster: api_v2_shadow
      runtime_fraction:                       # 5%, adjustable at runtime
        default_value: { numerator: 5, denominator: HUNDRED }
        runtime_key: shadow.api_search.percent
      trace_sampled: false                    # keep shadow spans out of traces
      request_headers_mutations:
      - append:
          header: { key: x-shadow-request, value: "1" }
          append_action: OVERWRITE_IF_EXISTS_OR_ADD
# clusters:
- name: api_v2_shadow
  type: STRICT_DNS
  connect_timeout: 0.25s
  circuit_breakers:                           # bound what the shadow can hold open
    thresholds: [{ max_connections: 64, max_pending_requests: 32, max_requests: 128 }]
  load_assignment:
    cluster_name: api_v2_shadow
    endpoints: [{ lb_endpoints: [{ endpoint: { address: { socket_address: { address: api-v2, port_value: 8000 } } } }] }]
```

**Istio.** `VirtualService` `mirror` (one destination) or `mirrors` (a list of `HTTPMirrorPolicy`), with `mirrorPercentage` for `mirror` and `percentage` per mirror policy [10]. Mirroring is "on a best effort basis" and the sidecar or gateway does not wait for the mirror [10]; the task page adds that mirrored responses "are discarded" and that hosts get the `-shadow` suffix [9]. An absent percentage means "all the traffic (100%) will be mirrored", and the max is 100 [10]. The old integer `mirror_percent` is still in the proto marked `deprecated = true`, hidden from docs, with the note "Use of integer `mirror_percent` value is deprecated" [11]. Istio documents the same task with a Gateway API `RequestMirror` filter and says it intends Gateway API to become the default traffic API [9].

**Kubernetes Gateway API.** `RequestMirror` on an `HTTPRoute` is an "Extended Support Feature: HTTPRouteRequestMirror", and responses from the mirror "MUST be ignored by the Gateway" [12]. GEP-3171 added sampling because mirroring was "an all or nothing feature" [13]; it reached the Standard channel in Gateway API v1.3.0, released April 24, 2025, with `percent` (integer) or `fraction` (numerator and denominator) [14]. The shipped type (v1.6.3) says "Only one of Fraction or Percent may be specified" and that with neither "100% of requests will be mirrored" [15]; the GEP draft text instead says Fraction takes priority when both are set [13], so trust the CRD, not the GEP. Illustrative, parsed only:

```yaml
apiVersion: gateway.networking.k8s.io/v1
kind: HTTPRoute
metadata: { name: search }
spec:
  parentRefs: [{ name: edge }]
  rules:
  - matches: [{ method: GET, path: { type: PathPrefix, value: /api/search } }]
    backendRefs: [{ name: search-v1, port: 8000 }]
    filters:
    - type: RequestMirror
      requestMirror:
        backendRef: { name: search-v2, port: 8000 }
        fraction: { numerator: 5, denominator: 1000 }   # 0.5%; omit both fields and you mirror 100%
```

**NGINX.** `ngx_http_mirror_module` (since 1.13.4) creates "background mirror subrequests" and "Responses to mirror subrequests are ignored" [16]. `mirror_request_body` defaults to `on`, which reads the client body before creating subrequests and disables unbuffered proxying [16]. The documented trap: a mirror subrequest runs in parallel, but a following request on the same client connection "will not be processed until the previous request and all its subrequest (including mirror subrequests) finish", so slow mirrors plus keep-alive degrade the primary [17]. An NGINX developer later called this "a known side-effect of how mirroring is implemented in nginx, and this is unlikely to change", with `keepalive_timeout 0` on the primary location as the workaround if losing client keep-alive is acceptable [18]. A reproduction measured the primary falling to about 2 requests per second behind a slow mirror and recommended mirroring only part of the traffic with `split_clients` [19]. The config below passed `nginx -t` on nginx 1.30.5 (run with `--add-host` entries for the two upstream names):

```nginx
events {}
http {
  upstream primary   { server api-v1:8000; keepalive 32; }
  upstream candidate { server api-v2:8000; }
  split_clients "${remote_addr}" $mirror_target {   # 5% of client addresses
    5%  candidate;
    *   "";
  }
  server {
    listen 8080;
    location /api/search {
      mirror /_shadow;
      mirror_request_body off;           # GET-only experiment
      proxy_http_version 1.1;
      proxy_set_header Connection "";
      proxy_pass http://primary;
    }
    location = /_shadow {
      internal;
      if ($mirror_target = "") { return 204; }
      proxy_connect_timeout 100ms;       # slow mirrors stall the next request
      proxy_read_timeout 500ms;          # on the same client connection
      proxy_set_header X-Shadow-Request 1;
      proxy_pass http://$mirror_target$request_uri;
    }
  }
}
```

**HAProxy.** Mirroring is an external SPOE agent, `spoa-mirror`, introduced with HAProxy 2.0; HAProxy streams request data to the agent, which sends a copy to `--mirror-url` [20]. HAProxy calls it "fire and forget" with "almost no impact on the time needed to process the request" (vendor-reported) [20]. Mirroring the body needs `option http-buffer-request`; sampling is an ACL such as `rand(100) le 10`, and a map file can switch mirroring on or off at runtime [20].

**Traefik.** A `mirroring` service type with `service`, `mirrors[].percent`, `mirrorBody` and `maxBodySize` [39]. By default "the whole request is buffered in memory while it is being mirrored"; if `percent` is unset "it defaults to 0, meaning no traffic will be sent to the mirror"; and the type "can be defined currently with the File provider or IngressRoute" [39], so not with Docker labels. The doc example comments say `mirrorBody` defaults to true and `maxBodySize` to -1 (unlimited), and a larger body is not mirrored [39]. The source (v3.7.14) shows the mirror copies are dispatched only after the main handler has served the request, in a pool goroutine, written to a `blackHoleResponseWriter`, with no mirroring if the request was cancelled during the main handler, and percent applied by a counter rather than a random draw [40]. The same four fields exist in v3.6.1 [41].

**AWS VPC Traffic Mirroring is a different tool.** It copies network packets "from an elastic network interface" to "out-of-band security and monitoring appliances" for inspection and threat monitoring [21]; packets arrive VXLAN-encapsulated on UDP 4789 with 54 extra bytes per IPv4 packet [22]. Nothing in that pipeline reassembles HTTP requests or replays them against a service, so it is a capture source for appliances, not a shadow [21][22].

**Cloud and edge.** Google Cloud's global external Application Load Balancer has `requestMirrorPolicy`, sent "on a fire and forget basis", unsupported for internet NEGs, serverless NEGs and Private Service Connect backends, and "Requests to the mirrored backend service do not generate any logs or metrics for Cloud Logging and Cloud Monitoring" [23]. Cloudflare documents no mirror setting; the building block is `ctx.waitUntil()`, which lets a Worker do work "without blocking returning a response", capped at 30 seconds after the response [24].

**In-process: Scientist.** `use` wraps the old code (control), `try` the new (candidate); `run` returns the control value, randomizes order, times both, compares and publishes [25]. Because sampling means the candidate may not run, the README states "Scientist is only safe for wrapping methods that aren't changing data" [25], and GitHub's launch post says "Scientist is not meant to be used for any code that has side-effects" and that they "only use Scientist on read operations" [26]. The blocks run "sequentially in random order", so data can change between them; the README suggests first running an experiment where both blocks call the control, to measure that noise [25].

**Recorded replay: GoReplay.** GoReplay is not a proxy; it "listens in the background for traffic on your network interfaces" [28]. It can write requests to a file and replay them preserving inter-request timing, faster with `--input-file "requests.gor|200%"`, and forever with `--input-file-loop` [29]. `--output-http-track-response` forwards replayed responses to outputs such as files [32]; middleware receives requests on stdin for "stripping private data, advanced rewriting, support for oAuth", sees original responses only with `--input-raw-track-response`, and must treat all messages as asynchronous [30]. `--http-allow-method` drops other verbs, `--http-header-limiter user-id:25%` samples consistently by header, and `--output-http-timeout` defaults to 5s [31][32].

**ML shadow mode.** SageMaker shadow tests route a copy of inference requests to a shadow variant inside the same endpoint; "Only the responses of the production variant are returned to the calling application", and shadow responses can be discarded or logged [33]. They are unavailable for serverless, asynchronous, multi-model and multi-container endpoints, among others [33].

| Mechanism | Layer | Sampling controlled by | Mirror response | Body handling | Side-effect guard | Cost to primary |
|---|---|---|---|---|---|---|
| Envoy [7][8] | L7 proxy | `runtime_fraction`, default all | ignored, stats kept | buffered, listener limit default 1MiB | none built in | buffer memory; async send |
| Istio [9][10] | mesh (Envoy) | `mirrorPercentage`, default 100% | discarded | as Envoy | none built in | as Envoy |
| Gateway API [12][15] | gateway CRD | `percent` or `fraction`, default 100% | must be ignored | implementation-defined | none built in | implementation-defined |
| NGINX [16][17] | L7 proxy | `split_clients` or none | ignored | read fully by default | none built in | slow mirror stalls keep-alive connections |
| HAProxy [20] | SPOE agent | ACLs, `rand()` | not reported | needs `http-buffer-request` | ACL by path or method | not reported (vendor says almost none) |
| Traefik [39][40] | L7 proxy | `percent`, default 0 | black-holed | in memory unless `maxBodySize` | none built in | sent after primary response |
| GCP ALB [23] | cloud LB | mirror percent | not waited for | not reported | none built in | no Cloud Logging for mirror |
| Scientist [25][26] | in-process | `enabled?`, `run_if` | compared, then dropped | n/a | rule: reads only | runs candidate inline |
| GoReplay [28][31][32] | packet capture, replay | `|N%`, header limiter | optional tracking | middleware can rewrite | `--http-allow-method` | passive capture; vendor says no effect on the app |
| SageMaker [33] | model endpoint | shadow test config | discard or log | n/a | n/a | shared endpoint |
| AWS VPC mirroring [21][22] | packets | filters | n/a | truncation | n/a | not an HTTP shadow |

## 3. Diffing responses and deciding what a difference means

**Why three instances.** Diffy multicasts each request to a candidate, a primary on known-good code and a secondary on the same known-good code [34]. Candidate versus primary differences are raw signal; primary versus secondary differences are noise, and Diffy compares the two rates, treating an error as ignorable when they are roughly the same [34][35]. Noise sources it names are server-generated timestamps, random generators and races in live downstream data [35]. A random boolean makes 25% of requests a false alarm, which is why Diffy judges aggregate frequencies, not single requests [35]. Twitter has archived Diffy; its original author maintains a fork, Opendiffy [34]. Diffy ignores POST, PUT and DELETE by default unless started with `-allowHttpSideEffects=true` [34].

**Normalise before comparing.** Netflix normalises timestamps, sorts unsorted lists, and transforms intended schema changes on the replay side before diffing [36]. It also records a lineage (data versions or checksums of every dependency) so mismatches caused by different dependency data can be discarded [36]. Google diffs at the protocol-buffer field level, tolerates differences such as list ordering, and recommends a diff "error budget" (for instance, accept up to 1% differing) or excluding hard-to-diff fields [1]. In GitHub's merge experiment, the remaining mismatches came from a missing time argument that made commit timestamps differ [27].

**What to measure.** Log status code, latency and response size for both paths so they can be compared request by request [1]. Netflix's summary counts responses per side, joined responses by correlation id, matches, mismatches and pass or fail per path [36]. For load, watch availability, latency, CPU, memory and garbage collection as the replay load factor changes [36]. If logs are sampled, account for the errors the sample missed [1].

**How long.** Ramp from a small percentage to 100% and, once at 100%, run over a typical load cycle, generally at least one day [1]. GitHub started at 1%, fixed mismatches and slow cases for 4 days, then ran 100% of requests for 24 hours with no mismatches before switching (GitHub-reported) [27]. Experiments "should run for just as long as is necessary to gain confidence rather than being left to run indefinitely" [26].

**When the comparison is meaningless.** A candidate under a sub-50% shadow sees little cache benefit and overstates load at 100% [2]; duplicating traffic above 100% inflates the cache hit rate instead [2]. Stateful systems need distinct, isolated data stores in identical starting states, all request types replayed, and state compared alongside responses, which led Netflix to other techniques [36]. Sequential in-process runs see data change between blocks [25].

The following runnable sketch applies these rules in-process (written for this doc; Scientist's control and candidate model [25], Google's asynchronous candidate call [1], Diffy-style normalisation [35]). It passed `python3 -m py_compile` and `ruff check --select E,F,B` on Python 3.13.5:

```python
"""In-process shadow experiment: serve control, run candidate off the request path."""
import random
import threading
import time
import uuid
from collections import Counter
from concurrent.futures import ThreadPoolExecutor

VOLATILE = {"generated_at", "request_id"}  # fields that differ on every call


def normalise(value):
    if isinstance(value, dict):
        return {k: normalise(v) for k, v in value.items() if k not in VOLATILE}
    if isinstance(value, list):
        return sorted((normalise(v) for v in value), key=repr)
    return value


class Experiment:
    def __init__(self, name, percent, workers=2, max_in_flight=500, seed=None):
        self.name, self.percent = name, percent
        self.enabled = True  # the kill switch: flip to False, shadow work stops
        self.sampler = random.Random(seed)
        self.pool = ThreadPoolExecutor(max_workers=workers)
        self.slots = threading.BoundedSemaphore(max_in_flight)
        self.lock = threading.Lock()
        self.stats, self.mismatches = Counter(), []
        self.timings = {"control": [], "candidate": []}

    def run(self, control, candidate, *args):
        start = time.perf_counter()
        result = control(*args)  # control errors propagate exactly as before
        self.timings["control"].append(time.perf_counter() - start)
        if self.enabled and self.sampler.random() * 100 < self.percent:
            if self.slots.acquire(blocking=False):  # never queue behind the candidate
                self.pool.submit(self._shadow, candidate, args, result)
            else:
                self._record("shed")
        return result

    def _record(self, outcome, detail=None):
        with self.lock:
            self.stats[outcome] += 1
            if detail is not None:
                self.mismatches.append(detail)

    def _shadow(self, candidate, args, control_result):
        try:
            start = time.perf_counter()
            observed = candidate(*args)
            with self.lock:
                self.timings["candidate"].append(time.perf_counter() - start)
            if normalise(observed) == normalise(control_result):
                self._record("match")
            else:
                self._record("mismatch", {"args": args, "candidate": observed})
        except Exception as exc:  # a candidate failure is data, never an outage
            self._record("candidate_error", {"args": args, "error": repr(exc)})
        finally:
            self.slots.release()


def search_v1(q):
    return {"q": q, "hits": sorted(["b", "a", q]), "generated_at": time.time()}


def search_v2(q):
    if q == "boom":
        raise RuntimeError("candidate bug")
    hits = ["a", "b", q] if q != "edge" else ["a", q]  # drops a hit for one input
    return {"q": q, "hits": hits, "generated_at": time.time(),
            "request_id": str(uuid.uuid4())}


def p50_us(samples):
    return sorted(samples)[len(samples) // 2] * 1e6


if __name__ == "__main__":
    exp = Experiment("search-v2", percent=25, seed=7)
    queries = ["x", "y", "edge", "boom", "z"] * 200
    served = [exp.run(search_v1, search_v2, q) for q in queries]
    exp.pool.shutdown(wait=True)
    sampled = sum(exp.stats.values())
    print(f"served {len(served)} responses from control; shadowed {sampled}")
    print("outcomes:", dict(sorted(exp.stats.items())))
    print(f"equivalence rate: {exp.stats['match'] / sampled:.1%}")
    print("inputs that diverged:", sorted({m["args"][0] for m in exp.mismatches}))
    print(f"p50 control {p50_us(exp.timings['control']):.1f} us, "
          f"candidate {p50_us(exp.timings['candidate']):.1f} us")
```

Observed output: `served 1000 responses from control; shadowed 277`, `outcomes: {'candidate_error': 54, 'match': 169, 'mismatch': 54}`, `equivalence rate: 61.0%`, `inputs that diverged: ['boom', 'edge']`; the timestamp and `request_id` differences were normalised away and the candidate exception never reached a caller. Unlike Scientist, which runs both blocks in the request [25], this runs the candidate after the control returns, so it trades same-moment inputs for zero added latency, the trade Google prefers [1].

## 4. Do's and don'ts, each tied to a documented failure

| Do | Don't | Documented reason |
|---|---|---|
| Shadow reads; for writes, stub the mutation after it is prepared, or send it to a temporary duplicate store | let a candidate write to the production store, invalidate its cache or call the real downstream | Scientist's rule against side effects [25][26]; Google's two options for mutating services, and the danger of going live still pointed at the duplicate [2] |
| Filter verbs at the mirror (`--http-allow-method GET`, path ACLs, Diffy's default) | assume the mirror knows which requests are safe | GoReplay [32], HAProxy [20], Diffy [34] |
| Remember that idempotency keys protect retries of the same request to the same API, not a second system | rely on a replayed idempotency key to make a shadow write safe | Stripe returns the saved first response, including 500 errors, for a reused key, and errors if parameters differ [42] |
| Provision every backend and third-party quota for 2x, and give frontends connection slack | forget that a full shadow doubles the cost of every query | Google CRE [2] |
| Make the mirror asynchronous with a short timeout and its own bounded pool | wait on the candidate in the request path | Google prefers asynchronous calls over timeouts alone [1]; NGINX keep-alive stall [17][18][19] |
| Mark shadow traffic sheddable and drop to 0% when CPU, memory or latency rise | keep shadow load during an incident | Google CRE [2] |
| Tag shadow requests (the `-shadow` host, a header) and filter them out of SLOs, analytics and billing | let mirrored requests count as user traffic, or assume the mirror is logged at all | Envoy suffix "is useful for logging" [7]; GCP mirror requests produce no Cloud Logging or Monitoring data [23]; trace sampling inherited by default [7] |
| Strip or tokenise personal data before it leaves the production boundary, keep retention short, and check where the mirror runs | copy bodies with personal data to a staging cluster in another region and keep them | GDPR principles of data minimisation and storage limitation, and data protection by design [43]; trivago mirrored a US-only service into an EU-region stage cluster [38] |
| Rewrite or refresh credentials in middleware for replay | replay captured tokens and expect them to work | GoReplay middleware handles OAuth and stateful token rewriting [30] |
| Ramp: 1%, then more, then 100% for a full load cycle | start at 100% on a service you have not sized | Google CRE [1]; GitHub [27] |
| Keep a one-step kill switch and a named owner who talks to SRE before the experiment | run a shadow nobody can turn off quickly | Google's 0% config push and SRE warning [1][2]; Scientist's `enabled?` [25]; HAProxy map toggle [20] |
| Set exit criteria up front (diff budget, duration, latency parity) | leave an experiment running indefinitely | Google's 1% diff budget [1]; Scientist [26] |
| Mind cache effects when sizing from a partial shadow | read capacity off a 10% shadow | Google CRE [2] |
| Replay at recorded timing first, then at 2x | treat replay timing as wall-clock real | GoReplay preserves inter-request gaps [29]; GitHub's timestamp mismatches [27] |

## 5. Choosing: shadow, canary, flag, replay or synthetic load

| Goal | First choice | Why, per sources | Signal you picked wrong |
|---|---|---|---|
| Capacity of a new backend before launch | live shadow ramped to 100%, optionally above 100% | realistic load test with live traffic over a load cycle [1][2] | cache hit rate distorted by sub-50% or duplicated traffic [2] |
| Correctness of a rewritten read path or function | in-process experiment (Scientist pattern) | compares values on production data, which tests cannot cover [26] | mismatches that are all timing or ordering noise; calibrate with control versus control [25] |
| Migrating a data store | dual writes plus read-path experiments, then cutover | Stripe verified reads from the new table with Scientist before switching writes [37]; Google requires a clear master and a revert path [2] | needing to shadow writes into the production store; stop and redesign [26] |
| Service rewrite or protocol change (REST to gRPC) | server-side or dedicated-service replay with offline diff | Netflix's approach for edge APIs and a REST to gRPC migration [36] | stateful flows that need identical isolated stores [36] |
| ML model, container or instance swap | platform shadow test (SageMaker shadow variant) | production variant answers, shadow is measured [33] | needing user-outcome metrics; shadows cannot show user impact, use a canary [4][33] |
| API contract change with intentional differences | replay with response transformation before diff | Netflix transforms expected changes on the replay side [36] | the transform grows into a second implementation |
| User acceptance of a feature | canary or flag-based release | exposes a subset of real users [4][6] | none of the users can be exposed yet; shadow instead [1] |
| Peak or holiday load with no traffic to copy | synthetic traffic | a dark launch can fake purchases from views at a sampled ratio, but only approximately [2] | the synthetic mix is the thing under test |
| Fast rollback of a whole environment | blue-green | switch the router, keep the old side idle [5] | you need to compare answers, not switch them |

## 6. Case studies (primary write-ups only)

| Case | What was mirrored | Side effects | What was measured | What went wrong or surprised |
|---|---|---|---|---|
| GitHub merges via libgit2 [27] | merge computation, Scientist, 1% then 100% | reads only, old result always returned | mismatches, timing, runs over 5000 ms | old code timed out; timestamps differed; found 2 Git bugs and 3 performance issues; p99 new roughly equal to p95 old (GitHub-reported) |
| GitHub permissions [26] | multi-year permissions rewrite | read operations only | mismatches and durations, Graphite for metrics and Redis for mismatch data | GitHub notes mismatch triage sometimes finds the bug in the legacy code or the data, not the rewrite [26] |
| Stripe Subscriptions [37] | reads from new versus old tables during dual writing | writes duplicated deliberately, reads compared | Scientist alerts on any inconsistency | migration of "hundreds of millions of Subscriptions objects" [37] |
| Twitter Diffy [34][35] | HTTP and Thrift requests to candidate, primary, secondary | POST, PUT, DELETE off by default | candidate versus noise disagreement rates | timestamps, randomness and live data races produce noise [35] |
| Netflix migrations [36] | device-driven, server-driven, then a dedicated Mantis replay service | stateless and idempotent systems; isolated stores otherwise | match rate, per-path pass and fail, load metrics | device forking wasted device resources and exposed attack surface; server forking coupled replay bugs to production [36] |
| trivago cross-cluster [38] | Istio mirror from a US-only production service to an EU stage cluster | stage must not treat mirror as live traffic | stage access logs | mirrors returned 404 until the stage gateway matched the `-shadow` host [38] |

No company write-up of an ML shadow deployment beyond vendor documentation was verified for this doc, so SageMaker [33] stands in.

## Agreed vs folklore (compressed)

| Claim | Verdict |
|---|---|
| "Mirroring is free because responses are discarded." | Folklore. Every shadowed query costs twice [2]; NGINX mirrors can stall the primary [17][18]; Traefik buffers whole bodies by default [39]. |
| "Shadowing proves correctness." | Half true. Diffing proves equivalence on sampled inputs within a noise floor [34][35]; it cannot show user impact [33], and mutations cannot be compared in parallel [1]. |
| "You can shadow writes if you roll back the database afterwards." | Folklore. The documented options are stubbing the mutation or a temporary duplicate store [2]; candidate writes to the shared store are "dangerous and incorrect" [26]. |
| "A service mesh makes it safe by default." | Folklore. Istio mirrors 100% when the percentage is absent [10] and adds no side-effect guard [9][10]. |
| "Replay is the same as mirroring." | Folklore. Replay can be offline, re-timed and transformed [29][36]; mirroring is a live fork [7]. |
| "Fire and forget means zero latency impact." | Only in the narrow sense that the proxy does not wait for the mirror: Envoy [7], Traefik [40]. False for NGINX with keep-alive [17], and buffering still costs memory [7][39]. |

## Synthesis (inferred)

Nothing in this section is cited; it is the author's reading of the evidence above.

**Runbook.**

- *Before:* write the question (correctness, capacity, or both) and the exit criteria: equivalence rate after normalisation, p99 latency within an agreed ratio, error rate parity, duration of at least one full weekly peak. Inventory every side effect the path can reach (DB writes, queues, email, payments, webhooks, caches, third-party calls) and give each one a stub, sandbox or duplicate store. Decide what leaves the production boundary and strip it. Pick the sampling knob and confirm its default (0% in Traefik, 100% in Istio, Gateway API and Envoy). Exclude login, one-time-code, CSRF-protected and rate-limited routes from the mirror: each consumes single-use state or quota a second time, or fails noisily in the candidate. Name an owner and a kill switch that works in one step.
- *During:* start at 1%, verify that shadow requests are tagged and excluded from SLO, analytics and billing dashboards, then ramp. Run a control-versus-control pass to measure the noise floor before reading candidate numbers. Watch the primary's latency and the shared dependencies, not just the candidate.
- *Exit:* stop when the criteria hold at 100% for the agreed window, or when the remaining mismatches are understood and accepted. Delete the mirror config, the shadow deployment and the stored bodies; a forgotten mirror is a standing cost and a data-retention problem.

**Reference setup for a small team.** One gateway rule mirroring 1% to 5% of GET traffic on one route to a candidate deployment with its own read-only replica credentials; candidate responses written to a small diff service (the Python sketch above grown into a process, or Opendiffy) that normalises, compares against a recorded primary response or a second known-good instance, and emits match, mismatch and error counters tagged by route; one dashboard with equivalence rate, candidate versus primary latency percentiles, and primary latency with and without the mirror. That is enough for most rewrites; a dedicated replay service only pays off at Netflix-like scale.

**Application note for mx projects (inferred, not executed against a live stack).** mx services register with the global Traefik router through Docker labels, and the router also loads a file provider from `~/.mech-crate/router/config/dynamic/`. Traefik's `mirroring` service cannot be declared in labels, so the first shadow experiment for an mx app is a file in that directory, picked up by `mx router reload`: a higher-priority router for the narrow, read-only slice of traffic, pointing at a mirroring service whose main service is the live app and whose single mirror is a candidate container on the `devmesh-traefik` network. The file below was loaded by Traefik v3.6.1 (the router template's version) and both objects reported `enabled`; container names follow the `<project>-<service>-1` convention and are placeholders.

```yaml
# ~/.mech-crate/router/config/dynamic/shadow-myapp.yml
http:
  services:
    myapp-shadowed:
      mirroring:
        service: myapp-live
        mirrorBody: false          # GET-only experiment; nothing to buffer
        maxBodySize: 1048576       # if mirrorBody is turned on later, cap it
        mirrors:
          - name: myapp-candidate
            percent: 5             # default is 0: an unset percent mirrors nothing
    myapp-live:
      loadBalancer:
        servers:
          - url: "http://myproj-myapp-1:8000"
    myapp-candidate:
      loadBalancer:
        servers:
          - url: "http://myproj-myapp-candidate-1:8000"
  routers:
    myapp-shadow:
      rule: "Host(`myapp.localhost`) && Method(`GET`) && PathPrefix(`/api/search`)"
      priority: 100
      service: myapp-shadowed
      entryPoints: [web]
```

Point the candidate at a copy of the database, not the dev database itself, and give it a log field such as `shadow=true` so its output never reads as the app's own. Because Traefik sends mirrors after the main response and drops their responses, the comparison has to happen elsewhere: have the candidate log a normalised response hash per request id, have the live app log the same, and join them. Delete the file to end the experiment; deleting it is the kill switch.
