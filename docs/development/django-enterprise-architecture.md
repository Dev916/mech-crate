---
title: "Enterprise Django: Layout, Service Layer, ORM, Migrations, APIs, Async, Tenancy, and Testing"
category: architecture
languages: [python]
complexity: advanced
use_cases:
  - "standing up a new Django service several teams will work in for years"
  - "deciding whether business logic belongs in models, managers, or a service layer"
  - "cutting N+1 queries and shipping schema changes to large tables without downtime"
  - "choosing between DRF and django-ninja, or between Celery and django.tasks"
summary: "Decision rules for large Django codebases on the 6.1 line: app boundaries and settings split, services versus fat models, ORM and migration technique at scale, DRF versus django-ninja, async and the 6.0 Tasks framework, multi-tenancy, and test structure."
provenance: researched
researched: 2026-09-18
sources:
  - https://www.djangoproject.com/download/
  - https://docs.djangoproject.com/en/dev/releases/6.1/
  - https://docs.djangoproject.com/en/dev/releases/6.0/
  - https://docs.djangoproject.com/en/5.2/releases/5.2/
  - https://docs.djangoproject.com/en/dev/misc/design-philosophies/
  - https://github.com/HackSoftware/Django-Styleguide
  - https://www.b-list.org/weblog/2020/mar/16/no-service/
  - https://forum.djangoproject.com/t/where-to-put-business-logic-in-django/282
  - https://12factor.net/config
  - https://django-environ.readthedocs.io/en/latest/quickstart.html
  - https://docs.djangoproject.com/en/dev/ref/applications/
  - https://docs.djangoproject.com/en/dev/topics/db/optimization/
  - https://docs.djangoproject.com/en/dev/topics/db/fetch-modes/
  - https://docs.djangoproject.com/en/dev/ref/models/querysets/
  - https://docs.djangoproject.com/en/dev/ref/models/fields/
  - https://docs.djangoproject.com/en/dev/topics/db/transactions/
  - https://docs.djangoproject.com/en/dev/howto/writing-migrations/
  - https://docs.djangoproject.com/en/dev/topics/migrations/
  - https://docs.djangoproject.com/en/dev/ref/contrib/postgres/operations/
  - https://www.django-rest-framework.org/community/release-notes/
  - https://django-ninja.dev/
  - https://django-ninja.dev/whatsnew_v1/
  - https://django-ninja.dev/guides/async-support/
  - https://drf-spectacular.readthedocs.io/en/latest/readme.html
  - https://www.loopwerk.io/articles/2024/drf-vs-ninja/
  - https://docs.djangoproject.com/en/dev/topics/async/
  - https://github.com/django/deps/blob/main/final/0014-background-workers.rst
  - https://docs.djangoproject.com/en/dev/topics/tasks/
  - https://www.loopwerk.io/articles/2026/django-tasks-review/
  - https://pypi.org/project/django-tasks/
  - https://channels.readthedocs.io/en/latest/introduction.html
  - https://jonathanadly.com/is-async-django-ready-for-prime-time
  - https://django-tenants.readthedocs.io/en/latest/
  - https://django-tenants.readthedocs.io/en/latest/use.html
  - https://pypi.org/project/django-tenants/
  - https://django-pgschemas.readthedocs.io/en/stable/overview/
  - https://github.com/citusdata/django-multitenant
  - https://pypi.org/project/django-multitenant/
  - https://docs.djangoproject.com/en/dev/topics/testing/tools/
  - https://docs.djangoproject.com/en/dev/topics/testing/overview/
  - https://pytest-django.readthedocs.io/en/latest/database.html
  - https://factoryboy.readthedocs.io/en/stable/orms.html
  - https://github.com/jazzband/django-silk
  - https://github.com/jmcarp/nplusone
  - https://pypi.org/project/nplusone/
---

# Enterprise Django: Layout, Service Layer, ORM, Migrations, APIs, Async, Tenancy, and Testing

State of practice as of 2026-09, describing the Django 6.1 line with 5.2 as the current LTS [1]. The seed sources are the Django release notes for 5.2, 6.0 and 6.1 [2][3][4], the framework's own database-optimization, migration, async, tasks and testing topics [12][13][17][18][26][28][39], and the two poles of the long-running architecture argument: the HackSoft Django Styleguide [6] and James Bennett's "Against service layers in Django" [7]. Ecosystem claims are checked against each project's own docs and its PyPI release record rather than blog summaries. Inline `[n]` keys to `sources`. Python snippets are illustrative: they are syntax-checked and lint-checked but not run, because they need a configured Django project.

Sibling reading in this corpus: `appendix-file-structure-theory.md` for the layout theory this section applies, `appendix-api-design.md` for the HTTP contract rules that section 6 deliberately does not repeat, and `database-design-guide.md` for schema modelling. `appendix-laravel.md` is the corpus's other framework doc and takes the opposite bet (domain kept clear of the ORM), which is worth reading against section 3. Security hardening, caching, connection pooling, observability and deployment are out of scope here; they belong to the Django production hardening doc.

## 1. Version landscape and an upgrade policy

The supported matrix published on djangoproject.com as of 2026-09 [1]:

| Series | Latest | Mainstream support ends | Extended support ends |
|---|---|---|---|
| 5.2 LTS [1] | 5.2.17 [1] | December 3, 2025 [1] | April 2028 [1] |
| 6.0 [1] | 6.0.8 [1] | August 4, 2026 [1] | April 2027 [1] |
| 6.1 [1] | 6.1.1 [1] | April 2027 [1] | December 2027 [1] |
| 6.2 LTS [1] | not yet released [1] | December 2027 [1] | April 2030 [1] |

