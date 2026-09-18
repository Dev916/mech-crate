---
title: "Django at Scale: Security Hardening, Performance, Observability, and Operations"
category: security
languages: [python]
complexity: advanced
use_cases:
  - hardening a Django project before it takes production traffic
  - choosing between CONN_MAX_AGE, the native psycopg pool, and PgBouncer
  - picking an application server, worker count and timeouts for a Django deployment
  - planning an upgrade against Django's LTS calendar and security release cadence
summary: "Production Django as of the 6.1 line: every check --deploy item and its real default, built-in CSP versus django-csp, the three mutually exclusive connection-pooling strategies, server and worker sizing, OpenTelemetry under pre-fork workers, and a risk-ordered readiness checklist."
provenance: researched
researched: 2026-09-18
sources:
  - https://docs.djangoproject.com/en/6.1/howto/deployment/checklist/
  - https://docs.djangoproject.com/en/6.1/ref/checks/
  - https://docs.djangoproject.com/en/6.1/ref/settings/
  - https://docs.djangoproject.com/en/6.1/topics/security/
  - https://docs.djangoproject.com/en/6.1/ref/csrf/
  - https://docs.djangoproject.com/en/6.1/releases/6.0/
  - https://docs.djangoproject.com/en/6.1/releases/6.1/
  - https://docs.djangoproject.com/en/6.1/releases/5.1/
  - https://docs.djangoproject.com/en/6.1/releases/5.2/
  - https://www.djangoproject.com/download/
  - https://docs.djangoproject.com/en/6.1/internals/security/
  - https://docs.djangoproject.com/en/6.1/releases/security/
  - https://docs.djangoproject.com/en/6.1/topics/auth/passwords/
  - https://django-axes.readthedocs.io/en/latest/2_installation.html
  - https://docs.allauth.org/en/latest/installation/quickstart.html
  - https://django-guardian.readthedocs.io/en/stable/
  - https://django-csp.readthedocs.io/en/latest/configuration.html
  - https://docs.djangoproject.com/en/6.1/topics/cache/
  - https://docs.djangoproject.com/en/6.1/ref/databases/
  - https://www.psycopg.org/psycopg3/docs/api/pool.html
  - https://www.pgbouncer.org/features.html
  - https://docs.djangoproject.com/en/6.1/topics/db/multi-db/
  - https://docs.djangoproject.com/en/6.1/ref/contrib/staticfiles/
  - https://whitenoise.readthedocs.io/en/stable/django.html
  - https://gunicorn.org/design/
  - https://gunicorn.org/reference/settings/
  - https://uvicorn.dev/deployment/
  - https://github.com/emmett-framework/granian/blob/master/README.md
  - https://docs.djangoproject.com/en/6.1/howto/deployment/asgi/
  - https://docs.djangoproject.com/en/6.1/ref/logging/
  - https://opentelemetry.io/docs/zero-code/python/
  - https://opentelemetry-python-contrib.readthedocs.io/en/latest/instrumentation/django/django.html
  - https://opentelemetry-python.readthedocs.io/en/stable/examples/fork-process-model/README.html
  - https://docs.sentry.io/platforms/python/integrations/django/
  - https://sre.google/sre-book/monitoring-distributed-systems/
  - https://docs.djangoproject.com/en/6.1/topics/testing/tools/
  - https://docs.djangoproject.com/en/6.1/ref/django-admin/
  - https://docs.djangoproject.com/en/6.1/howto/upgrade-version/
  - https://docs.djangoproject.com/en/6.1/topics/tasks/
  - https://docs.djangoproject.com/en/6.1/topics/migrations/
  - https://waffle.readthedocs.io/en/stable/
  - https://github.com/dfunckt/django-rules
---

# Django at Scale: Security Hardening, Performance, Observability, and Operations

State of practice as of 2026-09. The seed sources are Django's own deployment checklist [1], system check reference [2], settings reference [3] and security topic guide [4]; every default, version and "added in" claim below was read off the cited page rather than recalled. The version line described is Django 6.1, the current feature release, alongside Django 5.2, the current long-term support release [10]. Supporting claims come from the third-party projects Django leaves you to choose (axes, allauth, guardian, whitenoise, gunicorn, uvicorn, granian, psycopg, PgBouncer, OpenTelemetry, Sentry). Inline `[n]` keys to `sources`. Code is illustrative: settings fragments are compile-checked, not run.

Scope note: application structure, the service layer, ORM modelling, migrations authoring, the API layer and multi-tenancy belong to the enterprise Django architecture doc. This document owns what happens after the code is written. Container build and runtime practice lives in [docker-assembly-guide.md](docker-assembly-guide.md); cache-shape decisions at the data layer live in [database-design-guide.md](database-design-guide.md).

## 1. The deploy gate: `check --deploy` and the settings it audits

`manage.py check --deploy` "activates some additional checks that are only relevant in a deployment setting" and takes `--fail-level {CRITICAL,ERROR,WARNING,INFO,DEBUG}`, which "specifies the message level that will cause the command to exit with a non-zero status. Default is ERROR" [37]. That default is the trap: every deployment check is a **warning**, so the command exits 0 with a page of `security.W0xx` output unless you run `--fail-level WARNING` [2][37]. Point it at production settings with `--settings` or run it on the deployment itself [37].

The deploy-only checks are `security.W001` through `security.W022` plus `security.W025`, with `W007` and `W017` removed in earlier Django versions [2]. Silence the ones your load balancer already handles with `SILENCED_SYSTEM_CHECKS` rather than by lowering `--fail-level` [2].

The settings those checks audit, with the **documented defaults** [3]:

