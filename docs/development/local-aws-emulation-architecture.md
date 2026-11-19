---
title: "Local AWS Emulation as the Developer Lane: MiniStack, AWS Emulator, Floci and the Sandbox-Account Gate"
category: infra
languages: [bash, yaml, hcl, python]
complexity: advanced
use_cases:
  - designing the local development and CI architecture for an organization that hosts on AWS, as the counterpart to the corpus's Cloudflare Workers lane
  - choosing between MiniStack, Floci, the official AWS Emulator, LocalStack, moto and the service-specific tools (DynamoDB Local, SAM local, SST Live) after the 2026 LocalStack licence change
  - "deciding which tests may run against an emulator and which must run in a real sandbox account: the IAM, quota, networking and fidelity boundary"
  - wiring SDKs, Terraform and CDK at an emulator safely (AWS_ENDPOINT_URL precedence, allowed_account_ids, no real credentials in emulator jobs)
  - running an emulator in GitHub Actions and Testcontainers with per-test isolation, and adding an aws lane to the mx router
summary: "State of practice as of 2026-11-18. LocalStack archived its repo and moved to a token-gated commercial image in March 2026, MiniStack (MIT, 60+ services, Docker sidecars for RDS/ElastiCache/ECS) and Floci (MIT, about 32k stars) appeared as drop-in port-4566 replacements, and AWS shipped its own agent-built AWS Emulator on 2026-11-11 (CC0, 355 services scored, 72% of 46k published fidelity tests passing at v1.0.512, no IAM evaluation, one account and region). The recommended architecture for an AWS-hosted organization is three lanes: an emulator in the inner loop and in pull-request CI for data-path logic, and a real sandbox account as the gate for IAM, quotas, networking and anything below about 80% fidelity. Measured here: MiniStack and Floci both reach healthy in about 4 s at 25 to 36 MiB idle and return AWS-shaped errors for S3, SQS and DynamoDB; both accept any credentials as the account root. Against the Cloudflare lane, AWS has no local copy of its production runtime, so the real account carries more of the confidence."
provenance: researched
researched: 2026-11-19
sources:
  - https://ministack.org/
  - https://ministack.org/docs/architecture
  - https://ministack.org/docs/configuration
  - https://ministack.org/docs/limitations
  - https://ministack.org/docs/migrating-from-localstack
  - https://ministack.org/docs/iac
  - https://ministack.org/docs/services/
  - https://github.com/ministackorg/ministack
  - https://github.com/ministackorg/testcontainers-ministack
  - https://ministack.org/blog/aws-emulator
  - https://aws.amazon.com/blogs/aws/test-your-applications-against-aws-locally-with-aws-emulator/
  - https://docs.aws.amazon.com/emulator/latest/userguide/what-is-service.html
  - https://docs.aws.amazon.com/emulator/latest/userguide/fidelity.html
  - https://github.com/aws/aws-emulator
  - https://github.com/aws/aws-emulator/releases/download/v1.0.512/scores.json.gz
  - https://github.com/aws/aws-emulator-evaluation-suite
  - https://blog.localstack.cloud/the-road-ahead-for-localstack/
  - https://blog.localstack.cloud/2026-upcoming-pricing-changes/
  - https://blog.localstack.cloud/localstack-for-aws-release-2026-03-0/
  - https://www.localstack.cloud/pricing
  - https://docs.localstack.cloud/aws/developer-tools/security-testing/iam-policy-enforcement/
  - https://github.com/localstack/localstack
  - https://github.com/localstack/aws-cdk-local
  - https://github.com/localstack/terraform-local
  - https://github.com/floci-io/floci
  - https://github.com/localemu/localemu
  - https://codenote.net/en/posts/localstack-archived-oss-alternatives-comparison/
  - https://docs.getmoto.org/en/latest/docs/server_mode.html
  - https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/DynamoDBLocal.UsageNotes.html
  - https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/using-sam-cli-local-start-api.html
  - https://docs.aws.amazon.com/step-functions/latest/dg/test-and-debug.html
  - https://sst.dev/docs/live
  - https://github.com/minio/minio
  - https://docs.aws.amazon.com/lambda/latest/dg/testing-guide.html
  - https://docs.aws.amazon.com/wellarchitected/latest/framework/fnd01-bp04.html
  - https://docs.aws.amazon.com/prescriptive-guidance/latest/choosing-git-branch-approach/sandbox-environment.html
  - https://docs.aws.amazon.com/sdkref/latest/guide/feature-ss-endpoints.html
  - https://github.com/hashicorp/terraform-provider-aws/blob/main/website/docs/guides/custom-service-endpoints.html.markdown
  - https://github.com/hashicorp/terraform-provider-aws/blob/main/website/docs/index.html.markdown
  - https://docs.github.com/en/actions/tutorials/use-containerized-services/use-docker-service-containers
  - https://developers.cloudflare.com/workers/local-development/
  - https://developers.cloudflare.com/changelog/post/2025-09-16-remote-bindings-ga/
  - https://developers.cloudflare.com/workers/versions-and-deployments/version-urls/
  - https://news.ycombinator.com/item?id=47593285
  - https://news.ycombinator.com/item?id=47493657
  - https://www.infoq.com/news/2026/02/localstack-aws-community
  - docs/development/mx-cloudflare-deploy.md (corpus)
  - docs/development/INFRA_CONFIG.md (corpus)
  - crates/mx-lib/src/infra/config.rs, crates/mx-cli/src/commands/infra.rs, crates/mx-cli/src/commands/new.rs (mx repo)
  - https://aws.amazon.com/blogs/developer/new-improved-flexibility-when-configuring-endpoint-urls-with-the-aws-sdks-and-tools
  - docs/development/RECIPE_AUTHORING_GUIDE.md (corpus)
  - https://pypi.org/project/moto/