The roadmap lists 6.2 LTS for April 2027, then year-named releases (2028, 2029) each January, each LTS carrying the same three-year support period [1]. Support for the 4.2 LTS ended in April 2026 [4].

What moved in each line, restricted to what the release notes literally claim [2][3][4]:

- **5.2 (April 2, 2025, LTS)** added `CompositePrimaryKey` for multi-field primary keys, the `AlterConstraint` migration operation, automatic model imports in the `shell` command, `QuerySet.explain()` support for `memory` and `serialize` on PostgreSQL 17 and later, and constraint validation for `GeneratedField` [4]. It supports Python 3.10 through 3.14, with 3.14 added in 5.2.8 [4].
- **6.0 (December 3, 2025)** added the built-in Tasks framework, template partials (`{% partialdef %}` / `{% partial %}`), Content Security Policy support, and `AsyncPaginator` / `AsyncPage` [3]. It requires Python 3.12, 3.13 or 3.14, and changed `DEFAULT_AUTO_FIELD` to default to `BigAutoField` [3].
- **6.1 (August 5, 2026)** added fetch modes (`FETCH_ONE`, `FETCH_PEERS`, `FETCH_RAISE`), database-level `on_delete` options (`DB_CASCADE`, `DB_SET_NULL`, `DB_SET_DEFAULT`), and the `MAILERS` setting, which "will replace `EMAIL_BACKEND` and related `EMAIL_*` settings in Django 7.0" [2]. It dropped PostgreSQL 14, MySQL before 8.4 and MariaDB before 10.11, and raised the SQLite minimum from 3.31.0 to 3.37.0 [2].

**Upgrade policy that falls out of the table.** Running LTS-only means sitting on 5.2 until 6.2 lands in April 2027 and then absorbing two feature releases of change in one jump, with the Python floor moving from 3.10 to 3.12 on the way [3][4]. Running every feature release means an upgrade roughly every eight months but each one is small, and it is the only way to get fetch modes and database-level deletes before 2027 [1][2]. The deciding variable is usually the third-party stack: DRF 3.18.1 (2026-09-07) supports 5.2, 6.0 and 6.1 and dropped 4.2, 5.0 and 5.1 in 3.18.0 [20], and django-tenants 3.14.0 (2026-08-05) carries Django 5.2, 6.0 and 6.1 classifiers [35], so the mainstream packages track feature releases quickly; the long tail does not [38][45].

## 2. Layout: apps, boundaries, and the settings split

Django's own vocabulary is narrow: a **project** is "defined primarily by a settings module", and an **application** is "a Python package that provides some set of features", wired in through `INSTALLED_APPS` and combining models, views, templates, template tags, static files, URLs and middleware [11]. Nothing in the framework says an app is a bounded context; `AppConfig.name` and `AppConfig.label` must merely be unique across the project [11]. Where a capability boundary falls is therefore outside what Django specifies, and two shapes dominate in practice, both expressible within that definition [11]:

| Shape | What it is | Wins when | Costs |
|---|---|---|---|
| App-per-model-group | One app per noun cluster (`users`, `orders`, `payments`), each with `models.py`, `views.py`, `urls.py` [11] | A team owns the whole vertical and cross-app imports stay one-directional [11] | Nothing enforces direction; imports drift into a cycle that only `makemigrations` notices [18] |
| App-per-bounded-context with an internal split | One app per business capability, internally split into `models.py`, `selectors.py`, `services.py`, `apis.py` [6] | Logic spans several models inside the capability and you want one entry point per use case [6] | More files per app; the split only pays once a capability has real flow [6] |

Two hard constraints that bite when you rearrange apps late: "Apps without migrations must not have relations (`ForeignKey`, `ManyToManyField`, etc.) to apps with migrations" [18], and a `ForeignKey` across apps creates a migration dependency on the other app, which Django records and enforces [18]. A circular dependency is resolved manually "by breaking out one of the ForeignKeys in the circular dependency loop into a separate migration" [18]. Cross-app foreign keys are therefore the real boundary: cheap to add, and removable only through a hand-written migration sequence [18].

**Settings.** The twelve-factor litmus test is "whether the codebase could be made open source at any moment, without compromising any credentials" [9], and it objects specifically to named config groups because "as more deploys of the app are created, new environment names are necessary... resulting in a combinatorial explosion of config" [9]. Django's settings module is a Python module, so the practical compromise is a small package (`base.py` plus thin per-deploy modules selected by `DJANGO_SETTINGS_MODULE`) where the per-deploy modules contain switches, never values [9][11].

`django-environ` supplies typed reads with defaults, `read_env()` for a local `.env`, and URL parsers such as `env.db()` and `env.cache()` [10]. Its own guidance is that "the `.env` file should be specific to the environment and not checked into version control", with a committed `.env.dist` documenting the mandatory variables [10].

```python
# config/settings/base.py  (illustrative)
import environ

env = environ.Env(DEBUG=(bool, False))
env.read_env()

DEBUG = env("DEBUG")
SECRET_KEY = env("SECRET_KEY")
DATABASES = {"default": env.db("DATABASE_URL", default="postgres://localhost/app")}
CACHES = {"default": env.cache("REDIS_URL", default="locmemcache://")}
```

## 3. Where business logic lives

Django's own documentation, the most-cited community styleguide and the most-cited rebuttal to it disagree here, and all three are worth reading before choosing [5][6][7].

**Django's own philosophy is Active Record.** The design philosophies page states that "Models should encapsulate every aspect of an 'object,' following Martin Fowler's Active Record design pattern", and that "all the information needed to understand a given model should be stored *in* the model" [5]. The complementary rule for the other layers is that views should carry as little as possible and templates control "presentation and presentation-related logic, and that's it" [5].