| Setting | Default [3] | Production value | Check |
|---|---|---|---|
| `DEBUG` | n/a | `False`; "You must never enable debug in production" [1] | W018 |
| `ALLOWED_HOSTS` | n/a | explicit hostnames; "must not be empty in deployment" [2] | W020 |
| `SECRET_KEY` | n/a | 50+ chars, 5+ unique, no `django-insecure-` prefix [2] | W009 |
| `SECRET_KEY_FALLBACKS` | `[]` | previous keys during rotation, same strength bar | W025 |
| `SECURE_SSL_REDIRECT` | `False` | `True`, or redirect at the proxy | W008 |
| `SECURE_HSTS_SECONDS` | `0` | non-zero once you are certain | W004 |
| `SECURE_HSTS_INCLUDE_SUBDOMAINS` | `False` | `True` only if every subdomain is HTTPS | W005 |
| `SECURE_HSTS_PRELOAD` | `False` | `True` only if you intend to submit | W021 |
| `SESSION_COOKIE_SECURE` | `False` | `True` | W010/W011/W012 |
| `CSRF_COOKIE_SECURE` | `False` | `True` | W016 |
| `SECURE_CONTENT_TYPE_NOSNIFF` | `True` | leave alone | W006 |
| `SECURE_REFERRER_POLICY` | `'same-origin'` | leave alone or tighten | W022 |
| `SECURE_CROSS_ORIGIN_OPENER_POLICY` | `'same-origin'` | leave alone | n/a |
| `X_FRAME_OPTIONS` | `'DENY'` | leave alone | W019 |
| `SECURE_PROXY_SSL_HEADER` | `None` | set only if a trusted proxy terminates TLS | n/a |

Two defaults are already safe and get changed by accident more often than they get set: `SECURE_CONTENT_TYPE_NOSNIFF` is `True` and `X_FRAME_OPTIONS` is `'DENY'` [3]. `SECURE_HSTS_INCLUDE_SUBDOMAINS` carries an explicit warning that setting it incorrectly is irreversible for the lifetime of `SECURE_HSTS_SECONDS` [3], so ramp HSTS: minutes, then days, then a year, and only then preload.

`CSRF_COOKIE_SAMESITE` and `SESSION_COOKIE_SAMESITE` both already default to `'Lax'` [3]; there is no deployment check for them, and raising either to `'Strict'` breaks top-level navigation back into your site, so change them deliberately.

**Host header.** Django validates `Host` against `ALLOWED_HOSTS` inside `HttpRequest.get_host()`, and "this validation only applies via `get_host()`; if your code accesses the `Host` header directly from `request.META` you are bypassing this security protection" [4]. Django requires `ALLOWED_HOSTS` explicitly rather than trusting the web server, because "even seemingly-secure web server configurations are susceptible to fake `Host` headers" [4]. Wildcards are allowed, but the checklist attaches a condition: "If you use a wildcard, you must perform your own validation of the `Host` HTTP header, or otherwise ensure that you aren't vulnerable to this category of attacks" [1]. It also recommends a catch-all server block that closes the connection for unknown hosts [1].

**Key rotation.** `SECRET_KEY_FALLBACKS` defaults to `[]`; the rotation procedure is to set a new `SECRET_KEY`, move the previous value to the beginning of `SECRET_KEY_FALLBACKS`, then remove old values once they are no longer needed [3], and the checklist adds that old keys should be removed "in a timely manner" [1]. It backs "many of Django's security-critical features" [2], so a rotation that skips the fallback window invalidates every signed value at once [3].

```python
# settings/production.py (illustrative: needs django-environ installed)
import environ

env = environ.Env()

SECRET_KEY = env("SECRET_KEY")
SECRET_KEY_FALLBACKS = env.list("SECRET_KEY_FALLBACKS", default=[])
DEBUG = False
ALLOWED_HOSTS = env.list("ALLOWED_HOSTS")
CSRF_TRUSTED_ORIGINS = env.list("CSRF_TRUSTED_ORIGINS", default=[])

SECURE_SSL_REDIRECT = True
SECURE_HSTS_SECONDS = 31536000
SECURE_HSTS_INCLUDE_SUBDOMAINS = True
SESSION_COOKIE_SECURE = True
CSRF_COOKIE_SECURE = True

# DATABASE_URL looks like postgres://localhost/app in development.
DATABASES = {"default": env.db_url("DATABASE_URL")}
```

**CSRF.** `CSRF_TRUSTED_ORIGINS` defaults to `[]` [3]; `CsrfViewMiddleware` verifies the `Origin` header against the current host and that setting, and for HTTPS requests without an `Origin` header it "performs strict referer checking" [5]. The documented limitation is structural: "Subdomains within a site will be able to set cookies on the client for the whole domain. By setting the cookie and using a corresponding token, subdomains will be able to circumvent the CSRF protection. The only way to avoid this is to ensure that subdomains are controlled by trusted users" [5]. If you hand a subdomain to a third party, CSRF protection stops being a control you own [5]. `CSRF_USE_SESSIONS` moves the token into the session for sites that have one [5].

**Content Security Policy.** Django 6.0 added built-in CSP: "Built-in support for the Content Security Policy (CSP) standard is now available, making it easier to protect web applications against content injection attacks such as cross-site scripting (XSS)" [6]. The surface is `SECURE_CSP` and `SECURE_CSP_REPORT_ONLY` (both defaulting to `{}` [3]), `django.middleware.csp.ContentSecurityPolicyMiddleware`, and the `csp()` context processor for nonces [6]. Django 6.1 added the `csp_nonce_attr` template tag, applied CSP nonces to admin and built-in templates when the `csp()` context processor is configured, and added `security.W027`, which "warns when `ContentSecurityPolicyMiddleware` is enabled with `CSP.NONCE` in a CSP policy but `django.template.context_processors.csp` is not configured" [7]. `security.E026` fires when a CSP setting is not a dictionary [2].

