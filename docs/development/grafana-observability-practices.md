---
title: "Grafana in Practice: Dashboards, Alerting, the LGTM Stack, and Observability as Code"
category: infra
languages: [promql, logql, yaml]
complexity: intermediate
use_cases:
  - standing up Grafana with Prometheus, Loki, Tempo or Mimir for a service and deciding what goes on dashboards versus alerts
  - replacing CPU-and-everything alerting with symptom and multi-window burn-rate SLO alerts
  - choosing between file provisioning, Terraform, the Foundation SDK, Git Sync and gcx for dashboards and alert rules in git
  - planning a Grafana 12 to 13 upgrade, an Agent or Promtail to Alloy migration, or a self-hosted versus Grafana Cloud decision
summary: "Grafana 13.2 practice: version and support calendar, removals to plan around (Angular, legacy alerting, API keys, Agent, Promtail), RED/USE dashboards and sprawl control, query cost, Grafana-managed alerting with burn-rate SLOs, provisioning versus Git Sync, and hardening."
provenance: researched
researched: 2026-10-05
sources:
  - https://github.com/grafana/grafana/releases
  - https://grafana.com/docs/grafana/latest/upgrade-guide/when-to-upgrade/
  - https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v13-0/
  - https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v13-2/
  - https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v12-0/
  - https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v12-3/
  - https://grafana.com/docs/grafana/latest/breaking-changes/breaking-changes-v11-0/
  - https://github.com/grafana/grafana/blob/main/LICENSING.md
  - https://github.com/grafana/agent
  - https://grafana.com/docs/loki/latest/send-data/promtail/
  - https://grafana.com/docs/alloy/latest/introduction/
  - https://grafana.com/docs/alloy/latest/set-up/migrate/from-static/
  - https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/best-practices/
  - https://sre.google/sre-book/monitoring-distributed-systems/
  - https://grafana.com/docs/grafana/latest/visualizations/dashboards/assess-dashboard-usage/
  - https://grafana.com/docs/grafana/latest/visualizations/dashboards/build-dashboards/manage-library-panels/
  - https://grafana.com/docs/grafana/latest/datasources/prometheus/template-variables/
  - https://grafana.com/docs/grafana/latest/visualizations/panels-visualizations/query-transform-data/
  - https://grafana.com/docs/grafana/latest/visualizations/panels-visualizations/query-transform-data/transform-data/
  - https://grafana.com/docs/grafana/latest/visualizations/panels-visualizations/configure-data-links/
  - https://grafana.com/docs/grafana/latest/administration/data-source-management/
  - https://prometheus.io/docs/practices/rules/
  - https://prometheus.io/docs/practices/naming/
  - https://grafana.com/docs/loki/latest/get-started/labels/bp-labels/
  - https://grafana.com/docs/loki/latest/query/bp-query/
  - https://grafana.com/docs/loki/latest/get-started/labels/structured-metadata/
  - https://grafana.com/docs/tempo/latest/traceql/construct-traceql-queries/
  - https://grafana.com/docs/grafana/latest/fundamentals/exemplars/
  - https://grafana.com/docs/mimir/latest/manage/tools/mimirtool/
  - https://grafana.com/docs/grafana-cloud/observe-and-act/adaptive-telemetry/adaptive-metrics/
  - https://grafana.com/docs/grafana/latest/alerting/fundamentals/alert-rules/
  - https://grafana.com/docs/grafana/latest/alerting/alerting-rules/create-data-source-managed-rule/
  - https://grafana.com/docs/grafana/latest/alerting/fundamentals/alert-rule-evaluation/
  - https://grafana.com/docs/grafana/latest/alerting/fundamentals/alert-rule-evaluation/state-and-health/
  - https://grafana.com/docs/grafana/latest/alerting/fundamentals/notifications/notification-policies/
  - https://grafana.com/docs/grafana/latest/alerting/fundamentals/notifications/group-alert-notifications/
  - https://grafana.com/docs/grafana/latest/alerting/configure-notifications/mute-timings/
  - https://grafana.com/docs/grafana/latest/alerting/set-up/provision-alerting-resources/
  - https://grafana.com/docs/grafana/latest/alerting/guides/best-practices/
  - https://sre.google/workbook/alerting-on-slos/
  - https://grafana.com/docs/grafana-cloud/observe-and-act/alert-and-measure-reliability/slo/
  - https://grafana.com/docs/grafana/latest/alerting/set-up/configure-high-availability/
  - https://grafana.com/docs/grafana/latest/administration/provisioning/
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/
  - https://grafana.com/docs/grafana/latest/as-code/observability-as-code/git-sync/usage-limits/
  - https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis/
  - https://github.com/grafana/terraform-provider-grafana
  - https://github.com/grafana/grafana-foundation-sdk
  - https://github.com/grafana/gcx
  - https://github.com/grafana/grafanactl
  - https://github.com/grafana/dashboard-linter
  - https://github.com/monitoring-mixins/mixtool
  - https://grafana.com/docs/loki/latest/send-data/otel/
  - https://grafana.com/docs/mimir/latest/configure/configure-otel-collector/
  - https://grafana.com/docs/tempo/latest/configuration/
  - https://prometheus.io/docs/guides/opentelemetry/
  - https://github.com/grafana/beyla
  - https://grafana.com/docs/pyroscope/latest/introduction/
  - https://grafana.com/docs/grafana/latest/datasources/tempo/configure-tempo-data-source/
  - https://grafana.com/docs/grafana/latest/administration/correlations/
  - https://grafana.com/docs/grafana/latest/setup-grafana/set-up-for-high-availability/
  - https://grafana.com/docs/grafana/latest/setup-grafana/image-rendering/
  - https://grafana.com/docs/grafana/latest/administration/back-up-grafana/
  - https://grafana.com/docs/grafana/latest/setup-grafana/configure-access/configure-authentication/saml/
  - https://grafana.com/docs/grafana/latest/setup-grafana/configure-security/configure-team-sync/
  - https://grafana.com/docs/grafana/latest/administration/roles-and-permissions/access-control/
  - https://grafana.com/docs/grafana/latest/administration/service-accounts/
  - https://grafana.com/docs/grafana/latest/administration/service-accounts/migrate-api-keys/
  - https://grafana.com/docs/grafana/latest/setup-grafana/configure-security/configure-database-encryption/
  - https://grafana.com/docs/grafana/latest/administration/plugin-management/plugin-sign/
  - https://grafana.com/docs/grafana/latest/setup-grafana/configure-security/configure-security-hardening/
  - https://grafana.com/security/security-advisories/
  - https://grafana.com/security/security-advisories/cve-2026-76154/
  - https://grafana.com/security/security-advisories/cve-2026-13720/
  - https://henrikgerdes.me/blog/2025-11-grafana-mess/
  - https://grafana.com/docs/mimir/latest/release-notes/v3.0/
  - https://github.com/grafana/dashboard-linter/blob/main/docs/rules/target-rate-interval-rule.md
  - https://github.com/grafana/dashboard-linter/blob/main/docs/index.md
  - https://github.com/grafana/terraform-provider-grafana/releases/tag/v4.47.0
  - https://github.com/grafana/gcx/releases/tag/v1.4.0
