---
title: "HTTPS for a Traefik Router: ACME and Let's Encrypt in Production, Trusted Local Certificates for *.localhost Development"
category: infra
languages: [yaml, bash]
complexity: intermediate
use_cases:
  - putting real certificates on an mx-style global Traefik v3 router (ACME resolver, challenge choice, storage, renewal, redirect, TLS options) without hitting Let's Encrypt rate limits
  - choosing between HTTP-01, TLS-ALPN-01 and DNS-01, and configuring DNS-01 through Cloudflare for wildcards and hosts that are not reachable from the internet
  - serving https://<service>.localhost in local development with a certificate that Chrome, Firefox, Safari and curl all accept, and knowing why a "*.localhost" wildcard is rejected by every one of them
  - planning around the 2025-2029 Web PKI changes (OCSP gone, ARI, 160-hour and 45-day profiles, the 200/100/47-day validity schedule, no expiry emails) before they break a renewal
  - deciding between mkcert, Caddy's local CA, step-ca, a real wildcard over DNS-01 and Tailscale certificates for trusted HTTPS off the public internet
summary: "Operating guide, verified against Traefik v3.7.15 source and the Let's Encrypt, CA/Browser Forum, Cloudflare, Chromium and Mozilla primary pages as of 2026-10: how a Traefik ACME resolver really issues, stores and renews (including the renewal tiers, ACME profile support and the still-missing ARI), which challenge fits which situation, how to run DNS-01 through Cloudflare and sit behind its proxy, and the one finding that changes the mx router recipe: a *.localhost wildcard certificate is rejected by Chrome, Firefox and Apple's TLS stack, so local certs must list each hostname or use a three-label wildcard."
provenance: researched
researched: 2026-10-19
sources:
  - https://doc.traefik.io/traefik/reference/install-configuration/tls/certificate-resolvers/acme/
  - https://doc.traefik.io/traefik/reference/install-configuration/entrypoints/
  - https://doc.traefik.io/traefik/reference/routing-configuration/http/tls/tls-certificates/
  - https://doc.traefik.io/traefik/reference/routing-configuration/http/tls/tls-options/
  - https://doc.traefik.io/traefik/reference/routing-configuration/http/tls/overview/
  - https://doc.traefik.io/traefik/reference/routing-configuration/other-providers/docker/
  - https://doc.traefik.io/traefik/reference/install-configuration/observability/healthcheck/
  - https://raw.githubusercontent.com/traefik/traefik/v3.7.15/pkg/provider/acme/provider.go
  - https://raw.githubusercontent.com/traefik/traefik/v3.7.15/pkg/provider/acme/local_store_unix.go
  - https://raw.githubusercontent.com/traefik/traefik/v3.7.15/CHANGELOG.md
  - https://github.com/traefik/traefik/releases
  - https://github.com/traefik/traefik/issues/12461
  - https://github.com/traefik/traefik/pull/12479
  - https://github.com/traefik/traefik/issues/13808
  - https://github.com/traefik/traefik/issues/12946
  - https://community.letsencrypt.org/t/traefik-renewal-for-shortlived-cert-did-not-work/244100
  - https://letsencrypt.org/docs/challenge-types/
  - https://letsencrypt.org/docs/rate-limits/
  - https://letsencrypt.org/docs/staging-environment/
  - https://letsencrypt.org/docs/profiles/
  - https://letsencrypt.org/docs/integration-guide/
  - https://letsencrypt.org/2024/12/05/ending-ocsp
  - https://letsencrypt.org/2025/09/16/ari-rfc
  - https://letsencrypt.org/2024/04/25/guide-to-integrating-ari-into-existing-acme-clients
  - https://letsencrypt.org/2025/01/16/6-day-and-ip-certs/
  - https://letsencrypt.org/2026/01/15/6day-and-ip-general-availability
  - https://community.letsencrypt.org/t/upcoming-changes-to-let-s-encrypt-certificates/243873
  - https://letsencrypt.org/2025/05/14/ending-tls-client-authentication
  - https://letsencrypt.org/2025/06/26/expiration-notification-service-has-ended/
  - https://letsencrypt.org/docs/ct-logs/
  - https://cabforum.org/2025/04/11/ballot-sc081v3-introduce-schedule-of-reducing-validity-and-data-reuse-periods/
  - https://cabforum.org/working-groups/server/baseline-requirements/requirements/
  - https://scotthelme.co.uk/shorter-certificates-are-coming/
  - https://go-acme.github.io/lego/dns/cloudflare/
  - https://raw.githubusercontent.com/go-acme/lego/v5.5.2/CHANGELOG.md
  - https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/
  - https://developers.cloudflare.com/ssl/origin-configuration/origin-ca/
  - https://developers.cloudflare.com/ssl/origin-configuration/authenticated-origin-pull/
  - https://developers.cloudflare.com/ssl/edge-certificates/additional-options/always-use-https/
  - https://community.letsencrypt.org/t/how-does-lets-encrypt-work-when-i-use-cloudflare-proxy-with-ssl/242266
  - https://github.com/FiloSottile/mkcert
  - https://raw.githubusercontent.com/FiloSottile/mkcert/master/cert.go
  - https://github.com/FiloSottile/mkcert/releases
  - https://github.com/FiloSottile/mkcert/issues/232
  - https://github.com/FiloSottile/mkcert/issues/30
  - https://web.dev/articles/how-to-use-local-https
  - https://www.rfc-editor.org/rfc/rfc6761.html
  - https://codereview.chromium.org/1177933002
  - https://chromium.googlesource.com/chromium/src/+/main/net/cert/x509_certificate.cc
  - https://bugzilla.mozilla.org/show_bug.cgi?id=1220810
  - https://github.com/mozilla/gecko-dev/blob/master/security/nss/lib/mozpkix/lib/pkixnames.cpp
  - https://daniel.haxx.se/blog/2021/05/31/curl-localhost-as-a-local-host/
  - https://bugs.webkit.org/show_bug.cgi?id=160504
  - https://caddy.community/t/localhost-tld-in-safari-not-found/33815
  - https://caddyserver.com/docs/automatic-https
  - https://caddyserver.com/docs/caddyfile/directives/tls
  - https://smallstep.com/docs/step-ca/provisioners/
  - https://github.com/jsha/minica
  - https://tailscale.com/kb/1153/enabling-https
  - https://hstspreload.org/
  - https://doc.traefik.io/traefik/reference/routing-configuration/http/middlewares/headers/
  - https://major.io/p/wildcard-letsencrypt-certificates-traefik-cloudflare/
  - https://code.mendhak.com/use-wildcard-certificates-for-internal-infrastructure/
  - https://news.ycombinator.com/item?id=18842218
  - https://news.ycombinator.com/item?id=46279241
  - https://news.ycombinator.com/item?id=33383095
  - https://community.traefik.io/t/support-for-acme-ari-renewal-information/29461
  - mx repo (mech-crate): templates/router/docker-compose.yml, templates/router/config/traefik.yml, templates/router/config/dynamic/middlewares.yml, crates/mx-lib/src/router/mod.rs, docs/router.md, templates/recipes/*/docker/compose/service.yml
  - https://developers.cloudflare.com/fundamentals/api/get-started/create-token/
  - https://cert-manager.io/docs/configuration/acme/dns01/Cloudflare/
  - https://raw.githubusercontent.com/traefik/traefik/v3.7.15/pkg/provider/traefik/internal.go
---

# HTTPS for a Traefik Router: ACME in Production, Trusted Certificates Locally

Companion to `mx-app-playbook.md` section 5 (the router rule) and `mx-cloudflare-deploy.md`. The mx router is one global Traefik v3 container that owns host ports 80 and 443, watches the Docker socket with `exposedByDefault: false`, reads static config from `config/traefik.yml`, hot-reloads `config/dynamic/`, and mounts `letsencrypt/` at `/etc/traefik/acme` [68]. As shipped it has no `certificatesResolvers` block, `ping` sits on the `web` entrypoint, the dashboard API is `insecure: true`, and every recipe label says `entrypoints=web` [68]. Everything below is about turning that into HTTPS, first in production, then locally.

The numbers in this doc were read from the primary pages and from Traefik v3.7.15 source on 2026-10-19; where a claim comes only from a forum thread or an inference it is labelled.

## 0. Decisions in one table

| Situation | Do this | Why |
|---|---|---|
| Public host, port 80 reachable, one hostname per app | Let's Encrypt via `httpChallenge` on the `web` entrypoint, resolver referenced from router labels | Simplest; HTTP-01 "can only be done on port 80" and follows redirects, so the 80 to 443 redirect does not break it [17] |
| Port 80 blocked, 443 open | `tlsChallenge` (TLS-ALPN-01) | "Traefik must be reachable by Let's Encrypt through port 443" [1] |
| Wildcard, many hostnames, or host not reachable from the internet | `dnsChallenge` with a Cloudflare token scoped to `Zone:Read` + `DNS:Edit` | Only DNS-01 issues wildcards and validates names "whose webservers aren't exposed to the public internet" [17][34] |
| Behind Cloudflare's orange cloud | Full (strict) with a Let's Encrypt cert at the origin via DNS-01, or a Cloudflare Origin CA cert | Full (strict) validates the origin certificate; Origin CA certs are only trusted by Cloudflare [36][37] |
| Local `https://<service>.localhost` | mkcert with every hostname listed explicitly, or a three-label wildcard such as `*.dev.localhost`; load through the file provider | A `*.localhost` wildcard is rejected by Chrome, Firefox and Apple's TLS stack (section 6.2, verified) [49][51][44] |
| Team or phones need the same local certs without sharing a root key | A real `*.dev.example.com` wildcard via DNS-01 pointed at 127.0.0.1 or a LAN address | One public cert, no root CA to distribute; the hostname goes into CT logs [63][30] |
| Internal CA with ACME semantics | step-ca ACME provisioner (24 h default leaf lifetime) or Caddy's `tls internal` (12 h leaf) | Both are proper ACME or auto-renewing CAs rather than a static mkcert cert [57][56] |

## 1. The Web PKI in 2026: what changed and what is scheduled

This section exists because several "known facts" about Let's Encrypt stopped being true in 2025 and 2026, and a router built on them will fail silently at renewal time.

**OCSP is gone.** Let's Encrypt announced on 2024-12-05 that Must-Staple requests would fail from January 30, 2025, that OCSP URLs would be dropped from certificates on May 7, 2025 (CRL URLs added before that), and that "On this date we will turn off our OCSP responders" for August 6, 2025. The stated reason is privacy: with OCSP the CA "immediately becomes aware of which website is being visited from that visitor's particular IP address" [22]. Revocation is now CRL only.

**ARI is a standard, and the CA now tells you when to renew.** ACME Renewal Information is RFC 9773 (Let's Encrypt post of 2025-09-16) [23]. A client that implements it reads a `suggestedWindow` per certificate, renews at a random time inside the window, and sets `replaces` on the new order; such renewals "will now be exempt from all rate limits" [24][18]. Let's Encrypt's integration guide asks clients to check ARI "at least twice a day", and as a fallback to renew at two thirds of lifetime, which for 90-day certificates means "renewing 30 days before expiration" [21].

**Profiles replace the one-size certificate.** A client picks a profile per order [20]:

| Profile | Lifetime | Identifiers | Default | Status |
|---|---|---|---|---|
| `classic` | 90 days | DNS | yes | current |
| `tlsserver` | 45 days | DNS | no | current |
| `shortlived` | 160 hours (just over six days) | DNS, IP | no | announced 2025-01-16 to "greatly reduce the potential compromise window" [25]; generally available since 2026-01-15 [26] |
| `tlsclient` | 90 days with the Client Auth EKU | DNS | no | "no longer available" as of July 8, 2026 [20][28] |

Let's Encrypt's own schedule, posted 2025-12-15: in 2026 "you'll be able to opt-in to 45 day certificates for early adopters and testing via the tlsserver profile", in 2027 "we'll lower the default certificate lifetime to 64 days", "and then to 45 in 2028" [27]. The GA post says "Short-lived certificates are opt-in and we have no plan to make them the default at this time" and "Our default certificate lifetimes will be going from 90 days down to 45 days over the next few years" [26]. The classic profile moved to the Generation Y intermediates, which "do not contain the 'TLS Client Authentication' Extended Key Usage", on May 13, 2026 [27].

**The industry-wide ceiling is dropping on fixed dates.** CA/Browser Forum ballot SC-081v3 passed on 2025-04-11 (issuers 25 yes, 0 no, 5 abstain; consumers 4 yes) [31]. The current Baseline Requirements v2.3.1 (dated 2026-10-04) carry the schedule as dated rows: "2029-03-15 6.3.2 Maximum validity period of Subscriber Certificates is 47 days" and "2029-03-15 4.2.1 Domain Name and IP Address validation maximum data reuse period is 10 days" [32]. The intermediate steps are 200 days for certificates issued on or after March 15, 2026 and 100 days from March 15, 2027 [33][32]. Nothing a Let's Encrypt user does today exceeds those caps, but any manually rotated certificate elsewhere in your estate will.

**Client authentication left the public PKI.** Driven by Chrome's root program "June 2026 deadline to split TLS Client and Server Authentication into separate PKIs", the default profile dropped the Client Auth EKU on February 11, 2026 and the `tlsclient` escape hatch closed on July 8, 2026 [28]. If anything (an XMPP server, mTLS between your own services) relied on a Let's Encrypt cert as a client certificate, it needs a private CA, which is what Let's Encrypt itself recommends [28]. The Hacker News discussion of the XMPP write-up shows the breakage was real for federated protocols [65].

**Nobody will email you.** "This service ended on June 4, 2025." Let's Encrypt points at automation and at third-party monitors such as Red Sift Certificates Lite [29].

**Rate limits (page last updated 2026-08-05) [18]:**

| Limit | Value | Override |
|---|---|---|
| New certificates per registered domain | 50 per 7 days (refill 1 per 202 minutes) | on request |
| New certificates per exact set of identifiers | 5 per 7 days (refill 1 per 34 hours) | none |
| Authorization failures per identifier per account | 5 per hour | none |
| New orders per account | 300 per 3 hours | none needed in practice |
| New registrations per IP | 10 per 3 hours | none |

Staging has "significantly higher limits" (30000 duplicate certificates per week, 200 authorization failures per hour) and its roots, named things like "(STAGING) Pretend Pear X1", "are not present in browser/client trust stores" [19].

## 2. How a Traefik v3 ACME resolver actually behaves

Traefik's stable line is v3.7, with v3.7.15 released 2026-10-15; the mx router template pins `traefik:v3.6.1` [11][68]. Everything below was checked against the v3.7 reference pages and the v3.7.15 source.

### 2.1 Minimum configuration

```yaml
# config/traefik.yml (static). A resolver does nothing until a router or an
# entrypoint references it.
entryPoints:
  web:
    address: ":80"
  websecure:
    address: ":443"

certificatesResolvers:
  letsencrypt:
    acme:
      email: ops@example.com
      storage: /etc/traefik/acme/acme.json
      httpChallenge:
        entryPoint: web
  letsencrypt-staging:
    acme:
      email: ops@example.com
      storage: /etc/traefik/acme/acme-staging.json
      caServer: https://acme-staging-v02.api.letsencrypt.org/directory
      httpChallenge:
        entryPoint: web
```

Field facts from the reference page [1]:

- `caServer` defaults to `https://acme-v02.api.letsencrypt.org/directory`; `storage` defaults to `acme.json`; `keyType` defaults to `RSA4096`; `profile` ("Certificate profile to use") defaults to empty, which gives `classic`.
- `httpChallenge.entryPoint` must be reachable on port 80 and `tlsChallenge` means "Traefik must be reachable by Let's Encrypt through port 443".
- `dnsChallenge.provider` names a lego provider; `resolvers` are "DNS servers to resolve the FQDN authority"; `propagation.delayBeforeChecks` delays Traefik's own TXT pre-check and "is Useful if internal networks block external DNS queries"; `propagation.disableChecks` exists, and "disabling checks can prevent the challenge from succeeding".
- Fallback when the CA is unreachable: previously generated ACME certificates, then expired ACME certificates, then provided certificates.
- `acme.profile` and `acme.emailAddresses` were added in v3.4.0 (2025-05-05) [10], so profile selection needs a 3.4 or newer image.

### 2.2 The storage file is guarded in code

On non-Windows builds Traefik refuses to start a resolver whose storage file is readable by group or others: `if fi.Mode().Perm()&0o077 != 0 { return false, fmt.Errorf("permissions %o for %s are too open, please use 600", ...) }`, and it creates a missing file with `0o600` [9]. `mx router install` sets `0o600` on `letsencrypt/acme.json` after copying the template [68]. Two consequences: a bind mount that does not preserve modes (some Windows and macOS shared folders) fails with exactly that message, and a `cp` or restore that leaves the file at 644 does the same.

The reference page is explicit that the file "cannot be shared across multiple instances of Traefik" because "there is no way to ensure that the correct instance of Traefik receives the challenge request"; it points to Traefik Enterprise's distributed Let's Encrypt or to cert-manager on Kubernetes [1]. For one workstation router this is irrelevant; for a multi-node setup it is the first design constraint.

### 2.3 Renewal tiers, verbatim from source

Traefik does not read ARI. It derives a renewal window and a check interval from `certificatesDuration` (default 2160 hours, "exclusively used to determine renewal dates") [1]. The v3.7.15 table in `getCertificateRenewDurations` [8]:

| `certificatesDuration` | Renew when less than | Check every |
|---|---|---|
| 1 year or more | 4 months remain | 1 week |
| 90 days or more | 30 days | 1 day |
| 30 days or more | 10 days | 12 hours |
| 6 days or more | 2 days | 2 hours |
| 24 hours or more | 6 hours | 10 minutes |
| less than 24 hours | 20 minutes | 1 minute |

The six-day row was added by PR #12479 (merged 2026-01-21) in response to issue #12461, which showed that a `shortlived` certificate declared as `certificatesDuration: 160` previously fell into the 24-hour tier and got a six-hour window with a ten-minute retry, a combination the reporter noted "could run into rate limits very quickly if there's a configuration problem" [13][12]. A value below one hour is rejected outright [8]. "Certificates that are no longer used may still be renewed, as Traefik does not currently check if the certificate is being used before renewing" [1], which matters for the duplicate-certificate limit when hostnames come and go.

**ARI status.** The v3.7.15 ACME package contains no renewal-information call [8]; issue #13808 "Add support for certificate renewal tracking through ARI" and PR #13814 (both opened 2026-08-31) are open [14]. The lego library Traefik embeds has carried ARI for years ("Attempt to check ARI unless explicitly disabled" in its changelog) [35], so the gap is in Traefik's integration, as the community thread also concluded [67]. Practical reading: Traefik renewals get no rate-limit exemption and no early-renewal signal from the CA until that lands.

**Short-lived profile caveats.** With `profile: shortlived` and `certificatesDuration: 160` the 3.7 line renews with two days left, checking every two hours [8]. Two things to know before enabling it: a December 2025 report on 3.6.5 found renewals silently dropped the profile and came back as 90-day classic certificates [16]; the v3.7.15 source now passes `Profile: p.Profile` on both obtain and renew [8], so this is fixed in the current line, but pin a 3.7 image. And issue #12946 (open) says the v3.7 dashboard marks every short-lived certificate as expiring soon because its warning threshold is fixed [15].

### 2.4 Where the resolver is referenced decides how many certificates you get

Two ways to turn a resolver on [2][6]:

```yaml
# (a) per router, in the app's compose labels
- traefik.http.routers.${COMPOSE_PROJECT_NAME}-api.entrypoints=websecure
- traefik.http.routers.${COMPOSE_PROJECT_NAME}-api.tls.certresolver=letsencrypt
# optional explicit names instead of deriving them from Host():
- traefik.http.routers.${COMPOSE_PROJECT_NAME}-api.tls.domains[0].main=example.org
- traefik.http.routers.${COMPOSE_PROJECT_NAME}-api.tls.domains[0].sans=api.example.org,admin.example.org
```

```yaml
# (b) once, on the entrypoint (static config)
entryPoints:
  websecure:
    address: ":443"
    http:
      tls:
        certResolver: letsencrypt
```

Entrypoint TLS means "Enable TLS on every router attached to the entryPoint. If no certificate are set, a default self-signed certificate is generated by Traefik" and `certResolver` there applies "to all routers on this entryPoint" [2]. Domains are "automatically extracted from router rules" unless `tls.domains` is set, which "takes precedence" [5]. So option (b) orders one certificate per `Host()` rule of every container that opts in, including the stray test stack someone brings up with a public-looking hostname. The no-merge rule is the trap in the other direction: "Defining a router `tls` section (even empty, or with any field such as `certResolver`) replaces the entrypoint TLS configuration for that router (no merge)" [2], so a router that sets `tls.options` but forgets `certresolver` loses the entrypoint's resolver and gets the self-signed default.

For a wildcard covering a whole dev domain, option (b) plus explicit domains is the one-line form practitioners use with DNS-01: `--entrypoints.websecure.http.tls.domains[0].main=home.example.com` and `...domains[0].sans=*.home.example.com` [62].

### 2.5 Redirect, TLS options, default certificate

**HTTP to HTTPS.** On the entrypoint: `http.redirections.entryPoint.to: websecure`; `scheme` defaults to `https`, `permanent` to `true`, and the redirect router's priority to `MaxInt-1` [2]. Let's Encrypt's HTTP-01 "follows redirects, up to 10 redirects deep", only to `http:` or `https:` on ports 80 or 443, and "When redirected to an HTTPS URL, it does not validate certificates" [17], so the redirect and HTTP-01 coexist. Two source-level details matter for the mx template. First, the redirect is an internal router whose rule is ``HostRegexp(`^.+$`)`` and whose priority is the entrypoint's configured `priority` (default `MaxInt-1`); when the entrypoint's `allowACMEByPass` option is set the rule becomes ``HostRegexp(`^.+$`) && !PathPrefix(`/.well-known/acme-challenge/`)``, so the challenge path is never redirected [71][2]. Second, the `ping` router that `ping.entryPoint` creates has `Priority: math.MaxInt` [71], one above the redirect, which is why `ping.entryPoint: web` plus the compose `healthcheck` that runs `traefik healthcheck --ping` [68] should keep answering 200 after the redirect is added. Traefik's documented default for ping is its own `traefik` entrypoint on port 8080 [7]; this doc could not run the container to confirm the priority reasoning (section 7.3).

**TLS options live in the file provider only.** "TLS options are not supported by label or tag-based providers"; define them in `config/dynamic/` and reference them from labels with the provider suffix, `tls.options=myoptions@file` [4]. The option named `default` applies to every TLS router that does not name one. `sniStrict: true` means Traefik "won't allow connections from clients that do not specify a server_name extension or don't match any of the configured certificates"; the ALPN default is `h2, http/1.1, acme-tls/1` (keep `acme-tls/1` if you use `tlsChallenge`); TLS 1.3 cipher suites "are not configurable" [4]. When two routers on the same host and entrypoint disagree on options, both fall back to `default` [4].

```yaml
# config/dynamic/tls.yml
tls:
  options:
    default:
      minVersion: VersionTLS12
      sniStrict: true
```

**Default certificate.** Certificate selection is by SNI, and HTTP router rules "are evaluated after TLS has been established, so they do not influence certificate selection" [5]. With no match Traefik serves its generated self-signed certificate unless `tls.stores.default.defaultCertificate` (a file) or `defaultGeneratedCert` (resolver plus `domain.main`/`sans`) is set; "Any store definition other than the default one (named `default`) will be ignored" [3]. User-supplied certificates are declared only in the file provider's `tls.certificates` list with `certFile` and `keyFile` [3].

**HSTS.** The headers middleware exposes `stsSeconds`, `stsIncludeSubdomains`, `stsPreload` and `forceSTSHeader` [61]. Preloading requires `max-age` of at least 31536000 with `includeSubDomains` and `preload`, "applies to all subdomains, including internal subdomains that are not publicly accessible", and "cannot easily be undone" [60]; do not preload a domain whose dev subdomains still answer on plain HTTP.

## 3. Choosing the challenge

| | HTTP-01 | TLS-ALPN-01 | DNS-01 |
|---|---|---|---|
| Port that must be reachable from the internet | 80 ("can only be done on port 80") | 443 | none |
| Wildcards | no ("cannot be used to issue wildcard certificates") | no | yes |
| IP address certificates | yes (`shortlived` profile only) | yes | no ("cannot be used to validate IP Addresses") |
| Hosts not exposed to the internet | no | no | yes |
| Secret that lives on the router | none | none | a DNS API token ("Keeping API credentials on your web server is risky") |
| Multi-server | file must be on all servers | all servers | fine |

All rows from Let's Encrypt's challenge page [17] except the IP-address column, which combines it with the profiles page [20]. Let's Encrypt's own advice: "If you're unsure, go with your client's defaults or with HTTP-01" [17]. Validation can come from multiple network vantage points, so a DNS record that has propagated to one resolver but not others fails intermittently [17].

## 4. DNS-01 through Cloudflare

```yaml
certificatesResolvers:
  letsencrypt-dns:
    acme:
      email: ops@example.com
      storage: /etc/traefik/acme/acme-dns.json
      dnsChallenge:
        provider: cloudflare
        resolvers:
          - "1.1.1.1:53"
          - "1.0.0.1:53"
```

With the token in the router container's environment as `CF_DNS_API_TOKEN` (alias `CLOUDFLARE_DNS_API_TOKEN`). lego's provider page lists the minimum permissions as "Zone / Zone / Read" and "Zone / DNS / Edit", allows a split into a zone-read token and a DNS-edit token scoped to specific zones for least privilege, and defaults polling to 2 seconds, propagation timeout to 120 seconds and TTL to 120 seconds [34]. cert-manager documents the same two scopes [70]. Cloudflare's token UI has an "Edit zone DNS" template and shows the secret once [69].

Operational notes:

- `resolvers` should name public resolvers when the host's own DNS is split-horizon or filtered; `propagation.delayBeforeChecks` is the knob when internal networks block external DNS [1].
- Nothing about DNS-01 requires an A record that points at the router. That is how a LAN-only Traefik gets a publicly trusted `*.home.example.com`: public DNS holds only the `_acme-challenge` TXT records, private DNS points the names at the LAN address [62][63].
- The token is the renewal's single point of failure. Rotating or re-scoping it does not fail until the next renewal window, weeks later [62].
- Every public certificate is logged: "Let's Encrypt submits all certificates we issue to CT logs" [30]. Individual hostnames in SANs are therefore public; a wildcard discloses only the parent domain [63]. Tailscale states the same caveat for its certificates: the ledger "includes the fully qualified domain name of your devices" [59].

## 5. Behind Cloudflare's proxy

Cloudflare's encryption modes decide what the origin must present [36]:

| Mode | Origin certificate |
|---|---|
| Off, Flexible | none (Flexible is plaintext to the origin) |
| Full | any, including self-signed; no validation |
| Full (strict) | must validate: "issued by a public CA like Let's Encrypt or by Cloudflare Origin CA" |
| Strict (SSL-Only Origin Pull) | as Full (strict), and Cloudflare always connects over HTTPS |

Cloudflare "strongly recommends using Full or Full (strict) modes to prevent malicious connections to your origin" [36]. Two ways to satisfy Full (strict) at a Traefik origin:

1. **Origin CA certificate** loaded as a static `tls.certificates` entry in `config/dynamic/`. It is long-lived and free, but "Site visitors may see untrusted certificate errors if you pause Cloudflare or disable proxying on subdomains that use Cloudflare origin CA certificates" because "These certificates only encrypt traffic between Cloudflare and your origin server, not traffic from client browsers to your origin" [37].
2. **Let's Encrypt via DNS-01** (section 4), which keeps working if you ever grey-cloud the record.

Authenticated Origin Pulls adds client-certificate authentication of Cloudflare to the origin and is "not compatible with origins connected through Cloudflare Tunnel" [38]; Traefik's `clientAuth` TLS option can enforce it.

HTTP-01 through the proxy is a MEDIUM-confidence area. Cloudflare's "Always Use HTTPS" page says only that the feature "does not impact certificate validation. If you use HTTP DCV, you can still enable Always Use HTTPS", which is about Cloudflare's own edge certificates, not an origin's ACME client [39]. The Let's Encrypt community's reading is that validation passes through the orange cloud because Let's Encrypt follows the HTTPS redirect [17][40], and that what breaks it is an edge feature answering instead of the origin (a managed challenge, Bot Fight Mode, a WAF or redirect rule on the path) [40]. If you must use HTTP-01 behind the proxy, exempt `/.well-known/acme-challenge/` from such rules; otherwise use DNS-01 and avoid the question.

## 6. Local HTTPS for `<service>.localhost`

### 6.1 How the names resolve without DNS

RFC 6761 section 6.3 reserves "The domain 'localhost.' and any names falling within '.localhost.'" and says name resolution APIs "SHOULD always return the IP loopback address for address queries" and "SHOULD NOT send queries for localhost names to their configured caching DNS server(s)" [47]. Who implements it:

| Client | Behaviour | Since |
|---|---|---|
| Chromium | resolves names ending in `.localhost` to loopback without DNS, honouring `/etc/hosts` first | 2015 ("Resolve RFC 6761 localhost names to loopback") [48] |
| Firefox | `localhost` and `*.localhost` hard-coded to 127.0.0.1 and ::1 | Firefox 84, bug 1220810, pref `network.dns.native-is-localhost` [50] |
| curl | only the literal `localhost` is hard-coded; subdomains go to the OS resolver | 7.78.0 [52] |
| Safari / macOS resolver | WebKit bug 160504 closed MOVED because "this was implemented in OS frameworks below WebKit"; works on macOS 26, failed on 15.7 | macOS 26 Tahoe [53][54] |

Observed on the authoring machine (macOS 26, Darwin 25.6): `ping foo.localhost` and `ping a.dev.localhost` both answered from 127.0.0.1 with no `/etc/hosts` entry, so curl and Safari get loopback through the OS resolver there. On macOS 15 and earlier a July 2026 thread shows Safari failing on a `.localhost` name while Chrome, Firefox and curl worked, fixed by an `/etc/hosts` line [54].

### 6.2 The wildcard trap: `*.localhost` does not match `app.localhost` anywhere

`docs/router.md` in the mx repo tells you to run `mkcert "*.localhost" localhost 127.0.0.1` and load the result as the router's certificate [68]. That certificate is accepted for `https://localhost` and rejected for `https://api.localhost` by every mainstream TLS stack:

- **Chromium.** `VerifyHostname` computes the registry-controlled part of the reference name with `INCLUDE_UNKNOWN_REGISTRIES`, and the comment explains: "unknown top-level domains (such as 'intranet' domains or new TLDs/gTLDs not yet added to the registry controlled domain dataset) are also implicitly prevented. Because |reference_domain| must contain at least one name component that is not registry controlled, this ensures that all reference domains contain at least three domain components when using wildcards" [49]. `app.localhost` has two components; wildcard matching is disabled for it.
- **Firefox (mozilla::pkix).** "Like NSS, require at least two labels to follow the wildcard label." followed by `if (labelCount < 3) { return false; }` [51]. `*.localhost` has one label after the wildcard.
- **Apple's TLS stack.** Verified empirically below through curl built against Secure Transport.
- **mkcert itself warns.** Its `printHosts` matches `^\*\.[0-9a-z_-]+$` and prints "Warning: many browsers don't support second-level wildcards like %q" [42]; the author's answer to a user whose `*.localhost` cert showed as invalid in Chrome was that browsers do not support first-level wildcards and that "mkcert prints an error for those, but can't do anything else" [44]; on the equivalent `*.test` report: "It will work if you use a name like *.wild.test" [45].

**Empirical check, 2026-10-19.** A throwaway CA and a leaf with `subjectAltName=DNS:*.localhost,DNS:localhost` were served from 127.0.0.1:8443 by a Python TLS server; a second leaf with `DNS:*.dev.localhost` on port 8444.

| Client | URL | Result |
|---|---|---|
| curl 8.7.1 (SecureTransport, macOS), `--cacert` the test CA | `https://localhost:8443/` | `200` |
| same | `https://app.localhost:8443/` | `curl: (60) SSL: no alternative certificate subject name matches target host name 'app.localhost'` |
| same | `https://app.dev.localhost:8444/` | `200` |
| Chrome 155.0.8059.84 headless with `--log-net-log` | `https://localhost:8443/` | `cert_status` 4 (authority invalid only, the test CA is untrusted) |
| same | `https://app.localhost:8443/` | `cert_status` 5 (authority invalid plus common-name invalid) |
| same | `https://app.dev.localhost:8444/` | `cert_status` 4 |

The Chrome run does not install the test CA, so the authority bit is expected in every row; the name-mismatch bit appears only for the two-label wildcard, which is the point. (Chromium's `cert_status` bits: common name invalid is bit 0, authority invalid is bit 2, per `net/cert/cert_status_flags_list.h`.) A Firefox headless run was attempted and did not produce output in the time allowed; the pkix source is the evidence for Firefox.

**Fixes, in order of preference:**

```bash
# 1. List every hostname. mkcert names the files after the first host.
mkcert -install
cd ~/.mech-crate/router/letsencrypt
mkcert api.localhost admin.localhost app.localhost localhost 127.0.0.1 ::1
#  -> api.localhost+5.pem, api.localhost+5-key.pem

# 2. Or adopt a three-label scheme once and wildcard it.
mkcert "*.dev.localhost" localhost 127.0.0.1 ::1
#  -> _wildcard.dev.localhost+3.pem (matches api.dev.localhost, not api.localhost)
```

Option 1 means re-running mkcert when an app is added (Traefik hot-reloads the file provider, so no restart). Option 2 changes every recipe's default hostname from `<svc>.localhost` to `<svc>.dev.localhost`, which the `_ROUTER_HOST` override already allows per stack without editing shipped files [68]. Either way the dynamic file is:

```yaml
# ~/.mech-crate/router/config/dynamic/certs.yml
tls:
  certificates:
    - certFile: /etc/traefik/acme/api.localhost+5.pem
      keyFile: /etc/traefik/acme/api.localhost+5-key.pem
```

and the app labels switch to `entrypoints=websecure` plus `tls=true` [6][68]. X.509 wildcards "only go one level deep", as mkcert also reminds you, so `*.dev.localhost` does not cover `a.b.dev.localhost` [42].

### 6.3 mkcert mechanics worth knowing

mkcert's last release is v1.4.4 (2024-04-26) and the repository's last commit is from 2024; it is finished software, not abandoned, but it will not learn new trust stores [43]. `mkcert -install` writes the local root into the system store, Firefox (NSS, which needs `certutil` on macOS and Linux), Chrome, and Java when `JAVA_HOME` is set; `TRUST_STORES` restricts that to a comma-separated subset of `system`, `java`, `nss`; `CAROOT` relocates the CA directory; `mkcert -CAROOT` prints it [41]. The README's warning is unambiguous: "the `rootCA-key.pem` file that mkcert automatically generates gives complete power to intercept secure requests from your machine. Do not share it." and "mkcert is meant for development purposes, not production, so it should not be used on end users' machines" [41]. Google's local-HTTPS guide adds that team members "should install and run mkcert separately (not store and share the CA and certificate)" [46].

To trust the root somewhere else, copy `rootCA.pem` (never the key) and either set `CAROOT` and run `mkcert -install` there, or drop it into that system's CA bundle (`/usr/local/share/ca-certificates/<name>.crt` plus `update-ca-certificates` on Debian-family images; Node reads `NODE_EXTRA_CA_CERTS`) [41]. The router container itself needs no trust at all; it only presents the certificate.

### 6.4 Alternatives to a static mkcert certificate

- **Caddy's local CA.** Caddy treats `localhost`, names under `.localhost`, `.local`, `.internal` and `.home.arpa`, and IP addresses as internal and issues locally trusted certificates for them, installing a root named "Caddy Local Authority" and prompting for a password if it lacks permission [55]. With `tls internal` the leaf lifetime defaults to `12h`, the root to `3600d`, the intermediate to `7d`, and signing leaves with the root "is NOT recommended" [56]. Public issuance defaults to Let's Encrypt with ZeroSSL as fallback [55]. By 2022 the practitioner thread on mkcert already had people saying they "just bring up a local caddy instance" instead [66].
- **step-ca.** An ACME provisioner supporting `http-01`, `dns-01`, `tls-alpn-01` and `device-attest-01` (the last off by default), 24-hour default leaf lifetime, usable by Traefik by pointing `caServer` at `https://ca.internal/acme/<provisioner>/directory` [57]. This is the only option here that gives Traefik its normal ACME renewal loop against a private CA.
- **minica.** "a simple CA intended for use in situations where the CA operator also operates each host where a certificate will be used"; wildcards and IPs supported [58].
- **A real wildcard over DNS-01.** `*.dev.example.com` resolving to 127.0.0.1 or a LAN address, issued by Let's Encrypt through section 4. No root to distribute, works on phones, and was already the preferred pattern in the 2019 mkcert discussion ("you don't want HTTPS for localhost, you want HTTPS for a server in the local network") [64]. The hostname is public via CT [30].
- **Tailscale.** `tailscale cert` fetches a Let's Encrypt certificate for `<machine>.<tailnet>.ts.net` through a DNS-01 record Tailscale publishes; it needs MagicDNS and the HTTPS toggle, lasts 90 days, and renewal is yours unless an integration such as Caddy does it [59].

## 7. Applying this to the mx router

### 7.1 Local, today

1. `mkcert -install` once per workstation.
2. Generate one certificate listing every `.localhost` hostname the router serves (section 6.2, option 1) into `~/.mech-crate/router/letsencrypt/`, which is already mounted at `/etc/traefik/acme` [68].
3. Add `config/dynamic/certs.yml` and `config/dynamic/tls.yml` (sections 6.2 and 2.5). Both hot-reload.
4. In each app, set `entrypoints=websecure` and `tls=true`; keep `web` only if you add the redirect (7.3).
5. The devloop URL-discovery rule already infers `https` from a sibling `tls=true` label, so nothing else changes for agents [68].

### 7.2 Production

Add the resolver pair from section 2.1 (production and staging) to `config/traefik.yml`, pick the challenge with section 3, and reference `letsencrypt-staging` from labels on the first deploy. Staging issuance proves the challenge path end to end without consuming the five-per-week duplicate budget [18][19]; swap the label to `letsencrypt` once the staging certificate appears in the dashboard. Keep `acme.json` on a persistent, mode-600 path; recreating the container with an empty file re-orders every certificate [1][9]. Pin at least `traefik:v3.7` if you intend to set `profile` [10][16].

### 7.3 Things to test rather than assume

- **Ping behind the redirect.** The template's `ping.entryPoint: web` and the compose healthcheck share the entrypoint that would carry the HTTP to HTTPS redirect [68][2]. By source, the ping router outranks the redirect router (`math.MaxInt` versus `MaxInt-1`) [71], but Docker was unavailable on the authoring machine, so this was reasoned, not run. After enabling the redirect, run `docker exec mx-router traefik healthcheck --ping`; if it fails, give ping its own entrypoint, which is Traefik's documented default [7].
- **Dashboard exposure.** `api.insecure: true` publishes the dashboard on the mapped port over plain HTTP [68]; it is fine on a workstation and must not survive to a public host.
- **Browser caches.** After replacing a local certificate, Chrome may keep an HSTS or cert state for `.localhost`; a fresh profile is the quickest check.

## Agreed vs folklore (compressed)

| Claim heard in the wild | Status | Evidence |
|---|---|---|
| "One `*.localhost` cert covers all my local apps" | false | Chromium and pkix source, mkcert author, empirical test [49][51][44] |
| "Let's Encrypt emails you before expiry" | false since June 4, 2025 | [29] |
| "OCSP stapling makes Let's Encrypt revocation work" | false; responders off since August 6, 2025 | [22] |
| "Traefik renews on ARI" | false as of v3.7.15; issue open | [8][14] |
| "Traefik can't do ACME profiles" | false since v3.4.0 | [10][1] |
| "Short-lived certs are the Let's Encrypt default now" | false; opt-in `shortlived`, default still `classic` 90 days | [20][26] |
| "Wildcards need DNS-01" | true | [17] |
| "A Cloudflare Origin CA cert is a normal cert" | false; only Cloudflare trusts it | [37] |
| "HTTP-01 breaks behind the orange cloud" | not inherently; edge rules that intercept the path do | [17][40], MEDIUM |
| "acme.json must be 600" | true on Unix builds, enforced in code | [9] |
| "Firefox needs `/etc/hosts` for `foo.localhost`" | false since Firefox 84 | [50] |
| "Safari needs `/etc/hosts` for `foo.localhost`" | true on macOS 15 and earlier, false on macOS 26 | [53][54] |

## Synthesis (inferred)

**Decision ladder for a one-router shop.** Start with HTTP-01 on `web` and router-level `certresolver` labels; move to DNS-01 the day you need a wildcard, a LAN-only host, or you put Cloudflare in front; never use entrypoint-level `certResolver` on a router that also serves scratch stacks. Keep `profile` unset until Traefik ships ARI; a 160-hour certificate on a renewer with fixed windows and no CA feedback buys you a smaller compromise window at the price of a much smaller margin for a broken token or an outage, and Let's Encrypt's own default stays at 90 days through 2026.

**The hostname convention is the real decision.** The two-label `<svc>.localhost` scheme is pleasant to type and resolves everywhere modern, but it forces per-hostname SANs because no browser will match a wildcard against it. The three-label `<svc>.dev.localhost` scheme costs one more label and gets a single wildcard for every present and future app. For a framework that scaffolds hostnames, the second is the better default; for an existing fleet, re-running mkcert with the full list is the cheaper fix. Either is a template change to `docs/router.md`, whose current recipe produces a certificate that fails in every browser.

**What a renewal-safe router looks like.** One persistent, mode-600 storage file per resolver; a staging resolver that stays in the config forever so it can be swapped in during incidents; `resolvers` pointed at public DNS for DNS-01; an external expiry monitor because nobody emails and Traefik's dashboard warning threshold is fixed; and a calendar note for March 15, 2027, when the 100-day ceiling arrives for anything not issued through ACME.

**Why mkcert is still the right local tool despite its age.** Nothing in the browsers' trust-store handling changed in a way that breaks it, its root is per-machine by design, and the alternative that is also a renewing CA (step-ca) is more infrastructure than a workstation router warrants. The thing that aged is not mkcert but the folk recipe around it.

**What would change this doc.** Traefik merging ARI (issue #13808) changes section 2.3 and the profile advice; Let's Encrypt moving the default lifetime (planned 64 days in 2027) changes the renewal windows worth configuring; Apple or Chromium changing wildcard policy for unknown TLDs would retire section 6.2, which is unlikely given the stated rationale; a `templates/router` change that adds a resolver block retires section 7.

## Sources

1. Traefik, ACME certificate resolvers reference (v3.7): https://doc.traefik.io/traefik/reference/install-configuration/tls/certificate-resolvers/acme/
2. Traefik, EntryPoints reference (v3.7): https://doc.traefik.io/traefik/reference/install-configuration/entrypoints/
3. Traefik, TLS certificates reference (v3.7): https://doc.traefik.io/traefik/reference/routing-configuration/http/tls/tls-certificates/
4. Traefik, TLS options reference (v3.7): https://doc.traefik.io/traefik/reference/routing-configuration/http/tls/tls-options/
5. Traefik, HTTP TLS overview (v3.7): https://doc.traefik.io/traefik/reference/routing-configuration/http/tls/overview/
6. Traefik, Docker routing labels (v3.7): https://doc.traefik.io/traefik/reference/routing-configuration/other-providers/docker/
7. Traefik, Health check and ping (v3.7): https://doc.traefik.io/traefik/reference/install-configuration/observability/healthcheck/
8. Traefik source v3.7.15, `pkg/provider/acme/provider.go` (`getCertificateRenewDurations`, `ObtainRequest`/`RenewOptions` with `Profile`, no renewal-information call): https://raw.githubusercontent.com/traefik/traefik/v3.7.15/pkg/provider/acme/provider.go
9. Traefik source v3.7.15, `pkg/provider/acme/local_store_unix.go` (mode check and 0o600 creation): https://raw.githubusercontent.com/traefik/traefik/v3.7.15/pkg/provider/acme/local_store_unix.go
10. Traefik CHANGELOG at v3.7.15 (`acme.profile` under v3.4.0, 2025-05-05): https://raw.githubusercontent.com/traefik/traefik/v3.7.15/CHANGELOG.md
11. Traefik releases (v3.7.15, 2026-10-15) and Docker Hub `library/traefik` tags: https://github.com/traefik/traefik/releases
12. Traefik issue #12461, "Provide better renew period/interval for new LetsEncrypt shortlived (160h) certs" (2025-12-24): https://github.com/traefik/traefik/issues/12461
13. Traefik PR #12479, "Alter TLS renewal period" (merged 2026-01-21): https://github.com/traefik/traefik/pull/12479
14. Traefik issue #13808, "Add support for certificate renewal tracking through ARI" (2026-08-31), and PR #13814: https://github.com/traefik/traefik/issues/13808
15. Traefik issue #12946, "v3.7.x: all shortlived certs are marked with warning in new certificate dashboard" (2026-04-07): https://github.com/traefik/traefik/issues/12946
16. Let's Encrypt community, "Traefik renewal for shortlived cert did not work" (2025-12): https://community.letsencrypt.org/t/traefik-renewal-for-shortlived-cert-did-not-work/244100
17. Let's Encrypt, Challenge Types: https://letsencrypt.org/docs/challenge-types/
18. Let's Encrypt, Rate Limits (last updated 2026-08-05): https://letsencrypt.org/docs/rate-limits/
19. Let's Encrypt, Staging Environment: https://letsencrypt.org/docs/staging-environment/
20. Let's Encrypt, Certificate Profiles (last updated 2026-10-04): https://letsencrypt.org/docs/profiles/
21. Let's Encrypt, Integration Guide: https://letsencrypt.org/docs/integration-guide/
22. Let's Encrypt, "Ending OCSP Support in 2025" (2024-12-05): https://letsencrypt.org/2024/12/05/ending-ocsp
23. Let's Encrypt, "ACME Renewal Information (ARI) is now an RFC" (2025-09-16): https://letsencrypt.org/2025/09/16/ari-rfc
24. Let's Encrypt, "Guide to integrating ARI into existing ACME clients" (2024-04-25): https://letsencrypt.org/2024/04/25/guide-to-integrating-ari-into-existing-acme-clients
25. Let's Encrypt, "Six-day and IP address certificates" (2025-01-16): https://letsencrypt.org/2025/01/16/6-day-and-ip-certs/
26. Let's Encrypt, "Six-day and IP address certificates are now generally available" (2026-01-15): https://letsencrypt.org/2026/01/15/6day-and-ip-general-availability
27. Let's Encrypt community, "Upcoming Changes to Let's Encrypt Certificates" (2025-12-15, updated 2026-01-07): https://community.letsencrypt.org/t/upcoming-changes-to-let-s-encrypt-certificates/243873
28. Let's Encrypt, "Ending TLS Client Authentication" (2025-05-14, updated 2026-03-16): https://letsencrypt.org/2025/05/14/ending-tls-client-authentication
29. Let's Encrypt, "Expiration Notification Service Has Ended" (2025-06-26): https://letsencrypt.org/2025/06/26/expiration-notification-service-has-ended/
30. Let's Encrypt, Certificate Transparency logs: https://letsencrypt.org/docs/ct-logs/
31. CA/Browser Forum, Ballot SC-081v3 (2025-04-11): https://cabforum.org/2025/04/11/ballot-sc081v3-introduce-schedule-of-reducing-validity-and-data-reuse-periods/
32. CA/Browser Forum, Baseline Requirements v2.3.1 (2026-10-04), dated-change table: https://cabforum.org/working-groups/server/baseline-requirements/requirements/
33. Scott Helme, "Shorter certificates are coming" (2025-04-22): https://scotthelme.co.uk/shorter-certificates-are-coming/
34. lego, Cloudflare DNS provider: https://go-acme.github.io/lego/dns/cloudflare/
35. lego CHANGELOG at v5.5.2 (ARI entries): https://raw.githubusercontent.com/go-acme/lego/v5.5.2/CHANGELOG.md
36. Cloudflare, Encryption modes: https://developers.cloudflare.com/ssl/origin-configuration/ssl-modes/
37. Cloudflare, Origin CA certificates: https://developers.cloudflare.com/ssl/origin-configuration/origin-ca/
38. Cloudflare, Authenticated Origin Pulls: https://developers.cloudflare.com/ssl/origin-configuration/authenticated-origin-pull/
39. Cloudflare, Always Use HTTPS: https://developers.cloudflare.com/ssl/edge-certificates/additional-options/always-use-https/
40. Let's Encrypt community, "How does Let's Encrypt work when I use Cloudflare proxy with SSL": https://community.letsencrypt.org/t/how-does-lets-encrypt-work-when-i-use-cloudflare-proxy-with-ssl/242266
41. mkcert README: https://github.com/FiloSottile/mkcert
42. mkcert source, `cert.go` (`printHosts` wildcard warnings, file naming): https://raw.githubusercontent.com/FiloSottile/mkcert/master/cert.go
43. mkcert releases (v1.4.4, 2024-04-26): https://github.com/FiloSottile/mkcert/releases
44. mkcert issue #232, wildcard `*.localhost` shown invalid in Chrome: https://github.com/FiloSottile/mkcert/issues/232
45. mkcert issue #30, wildcard on a second-level domain: https://github.com/FiloSottile/mkcert/issues/30
46. web.dev, "How to use HTTPS for local development": https://web.dev/articles/how-to-use-local-https
47. RFC 6761, Special-Use Domain Names, section 6.3: https://www.rfc-editor.org/rfc/rfc6761.html
48. Chromium code review 1177933002, "Resolve RFC 6761 localhost names to loopback" (2015): https://codereview.chromium.org/1177933002
49. Chromium source, `net/cert/x509_certificate.cc`, `VerifyHostname`: https://chromium.googlesource.com/chromium/src/+/main/net/cert/x509_certificate.cc
50. Mozilla bug 1220810, "Consider hardcoding localhost names to the loopback address" (Firefox 84): https://bugzilla.mozilla.org/show_bug.cgi?id=1220810
51. mozilla::pkix, `pkixnames.cpp` (wildcard label-count rule): https://github.com/mozilla/gecko-dev/blob/master/security/nss/lib/mozpkix/lib/pkixnames.cpp
52. Daniel Stenberg, "curl localhost as a local host" (2021-05-31): https://daniel.haxx.se/blog/2021/05/31/curl-localhost-as-a-local-host/
53. WebKit bug 160504, "Localhost subdomains don't work" (RESOLVED MOVED, macOS 26): https://bugs.webkit.org/show_bug.cgi?id=160504
54. Caddy community, ".localhost TLD in Safari not found" (2026-07): https://caddy.community/t/localhost-tld-in-safari-not-found/33815
55. Caddy, Automatic HTTPS: https://caddyserver.com/docs/automatic-https
56. Caddy, `tls` directive (internal issuer): https://caddyserver.com/docs/caddyfile/directives/tls
57. Smallstep, step-ca provisioners (ACME): https://smallstep.com/docs/step-ca/provisioners/
58. minica: https://github.com/jsha/minica
59. Tailscale, Enabling HTTPS: https://tailscale.com/kb/1153/enabling-https
60. HSTS Preload List submission: https://hstspreload.org/
61. Traefik, Headers middleware (HSTS fields): https://doc.traefik.io/traefik/reference/routing-configuration/http/middlewares/headers/
62. Major Hayden, "Wildcard LetsEncrypt certificates with Traefik and Cloudflare" (2021-08-16): https://major.io/p/wildcard-letsencrypt-certificates-traefik-cloudflare/
63. mendhak, "Use wildcard certificates for internal infrastructure" (2023-03-11): https://code.mendhak.com/use-wildcard-certificates-for-internal-infrastructure/
64. Hacker News, "Show HN: Mkcert" (2019-01-07): https://news.ycombinator.com/item?id=18842218
65. Hacker News, "Upcoming Changes to Let's Encrypt Certificates" (2025-12-15) and "Upcoming changes to Let's Encrypt and how they affect XMPP server operators" (2026-02-09): https://news.ycombinator.com/item?id=46279241 and https://news.ycombinator.com/item?id=46950780
66. Hacker News, mkcert thread (2022-10-29): https://news.ycombinator.com/item?id=33383095
67. Traefik community, "Support for ACME ARI (Renewal Information)": https://community.traefik.io/t/support-for-acme-ari-renewal-information/29461
68. mx repo (mech-crate) at commit e266c26: `templates/router/docker-compose.yml`, `templates/router/config/traefik.yml`, `templates/router/config/dynamic/middlewares.yml`, `crates/mx-lib/src/router/mod.rs` (install sets 0o600 on acme.json), `docs/router.md` (HTTPS section), `templates/recipes/*/docker/compose/service.yml` (`entrypoints=web`), `~/.claude/skills/devloop/references/url-discovery.md`
69. Cloudflare, Create API token: https://developers.cloudflare.com/fundamentals/api/get-started/create-token/
70. cert-manager, Cloudflare DNS-01: https://cert-manager.io/docs/configuration/acme/dns01/Cloudflare/
71. Traefik source v3.7.15, `pkg/provider/traefik/internal.go` (redirect router rule and priority, `allowACMEByPass` exclusion, ping router at `math.MaxInt`): https://raw.githubusercontent.com/traefik/traefik/v3.7.15/pkg/provider/traefik/internal.go