```python
# Illustrative: Django 6.0+ built-in CSP.
from django.utils.csp import CSP

SECURE_CSP = {
    "default-src": [CSP.SELF],
    "script-src": [CSP.SELF, CSP.NONCE],
    "img-src": [CSP.SELF, "https:"],
}
```

Decision rule: on Django 6.0 or later use the built-in settings; `django-csp` remains the option for older lines and offers `CONTENT_SECURITY_POLICY` / `CONTENT_SECURITY_POLICY_REPORT_ONLY` dictionaries with `DIRECTIVES`, `EXCLUDE_URL_PREFIXES` and a `REPORT_PERCENTAGE` throttle, via `csp.middleware.CSPMiddleware` [17]. Do not run both, since each sets the same response header [6][17]. Django's own guidance warns against `EXCLUDE_URL_PREFIXES`-style carve-outs: "a vulnerability on an unprotected page (e.g., one allowing arbitrary script injection) may be leveraged to attack protected pages", so "excluding any route can significantly weaken the site's overall CSP protection" [4]. Ship report-only first (`SECURE_CSP_REPORT_ONLY` [6], or django-csp's report-only policy, "used to test the policy without breaking the site" [17]), read the reports, then enforce.

## 2. Security hardening beyond the checklist

**Password hashers.** The default `PASSWORD_HASHERS` list starts with `PBKDF2PasswordHasher` [13], and the default iteration count has been raised on every recent release: 720,000 to 870,000 in 5.1 [8], 870,000 to 1,000,000 in 5.2 [9], and 1,200,000 to 1,500,000 in 6.1 [7]. Django's own text says "The Password Hashing Competition panel, however, recommends immediate use of Argon2 rather than the other algorithms supported by Django" [13]. Switching is one install and a list reorder, and existing hashes upgrade on next login: "When users log in, if their passwords are stored with anything other than the preferred algorithm, Django will automatically upgrade the algorithm to the preferred one" [13].

```python
# python -m pip install django[argon2]
PASSWORD_HASHERS = [
    "django.contrib.auth.hashers.Argon2PasswordHasher",
    "django.contrib.auth.hashers.PBKDF2PasswordHasher",
    "django.contrib.auth.hashers.PBKDF2SHA1PasswordHasher",
    "django.contrib.auth.hashers.BCryptSHA256PasswordHasher",
    "django.contrib.auth.hashers.ScryptPasswordHasher",
]
```

**Brute force.** Django states plainly that it "does not throttle requests to authenticate users" and that you should deploy a plugin or web server module to throttle against brute-force attacks [4]. `django-axes` fills that gap by tracking failed authentication attempts and locking the offender out [14]. Its installation page puts `axes.backends.AxesStandaloneBackend` "to the top of `AUTHENTICATION_BACKENDS`" and `axes.middleware.AxesMiddleware` last in `MIDDLEWARE`, noting that the middleware "only formats user lockout messages and renders Axes lockout responses on failed user authentication attempts from login views" [14]. What gets counted together is `AXES_LOCKOUT_PARAMETERS`: `["ip_address"]`, `["username"]`, or `["username", "ip_address"]` [14]. Two operational details from the same page: `AXES_HTTP_RESPONSE_CODE` "default has been changed from 403 (Forbidden) to 429 (Too Many Requests)", and setting `AXES_ENABLED = False` "disables the Axes middleware, authentication backend and signal receivers" for tests [14].

**Identity.** `django-allauth` covers local accounts, social/OAuth2, SAML, OIDC, MFA and headless API flows in one dependency, split across `allauth.account`, `allauth.socialaccount`, `allauth.mfa`, `allauth.headless` and `allauth.idp`, and requires `allauth.account.middleware.AccountMiddleware` [15]. Those apps share one account layer, so adding SSO or MFA later is a settings change rather than a migration of the user model [15].

**Whole-site authentication.** Django 5.1 added `LoginRequiredMiddleware`, which "redirects all unauthenticated requests to a login page", with `login_not_required()` to opt individual views out [8]. Because it inverts the default, the failure mode changes from "forgot to protect a view" to "forgot to exempt one", and the second is visible on the first request [8].

**Object-level permissions.** Django's built-in permissions are per model, not per row. `django-guardian` supplies "an implementation of object permissions for Django providing an extra authentication backend" and supports Python 3.9+ and Django 4.2+ [16]. `rules` is the other shape: "a tiny but powerful app providing object-level permissions to Django, without requiring a database", wired in as `rules.permissions.ObjectPermissionBackend` so that `has_perm()` with an object argument evaluates a predicate instead of returning `False` [42]. Guardian fits grants that are data (one user shares one document); `rules` fits grants that are a function of the object ("no need to mess around with a database to figure out whether John really wrote that book" [42]).

**Uploads and media.** The concrete exploit is in Django's own docs: "an HTML file can be uploaded as an image if that file contains a valid PNG header followed by malicious HTML. This file will pass verification of the library that Django uses for `ImageField` image processing (Pillow)" [4]. The mitigation is domain isolation, and the docs are specific that a subdomain is not enough: serve uploads from a distinct top-level or second-level domain such as `usercontent-example.com`, because "it's not sufficient to serve content from a subdomain like `usercontent.example.com`" [4]. Also restrict allowable extensions, disable handlers like `mod_php` that execute static files as code, and cap request body size **in the web server**: "Form submissions containing files are not limited by `DATA_UPLOAD_MAX_MEMORY_SIZE`", and under ASGI "the entire request may be spooled to disk before file size validation" [4]. For reference, `DATA_UPLOAD_MAX_MEMORY_SIZE` and `FILE_UPLOAD_MAX_MEMORY_SIZE` both default to `2621440` (2.5 MB) and `DATA_UPLOAD_MAX_NUMBER_FIELDS` to `1000` [3]. The checklist puts it more bluntly: "Media files are uploaded by your users. They're untrusted! Make sure your web server never attempts to interpret them" [1].