**Bennett's argument against a service layer** is the strongest statement of the model-first position [7]. Its load-bearing claims: the models, plus "associated utility code, like custom `Manager` or `QuerySet` subclasses", "are the API exposed to other code", and therefore "are the place where the 'business logic' should be implemented"; swapping the data layer for something else "happens almost never" in his experience, so indirection built for that purpose never pays; and adopting a service layer means "whatever your business was previously, now your business is that plus developing and maintaining something close to your own private ORM" [7]. He also notes the collateral damage: generic views, auto-generated forms, DRF serializers and QuerySet caching all assume the ORM is visible, so hiding it behind a service boundary breaks them [7]. His closing advice to anyone committed to the pattern is to consider a Data Mapper ORM such as SQLAlchemy instead of fighting Django [7].

**HackSoft's Styleguide is the strongest statement of the opposite position** [6]. It places business logic in *services* ("functions, that mostly take care of writing things to the database") and *selectors* ("functions, that mostly take care of fetching things from the database"), names them `<entity>_<action>` for greppability, and explicitly forbids logic in model `save()`, in signals, in custom managers or querysets, in view methods and in DRF serializers, because that "fragments the business logic in multiple places, making it really hard to trace the data flow" [6]. Its validation rule is worth copying independently of the rest: "if you can do validation using Django's constraints, then you should aim for that", because "Less code to write, less code to maintain, the database will take care of the data even if it's being inserted from a different place"; `clean()` covers simple non-relational checks and the service calls `full_clean()` [6].

**The official forum thread has no official answer.** Practitioners there converge on a middle position: fat models "are okay at the start, but seem to become a problem the more the app grows"; "When you need to implement a process that writes to one model then I think a manager is the way to go", but "When you need to implement a process that needs to write to two or more models, then I think you might want to use the 'logic' module", whose payoff is that "The logic can be reused in views, api calls, celery tasks / cron jobs, management commands" [8]. Andrew Godwin reports having "used the 'services' type approach... with quite a bit of success" [8]; nothing in the thread establishes a Django project position [8].

**Decision rule that both sides actually agree on.** Single-model reads and writes belong on the model, manager or queryset, because that is where Django's own machinery can see them [5][7]. Multi-model writes, external side effects, and anything a view, a management command and a background task all need to call belong in a named function outside the model [6][8]. The disputed middle is single-model writes with a side effect; put those wherever the surrounding app already puts them, because consistency is worth more than the marginal argument [6][7].

```python
# billing/services.py  (illustrative)
from django.db import transaction

from billing.models import Invoice
from billing.notifications import notify_invoice_voided


@transaction.atomic
def invoice_void(*, invoice: Invoice, reason: str) -> Invoice:
    invoice.status = Invoice.Status.VOID
    invoice.void_reason = reason
    invoice.full_clean()
    invoice.save(update_fields=["status", "void_reason"])
    transaction.on_commit(lambda: notify_invoice_voided(invoice.pk))
    return invoice
```

The `on_commit` placement is not a style choice: callbacks registered with `transaction.on_commit()` run only after a successful commit, and "if the transaction is instead rolled back... the callback will be discarded, and never called" [16]. That is the correct way to make a side effect conditional on the write.

The queryset-first alternative for the read side, which both camps accept [6][7]:

```python
# billing/models.py  (illustrative)
from django.db import models


class InvoiceQuerySet(models.QuerySet):
    def outstanding(self):
        return self.filter(status=Invoice.Status.OPEN, balance__gt=0)

    def for_customer(self, customer_id):
        return self.filter(customer_id=customer_id)


class Invoice(models.Model):
    class Status(models.TextChoices):
        OPEN = "open"
        VOID = "void"

    objects = InvoiceQuerySet.as_manager()
```

## 4. The ORM at scale

Django's optimization page opens with a caveat worth quoting before any of the technique: "**All** of the suggestions below come with the caveat that in your circumstances the general principle might not apply, or might even be reversed", and "remember to profile after every change" [12].

**N+1 is now a configuration choice, not only a discipline.** Django 6.1 added fetch modes [2]. `FETCH_ONE` is the historical default and produces one query per instance for a missing field, the pattern "known as the 'N+1 queries problem'" [13]. `FETCH_PEERS` fetches the missing field "for the current instance and its 'peers', instances that came from the same initial `QuerySet`", and "can reduce most cases of the 'N+1 queries problem' to two queries without much effort" [13]. `FETCH_RAISE` raises `FieldFetchBlocked` and "can prevent unintentional queries in performance-critical sections of code" [13]. Fetch modes cover `ForeignKey`, `OneToOneField` and its reverse accessor, fields deferred by `defer()` or `only()`, and generic relations, and Django "copies the fetch mode of an instance to any related objects it fetches" [13]. Peers are tracked as weak references "to avoid memory leaks where some peer instances are discarded" [13]. The documented way to make it project-wide is a custom manager [12][13]:

```python
# catalog/models.py  (illustrative, Django 6.1 and later)
from django.db import models


class BookManager(models.Manager):
    def get_queryset(self):
        return super().get_queryset().fetch_mode(models.FETCH_PEERS)


class Book(models.Model):
    title = models.TextField()
    author = models.ForeignKey("Author", on_delete=models.CASCADE)

    objects = BookManager()
```

The docs are explicit that this does not retire the explicit tools: "When the `FETCH_PEERS` fetch mode is not appropriate or efficient enough, use `select_related()` and `prefetch_related()`" [12].