---

# Grafana in Practice: Dashboards, Alerting, the LGTM Stack, and Observability as Code

State of practice as of 2026-10. The version line described is Grafana 13.2 (13.2.3 shipped on 2026-09-29), alongside the still-supported 13.1, 13.0 and 12.4 lines [1][2]. The seed sources are Grafana's own release cadence page [2], the 12.0, 13.0 and 13.2 "What's new" pages [3][4][5], the dashboard best-practices guide [13] and the Grafana Alerting fundamentals [31][33]; every version, date, default and edition restriction below was read off the cited page, not recalled. Supporting claims come from the Prometheus, Loki, Tempo, Mimir and Alloy docs, the Google SRE books [14][40], and Grafana's security advisories [72]. Inline `[n]` keys to `sources`. Edition labels follow the docs: **OSS** (open source), **Enterprise**, **Cloud**. The Prometheus rules and the Grafana provisioning files below were executed (promtool and a Grafana 13.2.3 container); LogQL, TraceQL and the Alloy snippet are marked where they were not.

Scope note: this is about running Grafana and the signals behind it as a team, not a product tour. Container build and compose practice lives in [docker-assembly-guide.md](docker-assembly-guide.md); event and stream design in [appendix-streams.md](appendix-streams.md); the dependency-hygiene side of plugins and collectors in [supply-chain-security-cargo-npm.md](supply-chain-security-cargo-npm.md).

## 1. Version landscape and removals to plan around

**Cadence.** Minor releases come "every other month", a major "once a year, in April/May", and patch releases "at least every month" [2]. Each minor is supported for 9 months and the last minor of a major gets 15 months [2]. The documented support table [2]:

| Line | Released [2] | Support ends [2] | Note |
|---|---|---|---|
| 12.4.x | 2026-02-24 | 2027-05-24 | last 12.x minor, extended window |
| 13.0.x | 2026-04-14 | 2027-01-09 | |
| 13.1.x | 2026-06-23 | 2027-03-20 | |
| 13.2.x | 2026-08-18 | 2027-05-18 | current; 13.2.3 on 2026-09-29 [1] |
| 13.3.x | 2026-10-20 | 2027-07-20 | scheduled |

Grafana recommends the "minor / bi-monthly" upgrade strategy, with changelog review, a test environment first, a database backup before every upgrade and plugin updates before core updates [2]. Patch releases land on several lines at once: v12.4.12, v13.0.10, v13.1.7 and v13.2.3 all shipped on 2026-09-29 [1].

**What the last three majors changed in practice:**