**Signed cookies.** Django 6.1 changed the default of `SIGNED_COOKIE_LEGACY_SALT_FALLBACK` to `False` and deprecated the setting: "Signed cookies now use an unambiguous salt derivation by default. Set `SIGNED_COOKIE_LEGACY_SALT_FALLBACK` to `True` to continue accepting legacy signed cookies" [7]. Flip it to `True` for one deploy across the upgrade, then remove it before it is dropped [7].

**Tracking advisories.** Security support covers the main branch, "the two most recent Django release series", and LTS releases for their specified period [11]. Roughly one week before disclosure the team notifies django-announce "of the date and approximate time of the upcoming security release, as well as the severity of the issues" [11], so subscribing to that list buys a week of scheduling notice. The disclosed-issue archive carries a CVE per issue with affected versions and per-version patches [12]; the 2026-08-04 batch (CVE-2026-15307, CVE-2026-15337, CVE-2026-15830, CVE-2026-15920) shipped patches for 6.1, 6.0 and 5.2 [12], which is exactly the "two most recent Django release series" plus the LTS window [11]. Anything older receives no patch [11].

## 3. Performance in production

### 3.1 Caching layers

Django's default cache is `LocMemCache`, and the docs are blunt: "Each process will have its own private cache instance, which means no cross-process caching is possible ... it's probably not a good choice for production environments" [18]. With four worker processes that is four caches: a value one worker writes is invisible to the other three [18]. Move to `django.core.cache.backends.redis.RedisCache`, which takes a single URL or a list where the first entry is the leader and the rest are read replicas, and wants `redis-py` installed with `hiredis-py` recommended [18].

| Layer | Mechanism | Reach for it when | Source |
|---|---|---|---|
| Per-site | `UpdateCacheMiddleware` first and `FetchFromCacheMiddleware` last in `MIDDLEWARE`, plus `CACHE_MIDDLEWARE_SECONDS` / `_KEY_PREFIX` / `_ALIAS` | the whole site is anonymous and mostly static; caches GET and HEAD responses with status 200 | [18] |
| Per-view | `@cache_page(60 * 15)`, optionally in the URLconf to decouple view from policy | a handful of expensive read-only endpoints | [18] |
| Fragment | `{% cache 500 sidebar request.user.username %}` | one costly region of an otherwise personalised page | [18] |
| Low-level | `cache.get_or_set`, `get_many` / `set_many`, `touch`, `incr` | computed values with your own invalidation | [18] |

The correctness hazard is `Vary`. `CacheMiddleware` "varies on `Authorization` automatically" [18], but personalisation carried in a session cookie is not covered unless you say so with `@vary_on_cookie` or `patch_vary_headers` [18]. Rule: per-site caching is for pages that are identical for every anonymous visitor, and everything personalised gets fragment or low-level caching keyed on the varying value, as in the documented `{% cache 500 sidebar request.user.username %}` form [18].

Sessions are a cache decision too. `SESSION_ENGINE` defaults to `'django.contrib.sessions.backends.db'` with `SESSION_COOKIE_AGE` of `1209600` seconds (2 weeks) [3], which means a database write per request for logged-in users. The checklist recommends cached sessions for performance, and regular clearing of old sessions if you stay on the database backend [1]; `clearsessions` "can be run as a cron job" [37].

### 3.2 Database connections: three strategies, pick exactly one

`CONN_MAX_AGE` defaults to `0` and `CONN_HEALTH_CHECKS` to `False` [3], so out of the box every request opens and closes a PostgreSQL connection. The checklist notes persistent connections give "a nice speed-up" when connection setup is a significant share of request time [1].

Django 5.1 added a native pool: "Django 5.1 also introduces connection pool support for PostgreSQL. As the time to establish a new connection can be relatively long, keeping connections open can reduce latency", configured with `"pool": True` or a dict passed to psycopg's `ConnectionPool`, and requiring `psycopg[pool]` [8][19]. Django 5.2 extended pooling to Oracle [9]. The pool is per alias per process: "Django maintains a separate pool for each database alias in each process" [19].

The constraint that bites: "The `CONN_MAX_AGE` setting must be `0` when connection pooling is enabled. Configure connection lifetime with the `max_lifetime` pool option instead" [19]. And unless you supply a `check` callback, `CONN_HEALTH_CHECKS` decides whether connections are validated on checkout from the pool [19].

| Strategy | Config | Connections to Postgres | Costs |
|---|---|---|---|
| Per-request connect | `CONN_MAX_AGE = 0` (default [3]) | one per in-flight request | connection setup on every request [1] |
| Persistent connections | `CONN_MAX_AGE = N`, `CONN_HEALTH_CHECKS = True` | one per worker process, held | idle connections scale with workers, not with load |
| Native psycopg pool | `OPTIONS = {"pool": {...}}`, `CONN_MAX_AGE = 0` [19] | bounded per process per alias [19] | needs `psycopg[pool]`; still per process |
| External pooler | PgBouncer in front | bounded across the whole fleet [21] | mode-dependent feature loss (below) |