| Technique | What it does | Applies to | Trap |
|---|---|---|---|
| `select_related` | SQL join, related fields in the same `SELECT` [14] | `ForeignKey`, `OneToOneField` only [14] | Widens every row; useless for multi-valued relations [14] |
| `prefetch_related` | Separate lookup per relationship, joined in Python [14] | Also M2M, reverse FK, `GenericRelation` [14] | Filtering the relation afterwards issues a fresh query and ignores the prefetch [14] |
| `Prefetch(..., queryset=, to_attr=)` | Custom queryset per prefetch, result stored on a plain attribute [14] | Filtered or reordered prefetches [14] | `to_attr` bypasses the related-manager cache, so it is a list, not a manager [14] |
| `only` / `defer` | Restrict columns loaded [12] | Wide rows with large text fields [12] | "the ORM will have to go and get them in a separate query, making this a pessimization if you use it inappropriately" [12] |
| `iterator(chunk_size)` | Streams without the queryset cache [14] | One-pass scans over large result sets [14] | Prefetches are only honoured "as long as `chunk_size` is given"; the implicit default is 2000 when nothing is prefetched [14] |
| `values` / `values_list` | Dicts or tuples instead of model instances [12] | Read-only projections and template data [12] | Loses model methods and property access [12] |
| `count` / `exists` / `contains` | Push the question into SQL [12] | Existence and cardinality checks [12] | "If you are going to need other data from the QuerySet, evaluate it immediately" [12] |

`iterator()` streams via server-side cursors on Oracle and PostgreSQL, and on PostgreSQL "server-side cursors will only be used when the `DISABLE_SERVER_SIDE_CURSORS` setting is `False`"; MySQL "doesn't support streaming results", so its driver loads the entire result set into memory [14].

**Bulk operations trade signals for throughput.** `QuerySet.update()` and `delete()` "cannot call the `save()` or `delete()` methods of individual instances, which means that any custom behavior you have added for these methods will not be executed, including anything driven from the normal database object signals" [12]. `bulk_create()` carries its own list: `save()` is not called and `pre_save` / `post_save` are not sent, it does not work with multi-table-inheritance children or many-to-many relations, it casts the input to a list (fully evaluating a generator), and when the primary key is an `AutoField` or has a `db_default` the attribute "can only be retrieved on certain databases (currently PostgreSQL, MariaDB, and SQLite)" [14].

**Push work into the database where 5.2 and later let you.** `GeneratedField` computes a column in the database; its expression "should be deterministic and only reference fields within the model (in the same database table)", generated fields cannot reference other generated fields, and PostgreSQL before 18 only supports persisted columns while Oracle before 23ai only supports virtual ones [15]. `Field.db_default` sets a database-level default that "will be used when inserting rows outside of the ORM or when adding a new field in a migration", while a Python `default` still wins for instances created in Python [15]. `CompositePrimaryKey` (5.2) "must be defined as the model's `pk` attribute" [15]. Constraints are the Styleguide's first recommendation for validation, on the stated grounds of less code to write and less code to maintain [6].

**Locking.** `select_for_update(nowait, skip_locked, of, no_key)` locks rows to the end of the transaction, and evaluating it in autocommit mode raises `TransactionManagementError` "because the rows are not locked in that case" [14]. Two traps: on SQLite it silently has no effect [14], and inside a `TestCase` it "will (perhaps unexpectedly) pass without raising a `TransactionManagementError`" because `TestCase` already wraps the test in a transaction, so real coverage needs `TransactionTestCase` [14].

**Transactions.** `ATOMIC_REQUESTS` is simple but "makes it inefficient when traffic increases. Opening a transaction for every view has some overhead" [16]. Nested `atomic()` blocks create savepoints rather than new transactions [16]. The sharpest rule: "Avoid catching exceptions inside `atomic`", because after a `DatabaseError` "the transaction is broken" and any further query raises `TransactionManagementError`; catch *around* an inner `atomic()` block instead [16]. Use `durable=True` when a block must be the outermost one [16].

**Finding the N+1s.** `QuerySet.explain()` returns the execution plan and "is supported by all built-in database backends except Oracle", with PostgreSQL accepting `TEXT`, `JSON`, `YAML` and `XML`; the `ANALYZE` flag actually executes the query, which "could result in changes to data if there are triggers" [14]. django-silk records per-request query counts, times and stack traces, adds `@silk_profile` for blocks, and offers `SILKY_INTERCEPT_PERCENT` for sampling, but warns that "if Silk is used in production under heavy volume with large bodies this can have a huge impact on space/time performance" [43]. nplusone detects both lazy loads and eager loads that are never accessed, and `NPLUSONE_RAISE` makes those failures fatal in tests [44]; note before adopting it that its last PyPI release is 1.0.0 from 2018-05-21 [45], and that it says it "should only be used for development and should not be deployed to production environments" [44]. In tests, `assertNumQueries` is the in-tree equivalent and needs no dependency [39].

## 5. Migrations at scale

The zero-downtime sequence Django documents for adding a unique column to a populated table is four migrations, not one [17]:

1. Add the field as `null=True` and without `unique=True`, so no constraint is created up front [17].
2. A data migration that backfills, with `atomic = False` so the table is not held in one transaction: "For use cases such as performing data migrations on large tables, you may want to prevent a migration from running in a transaction by setting the `atomic` attribute to `False`" [17].
3. `AlterField` to make the column unique [17].
4. Deploy code that writes the column, only after the backfill is complete, because there is "a race condition if you allow objects to be created while this migration is running" [17].