| Change | Status as documented | Practice consequence |
|---|---|---|
| Legacy alerting | "Legacy alerting is entirely removed" in v11; "Grafana v10.4.x is the last version that offers migration" [7] | an instance still on legacy alerting must pass through 10.4 first [7] |
| Angular plugins | "deprecated and turned off by default in Grafana 11 and is now being removed Grafana v12" [5] | Angular panels are gone; audit plugins before any 11 to 12+ jump [5][7] |
| API keys | 12.3 removed `apikeys:*` permissions "following the deprecation and removal of API Keys" [6] | use service account tokens [68] |
| Scenes | v13 "removed the the ability to disable Scenes" [3] | no opt-out from the Scenes dashboard runtime [3] |
| Dynamic dashboards, schema v2 | experimental in 12.0, where a migrated dashboard "can't be migrated back" [5]; GA and "on by default" in 13.0, existing dashboards "migrated to the new schema when you open" them [3] | existing dashboards change schema the first time they are opened on 13 [3] |
| Git Sync | experimental in 12.0 [5]; GA in 13.0 for Cloud and self-managed [3]; GitHub Enterprise support GA in 13.2 for Cloud and Enterprise [4] | dashboards and folders only (section 5) [45] |
| `/apis` | 13.0 "officially deprecating the /api path in favor of the new /apis path" [3] | new automation should target `/apis` [46] |
| Unified storage | 13.0 warning: "A migration bug in Grafana v13.0.0 might cause dashboards and folders to be lost or reverted when upgrading from Grafana v12.x.x with Git Sync enabled" [3] | Git Sync users skip 13.0.0 and back up first [3] |
| Image Renderer plugin | "Starting from Grafana 13, support for the plugin is fully removed" [3] | run the separate renderer service [62] |

**Collector removals.** "Grafana Agent has reached End-of-Life (EOL) on November 1, 2025" and the repository is archived, with Alloy as the named replacement [9]. "Promtail is end of life (EOL) as of March 2, 2026", and "All future feature development will occur in Grafana Alloy" [10]. Alloy ships an `alloy convert` command that turns an Agent Static configuration into Alloy syntax [12].

**Licence.** The default licence for the Grafana repository is AGPL-3.0-only, with listed package directories such as `packages/grafana-ui/` under Apache-2.0 [8].

## 2. Dashboard design strategy

**Pick the method per layer.** Grafana's guide frames USE (utilization, saturation, errors) for resources, RED (rate, errors, duration) for services, and the four golden signals (latency, traffic, errors, saturation) [13]. Its one-line rule: "The USE method tells you how happy your machines are, the RED method tells you how happy your users are" [13]. The SRE book adds that dashboards "should answer basic questions about your service, and normally include some form of the four golden signals" [14].

**Maturity model (still published).** The guide's low, medium and high levels map directly to a checklist [13]:

- [ ] Medium: "Prevent sprawl by using template variables", "Hierarchical dashboards with drill-downs to the next level", "Compare like to like: split service dashboards when the magnitude differs", "Version-controlled dashboard JSON" [13].
- [ ] High: "Actively reducing sprawl", "Consistency by design" through scripting libraries, "No editing in the browser. Dashboard viewers change views with variables", "Browsing for dashboards is the exception, not the rule" [13].
- [ ] Every dashboard passes "A dashboard should tell a story or answer a question" [13].

**Sprawl control.** Use query variables instead of per-node copies [13]; library panels, where "that change propagates to all instances" [16]; usage insights to "find most-used, broken, and unused dashboards", which is Enterprise and Cloud only [15]; restore of deleted dashboards, GA in 13.0 [3]; and provisioned dashboards with `allowUiUpdates: false`, which refuse UI saves [43].

**Variables.** With multi-value or "Include All", "the variable value becomes a regular expression pattern", so queries must use `=~` [17]. For `rate()` and `increase()`, "Always use `$__rate_interval` instead of a fixed interval or `$__interval`", because a fixed `[5m]` "breaks at different zoom levels" and `$__interval` "can be too small for rate()" [17]. The rule is `max($__interval + scrape_interval, 4 * scrape_interval)`, where scrape_interval is the per-query Min step or else the data source's Scrape interval setting [17]. That setting must match Prometheus: if it "is left at the default 15s but your actual Prometheus scrape interval is 60s, $__rate_interval calculates too small a window" [17]. The rule is scoped: `$__rate_interval` is "designed for use with rate() and increase()" [17], and the dashboard linter's `target-rate-interval-rule` "Checks that each target uses $__rate_interval" [78].

**Units, thresholds, links.** Prometheus names carry base units (`_seconds`, `_bytes`, `_total`) [23]; set the matching panel unit so axes read correctly, which the dashboard linter's `panel-units-rule` checks ("Checks that each panel uses has valid units defined") [78]. Data links "allow you to link to other panels, dashboards, and external resources" while keeping the source panel's context [20], which is how drill-down from a RED overview to a per-instance view is wired [13].

**Transformations versus query-side work.** Transformations manipulate data "returned by a query before the system applies a visualization" [19], and using "the output of one transformation as the input to another transformation" "results in a performance gain" [19].