psycopg's `ConnectionPool` defaults are `min_size=4`, `max_size=None`, `timeout=30.0`, `max_lifetime=3600.0` (1 hour), `max_idle=600.0`, `reconnect_timeout=300.0` and `num_workers=3` [20]. Note the multiplication: `min_size=4` per alias per process, across N worker processes across M containers, is 4NM connections held open before any traffic arrives [20]. Size `min_size` from the floor you actually want rather than from the default [20].

**PgBouncer.** The modes are documented as session pooling ("a server connection will be assigned to it for the whole duration it stays connected"), transaction pooling ("a server connection is assigned to a client only during a transaction"), and statement pooling, "transaction pooling with a twist: multi-statement transactions are disallowed" [21]. Transaction pooling is the mode that actually multiplexes connections, and it is also the mode with a published incompatibility list [21]. PgBouncer lists as unsupported in transaction pooling: `SET`/`RESET`, `LISTEN`, `WITH HOLD CURSOR`, protocol-level prepared plans (requires configuration), `PREPARE`/`DEALLOCATE`, `PRESERVE`/`DELETE ROWS` temporary tables, `LOAD`, and session-level advisory locks [21].

Django names the concrete consequence: "Using a connection pooler in transaction pooling mode (e.g. PgBouncer) requires disabling server-side cursors for that connection", because server-side cursors are local to a connection and a later transaction may land on a different one [19]. The fix is `"disable_server_side_cursors": True` in `OPTIONS`, or session pooling mode, or wrapping each server-side-cursor queryset in `atomic()` [19].

```python
# Illustrative: native psycopg pool, mutually exclusive with CONN_MAX_AGE.
DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": "app",
        "HOST": "localhost",
        "CONN_MAX_AGE": 0,  # required when "pool" is set
        "CONN_HEALTH_CHECKS": True,
        "OPTIONS": {
            "pool": {"min_size": 2, "max_size": 8, "max_lifetime": 1800},
        },
    },
}

# Illustrative: behind PgBouncer in transaction pooling mode instead.
PGBOUNCER_OPTIONS = {"disable_server_side_cursors": True}
```

Decision rule: because the native pool is per process per alias [19], one process pool per container is enough until connection count at the database becomes the limit; once many containers each hold a pool, move the bound to PgBouncer and set the in-process strategy back to per-request or a very small pool. Never stack a large `CONN_MAX_AGE` under a transaction-mode pooler: Django already requires `CONN_MAX_AGE = 0` alongside its own pool [19], and transaction mode drops the session state a long-lived connection assumes [21].

### 3.3 Read replicas

Django ships a `PrimaryReplicaRouter` example with `db_for_read` returning a random replica, `db_for_write` returning `"primary"`, and `allow_relation` accepting pairs from the pool, wired through `DATABASE_ROUTERS` [22]. The docs then disown it: the configuration "is also flawed: it doesn't provide any solution for handling replication lag ... It also doesn't consider the interaction of transactions with the database utilization strategy" [22]. Because the shipped router has no answer for replication lag [22], treat replica routing as a per-view opt-in for reports and dashboards rather than a global default, and never read back from a replica in the request that wrote.

### 3.4 Query budgets in CI

`assertNumQueries(num, func, *args, **kwargs)` "asserts that when `func` is called with `*args` and `**kwargs` that `num` database queries are executed", and also works as a context manager [36]. Because it asserts a count rather than a duration, it turns an N+1 regression into a deterministic test failure [36].

```python
# Illustrative: a query budget test (needs Django installed to run).
from django.test import TestCase
from django.urls import reverse


class DashboardQueryBudget(TestCase):
    def test_dashboard_stays_within_budget(self):
        url = reverse("dashboard")
        with self.assertNumQueries(6):
            self.client.get(url)
```

Django 6.1 added a second lever, fetch modes [7]. `FETCH_PEERS` "fetches a missing field for all instances that came from the same `QuerySet` ... It can reduce most cases of the 'N+1 queries problem' to two queries without any work to maintain a list of fields to prefetch", and `FETCH_RAISE` raises `FieldFetchBlocked` instead of issuing the query [7]. `FETCH_RAISE` is the stricter budget of the two: it fails on the specific unfetched field access rather than on an aggregate count [7].

### 3.5 Static files

`ManifestStaticFilesStorage` appends the MD5 hash of content to filenames, rewrites `@import`, `url()` and source-map references, and stores the mapping in `staticfiles.json` at `collectstatic` time; it requires `DEBUG = False` and a completed `collectstatic`, raises `ValueError` for files missing from the manifest unless `manifest_strict` is disabled, and gives up after `max_post_process_passes`, which "defaults to 5" [23]. The documented warning is that it "typically shouldn't be used when running tests as `collectstatic` isn't run as part of the normal test setup" [23].

WhiteNoise's `CompressedManifestStaticFilesStorage` adds gzip and Brotli, and its middleware "should be placed directly after the Django `SecurityMiddleware` (if you are using it) and before all other middleware" [24]. It "sends appropriate cache headers with your static content", which is what lets a CDN serve repeat requests without touching the application [24]. It is explicitly "not suitable for serving user-uploaded 'media' files" [24], which lines up with the separate-domain rule in section 2.

### 3.6 Application server and worker sizing

Django's ASGI deployment page names Daphne, Granian, Hypercorn and Uvicorn as ASGI servers, and notes that "Django's default ASGI handler will run all your code in a synchronous thread" unless you write your own async handler, in which case "do not call blocking synchronous functions or libraries in any async code" [29].