```python
# billing/migrations/0042_backfill_invoice_uuid.py  (illustrative)
import uuid

from django.db import migrations, transaction


def backfill(apps, schema_editor):
    Invoice = apps.get_model("billing", "Invoice")
    while Invoice.objects.filter(uuid__isnull=True).exists():
        with transaction.atomic():
            for row in Invoice.objects.filter(uuid__isnull=True)[:1000]:
                row.uuid = uuid.uuid4()
                row.save(update_fields=["uuid"])


class Migration(migrations.Migration):
    atomic = False
    dependencies = [("billing", "0041_invoice_uuid")]
    operations = [migrations.RunPython(backfill, migrations.RunPython.noop)]
```

Three details in that snippet are load-bearing [17][18]. `apps.get_model()` gives the historical model, which is what keeps the migration replayable after the model class changes [18]. The batched `while` loop is Django's own documented shape for large tables under `atomic = False` [17]. Passing `migrations.RunPython.noop` as the reverse is what makes the migration reversible; "If this callable is omitted, migrating backwards will raise an exception" [18].

**PostgreSQL-specific lock avoidance** lives in `django.contrib.postgres.operations` [19]:

| Operation | Effect | Requirement |
|---|---|---|
| `AddIndexConcurrently` | `CREATE INDEX CONCURRENTLY` [19] | "The `CONCURRENTLY` option is not supported inside a transaction" [19] |
| `RemoveIndexConcurrently` | `DROP INDEX CONCURRENTLY` [19] | Same transaction restriction [19] |
| `AddConstraintNotValid` | Adds a constraint "but avoids validating the constraint on existing rows" [19] | Must be a separate migration from validation [19] |
| `ValidateConstraint` | "Scans through the table and validates the given check constraint on existing rows" [19] | Doing both in one atomic migration "has the same effect as `AddConstraint`" [19] |

The documented failure mode is worth restating: performing `AddConstraintNotValid` and `ValidateConstraint` "in a single non-atomic migration, may leave your database in an inconsistent state if the `ValidateConstraint` operation fails" [19].

```python
# billing/migrations/0043_invoice_uuid_index.py  (illustrative)
from django.contrib.postgres.operations import AddIndexConcurrently
from django.db import migrations, models


class Migration(migrations.Migration):
    atomic = False
    dependencies = [("billing", "0042_backfill_invoice_uuid")]
    operations = [
        AddIndexConcurrently(
            model_name="invoice",
            index=models.Index(fields=["uuid"], name="billing_invoice_uuid_idx"),
        )
    ]
```

**`SeparateDatabaseAndState`** is how you tell Django the state changed without letting it generate the DDL, the documented example being a rename executed as `RunSQL` while the state operations describe the new model shape [17]. It is the escape hatch for whenever the SQL you need and the SQL Django would emit differ [17].

**Squashing** has one real constraint and one real procedure [18]. The constraint: the optimizer cannot see through `RunSQL` or `RunPython` operations "unless they are marked as `elidable`" [18], so a history full of data migrations squashes badly. The procedure is two releases, not one: "squash, keeping the old files, commit and release, wait until all systems are upgraded with the new release... and then remove the old files, commit and do a second release" [18]. Transitioning a squashed migration to a normal one means deleting the replaced files, repointing dependents, and "Removing the `replaces` attribute" [18]. If a deleted migration's name might be reused, clear it from the migrations table with `migrate --prune` [18]. Note that `atomic` "doesn't have an effect on databases that don't support DDL transactions (e.g. MySQL, Oracle)", so the non-atomic technique above is a PostgreSQL-shaped answer [17].

## 6. The API layer

Both frameworks are current: DRF 3.18.1 shipped 2026-09-07 supporting Django 5.2, 6.0 and 6.1, having removed the deprecated CoreAPI support in 3.17.0 in favour of OpenAPI-based generation [20]; django-ninja advertises "Very high performance thanks to Pydantic and async support" and OpenAPI plus JSON Schema as its standards base [21].

| Axis | DRF | django-ninja |
|---|---|---|
| Validation engine | Serializers, pure Python [20] | Pydantic schemas [21] |
| Version 1 performance claim | No self-reported figure in the release notes [20] | "average project can gain some 10% performance increase on average, while some edge parsing/serializing cases can give you 4x boost" moving to Pydantic 2 [22] |
| Async | No async support announced in the release notes through 3.18.1 [20] | `async def` operations routed automatically alongside sync ones [23] |
| OpenAPI | Built-in generator, with drf-spectacular as the widely used fork [20][24] | Generated from type hints as a first-class feature [21] |
| Cross-cutting permissions | One `BasePermission` class applied across endpoints [25] | Decorator per operation, repeated per CRUD method [25] |
| Boilerplate for plain CRUD | A `ModelViewSet` measured at four lines in one head-to-head [25] | Roughly thirty lines for the same endpoints in that comparison [25] |

The table records two different bets rather than a ranking: the performance figure is django-ninja's own measurement of its Pydantic 1 to 2 move rather than a comparison against DRF [22], and no primary source in this survey publishes a DRF-versus-ninja throughput benchmark [20][21][22][25]. The boilerplate and permissions figures come from one practitioner comparison whose conclusion was "Django Ninja is not for me" for complex APIs, on the grounds that DRF centralises cross-cutting concerns better [25].

**Decision rule.** If the API is mostly model-shaped CRUD with per-object permissions and a large existing DRF investment, DRF's generic machinery is the cheaper path [25]. If the API is procedural, the payloads are hand-shaped, or the handlers are genuinely I/O-bound and you intend to run ASGI, django-ninja's typing and native async are the reason to switch [21][23].

**OpenAPI.** drf-spectacular generates OpenAPI 3.0.3, 3.1 and 3.2 from DRF, exists because the in-tree generator "is/was lacking" for non-toy schemas, and customises per view with `@extend_schema` [24]. Take its own stability warning seriously when pinning: it "deliberately stays below version 1.x.x to signal that every new version may potentially break you" [24].