**Dashboard or alert?** Grafana's guide: "alerting should be done on RED dashboards", because USE "reports on causes" and RED on symptoms [13]. Grafana's alerting guide keeps infrastructure signals to "support diagnosis and prevention", not to replace symptom alerts [39].

## 3. Query cost and performance

**PromQL.** Name recording rules `level:metric:operations` and compute ratios by aggregating numerator and denominator separately, then dividing, rather than averaging ratios [22]. Every label combination "represents a new time series", so user IDs, emails and other unbounded values never become labels [23].

**Panel controls.** Max data points sets "the maximum number of data points for each series returned"; Min interval bounds the auto interval and is "typically the minimum scrape interval"; Relative time overrides the dashboard range per panel; and Grafana "supports up to 26 queries per panel" [18].

**Query caching is Enterprise and Cloud only** ("Available in Grafana Enterprise and Grafana Cloud") [21]. It helps less than expected for the LGTM backends: "Elasticsearch, Prometheus, and Loki, cache queries themselves, so Grafana query caching does not significantly improve performance" [21]. In-memory caching "can increase Grafana's memory footprint", and Redis or Memcached is "highly recommended" in production [21].

**LogQL for Loki.** Labels are for low-cardinality context, "ideally limited to tens of values" [24]; "Do not extract ephemeral values like a trace ID or an order ID into a label" [24]; keep a tenant under "100,000 active streams" [24]. High-cardinality context that is not in the line goes into structured metadata, which attaches metadata "without indexing them" [26]. Query order: narrow the time range, use the most specific label selector, line filters (`|=`, `!=`) before regex, and "parser expressions only after line filter expressions" [25]. The following query is illustrative (not executed against a Loki instance):

```logql
sum by (type) (count_over_time({service_name="rust-worker", env="prod"} |= "job failed" | json | duration_seconds > 5 [5m]))
```

**TraceQL for Tempo.** Curly braces select spans; start broad and filter, prefer intrinsics such as `trace:duration` and `trace:rootService`, and join conditions with `&&` [27]. This query is taken from the TraceQL docs and is illustrative here [27]:

```traceql
{ resource.service.name="frontend" && name = "POST /api/orders" && status = error }
```

**Exemplars** are "a specific trace representative of measurement taken in a given time interval", the bridge from a latency panel to one trace [28].

**Cardinality control.** `mimirtool analyze grafana` and `analyze ruler` list the metrics that dashboards and rules use, and `analyze prometheus` then shows which stored metrics are *not* used [29]; it works against Mimir, Prometheus or Cloud [29]. Adaptive Metrics recommends aggregating "underutilized metrics into lower-cardinality versions" and is documented under Grafana Cloud [30].

## 4. Alerting strategy

**Grafana-managed by default.** Data-source-managed rules live in Mimir or Loki; Grafana's own note: "We recommend using Grafana-managed alert rules whenever possible" [32]. In the documented comparison, Grafana-managed rules work with all backend data sources that support alerting, mix data sources, add expressions, handle no data and error states and attach images, while data-source-managed rules can be created only for Mimir and Loki and do none of the rest [32]. New Grafana Cloud stacks no longer provision data-source-managed alerting for the default Loki and Prometheus sources (Cloud only) [32].

**Evaluation.** Three settings decide firing: the evaluation group (interval), the pending period ("how long the condition must be met to start firing") and keep firing for (flap suppression) [33]. Grafana's best-practice guide: "Require issues to persist before alerting. Set a pending period" [39]. A query with no data triggers a separate `DatasourceNoData` alert by default and an evaluation error a `DatasourceError` alert [34]; those instances have different labels, so "existing silences, mute timings, and notification policies applied to the original alert may not apply to them" [34]. "Keep Last State" smooths transient data source issues but "may not be appropriate" where monitoring is critical [34].

**Routing.** Notification policies are "not a list, but rather are structured according to a tree", rooted at the default policy; matching stops at the first sibling unless "Continue matching siblings" is enabled [35]. Default grouping is by `alertname` and `grafana_folder` [36]. Timer defaults: group wait 30 seconds, group interval 5 minutes, repeat interval 4 hours [36].

A mute timing uses time intervals "that can repeat periodically" and is attached to policies (weekends, maintenance windows); a silence matches labels with "a fixed start and end time" (one incident, one deploy) [37]. Neither stops evaluation; both "only prevent notifications from being created" [37].

**Symptoms, then SLOs.** The SRE book separates "what's broken" (symptom) from "why" (cause) [14]; Grafana's guide says "Alerts should primarily detect user-facing failures" while infrastructure alerts should "support diagnosis and prevention" [39], and to "Graduate symptom-based alerts into SLOs" when they fire often [39]. The SRE Workbook's multiwindow, multi-burn-rate method uses a short window of "1/12 the duration of the long window" and these starting parameters for a 99.9% SLO [40]:

| Severity [40] | Long window | Short window | Burn rate | Budget consumed |
|---|---|---|---|---|
| Page | 1 hour | 5 minutes | 14.4 | 2% |
| Page | 6 hours | 30 minutes | 6 | 5% |
| Ticket | 3 days | 6 hours | 1 | 10% |

Applied to the `rust-worker` metrics (`jobs_processed_total{status}`) with the naming and ratio rules from section 3 [22][40]; this file passes `promtool check rules` and a `promtool test rules` case where a 3% failure ratio fires the page alert by minute 90:

```yaml
groups:
  - name: rust-worker-slo-recording
    interval: 30s
    rules:
      - record: job:jobs_failed_per_processed:ratio_rate5m
        expr: sum by (job) (rate(jobs_processed_total{status="failed"}[5m])) / sum by (job) (rate(jobs_processed_total[5m]))
      - record: job:jobs_failed_per_processed:ratio_rate30m
        expr: sum by (job) (rate(jobs_processed_total{status="failed"}[30m])) / sum by (job) (rate(jobs_processed_total[30m]))
      - record: job:jobs_failed_per_processed:ratio_rate1h
        expr: sum by (job) (rate(jobs_processed_total{status="failed"}[1h])) / sum by (job) (rate(jobs_processed_total[1h]))
      - record: job:jobs_failed_per_processed:ratio_rate6h
        expr: sum by (job) (rate(jobs_processed_total{status="failed"}[6h])) / sum by (job) (rate(jobs_processed_total[6h]))
      - record: job:jobs_failed_per_processed:ratio_rate3d
        expr: sum by (job) (rate(jobs_processed_total{status="failed"}[3d])) / sum by (job) (rate(jobs_processed_total[3d]))
  - name: rust-worker-slo-alerts
    rules:
      # 99.9% job success objective: error budget 0.001.
      - alert: WorkerErrorBudgetBurnFast
        expr: |
          (job:jobs_failed_per_processed:ratio_rate1h > (14.4 * 0.001)
            and job:jobs_failed_per_processed:ratio_rate5m > (14.4 * 0.001))
          or
          (job:jobs_failed_per_processed:ratio_rate6h > (6 * 0.001)
            and job:jobs_failed_per_processed:ratio_rate30m > (6 * 0.001))
        labels:
          severity: page
        annotations:
          summary: "{{ $labels.job }} is burning its job error budget fast"
          runbook_url: "https://runbooks.example.invalid/rust-worker/error-budget"
      - alert: WorkerErrorBudgetBurnSlow
        expr: |
          job:jobs_failed_per_processed:ratio_rate3d > 0.001
          and job:jobs_failed_per_processed:ratio_rate6h > 0.001
        labels:
          severity: ticket
        annotations:
          summary: "{{ $labels.job }} spent 10% of its 30-day error budget in 3 days"
```

Grafana SLO creates SLOs with their alert rules, an SLO dashboard showing "burn rate, error budget, SLI results", and maintenance windows that "pause error budget consumption and burn rate alerting"; it is documented under Grafana Cloud [41].

**Alerting HA.** Every instance evaluates every rule; the Alertmanagers gossip on TCP and UDP port 9094 to avoid duplicate notifications, choosing "availability over consistency" [42]. Memberlist is preferred and Redis is the fallback "only in environments where direct communication between Grafana servers is not possible" [42]. `ha_single_node_evaluation = true` makes one automatically chosen primary instance evaluate rules, and requires HA clustering to be configured [42].

## 5. Observability as code

**File provisioning (OSS and Enterprise, not Cloud for alerting).** Data sources, dashboards and alerting resources load from YAML at startup [43][38]. Provisioning locks the UI: "You cannot edit provisioned resources from files in the Grafana UI", and "Provisioning with configuration files is not available in Grafana Cloud" [38]. Resources stay editable only in the source that created them; file-provisioned alerting cannot be edited "in Terraform or from within Grafana" [38]. For dashboards, `allowUiUpdates: false` raises "Cannot save provisioned dashboard", while `allowUiUpdates: true` saves to the database and is overwritten by the next file change [43]. Environment variables interpolate in provisioning files but "not in the dashboard definition files themselves", and a literal `$` must be written `$$` [43]. `prune: true` deletes data sources removed from the file; with several Grafana instances, version each data source so only newer files apply [43].

These three files booted cleanly on Grafana 13.2.3 with the provisioning directory mounted read-only; the API then reported the data source as read-only, the dashboard as provisioned, and the contact point, policy and rule with provenance `file`:

```yaml
# provisioning/datasources/prometheus.yml
apiVersion: 1
prune: true
datasources:
  - name: Prometheus
    uid: prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: 30s   # must equal the Prometheus scrape_interval
---
# provisioning/dashboards/rust-worker.yml
apiVersion: 1
providers:
  - name: rust-worker
    type: file
    disableDeletion: true
    allowUiUpdates: false
    updateIntervalSeconds: 30
    options:
      path: /var/lib/grafana/dashboards
      foldersFromFilesStructure: true
---
# provisioning/alerting/rust-worker.yml
apiVersion: 1
contactPoints:
  - orgId: 1
    name: oncall-webhook
    receivers:
      - uid: oncall-webhook
        type: webhook
        settings:
          url: ${ONCALL_WEBHOOK_URL}
policies:
  - orgId: 1
    receiver: oncall-webhook
    group_by: ['grafana_folder', 'alertname']
groups:
  - orgId: 1
    name: rust-worker-slo
    folder: rust-worker
    interval: 1m
    rules:
      - uid: rw-burn-fast
        title: Worker error budget burn (fast)
        condition: C
        data:
          - refId: A
            relativeTimeRange: {from: 600, to: 0}
            datasourceUid: prometheus
            model:
              refId: A
              instant: true
              expr: >-
                (job:jobs_failed_per_processed:ratio_rate1h > (14.4 * 0.001)
                and job:jobs_failed_per_processed:ratio_rate5m > (14.4 * 0.001))
                or (job:jobs_failed_per_processed:ratio_rate6h > (6 * 0.001)
                and job:jobs_failed_per_processed:ratio_rate30m > (6 * 0.001))
          - refId: C
            datasourceUid: __expr__
            model:
              refId: C
              type: threshold
              expression: A
              conditions:
                - evaluator: {type: gt, params: [0]}
        for: 2m
        noDataState: OK
        execErrState: Error
        labels:
          severity: page
        annotations:
          runbook_url: https://runbooks.example.invalid/rust-worker/error-budget
```

**The tool landscape, verified status.**

| Tool | Manages | Status (2026-10) |
|---|---|---|
| File provisioning | data sources, dashboards, alerting, folders [43][44] | stable; alerting files not on Cloud [38] |
| Terraform provider | dashboards, folders, data sources, rule groups, contact points, policies, mute timings, service accounts [47] | v4.47.0, 2026-09-30 [79] |
| Foundation SDK | typed builders in Go, TypeScript, Python, PHP, Java; "best suited for Grafana >= 12" [48] | official [44][48] |
| Grafonnet (Jsonnet) | dashboard JSON generation | "Grafonnet is not officially supported by Grafana. Instead, use the Foundation SDK" [44] |
| Git Sync | "only supports dashboards and folders"; up to four nested folders; Pure Git needs Smart HTTP protocol v2 over HTTPS [45] | GA in 13.0 [3]; Cloud Free: 1 repository, 20 resources [45] |
| `gcx` CLI | dashboards, alerts, SLOs, queries over the `/apis` layer, Grafana 12+ [49][44] | generally available badge [49]; v1.4.0, 2026-10-02 [80] |
| `grafanactl` | predecessor CLI | "being deprecated" in favour of gcx, announced for archiving on June 1st, 2026 [50] |
| Grafana Operator | dashboards, folders, data sources as Kubernetes custom resources [44] | listed as an additional tool [44] |

Git Sync guidance: "Do not sync more than 1,000 resources per repository connection as of today", and "Full-instance sync is experimental" [45]. The `/apis` layer is Kubernetes-style, `/apis/<GROUP>/<VERSION>/namespaces/<NAMESPACE>/<RESOURCE>[/<NAME>]`, with `default` as the namespace for organization 1, for example `GET /apis/dashboard.grafana.app/v1/namespaces/default/dashboards/production-overview` [46].

**Review in CI.** `dashboard-linter` lints dashboards that use a Prometheus data source [51]; its rule doc, headed `rate-interval-rule` (listed in the index as `target-rate-interval-rule`), "Checks that every target with a `rate`, `irate` or `increase` function uses `$__rate_interval` for the range of the data to process" [77][78]. `mixtool` is "a helper for easily working with jsonnet mixins" [52].

## 6. Collection and correlation

**Alloy is the collector.** Alloy is "an OpenTelemetry Collector distribution with built-in Prometheus pipelines and native support for Loki, Pyroscope, and other observability backends" [11]. OTLP ingestion paths, all documented:

| Backend | OTLP endpoint | Caveat |
|---|---|---|
| Mimir | `otlphttp` exporter to `http://<mimir-endpoint>/otlp` [54] | exponential histograms need native histogram ingestion enabled first [54] |
| Loki | `otlphttp` exporter to `http://<loki-addr>/otlp` [53] | needs `allow_structured_metadata`; "enabled by default in Loki 3.0 and later" [53] |
| Tempo | OTLP receiver on gRPC 4317 and HTTP 4318, "default localhost" [55] | bind an external address explicitly [55] |
| Prometheus | `--web.enable-otlp-receiver` serves `/api/v1/otlp/v1/metrics` [56] | off by default because it lacks authentication [56] |

Loki's OTLP path stores a fixed set of resource attributes (for example `k8s.namespace.name`, `deployment.environment.name`) as index labels and the rest as structured metadata [53], which matches the label guidance in section 3 [24].