| Server | Model | Reach for it when | Source |
|---|---|---|---|
| gunicorn `sync` | pre-fork, one request per worker, no keep-alive, "requires a buffering proxy (nginx, HAProxy) for production" | plain WSGI Django behind a proxy | [25] |
| gunicorn `gthread` | thread pool per worker, supports keep-alive | mixed workloads, moderate concurrency | [25] |
| gunicorn `gevent` | greenlets, thousands of concurrent connections; "may require patches for some libraries (e.g., psycogreen for Psycopg)" | I/O-bound, websockets, long polling | [25] |
| uvicorn | `spawn`, not pre-fork, so the multiprocess manager works on Windows | ASGI Django; nginx in front "may not be necessary, but is recommended for additional resilience" | [27] |
| granian | Rust (Hyper + Tokio), serves ASGI/3, RSGI and WSGI, HTTP/1 and HTTP/2, HTTPS, mTLS, websockets | one dependency for both ASGI and WSGI, or HTTP/2 at the app server | [28] |

Sizing. Gunicorn's own guidance is `workers = (2 x CPU cores) + 1` as a starting point, with the caveat that workers are not clients and that gunicorn typically needs only 4 to 12 workers to handle heavy traffic, since too many "waste resources and can reduce throughput" [25]. Granian defaults `--workers` to 1 and advises that "matching the amount of CPU cores for the workers is generally the best starting point; on containerized environments like docker or k8s is best to have 1 worker per container though and scale your containers using the relevant orchestrator" [28]. In containers the second rule dominates, because the orchestrator is already the thing scaling you [28].

Timeouts, from gunicorn's generated settings reference [26]: `timeout` defaults to `30` and kills workers silent that long; `graceful_timeout` defaults to `30`; `keepalive` defaults to `2`; `max_requests` and `max_requests_jitter` both default to `0`, so periodic worker recycling is opt-in; `worker_class` defaults to `'sync'`; `forwarded_allow_ips` defaults to `'127.0.0.1,::1'`; and `http_protocols` defaults to `'h1'`, so HTTP/2 is opt-in [26]. Set `max_requests` with non-zero `max_requests_jitter` to bound leak growth without restarting every worker at once [26]. Behind a proxy, uvicorn reads `X-Forwarded-For` and `X-Forwarded-Proto`, but "as anyone can set these headers you must configure which 'clients' you will trust to have set them correctly", which is what `--forwarded-allow-ips` does [27]; gunicorn's equivalent trust list is `forwarded_allow_ips` [26]. Getting that trust list wrong while `SECURE_PROXY_SSL_HEADER` is set lets a client decide the answer to `request.is_secure()` [3][27].

A note on the gunicorn plus uvicorn combination: uvicorn's docs now carry "the `uvicorn.workers` module is deprecated and will be removed in a future release. You should use the `uvicorn-worker` package instead" [27].

## 4. Observability

**Logging.** The built-in loggers are worth wiring individually rather than catching `django` at the root [30]:

| Logger | What it emits | Level |
|---|---|---|
| `django.request` | request handling; 5XX as ERROR, 4XX as WARNING, with `status_code` and `request` in the record | ERROR / WARNING |
| `django.security.*` | `SuspiciousOperation` and other security errors; sub-loggers include `django.security.DisallowedHost` and `django.security.csrf` | WARNING, ERROR when it reaches the WSGI handler |
| `django.db.backends` | every application-level SQL statement, with `duration`, `sql`, `params`, `alias` | DEBUG |
| `django.db.backends.schema` | SQL run by migrations (not by `RunPython`) | DEBUG |
| `django.contrib.sessions` | non-fatal `cached_db` session errors, with traceback | ERROR |
| `django.dispatch` | signal receiver failures, with `receiver` and `err` | ERROR |

Two consequences. First, "requests logged to `django.security` aren't logged to `django.request`" and requests resulting in a 400 go only to `django.security` [30], so a handler attached to `django.request` alone silently drops host-header and CSRF attacks. Second, do not plan on `django.db.backends` for production slow-query logging: "For performance reasons, SQL logging is only enabled when `settings.DEBUG` is set to `True`, regardless of the logging level or handlers installed" [30]. Production slow queries belong to the database's own slow-query log, or to an agent that instruments the driver the way Sentry's Django integration instruments database queries [34].

For error email, `AdminEmailHandler` sends to `ADMINS` and the default configuration gates it behind the `RequireDebugFalse` filter [30]; `ADMINS` receive 500s and `MANAGERS` receive 404s [1]. The checklist's own verdict is the one to follow: "Error reporting by email doesn't scale very well ... Consider using an error monitoring system such as Sentry before your inbox is flooded by reports" [1]. `CallbackFilter` is the documented escape hatch for dropping noisy records such as `UnreadablePostError` [30].

**Tracing.** OpenTelemetry's zero-code path is `opentelemetry-bootstrap -a install` followed by running under `opentelemetry-instrument`, with Django among the auto-instrumented frameworks and configuration via `OTEL_SERVICE_NAME`, `OTEL_TRACES_EXPORTER` and `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` [31]. The programmatic form is `DjangoInstrumentor().instrument()`, with `request_hook` and `response_hook` callbacks and `OTEL_PYTHON_DJANGO_TRACED_REQUEST_ATTRS` for pulling request attributes onto the span [32]. One ordering detail that costs an afternoon: the instrumentation "creates the span before Django middlewares run", so `request.user` is not available in `request_hook` and must be read in `response_hook` [32].

The pre-fork trap is the important one. "The `BatchSpanProcessor` is not fork-safe and doesn't work well with application servers (Gunicorn, uWSGI) which are based on the pre-fork web server model", because "during the fork, the child process inherits the lock which is held by the parent process and deadlock occurs" [33]. Initialise tracing inside gunicorn's `post_fork` hook rather than at import time [33].