Versioning, pagination and error-shape conventions are not repeated here; `appendix-api-design.md` in this corpus owns those, and both frameworks emit whatever contract you choose [20][21].

## 7. Async Django and background work

**What async buys and what it costs.** Async views "will still work under WSGI, but with a small per-request adaptation cost", and the benefit, "the ability to service hundreds of connections without using Python threads", requires deploying under ASGI [26]. Django quantifies the sync/async crossing itself: "tens of microseconds in the in-request ASGI path, where the running event loop is reused, and a few hundred microseconds in the cold-start path used by management commands, background tasks, and scripts" [26]. The corollary the same page draws: "If you find yourself wrapping individual rows or operations in a tight loop, restructure your code so the loop runs inside a single `sync_to_async()`... crossing" [26].

**The async ORM has two hard limits.** "Transactions do not yet work in async mode", and the recommendation is to "write that piece as a single synchronous function and call it using `sync_to_async()`" [26]. Persistent connections set through `CONN_MAX_AGE` "should also be disabled in async mode" [26]. Everything else is available through `a`-prefixed variants of the query methods and `async for` over querysets [26]. Calling a sync-only part of Django from a running event loop raises `SynchronousOnlyOperation` [26]; `DJANGO_ALLOW_ASYNC_UNSAFE` disables that check, and the docs say "do not use this in production environments" [26].

`sync_to_async` defaults to `thread_sensitive=True`, under which "the sync function will run in the same thread as all other `thread_sensitive` functions"; `thread_sensitive=False` spins a fresh thread per invocation [26]. That default is why naive parallelism across `sync_to_async` calls does not parallelise: the calls serialise on one thread unless you opt out, and opting out is what makes ORM use unsafe [26].

```python
# billing/views.py  (illustrative)
from asgiref.sync import sync_to_async
from django.http import HttpResponse

from billing.models import Invoice
from billing.services import invoice_void


async def void_invoice_view(request, pk: int) -> HttpResponse:
    invoice = await Invoice.objects.aget(pk=pk)
    # Transactions do not work in async mode: cross once, not per row.
    await sync_to_async(invoice_void)(invoice=invoice, reason="duplicate")
    return HttpResponse(status=204)
```

django-ninja documents the same shape and adds one ORM-specific trap: because "Django querysets are lazily evaluated", wrapping a queryset-returning call in `sync_to_async()` does not move the query off the event loop; force evaluation with `list()` or use `async for` [23].

**Channels is for protocols, not for speed.** It "wraps Django's native asynchronous view support, allowing Django projects to handle not only HTTP, but protocols that require long-running connections too", and adds integrations with Django's auth and session systems plus the consumer abstraction [31]. Channel layers "are an optional part of Channels" [31]. Plain async views need none of it [31].

**The Tasks framework is an interface, not a queue.** Django 6.0 added `django.tasks`: a `@task` decorator on module-level functions, `enqueue()` / `aenqueue()`, a `TASKS` setting, and `TaskResult` [28]. The sentence that determines every architecture decision downstream is in the release notes: "Django handles task creation and queuing, but does not provide a worker mechanism to run tasks. Execution must be managed by external infrastructure, such as a separate process or service" [3]. The shipped backends are `ImmediateBackend` (runs inline, for development and tests) and `DummyBackend` (stores results without executing); "Production systems should rely on backends that supply a worker process and a durable queue implementation" [28].

DEP 14 states the intent plainly: "This proposal doesn't seek to replace existing tools... The primary motivation is creating a shared API contract between worker libraries and developers", because "Writing a production-ready task runner is a complex and nuanced undertaking" [27]. It lists as explicitly out of scope for the first pass: completion and failure hooks, bulk queueing, automated task retrying, "A generic way of executing task runners", and observability into task queues [27]. The DEP describes a `DatabaseBackend` as the getting-started path [27]; that backend did not ship in 6.0 [3][28], and Django 6.1's Tasks changes were limited to `task()` accepting `**kwargs` forwarded to the backend's `task_class` and to `Task` and `TaskResult` becoming picklable [2]. Practitioner reaction has focused on exactly that gap: "There is no worker process and no production-ready backend" [29]. The third-party `django-tasks` package (0.12.0, 2026-02-06) is the reference implementation that supplies one [30].

Because "both Task arguments and return values are serialized to JSON, they must be JSON-serializable", and "complex objects such as model instances, or built-in types like `datetime` and `tuple` cannot be used in Tasks without additional conversion" [28]. Enqueue from inside `transaction.on_commit()` so a worker cannot pick the task up before the row it needs exists [28].

```python
# billing/tasks.py  (illustrative, Django 6.0 and later)
from functools import partial

from django.db import transaction
from django.tasks import task

from billing.models import Invoice


@task(queue_name="billing")
def invoice_email_send(invoice_id: int) -> None:
    ...


def invoice_issue(*, customer_id: int) -> Invoice:
    with transaction.atomic():
        invoice = Invoice.objects.create(customer_id=customer_id)
        transaction.on_commit(partial(invoice_email_send.enqueue, invoice.pk))
    return invoice
```

**Choosing.** Use `django.tasks` as the call-site API regardless, because it is the contract third-party backends are converging on [27]. Behind it, pick a backend by what DEP 14 left out: if you need retries, scheduling, chaining or queue observability, that is the territory of a dedicated worker library (DEP 14 names Celery and RQ; django-q2 and Dramatiq occupy the same space), because none of those capabilities are in Django's first pass [27][29].