---

# Local AWS Emulation as the Developer Lane

State of practice as of 2026-11-18. Inline `[n]` cites key to the `sources` list in the frontmatter. "Verified here" marks measurements made for this doc: MiniStack 1.5.42 and Floci (`latest` on 2026-11-18 for the startup and memory rows, the pinned 2.5.0 on 2026-11-19 for the behaviour rows) were run as plain Docker containers (Docker 29.8 under OrbStack on an arm64 Mac) and driven with AWS CLI 2.27.38; the AWS Emulator was not run (on macOS it needs an image built from AWS's Dockerfile) but its published fidelity file for v1.0.512 was downloaded and summed. The Terraform block passed `terraform validate` against `hashicorp/aws` v6.73.0 under Terraform 1.15.5, the compose block passed `docker compose config`, and the Python and bash blocks compile; the Java block is quoted from the Testcontainers module README and was not compiled. Hacker News comments are quoted as practitioner signal and marked as such. The mx-specific proposal at the end sits under Synthesis (inferred).

## 1. Why this doc exists now: the 2026 shake-up

For a decade "local AWS" meant LocalStack. That ended in three steps. On 2025-12-18 LocalStack announced the Community image would stop receiving updates after March 2026 and that a single image "will require authentication via an auth token for use" [17]. Release 2026.03.0 (2026-03-23) made `localstack/localstack` and `localstack/localstack-pro` the same token-gated image; a bypass variable worked until 2026-04-06, and state from earlier versions cannot be reused [19]. The GitHub repository was archived the same day (the Hacker News thread reporting it is dated 2026-03-23, as is the repository's last push) [45][22]; it still carries about 65k stars and its licence file is still Apache-2.0, so "LocalStack went BSL" (a claim found only in secondary sources) is not supported by the primary evidence: the code that exists is Apache [22], the new distribution is commercial [19][20]. Current plans: Hobby (free, non-commercial use only), Base ($39 per user per month billed annually, $45 monthly, with "basic IAM policy enforcement"), Ultimate ($89 per user per month billed annually) and Enterprise (custom) [20]. CI minutes became unlimited on every plan subject to fair use, but an organization's CI still needs paid seats because Hobby forbids commercial use [18][20]. InfoQ quoted the founders' reasoning as needing "a distribution model that lets us engage directly with users" [46]; the 216-point Hacker News thread split between calling it a rug pull and calling the complaint "a sense of perpetual entitlement over the work of others", with one commenter who described their company as paying customers calling the Cloud Pods and ephemeral-instance features "pretty half-baked" [45].

Three kinds of replacement appeared within weeks:

| Option | Licence | Shape | Status on 2026-11-18 |
|---|---|---|---|
| **MiniStack** | MIT | Python/hypercorn, one process on port 4566, 60+ services, real Docker sidecars for RDS, ElastiCache, ECS, EKS, Lambda, OpenSearch; multi-account and multi-region state | repo created 2026-03-24 (the day after the archive), v1.5.42 released 2026-11-18, ten releases between 2026-10-26 and 2026-11-18, about 5.2k stars, 549 forks, 5.2M Docker Hub pulls, two maintainers with 518 and 265 commits then a long tail [1][2][7][8] |
| **Floci** | MIT | Quarkus native binary or container on port 4566, "drop-in replacement: swap the image and keep going", also a native binary for Linux, macOS and Windows | repo created 2026-02-18, about 31.9k stars and 3.5k forks, pushed 2026-11-18 [25]; compared with the other replacements in [27] |
| **LocalEmu** | Apache-2.0 fork of the archived Community code | `pip install`, image `localemu/localemu`, claims 133 services | created 2026-04-06, about 228 stars, pushed 2026-11-06 [26] |
| **AWS Emulator** (official) | CC0-1.0 (`vendor/` Apache-2.0) | Rust, HTTPS on `127.0.0.1:9297`, built from the public API models, written and maintained by AI agents, one account and one region, no IAM evaluation | GA 2026-11-11; about 9.3k stars; several releases per day (v1.0.504 to v1.0.512 all on 2026-11-18) [11][12][14] |
| **moto** server mode | Apache-2.0 | `motoserver/moto` on port 5000 with a `/moto-api/reset` endpoint | active, v5.2.5 on 2026-10-27 [28][52] |
| **MinIO** as an S3 stand-in | AGPL-3.0 | | archived; do not start new work on it [33] |

MiniStack's own homepage comparison claims about 2 s startup, about 30 MB idle and a 280 MB image against LocalStack's 15 to 30 s, 500 MB and 1 GB, and tags several services (EMR, EBS, EFS, ALB, WAFv2, Batch, EKS) as "Pro-only on LocalStack" [1]. The homepage also carries a banner pointing at the official AWS Emulator, and MiniStack's blog post on it is a neutral summary of AWS's announcement rather than a head-to-head [1][10].

## 2. The architecture: three lanes, one boundary

AWS's own position is the starting constraint. The Lambda testing guide's best-practice heading is "Use emulators sparingly"; it says "You cannot fully replicate a cloud environment locally. This includes queues, database tables, event buses, and security policies", and that when you do use emulation "you must still test in the cloud to verify configuration and to test interactions with cloud services that can only be simulated or mocked" [34]. Its worked example is the false confidence an emulator gives on IAM: the local call "does not originate from the Lambda service", so a function whose role lacks `s3:CreateBucket` passes locally and "likely fails when deployed" [34]. The same guide lists the other classes emulation misses: SQS visibility timeouts, payloads over the size quota, Lambda timeout and memory settings, and "production security policies, inter-service configurations, or exceeding Lambda quotas" [34].

The Well-Architected Framework supplies the other half: FND01-BP04 recommends a sandbox account per developer with few preventative controls, detective guardrails and cost controls, plus ephemeral pull-request environments that clean themselves up, and rates the risk of not doing so "Medium" [35]. Prescriptive Guidance defines the sandbox as where developers have "full access" and deploy from their own workstation before the shared dev environment [36].

Put together, the architecture for an AWS-hosted organization is:

| Lane | Runs where | What it is for | What it must not be trusted for |
|---|---|---|---|
| **Inner loop** | emulator on the workstation (via the mx router, section 10) | fast iteration on data-path code: S3 objects, SQS/SNS flows, DynamoDB access patterns, EventBridge rules, Lambda handler wiring, Step Functions definitions | IAM, quotas, networking, cross-account, anything scoring low in section 4 |
| **Pull-request CI** | emulator as a service container or Testcontainer | the same integration suite on every push, with no AWS credentials in the job | same as above |
| **sandbox gate** | per-developer sandbox account or ephemeral per-PR stack in a real non-production account | execution-role policies, resource policies, service quotas, VPC and DNS, cross-region and cross-account behaviour, CloudFormation and CDK deploy paths, anything the emulator returns `NotImplemented` for | nothing; this is the confidence source |

The boundary rule: **a test moves from the emulator lanes to the sandbox gate the moment its pass depends on IAM, a quota, a network path, a region or account boundary, or a service whose published fidelity is below the team's threshold** (section 4 gives numbers). Everything else runs in the emulator lanes because they are free, fast and credential-free.

## 3. Choosing the emulator

Measured here, same machine, same Docker daemon, `docker run -d -p <port>:4566 <image>` then polling the health endpoint every 250 ms:

| | MiniStack 1.5.42 | Floci (2.5.0 for behaviour rows) |
|---|---|---|
| container start to healthy | 4.55 s | 4.02 s |
| memory after 3 s idle (`docker stats`) | 36.0 MiB | 24.7 MiB |
| image size on disk (`docker image inspect`) | 174 MB | 314 MB |
| `aws s3 mb` / `cp` / `head-object` | ok, ETag is the MD5 | ok |
| `aws sqs create-queue` / `send` / `receive` | ok, queue URL carries account `000000000000` | ok |
| DynamoDB `put-item` with a key of the wrong type | `ValidationException: ... Type mismatch for key pk expected: S actual: N` (AWS wording) | identical wording |
| DynamoDB conditional put on an existing item | `ConditionalCheckFailedException: The conditional request failed` | identical |
| S3 `get-object` on a missing key | `NoSuchKey` | identical |
| SQS `send-message` to a FIFO queue without `MessageGroupId` | `MissingParameter: The request must contain the parameter MessageGroupId.` | identical |
| `sts get-caller-identity` with access key `test` | `arn:aws:iam::000000000000:root` | identical |
| a 12-digit access key as a second account | bucket list empty for `111111111111`, present for `test`: accounts are isolated | identical; two 12-digit accounts each created a bucket named `uploads` |
| `sqs list-queues --region eu-west-1` after creating in `us-east-1` | empty: regions are isolated | identical |
| state reset | `POST /_ministack/reset` answers `{"reset": "ok"}`, bucket list empty afterwards | no reset endpoint found; recreate the container (memory mode) or drop its volume |
| health endpoint | `/_ministack/health`, also `/_localstack/health` | `/_floci/health`, also `/_localstack/health` |

The startup numbers include Docker's own container start, which is why they exceed MiniStack's "~2s" claim; both are several times faster than the 15 to 30 s MiniStack attributes to LocalStack, a figure not re-measured here [1]. MiniStack's documented shape: all state is in-memory Python dicts with persistence off by default and three opt-in layers (`PERSIST_STATE=1` JSON snapshots, `S3_PERSIST=1` object bytes, `RDS_PERSIST=1` named volumes), with the docs advising all three on for local dev and all off for CI [3]. A 12-digit `AWS_ACCESS_KEY_ID` becomes the account id; non-numeric keys fall back to `000000000000`; state is per region from the SigV4 scope except S3 buckets (global, home-region redirect) and Aurora DSQL [4][8]. IAM policies are evaluated only under `AUTH=true`, which also turns on SigV4 validation [3][4]. The services page lists "60+ services, 10 fully featured, 45+ partial"; the limitations page lists four classes of gap: events "stored but not dispatched" (CloudWatch alarms to non-SNS targets, EventBridge API-destination retries, Step Functions logging), metadata-only services (EC2 networking, CloudFront caching, EFS, EMR, WAFv2 enforcement, Batch jobs auto-succeed), "impossible locally" (VPC networking, DNS propagation, SMTP delivery, a real Kubernetes control plane), and by-design leniency in validating optional fields [4][7]. What MiniStack lacks relative to LocalStack Pro is the developer-experience layer: no hot reload, IDE plugins, Cloud Pods or extensions API [5]. The official Testcontainers module is `org.ministack:testcontainers-ministack:0.1.5` [9].

**The official AWS Emulator** is a different animal. It "supports the APIs of AWS services that have an API model in the AWS API models repository", aims for "the same results as AWS for valid calls, and the same errors for invalid ones", and returns `HTTP 501 NotImplemented` for anything it has not implemented "so your tests fail right away" [11][13]. It requires SigV4 on every request, generates `EMULATOR`-prefixed keys at each start and rejects any other key, uses the single account `000000000000`, and sends anonymous usage data by default [11][12]. AWS states the limits itself: "The emulator doesn't evaluate AWS Identity and Access Management (IAM) policies and resource-based policies the way AWS does. Don't use it to test security"; it "accepts any Region and account ID in a request, but it doesn't keep resources separate by Region or account"; behaviour is checked against `us-west-2` only, except global services, which are checked in `us-east-1`; everything is in memory and "the emulator deletes all resources when it stops"; and "We haven't tested the emulator for long-running use or load testing" [12]. It is "built and maintained by agents from the AWS API models" with updates "many times a day" [11][12]. Linux gets RPM and Debian packages; macOS and Windows build a Docker image from AWS's Dockerfile because "We don't publish a ready-made image"; Rust tests can embed it in-process [11][12]. "AWS Emulator has no additional service fee" and needs no AWS account [12].

**Decision.** Pick the port-4566 family (MiniStack or Floci) as the default emulator for the two emulator lanes: both are a one-line swap for existing LocalStack configuration, both run as a plain container with no token, and MiniStack adds Docker-backed RDS, ElastiCache, ECS, EKS and Lambda sidecars that nothing else offers free [1][5][25]. Prefer Floci when the team wants a native binary or the larger community; prefer MiniStack when the real-database sidecars or a one-call reset endpoint matter. On every behaviour row measured above the two were indistinguishable, so the choice rests on those differences and on maintenance risk, not on fidelity. The mx `aws` lane (section 10) defaults to Floci 2.5.0 and keeps the image a single variable. Run the AWS Emulator alongside for control-plane and error-shape checks, where its published fidelity is strongest, and watch its scores: it is the only one of the three that publishes a test-by-test conformance file, and that file is the best available proxy for how much any emulator can be trusted per service (section 4). Do not start new pipelines on LocalStack's Hobby plan (non-commercial only) or on frozen Community images (no patches) [18][20].

## 4. What emulation cannot tell you, with numbers

The AWS Emulator publishes, with every release, a `scores.json.gz` asset listing each fidelity test (a "unit") with its outcome, the services it touches, its kind ("behavioral" units assert a workflow's results and state, "wire" units assert request and response shape) and the failing step [13][15]. Summed here from the v1.0.512 file (scored 2026-11-18T17:03Z, commit `1591f1d`): 46,026 units, 33,227 pass, 12,712 fail, 87 skipped, so **72.2% overall** (72.3% with the skipped units excluded); behavioral units pass at 78.7% and wire units at 53.0%; counting only the 38,782 units that touch a single service, 355 services are scored and the median pass rate among those with at least 20 units is 85.3% [15]. Per service, single-service units only, sorted by pass rate:

| Service | Pass | Service | Pass |
|---|---|---|---|
| SNS | 281/296 = 94.9% | Step Functions | 501/626 = 80.0% |
| CloudWatch | 790/865 = 91.3% | STS | 146/186 = 78.5% |
| KMS | 305/335 = 91.0% | DynamoDB | 1647/2100 = 78.4% |
| Secrets Manager | 212/236 = 89.8% | OpenSearch | 279/370 = 75.4% |
| API Gateway v1 | 737/823 = 89.6% | ECS | 473/639 = 74.0% |
| S3 | 1157/1295 = 89.3% | ElastiCache | 138/196 = 70.4% |
| EventBridge | 275/308 = 89.3% | API Gateway v2 | 209/312 = 67.0% |
| SQS | 320/359 = 89.1% | RDS | 320/516 = 62.0% |
| Kinesis | 235/266 = 88.3% | Lambda | 498/810 = 61.5% |
| IAM (control plane) | 637/726 = 87.7% | EC2 | 1111/1815 = 61.2% |
| Cognito user pools | 750/856 = 87.6% | SSM | 298/489 = 60.9% |
| CloudWatch Logs | 581/667 = 87.1% | CloudFormation | 488/2639 = 18.5% |

CloudFormation at 18.5% is the standout, which is why the sandbox gate owns deploy paths (section 10 draws the general lesson). AWS's own framing: "Today, the emulator matches AWS most closely for control plane APIs. Fidelity for data plane APIs, and support for infrastructure as code tools such as AWS CloudFormation, Terraform, and the AWS CDK, varies by service" [13]. The IAM row measures the IAM control plane (create role, attach policy), not policy evaluation, which the emulator does not do [12].

MiniStack publishes no equivalent file; its claim is "5,700+ tests passing across Python and Java" [1]. The practitioner signal on its fidelity is mixed. In the 317-point Hacker News thread one commenter claiming "detailed knowledge of dynamodb" wrote that the code "does not properly mimic the service exceptions, input validations, eventual consistency, or edge cases"; another wrote that the real LocalStack problem "was always drift. Tests pass locally, then something breaks in staging because the S3 response format is slightly different or DynamoDB throttling doesn't match", and that their team "just switched to short-lived real AWS environments for integration tests"; a reply framed the emulator as "a sniff-test" that gates the expensive real run [44]. The spot checks in section 3 found AWS-wording errors for the common DynamoDB, S3 and SQS mistakes, which is consistent with "good on the common path, unverified on the edges". The other thread's recurring counter-position was to "run test and develop in a real isolated AWS account with security policies (guardrails)", answered by a complaint that CloudFormation deployments "take inordinate amounts of time" and that pushing a new ECS task alone takes over ten minutes, which is the latency the emulator lanes exist to remove [45].

Three facts hold for every emulator in this doc and decide what belongs in the sandbox gate:

1. **IAM is lenient or absent by default.** AWS Emulator never evaluates policies [12]; LocalStack's enforcement is off unless `ENFORCE_IAM=1`, and only on paid plans [21]; MiniStack checks only under `AUTH=true` [4]. Verified here: both MiniStack and Floci answered `sts get-caller-identity` for the key `test` with the account root ARN.
2. **Account and region boundaries are partial.** The AWS Emulator keeps one account and one region [12]; DynamoDB Local does not model accounts or regions but, unless started with `-sharedDb`, keeps a separate database file per access key id and region [29]; MiniStack isolates accounts and regions (verified here for S3 and SQS), with S3 bucket names global across regions and Aurora DSQL shared across regions [4][8].
3. **Service-specific tools document their own divergence.** DynamoDB Local ignores provisioned throughput, applies `CreateTable` instantly, has no `TransactionConflictException`, treats table names case-insensitively, and silently ignores vector indexes [29]. `sam local start-api` runs handlers in Lambda runtime containers but any AWS call the handler makes reaches real AWS resources [34], and SAM "does not guarantee 100% fidelity" for Lambda authorizers [30]. Step Functions Local is now "unsupported"; AWS points at the TestState API instead [31].

## 5. Wiring clients at an emulator without ever hitting production

**SDKs and CLI.** Since 2023 the AWS SDKs and CLI read `AWS_ENDPOINT_URL` [50] and service-specific `AWS_ENDPOINT_URL_<SERVICE>` (and the equivalent `endpoint_url` / `services` settings in the config file). Supported: CLI v2, Go v2, Java 2.x, JavaScript v3, Python, .NET, PHP, Ruby, Rust, Kotlin, Swift, C++; not supported: Go v1, Java 1.x, JavaScript v2 [37]. Precedence, highest first: an endpoint set in code or `--endpoint-url`; the service-specific variable; `AWS_ENDPOINT_URL`; the profile's `services` section; the profile's `endpoint_url`; the default. `AWS_IGNORE_CONFIGURED_ENDPOINT_URLS=true` ignores everything but endpoints set in code [37]. The emulator-lane environment is therefore:

```bash
# Emulator lane (port-4566 family). Nothing here is a real credential.
export AWS_ENDPOINT_URL=http://aws.myproj.localhost   # mx router hostname (section 10); http://aws:4566 inside compose
export AWS_ACCESS_KEY_ID=000000000000                 # MiniStack: a 12-digit key IS the account id
export AWS_SECRET_ACCESS_KEY=test
export AWS_REGION=us-east-1                           # JavaScript v3 and Java 2.x read AWS_REGION
export AWS_DEFAULT_REGION=us-east-1                   # the CLI and boto3 also read this one
export AWS_EC2_METADATA_DISABLED=true                 # never fall through to IMDS
unset AWS_PROFILE AWS_SESSION_TOKEN                   # a lingering profile is how emulator jobs reach production

# AWS Emulator uses its own generated keys and config files; abridged from the launch post [11],
# which also sets AWS_EC2_METADATA_DISABLED, AWS_REGION and AWS_DEFAULT_REGION:
#   export AWS_ENDPOINT_URL=https://127.0.0.1:9297
#   export AWS_SHARED_CREDENTIALS_FILE=~/.aws/emulator/9297/emulator_credentials
#   export AWS_CONFIG_FILE=~/.aws/emulator/9297/config
#   export AWS_DISABLE_HOST_PREFIX_INJECTION=true
#   unset AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY AWS_SESSION_TOKEN AWS_PROFILE
```

S3 behind a hostname router needs path-style addressing (`s3_use_path_style` in Terraform, `forcePathStyle: true` in the JavaScript SDK, `addressing_style = path` in boto3 config), because virtual-host-style requests put the bucket name in the `Host` header and the router's `Host()` rule would not match [6][38].

**Terraform.** The AWS provider honours `AWS_ENDPOINT_URL` and `AWS_ENDPOINT_URL_<SERVICE>` using Go SDK v2 service ids, and HashiCorp calls custom endpoints "best effort", testing the provider only against real AWS [38]. An adapted local block (flags from [6][38]; list an `endpoints` entry for every service the configuration uses, since an unlisted service goes to the real endpoint and fails on the fake credentials, or set `AWS_ENDPOINT_URL` in the environment, which the provider honours [38]):

```hcl
provider "aws" {
  region                      = "us-east-1"
  access_key                  = "000000000000" # emulator account id; not a credential
  secret_key                  = "test"
  s3_use_path_style           = true
  skip_credentials_validation = true
  skip_metadata_api_check     = true
  skip_requesting_account_id  = true
  endpoints {
    # one entry per service the configuration touches; MiniStack's page lists sixteen [6]
    dynamodb       = "http://aws.myproj.localhost"
    events         = "http://aws.myproj.localhost"
    iam            = "http://aws.myproj.localhost"
    lambda         = "http://aws.myproj.localhost"
    s3             = "http://aws.myproj.localhost"
    secretsmanager = "http://aws.myproj.localhost"
    sns            = "http://aws.myproj.localhost"
    sqs            = "http://aws.myproj.localhost"
    sts            = "http://aws.myproj.localhost"
  }
}
```

MiniStack's own Terraform page documents the same flags for provider v5 and v6 and multi-account via provider aliases with distinct 12-digit keys [6]. For the sandbox gate's provider block, add `allowed_account_ids = ["<sandbox id>"]`: the provider documents it "to prevent you from mistakenly using an incorrect one (and potentially end up destroying a live environment)" [39]. The emulator block should not rely on it, since `skip_requesting_account_id` can stop the provider from learning an account id at all [38][39].

**CDK and the LocalStack wrappers.** `cdklocal` and `tflocal` are both marked as no longer receiving updates, pointing at LocalStack's new `lstk` CLI [23][24]. `cdklocal` worked by injecting `AWS_ENDPOINT_URL` and, for CDK 2.177.0 and later, by removing AWS configuration variables such as `AWS_PROFILE` before invoking `cdk`, so that a configured profile could not send the deploy to real AWS [23]; `tflocal` wrote a `localstack_providers_override.tf` that pointed every endpoint at `localhost:4566` [24]. MiniStack still documents `cdklocal bootstrap` and `cdklocal deploy` and says LocalStack's `awslocal` and `tflocal` keep working against it [6]. The mechanism that outlives any wrapper is the SDK's own: `AWS_ENDPOINT_URL` set and `AWS_PROFILE` unset, which is what `cdklocal` did on the caller's behalf [23][37].

## 6. CI: service container, Testcontainers, isolation

GitHub Actions service containers run only on Linux runners; a job on the runner reaches them at `localhost:<mapped port>`, a job inside a container uses the service label as hostname [40].

```yaml
# .github/workflows/integration.yml — emulator lane; the job holds no AWS credentials at all
jobs:
  integration:
    runs-on: ubuntu-latest
    services:
      aws:
        image: floci/floci:2.5.0   # pin the tag; the same image the inner loop runs (MiniStack: ministackorg/ministack:<tag>)
        ports: ["4566:4566"]
    env:
      AWS_ENDPOINT_URL: http://localhost:4566
      AWS_ACCESS_KEY_ID: "000000000000"
      AWS_SECRET_ACCESS_KEY: test
      AWS_REGION: us-east-1
      AWS_DEFAULT_REGION: us-east-1
      AWS_EC2_METADATA_DISABLED: "true"
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v7
        with: { python-version: "3.13" }
      - run: pip install boto3 pytest
      - run: curl --retry 20 --retry-connrefused --retry-delay 1 -sf http://localhost:4566/_localstack/health   # served by Floci and MiniStack
      - run: pytest tests/integration -q
```

Per-test isolation has two mechanisms. Namespacing by account works on both port-4566 emulators and lets tests run in parallel against one instance: a 12-digit access key is its own account in Floci and in MiniStack (verified here on both), which the AWS Emulator does not offer [8][12][25]. A whole-emulator reset suits a serial suite, and it is emulator-specific: MiniStack has `POST /_ministack/reset` (verified here), moto has `POST /moto-api/reset`, and Floci and the AWS Emulator are reset by restarting them [8][12][28]. A reset under a parallel runner erases other workers' resources, so the portable default is the per-test account.

```python
# tests/integration/conftest.py — the emulator is started outside pytest (compose or the CI service container)
import os, uuid, boto3, pytest

ENDPOINT = os.environ["AWS_ENDPOINT_URL"]

@pytest.fixture
def account():
    """Each test gets its own account: a 12-digit access key is the account id in Floci and MiniStack."""
    acct = f"{uuid.uuid4().int % 10**12:012d}"
    return boto3.Session(aws_access_key_id=acct, aws_secret_access_key="test", region_name="us-east-1")

def test_upload_lands_in_bucket(account):
    s3 = account.client("s3", endpoint_url=ENDPOINT)
    s3.create_bucket(Bucket="uploads")          # no clash: every test's account has its own namespace
    s3.put_object(Bucket="uploads", Key="a.txt", Body=b"hi")
    assert s3.get_object(Bucket="uploads", Key="a.txt")["Body"].read() == b"hi"
```

For JVM projects the Testcontainers module gives the same lifecycle in-process [9]:

```java
// testImplementation 'org.ministack:testcontainers-ministack:0.1.5'   (verbatim from the module README [9])
try (MiniStackContainer ministack = new MiniStackContainer()) {
    ministack.start();
    S3Client s3 = S3Client.builder()
        .endpointOverride(URI.create(ministack.getEndpoint()))
        .region(ministack.getRegion())
        .credentialsProvider(StaticCredentialsProvider.create(
            AwsBasicCredentials.create(ministack.getAccessKey(), ministack.getSecretKey())))
        .forcePathStyle(true)
        .build();
    s3.createBucket(b -> b.bucket("my-bucket"));
}
```

Pin the image tag. MiniStack cut ten releases between 2026-10-26 and 2026-11-18 and the AWS Emulator cuts several a day; an unpinned `latest` turns a flaky upstream fix into a red main branch [8][14].

## 7. The sandbox gate

The gate is a real AWS account, so it is the one lane with cost and blast radius, and the one lane whose results can be trusted for IAM, quotas and networking [34][35]. The AWS-native mechanics: account vending through Control Tower Account Factory or AFT and the Innovation Sandbox solution with budget alerts and scheduled cleanup, Budgets, an SCP denying expensive instance types, and TTL tags with automatic deletion [35]. FND01-BP04 says to "match the investment to organizational scale": a startup is well served by per-developer sandboxes plus one shared dev account [35]. For serverless teams the fastest gate is not a full deploy per change: SST's Live mode deploys stub functions to a personal stage and forwards their invocations to the developer's machine over AppSync Events, so real AWS triggers, roles and quotas apply while the handler code runs locally; SST warns to use it only in a personal stage and that the stubs time out if the CLI stops [32]. AWS's own guide makes the cost argument for cloud iteration: cloud test environments can cost less than "the developer time to set up a complex local environment", and SAM Accelerate, CDK watch and SST shorten the loop [34]. For Step Functions, TestState is now the supported way to test state logic without a deploy [31].

## 8. Compared with the Cloudflare lane

The corpus's Cloudflare doc describes mx's production path as Workers plus Containers behind a Durable Object, with local development staying on the mx router: "Keep local dev on the mx router; Cloudflare is the production path. The two coexist: same Dockerfile family, different front doors" [47]. What Cloudflare gives a developer locally that AWS does not: `wrangler dev` runs the Worker through Miniflare in a local `workerd` process, and "`workerd` also powers the hosted Workers platform", so the local engine is the one that powers production, although the hosted platform adds layers the local one lacks; "All resources your Worker is bound to in your Wrangler configuration are simulated locally" (KV, R2, D1, Queues, Durable Objects) by the vendor itself; remote bindings (`remote: true`), GA since Wrangler 4.37.0 on 2025-09-16, let the Worker run locally against deployed resources; and `wrangler versions upload --preview-alias` gives per-version URLs, though not for Workers that implement a Durable Object, which includes Containers, so not for mx's own production path [41][42][43]. Cloudflare also documents its gaps plainly: no local simulation for Workers AI, Browser Run, Vectorize, mTLS, the Images API or Dispatch Namespaces, and "Miniflare is not a security boundary for untrusted code" [41].

| | Cloudflare lane | AWS lane |
|---|---|---|
| local runtime | the production engine (`workerd`), minus the hosted platform's extra layers [41] | none; Lambda runtime containers via SAM are the closest, and the handler's own AWS calls reach real AWS [30][34] |
| local service simulation | first-party, in the same CLI [41] | third-party (MiniStack, Floci) or an official emulator that reached GA on 2026-11-11 [11] |
| "local code, real resources" | remote bindings, GA [42] | SST Live for Lambda; otherwise deploy to a sandbox [32][34] |
| security semantics locally | n/a (bindings are capabilities, not IAM) | absent in the AWS Emulator, opt-in in MiniStack (`AUTH=true`), paid-only in LocalStack [12][3][21] |
| per-change preview | version URLs per upload, except for Workers with Durable Objects or Containers [43] | ephemeral per-PR stack in a sandbox account [35] |
| breadth to emulate | small, bounded platform | RDS, ECS, EKS, VPC: MiniStack answers with real sidecar containers, which nothing in the Cloudflare model needs [2] |

The consequence for the architecture: on Cloudflare the local lane carries most of the confidence and the deploy is the check; on AWS the emulator lanes carry speed and the sandbox gate carries the confidence. Both stacks share the mx router as the front door for local work [47].

## 9. What practitioners argue about

Two Hacker News threads record the objections; these are comment claims, not facts [44][45]:

- **A clone cannot "maintain compatibility with tons of AWS services while not getting paid for it"** (the top comment's words). The strongest answer arrived eight months later from AWS itself, which now maintains an emulator generated from the API models by agents and publishes a conformance file [11][13]. Whether a two-maintainer MIT project keeps pace is the bus-factor question in section 10.
- **Generated text, unreviewed code.** Several commenters distrusted MiniStack for signs of generated text: a misaligned ASCII diagram in the README and a "Copyright (c) 2024 MiniStack Contributors" line in a project created in 2026 [44]. The AWS Emulator is written entirely by agents by design [11]. The useful response is not to argue about authorship but to demand a published conformance suite, which only AWS provides today [13][16].
- **Drift burned us, so we moved integration tests to short-lived real environments.** This is the sandbox gate; the thread's own reply, that the emulator is the sniff test before the expensive run, is the three-lane model [44].
- **Just "run test and develop in a real isolated AWS account with security policies (guardrails)".** The rebuttal in the same thread was deploy latency measured in minutes per trivial change [45]; the emulator lanes exist to remove that latency, not to replace the account.

## 10. Synthesis (inferred)

The following are this doc's own conclusions, not statements by the sources.

**Treat the emulator as a swappable implementation detail behind the standard endpoint variables.** Three independent emulators now speak the port-4566 dialect and AWS's own speaks the same SDK protocol on 9297. Every piece of project configuration should reference `AWS_ENDPOINT_URL` and the emulator's hostname, never a product name, so that a maintainer disappearing (MiniStack is effectively two people) or a fidelity gap closing elsewhere is an image-tag change. The one place a product name may appear is the reset endpoint, which differs per emulator; wrap it in a single fixture.

**Use the AWS fidelity file as the threshold instrument for every emulator.** AWS's per-service scores are the only published, test-level map of how emulable each service's behaviour is today. A team that adopts MiniStack or Floci can still use the AWS table to decide which services' tests belong in the sandbox gate: anything with a low control-plane or data-plane score (CloudFormation, EC2, Lambda configuration, SSM, RDS) is unlikely to be more faithful in an emulator that publishes no scores at all. Re-sum the file each quarter, and compare only scores produced with the same version of the fidelity suite, as AWS itself cautions [13]. The pattern in the v1.0.512 table is also a general lesson about emulation: the services at the top are stateless APIs over simple stores, the services at the bottom orchestrate other services or a real network.

**Folding this into mx: an `aws` infra lane, not an app recipe.** mx already distinguishes recipes (app scaffolds under `templates/recipes/`: laravel, nuxt, rust-api) from infra providers (`templates/infra/<provider>/` plus `templates/make/<provider>.mk` and the `mx infra setup|link|inspect` credential model) [48][49]. The `InfraProvider::Aws` variant exists and `mx infra setup aws` already writes `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_REGION` to the global `aws.env`, but `mx new --infra aws` prints "infrastructure not yet implemented" and there is no `templates/infra/aws/` [49]. The proposal is therefore a sibling of the Cloudflare lane with the three lanes of section 2 mapped onto mx's existing seams:

1. **Inner loop on the router.** `templates/infra/aws/compose.yml` adds one `aws` service running the emulator image on the `devmesh-traefik` network with the standard labels, so the endpoint is a hostname (`http://aws.<project>.localhost`, following the `<name>.localhost` default the recipe guide uses for every router host) and never a published port, which is the corpus's standing rule for local URLs [51]. App containers get `AWS_ENDPOINT_URL=http://aws:4566` injected by the compose env (resolved over the project's own `default` network; like every mx service on the shared router network, the bare name is also visible to other projects, so apps should not rely on it there) and the fake account credentials from a committed `aws.local.env`; the global `aws.env` with real keys is never mounted into the emulator lane. Path-style S3 is forced because the router matches on `Host`.

   ```yaml
   # templates/infra/aws/compose.yml (illustrative; the labels follow the recipe guide's router section [51])
   services:
     aws:
       image: ${AWS_EMULATOR_IMAGE:-floci/floci:2.5.0}   # the lane's default; swap for ministackorg/ministack or a pinned fork
       environment:
         FLOCI_HOSTNAME: ${AWS_ROUTER_HOST:-aws.{{DOMAIN}}}   # see the note below; required behind a hostname
         MINISTACK_HOST: ${AWS_ROUTER_HOST:-aws.{{DOMAIN}}}   # the same setting under MiniStack's name
         FLOCI_STORAGE_MODE: hybrid       # local dev keeps state across restarts; CI uses memory
         FLOCI_STORAGE_PERSISTENT_PATH: /app/data
       volumes:
         - aws-state:/app/data
         # - /var/run/docker.sock:/var/run/docker.sock   # opt in only for the RDS/ElastiCache/ECS sidecars [8]; it grants host-root-equivalent access
       networks: [default, devmesh-traefik]
       labels:
         - traefik.enable=true
         - traefik.http.routers.${COMPOSE_PROJECT_NAME}-aws.rule=Host(`${AWS_ROUTER_HOST:-aws.{{DOMAIN}}}`)
         - traefik.http.routers.${COMPOSE_PROJECT_NAME}-aws.entrypoints=web
         - traefik.http.services.${COMPOSE_PROJECT_NAME}-aws.loadbalancer.server.port=4566
         - traefik.docker.network=devmesh-traefik
   volumes:
     aws-state:
   networks:
     devmesh-traefik:
       external: true
   ```

   The lane as built (bd:mech-crate-7vf) defaults to Floci 2.5.0, the owner's choice between the two port-4566 emulators; MiniStack stays a one-variable swap. Three things only a live run showed, each reproduced on both Floci 2.5.0 and MiniStack 1.5.42. Both emulators read `<label>.<their own host>` as a virtual-hosted S3 bucket and default that host to `localhost`, so behind a router hostname ending in `.localhost` every request was taken for bucket `aws` (`s3 mb` answered `NoSuchBucket`) until the emulator was told its hostname (`FLOCI_HOSTNAME`, `MINISTACK_HOST`). Returned SQS queue URLs carry the internal port (`http://aws.<project>.localhost:4566/...`); the AWS CLI sends to `AWS_ENDPOINT_URL` and ignores that host, so the URL is passed through unchanged. And Floci 2.5.0 has no reset endpoint that was found, so the lane's `make aws-reset` recreates the container with an empty volume, which works for any emulator.

2. **Pull-request CI** reuses the same image tag from the same variable in a service container (section 6), so the inner loop and CI run one emulator version.
3. **sandbox gate** is where `mx infra link aws` earns its keep: `make aws-deploy` (mirroring `cf-deploy`) reads the linked real credentials, requires `allowed_account_ids` to name the sandbox, and runs the same integration suite with `AWS_ENDPOINT_URL` unset. A `make aws-vars` target like `cf-vars` shows which scope supplied each value, and a guard refuses to run the deploy target while `AWS_ENDPOINT_URL` is set, which is the one-line defence against the "profile leaked into an emulator job" failure that `cdklocal` had to special-case.

   The split that matters is which `.env` each lane sees. The emulator lane sees only committed fakes; the gate sees only the linked real file. A developer cannot point production credentials at an emulator, nor an emulator endpoint at production, without editing two files on purpose.

**What to measure before adopting.** Run the team's own integration suite against MiniStack, Floci and the AWS Emulator for one sprint and count the tests that pass on one and fail on another; those disagreements are either emulator bugs worth an issue or tests that belong in the sandbox gate. This is cheaper than trusting any vendor's service count.

## 11. Checklist

- [ ] Emulator chosen from the port-4566 family and pinned by tag; AWS Emulator scores file re-summed quarterly for the threshold table.
- [ ] Three lanes written down with the boundary rule: IAM, quotas, networking, account and region boundaries, and low-fidelity services go to the sandbox gate.
- [ ] Emulator lanes hold no real credentials: 12-digit fake key, `AWS_PROFILE` unset, `AWS_EC2_METADATA_DISABLED=true`, IMDS unreachable.
- [ ] Endpoint set only through `AWS_ENDPOINT_URL` / `AWS_ENDPOINT_URL_<SERVICE>` or the Terraform `endpoints {}` block; no `cdklocal` or `tflocal` in new pipelines.
- [ ] Path-style S3 wherever the emulator sits behind a hostname router.
- [ ] Sandbox provider block carries `allowed_account_ids`; deploy target refuses to run with `AWS_ENDPOINT_URL` set.
- [ ] Per-test account for isolation (a reset only for serial suites, and only where the emulator has one); CI image identical to the inner-loop image.
- [ ] Sandbox accounts vended with budgets, SCPs and TTL cleanup; SST Live or SAM Accelerate for serverless inner loops that need real triggers.
- [ ] No new work on LocalStack Hobby (non-commercial) or MinIO (archived).
- [ ] mx: `templates/infra/aws/` lane filed and built as the sibling of the Cloudflare lane; emulator reachable at `aws.<project>.localhost` through the router, never a port.