```python
# gunicorn.conf.py (illustrative: needs the OTel SDK installed)
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor


def post_fork(server, worker):
    server.log.info("Worker spawned (pid: %s)", worker.pid)
    resource = Resource.create(attributes={"service.name": "api-service"})
    trace.set_tracer_provider(TracerProvider(resource=resource))
    exporter = OTLPSpanExporter(endpoint="http://localhost:4317")
    trace.get_tracer_provider().add_span_processor(BatchSpanProcessor(exporter))
```

**Errors and spans from Sentry.** The Django integration instruments the middleware stack, signals, database queries, Redis commands and access to Django caches, and lets you name transactions by URL pattern or by view function via `transaction_style` [34]. `send_default_pii` controls whether user data and request headers are attached, so it is the one flag that decides whether your error store holds personal data [34].

**Metrics that matter.** The four golden signals are latency ("the time it takes to service a request"), traffic ("a measure of how much demand is being placed on your system"), errors ("the rate of requests that fail, either explicitly (e.g., HTTP 500s), implicitly ... or by policy"), and saturation ("how 'full' your service is") [35]. Two refinements transfer directly to Django. "It's important to distinguish between the latency of successful requests and the latency of failed requests" [35]: a fast 500 flatters a p95 that a slow 200 would fail. And "many systems degrade in performance before they achieve 100% utilization, so having a utilization target is essential" [35], which for Django means alerting on connection pool and worker saturation well before they hit their ceiling, since both of those are the resources that hit it first.

## 5. Operations

**Deployment order.** Three commands, each closing a different failure. `migrate --check` "makes `migrate` exit with a non-zero status when unapplied migrations are detected" and `makemigrations --check` "makes `makemigrations` exit with a non-zero status when model changes without migrations are detected" [37]; the second belongs in CI so a model change without a migration never reaches a deploy. `sqlmigrate` "takes migration names and returns the SQL they would run ... without actually running the migration", which is how you get a reviewable diff before a schema change lands [40]. `collectstatic` must run before traffic reaches code that renders hashed asset URLs, because `ManifestStaticFilesStorage` raises `ValueError` for anything missing from `staticfiles.json` [23].

**Background work.** Django 6.0 added a built-in Tasks framework for running code outside the HTTP request and response cycle, configured with the `TASKS` setting and used via the `@task` decorator and `enqueue()` [6]. Read the boundary carefully before planning around it: "Django does not provide a worker mechanism to run Tasks. The actual execution must be handled by infrastructure outside Django", and "Django includes backends suitable for development and testing only. Production systems should rely on backends that supply a worker process and a durable queue implementation" [39]. The shipped backends are `ImmediateBackend` and `DummyBackend` [39]. What 6.0 standardised is the interface; the runtime is still yours to choose and operate [39].

**Feature flags.** "Waffle is feature flipper for Django. You can define the conditions for which a flag should be active, and use it in a number of ways" [41]. Paired with the deployment order above, a flag is what lets new code ship dark and be switched on only after the migration has landed [41].

**Configuration and secrets.** The checklist's own example reads `SECRET_KEY` from the environment or a file [1], and says database passwords "are very sensitive" and should be protected "exactly like `SECRET_KEY`" [1]. Rotation is the `SECRET_KEY_FALLBACKS` procedure from section 1 [3].

**Backups.** The checklist does not hedge: "If you haven't set up backups for your database, do it right now!" [1], and it separately tells you to check your backup strategy for user-uploaded files [1].

**Containers.** Base image, layer and runtime practice for Django containers is covered in [docker-assembly-guide.md](docker-assembly-guide.md) and not repeated here. The Django-specific hooks are: run `check --deploy --fail-level WARNING` as a build or start-time gate [2][37], run `collectstatic` at build time so the image is self-contained [23], and size workers per container rather than per host [28].

**Upgrades.** The support window is published: Django 5.2 is the current LTS, with extended support to April 2028; 6.0's mainstream support ended 2026-08-04 with extended support to April 2027; 6.1 has mainstream support to April 2027 and extended to December 2027; 6.2 arrives in April 2027 as the next LTS [10]. From Django 2028 the scheme changes: "Beginning with Django 2028, feature releases will use the version format YYYY and will be released every January", and "starting with Django 2028, every feature release receives the same three-year support period" [10]. Django 6.1 already renamed the deprecation warning classes to match, `RemovedInDjango70Warning` becoming `RemovedInDjango2028Warning` [7].

The upgrade procedure is explicit: go one feature release at a time using the latest patch of each, and "the same incremental upgrade approach is recommended when upgrading from one LTS to the next" [38]. Before upgrading, "resolve any deprecation warnings raised by your project while using your current version of Django" [38]. The mechanism is `python -Wa manage.py test`, or `PYTHONWARNINGS=always pytest tests --capture=no` [38]. Run that in CI and fail the build on `RemovedInDjango2028Warning`, since the warning classes are named for the calendar version in which the features they mark are removed [7]; a green build full of warnings is a scheduled breakage [38]. Clear your cache after upgrading to avoid pickle compatibility problems with cached objects [38].

Python floors move with Django: 6.0 and 6.1 both support Python 3.12, 3.13 and 3.14 [6][7], and "the Django 5.2.x series is the last to support Python 3.10 and 3.11" [6].

## Agreed vs folklore (compressed)