## 8. Multi-tenancy

Each row below states the isolation model as its own documentation describes it, with the operational consequences that follow from it [33][34][36][37]:

| Model | Isolation | Migration cost | Where it breaks |
|---|---|---|---|
| Database per tenant [36] | Strongest; separate databases [36] | One `migrate` run per database, plus routing [36] | Connection count and operational surface grow linearly [36] |
| Schema per tenant (`django-tenants`) [33] | Per-tenant PostgreSQL schema selected by `search_path` [33] | `migrate_schemas` walks every tenant schema [34] | "for a large number of tenants (thousands) the schema approach might not be feasible, and as of now, there is no clear way for implementing tenant sharding" [36] |
| Row-level tenant key [37] | Application-enforced filter on a `tenant_id` column [37] | One `migrate` for everyone [37] | A single missed filter leaks across tenants [37] |

**Schema per tenant.** django-tenants stores tenants in a table on the `public` schema and matches on hostname: "Whenever a request is made, the host name is used to match a tenant in the database. If there's a match, the search path is updated to use this tenant's schema" [33]. `migrate_schemas` "will also respect the `SHARED_APPS` and `TENANT_APPS` settings, so if you're migrating the `public` schema it will only migrate `SHARED_APPS`. If you're migrating tenants, it will only migrate `TENANT_APPS`" [34]. For large tenant counts it offers `--executor=multiprocessing`, tuned by `TENANT_MULTIPROCESSING_MAX_PROCESSES` (default 2) and `TENANT_MULTIPROCESSING_CHUNKS` (default 2) [34]; per-tenant management commands run through `tenant_command` and `all_tenants_command` [34]. `TENANT_LIMIT_SET_CALLS` makes the library "set the search path only once per request", which matters because setting it on every database operation is the documented cost of this design [34]. One sharp edge: "Schema names and domain names have different validation rules. Underscores (`_`) and capital letters are permitted in schema names but they are illegal for domain names", and domains may contain a dash which schema names may not [33]. django-tenants 3.14.0 (2026-08-05) carries Django 5.2, 6.0 and 6.1 classifiers, so it tracks current Django [35].

django-pgschemas frames the same tradeoff in its own terms: the schema approach wins on "Simplicity: barely make any changes to your current code to support multi-tenancy. Plus, you only manage one database" and on "Performance: make use of shared connections, buffers and memory", and loses on scalability at thousands of tenants [36].

**Row level.** django-multitenant implements the shared-table model: models inherit `TenantModel` and declare `TenantMeta.tenant_field_name`, `set_current_tenant()` scopes the request, and thereafter filters, joins, updates, aggregates and subqueries are scoped automatically; `TenantForeignKey` replaces `ForeignKey` so the database can enforce composite foreign keys including `tenant_id` [37]. Check currency before adopting: its latest PyPI release is 4.1.1 from 2023-12-18, with classifiers topping out at Python 3.11 and no Django 5.x or 6.x classifier [38]. A hand-rolled equivalent (middleware setting a context variable plus a default manager that filters on it) is the common alternative and has the same failure mode, which is that every raw SQL statement, management command and data migration "start with access to everything and must explicitly narrow down" [36].

**Choosing.** Regulated tenants who will ask what physically separates their rows are the case that justifies schema-per-tenant or database-per-tenant [36]. Thousands of small tenants, or any expectation of sharding, argue for row-level [36]. The migration cost is asymmetric: row-level to schema-per-tenant is a known path, and the reverse is not documented anywhere in this survey [36].

## 9. Testing

**Pick the base class by what you are testing, not by habit.** "A `TransactionTestCase` resets the database after the test runs by truncating all tables", while "A `TestCase`, on the other hand, does not truncate tables after a test. Instead, it encloses the test code in a database transaction that is rolled back at the end of the test" [39]. `TestCase` is therefore faster, and `TransactionTestCase` is required when the behaviour under test *is* transactional: "you cannot test that a block of code is executing within a transaction, as is required when using `select_for_update()`" [39]. In a multi-database setup the `databases` attribute (or `'__all__'`) declares which databases must be flushed [39]. `serialized_rollback` restores initial data per test case but "will slow down that test suite by approximately 3x" [40].

**Fixtures.** `setUpTestData()` creates class-level data once per `TestCase` and "allows for faster tests as compared to using `setUp()`", with two caveats: the objects "must support creating deep copies with `copy.deepcopy()`", and on a database without transaction support it degrades to running before every test [39]. factory_boy is the usual complement: `DjangoModelFactory` is the required base for Django model factories, and its `django_get_or_create` option has one trap worth memorising, that "any new values passed to the Factory are **not** used to update an existing model" [42]. The Styleguide's file layout mirrors the code layout, with `tests/models/`, `tests/selectors/` and `tests/services/`, and reserves exhaustive coverage for services while mocking "async task calls & everything that goes outside the project" [6].

**Query budgets are the cheapest regression guard you own.** `assertNumQueries` "Asserts that when `func` is called with `*args` and `**kwargs` that `num` database queries are executed", and works as a context manager [39]:

```python
# catalog/tests/test_book_list.py  (illustrative)
from django.test import TestCase

from catalog.models import Book


class BookListQueryBudget(TestCase):
    def test_list_stays_within_budget(self):
        with self.assertNumQueries(2):
            titles = [b.author.name for b in Book.objects.all()]
        self.assertIsInstance(titles, list)
```

`captureOnCommitCallbacks(execute=True)` is the companion for anything registered through `transaction.on_commit()`, since a plain `TestCase` never commits: it "captures `transaction.on_commit()` callbacks" and, with `execute=True`, runs them on exit "if no exception occurred", emulating a commit [39]. Without it, every `on_commit` side effect is silently untested [39].