**eBPF auto-instrumentation.** Beyla produces "OpenTelemetry web transaction trace spans and Rate-Errors-Duration (RED) metrics" without code changes, and "has been donated to the CNCF OpenTelemetry Project, under the project name OpenTelemetry eBPF Instrumentation or OBI" [57]. New development happens in the upstream OBI repository [57]. For profiles, Pyroscope is "a multi-tenant continuous profiling aggregation system", architecturally aligned with Mimir, Loki and Tempo [58].

**Correlation wiring.** The Tempo data source has trace to logs and trace to metrics settings, and Loki derived fields link logs back to traces [59]. Correlations generalise this: "A correlation defines how data in one data source is used to query data in another data source or to generate an external URL", configurable by provisioning, the admin page or Explore [60]. Exemplars link metric panels to traces [28].

## 7. Operating Grafana

**Database and HA.** Grafana defaults to an embedded SQLite database; "For high availability, you must use a shared database", MySQL or Postgres [61]. The shared database "tracks session information, so your load balancer won't need to provide session affinity" [61]. Alerting HA needs the separate gossip setup from section 4 [42][61].

**Rendering, backups, upgrades.** Image rendering runs as the separate `grafana/grafana-image-renderer` service reached through `[rendering] server_url`; the old plugin is deprecated and removed in 13 [62][3]. Back up the configuration files, plugin data and the database; for SQLite, "shut down your Grafana service before backing up" [63]. Follow the minor / bi-monthly strategy and back up before every upgrade [2].

**Identity and access, by edition.**

| Capability | OSS | Enterprise | Cloud |
|---|---|---|---|
| Basic roles (Grafana admin, Org admin, Editor, Viewer, None) | yes [66] | yes [66] | yes [66] |
| Fixed and custom RBAC roles | no | yes [66] | yes [66] |
| SAML login | no | yes [64] | yes [64] |
| Team sync from LDAP, OAuth or SAML groups | no | yes [65] | yes [65] |
| Data source permissions and query caching | no | yes [21] | yes [21] |
| KMS-backed database encryption, AES-GCM | no | yes [69] | not reported |

Without data source permissions, "a user with the Viewer role can issue any possible query to a data source, not just queries that exist on dashboards to which they have access" [21].

**Service accounts.** "API keys are deprecated" and service accounts replace them [68]. Service account tokens have no expiry by default; set `token_expiration_day_limit` to bound them [67]. Create several tokens per account to rotate or replace a compromised one [67].

**Secrets.** Data source credentials go in `secureJsonData`, which is encrypted before storage [43][69]. Since v9.0 Grafana uses envelope encryption: data encryption keys wrapped by a key encryption key from `secret_key` or a KMS (Enterprise) [69]. Rotate data keys through `/encryption/rotate-data-keys` [69].

**Plugins.** "If a plugin is unsigned, then Grafana neither loads nor starts it" [70]; signature levels are Private, Community and Commercial [70]. Angular plugins cannot load at all from v12 [5].

**Hardening settings** [71]:

```ini
[security]
cookie_secure = true
cookie_samesite = strict          ; default lax; check OAuth/SAML flows first
content_security_policy = true

[metrics]
basic_auth_username = metrics
basic_auth_password = ${GRAFANA_METRICS_PASSWORD}
```

The metrics setting matters because "By default, metrics from Grafana itself can be accessed without authentication" [71].

**Advisory process, two worked examples.** Grafana publishes CVEs with CVSS scores and fixed versions per supported line [72]. CVE-2026-76154 (High, 7.3, published 2026-09-17): a stored XSS in the Geomap panel's MapLibre base layer lets "a user with the Editor role" run JavaScript in another session, "enabling escalation to Org Admin"; its Fixed Versions field reads "<12.3.0 >=12.4.11 <13.0.0 >=13.0.9 <13.1.0 >=13.1.6 <13.2.0 >=13.2.2", so releases before 12.3.0 are listed as unaffected and the fixes are 12.4.11, 13.0.9, 13.1.6 and 13.2.2 [73]. CVE-2026-13720 (Medium, 5.4, published 2026-09-29): an Editor could set file-provisioning annotations through the dashboard API, after which "administrators can no longer update or delete it through Grafana"; its Fixed Versions field begins "<12.0.0", so releases before 12.0.0 are listed as unaffected, and the fixes are 12.4.12, 13.0.10, 13.1.7 and 13.2.3 [74]. Both start from the Editor role [73][74], and both fixes required the 2026-09 patch releases on every line [1][74].

## Agreed vs folklore (compressed)