- **Agreed.** `check --deploy` is the baseline and is designed to be run against production settings or in CI [37]. HTTPS-only cookies, HSTS, explicit `ALLOWED_HOSTS` are settings with documented defaults that are wrong for production [3]. Argon2 is recommended over the Django default by the body Django cites [13]. Django deliberately ships no login throttling and no task worker, and says so [4][39]. Pooling and `CONN_MAX_AGE` are mutually exclusive by documentation, not by convention [19].
- **"`DEBUG = False` is enough": folklore.** It clears exactly one check, `security.W018` [2]. It leaves `SECURE_SSL_REDIRECT` at `False`, `SECURE_HSTS_SECONDS` at `0`, `SESSION_COOKIE_SECURE` at `False` and `CSRF_COOKIE_SECURE` at `False` [3], all of which the same command reports. It also turns off SQL logging, since `django.db.backends` only emits when `DEBUG` is `True` [30], so the setting that hardens the app is also the one that removes your query visibility.
- **"PgBouncer just works": folklore in transaction mode.** The unsupported list is published: `SET`/`RESET`, `LISTEN`, `WITH HOLD CURSOR`, protocol-level prepared plans, `PREPARE`/`DEALLOCATE`, session-level advisory locks and more [21], and Django separately requires `disable_server_side_cursors` for that mode [19]. Session pooling keeps the features and loses most of the multiplexing, so the choice is a trade rather than a free win [21].
- **"Cache everything": folklore.** Per-site caching only stores GET and HEAD responses with status 200 [18], and correctness depends on `Vary`; `CacheMiddleware` varies on `Authorization` automatically but not on your session cookie unless you ask [18]. The default `LocMemCache` is per process, so "we added caching" with the default backend is N private caches, not one [18].
- **"gunicorn sync workers cannot scale": overstated.** Sync is the documented default for "CPU-bound apps behind a proxy", and its real constraints are no keep-alive and a required buffering proxy, not a throughput ceiling [25]. The same page caps the useful worker count in the 4 to 12 range regardless of class [25]. Switch classes for long blocking calls, streaming, long polling or websockets [25], not because sync sounds slow.
- **"Rename `/admin/` to harden it": not a documented control.** Nothing in Django's security topic guide or check reference lists URL obscurity as a mitigation [2][4]. The documented controls are throttling (which Django does not provide [4]), forced authentication such as `LoginRequiredMiddleware` [8], `SESSION_COOKIE_SECURE` plus HSTS [3], and staying inside the patch window [11][12].

## Synthesis (inferred)

**Production readiness checklist, ordered by risk.** Each rung is cheap relative to the blast radius of the one above it.

1. `DEBUG = False`, `ALLOWED_HOSTS` explicit, `SECRET_KEY` from the environment and never in git. A miss here leaks source, settings and locals to anyone who can trigger a 500.
2. `check --deploy --fail-level WARNING` exits 0 in CI, with any `SILENCED_SYSTEM_CHECKS` entry carrying a comment naming the compensating control.
3. TLS everywhere: `SECURE_SSL_REDIRECT`, both `*_COOKIE_SECURE` flags, HSTS ramped (minutes, days, a year) before `includeSubDomains` and long before preload.
4. Login throttling installed and actually firing, verified by a deliberate lockout in staging. This is the highest-value control Django does not give you.
5. Uploads on a separate registrable domain, extensions allowlisted, body size capped in the web server. The PNG-header-plus-HTML exploit needs no authentication.
6. Backups running and one restore drill completed. Everything below is availability; this is survival.
7. A non-`LocMemCache` cache backend, cached or cleaned sessions, and a connection strategy chosen on purpose (one of per-request, persistent, in-process pool, external pooler).
8. `collectstatic` at build time with a manifest storage, assets behind a CDN.
9. Structured logging with a request id, `django.security.*` routed somewhere a human sees, errors going to an error tracker rather than to `ADMINS`.
10. Traces initialised after fork, with the p95 of successful requests and DB pool saturation on a dashboard.
11. Query budgets on the three hottest endpoints, CSP in report-only, deprecation warnings failing the build.

**A request id is the cheapest observability primitive and Django does not ship one.** Generate or accept one in the outermost middleware, put it in a `contextvars` variable, add it to every log record via a filter, echo it in the response header, and stamp it as a span attribute. Without it, correlating a Sentry event to a slow-query log entry to a load balancer access log is manual work; with it, it is a grep. Do this before adding a second observability vendor.

**What to measure in the first week after launch.** Instrument these seven and ignore everything else until one of them moves:

| Signal | Why this one | Action when it moves |
|---|---|---|
| p95 latency of 2xx responses, per URL pattern | golden-signal latency, split from failures so a fast 500 cannot flatter it | profile the top pattern, do not tune globally |
| 5xx rate, split by view | golden-signal errors | one view dominating means a bug, spread evenly means infrastructure |
| DB connections in use versus the server's `max_connections` | the ceiling that arrives first for Django, and the one pool config controls | lower `min_size` per process before raising `max_connections` |
| worker saturation (busy workers over total) | pre-fork servers queue silently once every worker is busy | add workers only if CPU is idle, otherwise add containers |
| queries per request on the hottest endpoint | N+1 regressions are step functions, not drifts | pin it with `assertNumQueries` so it never regresses twice |
| cache hit ratio per cache alias | distinguishes "cache is cold" from "cache is wrong" | a ratio near zero usually means a `Vary` or key-prefix bug |
| worker restarts and OOM kills | memory growth shows up here before it shows up in latency | set `max_requests` with jitter while you hunt the leak |

**The three things most likely to be wrong on day one**, from the shape of the defaults rather than from any single source: the cache is still `LocMemCache` so every worker has its own; `CONN_MAX_AGE` is still `0` so every request pays connection setup; and `check --deploy` has never been run with `--fail-level WARNING` so it has always exited 0. All three are single-line fixes and all three are invisible until load arrives.