**pytest-django.** "By default your tests will fail if they try to access the database"; the `django_db` marker grants access, and the default mode "rolls back transactions, to isolate tests from each other" [41]. `@pytest.mark.django_db(transaction=True)` is the `TransactionTestCase` equivalent, and "The downside of this is that these tests are much slower to set up due to the required flushing of the database" [41]. `databases=[...]` or `'__all__'` selects databases; `--reuse-db` preserves the test database between runs and `--reuse-db --create-db` forces a rebuild after schema changes [41].

**Async tests.** "your tests must be `async def` methods on the test class... Django will automatically detect any `async def` tests and wrap them so they run in their own event loop" [39]. `AsyncClient` mirrors the sync client but every request must be awaited, headers skip the `HTTP_` prefix, and views receive an `ASGIRequest` [39]. Decorators that are not async-aware must be applied with `async_to_sync()` *inside* them on the test method [39].

## Agreed vs folklore (compressed)

**Agreed** (multiple independent primary sources): profile before optimising, and re-profile after every change [12] · N+1 is the query pattern Django's own docs name and 6.1 turned into a configurable fetch mode rather than a discipline problem [2][12][13] · bulk operations skip `save()` and signals, always [12][14] · large-table schema change is a multi-migration sequence, never one migration [17][19] · `on_commit` is the correct place for side effects that must not fire on rollback [16][28] · Django's Tasks framework ships no worker, by design and by DEP [3][27][28] · async transactions do not exist yet [26] · schema-per-tenant does not scale to thousands of tenants [36].

**Folklore, rebutted by primary sources**: "fat models always" (Django's philosophy is Active Record [5], but practitioners in Django's own forum report fat models becoming a problem as the app grows [8], and the Styleguide's counter-position is a documented production practice [6]) · "you need a service layer for everything" (Bennett's objection that you end up "developing and maintaining something close to your own private ORM" is the strongest argument against blanket adoption, and neither camp disputes that single-model logic belongs on the model [5][7]) · "DRF is too slow" (the only published figure in this survey is django-ninja's self-reported 10% average and 4x edge-case gain from moving to Pydantic 2, which measures ninja against itself, not against DRF [22]) · "Django 6.0 replaced Celery" (the release notes say the opposite, and DEP 14 lists retries, bulk queueing and observability as out of scope [3][27]).

**Asserted without published measurement**: the widely repeated claim that async Django now matches FastAPI throughput traces to a practitioner post that presents no throughput, latency or load-test figures of its own [32]. Treat it as an untested hypothesis, and measure your own stack [26][32].

**White space** (verified absent): DEP 14's `DatabaseBackend` is described in the accepted DEP [27] but is not among the backends Django 6.0 or 6.1 ship [3][28][2] · no primary source in this survey publishes a DRF-versus-django-ninja throughput benchmark [20][21][22][25] · no documented path from schema-per-tenant back to row-level tenancy [36] · nplusone, still the best-known N+1 detector, has had no release since 2018 [45].

## Synthesis (inferred)

**A decision rule for where logic lives.** Score the operation on three questions: does it write more than one model; does it have an effect outside the database; is it called from more than one entry point. Zero yeses means it belongs on the model, manager or queryset, where Django's generic views, forms and serializers can still see it. Two or more yeses means it belongs in a named module-level function with keyword-only arguments, wrapped in `atomic`, registering side effects through `on_commit`. One yes is a coin flip, and the tiebreaker is whatever the surrounding app already does. This rule reconciles the two camps rather than choosing between them: Bennett's objection is to blanket indirection, not to named functions, and the Styleguide's value is the naming convention and the greppability, not the layer itself.

**A starter layout for a new enterprise Django service.**

```text
config/
  settings/{base,local,production}.py   # switches only; values come from env
  asgi.py wsgi.py urls.py
<capability>/                           # one app per business capability
  models.py            # fields, constraints, TextChoices, thin properties
  managers.py          # queryset methods for reusable reads
  services.py          # multi-model writes, side effects, one per use case
  selectors.py         # reads that span relations or apply visibility rules
  apis.py              # HTTP layer only: parse, call one service, render
  tasks.py             # @task wrappers; bodies delegate to services.py
  tests/{models,selectors,services,apis}/
```

Decisions I would make on day one and not revisit: PostgreSQL only, so the concurrent-index and not-valid-constraint operations stay available; every cross-app relation reviewed as a boundary commitment, because removing one later is a multi-migration exercise; `FETCH_PEERS` set through a base manager if you are on 6.1, and `assertNumQueries` budgets on the three hottest list endpoints if you are not; a query budget in CI from the first week, because retrofitting one to an existing suite means triaging every existing regression at once.

**On the version line.** Track feature releases, not LTS, unless a vendored dependency forces otherwise. The 5.2-to-6.2 jump in April 2027 bundles a Python floor move, a `DEFAULT_AUTO_FIELD` change, three dropped database versions and the `EMAIL_*` to `MAILERS` transition into one window; taking them one release at a time is cheaper than taking them together, and it is the only way to have fetch modes and database-level cascades before 2027.

**On background work.** Adopt `django.tasks` as the call-site API immediately, even if the backend behind it is Celery, because the DEP's whole purpose is to make that swap cheap later. Keep task bodies to three lines: load by id, call a service, return. The JSON-only argument constraint is a feature in that shape, since it forces the id-not-object discipline that makes tasks safe to retry.

**Where this doc stops.** Caching, connection pooling, security headers, rate limiting, logging and deployment topology are deliberately absent; they belong to the Django production hardening doc, and the two together are the full picture.