| Claim | Verdict |
|---|---|
| "More dashboards means more observability." | Folklore. Grafana's maturity model treats sprawl as the low level and "Actively reducing sprawl" as the high one [13]. |
| "Alert on CPU and every other cause." | Folklore. Alert on symptoms; keep cause signals for diagnosis [13][14][39]. |
| "Label everything in Loki like it is Elasticsearch." | Folklore. Labels should have tens of values; ephemeral IDs go in filters or structured metadata [24][26]. |
| "Dashboard JSON in git is observability as code." | Half true. Version-controlled JSON is only the medium maturity level; high adds consistency by design and no browser editing [13]. |
| "Grafana stores your metrics." | Folklore. Grafana's own database holds "users, dashboards, and other persistent data" [61]; telemetry goes to backends such as Loki and Pyroscope [11][58]. |
| "Mimir 3 needs Kafka." | Overstated in a widely read critique [75]; the 3.0 notes add Kafka-based ingest storage but say "classic architecture is still supported" [76]. |
| "Grafana churns its collectors." | Agreed in substance: Agent EOL 2025-11-01 and Promtail EOL 2026-03-02, both replaced by Alloy [9][10][75]. |

## Synthesis (inferred)

Nothing in this section is cited; it is the author's reading of the evidence above.

**Starter plan for a small team (ordered):**

1. Run Grafana 13.2 with PostgreSQL from day one, even single-node; SQLite blocks the later HA step and complicates backups.
2. Collect with Alloy only. Do not start new Agent or Promtail deployments.
3. Provision data sources and dashboard providers from files in the service repo; set `allowUiUpdates: false` for anything that pages.
4. One RED dashboard per service, one USE dashboard per host class, drill-down links between them, template variables instead of copies.
5. Write SLO recording rules and multi-window burn-rate alerts before any cause alert. Add cause alerts only as tickets.
6. Put `dashboard-linter` and `promtool check rules` plus `promtool test rules` in CI.
7. Service accounts with `token_expiration_day_limit` set; treat Editor as a privileged role given the 2026 advisories.
8. Subscribe to the advisory feed and take every patch release on your line within a week.

**Decision rules:**

| Situation | Choose |
|---|---|
| 1 to 3 engineers, one Grafana | file provisioning in the app repo, Prometheus-format rules checked by promtool |
| Team already runs Terraform | Terraform provider for folders, data sources, alerting and service accounts; dashboards via Foundation SDK output |
| Many dashboard authors who want the UI | Git Sync, kept under 1,000 resources per connection; alerts still via Terraform or files |
| Kubernetes-native platform | Grafana Operator custom resources |
| Panel shows a counter rate | `$__rate_interval`; keep `$__interval` for `*_over_time` bucketing of gauges |
| Existing Jsonnet mixins | keep them building through mixtool; write new dashboards with the Foundation SDK |
| Need SAML, team sync, RBAC, query caching or usage insights | Enterprise or Cloud; OSS cannot provide them |
| Small metric volume, no compliance constraint, nobody to operate Mimir and Loki | Grafana Cloud |
| Data residency, air gap, or a platform team already running Prometheus | self-hosted |

**Application note: a recipe-level observability add-on for `rust-worker`.** The recipe already exposes `jobs_processed_total{status,type}`, `job_duration_seconds`, `queue_depth{channel}`, `active_workers` and `job_retries_total` on port 9090, and its compose service carries `prometheus.scrape=true`, `prometheus.port=9090` and `prometheus.path=/metrics` labels with two replicas. An add-on would ship, next to `docker/compose/service.yml`:

- `observability.yml` compose include with Alloy, Prometheus (remote-write receiver enabled) and Grafana, Grafana reached through the mx router rather than a host port.
- `config.alloy` that discovers containers by the existing labels, so both replicas are scraped without a static target list. It passes `alloy validate` on Alloy v1.20.1, but has not been run against a live Docker socket:

```alloy
discovery.docker "workers" {
  host = "unix:///var/run/docker.sock"
}
discovery.relabel "workers" {
  targets = discovery.docker.workers.targets
  rule {
    source_labels = ["__meta_docker_container_label_prometheus_scrape"]
    regex         = "true"
    action        = "keep"
  }
  rule {
    source_labels = ["__meta_docker_container_label_com_docker_compose_service"]
    target_label  = "job"
  }
}
prometheus.scrape "workers" {
  targets         = discovery.relabel.workers.output
  scrape_interval = "30s"
  forward_to      = [prometheus.remote_write.metrics.receiver]
}
prometheus.remote_write "metrics" {
  endpoint {
    url = "http://prometheus:9090/api/v1/write"
  }
}
```

- `grafana/provisioning/` with the three files from section 5, and the section 4 rules mounted into Prometheus.
- One provisioned RED dashboard: jobs per second by `type` (`rate(...[$__rate_interval])`), failed-job ratio with a 0.001 threshold, p95 of `job_duration_seconds` via `histogram_quantile`, and `queue_depth` as the saturation panel. The boot test provisioned such a four-panel dashboard alongside the section 5 files.
- Alerts: the two burn-rate rules, a ticket for `up == 0` on the worker job, and a ticket (not a page) for `queue_depth` growing for 30 minutes, because backlog is a worker's closest thing to a latency symptom but still a cause signal for the producer.
