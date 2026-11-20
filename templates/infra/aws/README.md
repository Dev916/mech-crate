# AWS Infrastructure

Local development and CI for a project that hosts on AWS. Three lanes:

| Lane | Runs where | Trust it for | Do not trust it for |
|------|------------|--------------|---------------------|
| Inner loop | emulator on your machine (`make aws-up`) | data-path code: S3, SQS, SNS, DynamoDB, EventBridge, Lambda wiring | IAM, quotas, networking, cross-account |
| Pull-request CI | the same emulator image as a service container | the same integration suite, with no AWS credentials in the job | same as above |
| Sandbox gate | a real non-production AWS account (`make aws-sandbox`) | everything, including IAM and deploys | nothing |

The reasoning, emulator comparison and fidelity numbers are in the techniques
corpus: `docs/development/local-aws-emulation-architecture.md` in mech-crate.

## Emulator lane

Every `make aws-<cmd>` target is also `mx aws <cmd>`, with the command line
passed as plain arguments: `mx aws cli s3 ls` runs exactly
`make aws-cli c="s3 ls"`. `mx aws --help` lists them all.

```bash
mx aws up                   # start the emulator in this project's compose project
mx aws health               # wait until it answers
eval "$(mx aws env)"        # export the emulator endpoint and fake credentials
aws s3 mb s3://uploads      # talks to the emulator, not to AWS
mx aws cli s3 ls            # same, without touching your shell
mx aws reset                # recreate the emulator with empty state
mx aws down                 # --volumes also drops its state
```

- Workstation endpoint: `http://aws.{{PROJECT_NAME}}.localhost` through the mx router
  (override the hostname with `AWS_ROUTER_HOST`). No host port is published.
- Inside compose: `http://aws:4566`. Add `infra/aws/aws.local.env` to a
  service's `env_file` list.
- S3 needs path-style addressing behind the router: `s3_use_path_style = true`
  in Terraform, `forcePathStyle: true` in the JavaScript SDK,
  `addressing_style = path` in boto3 config.
- The emulator is Floci by default (`floci/floci`, MIT). Swap it by setting
  `AWS_EMULATOR_IMAGE` (for example `ministackorg/ministack:<tag>`); anything
  that serves port 4566 and `/_localstack/health` works with these targets.
- The compose file tells the emulator its own hostname (`FLOCI_HOSTNAME`,
  `MINISTACK_HOST`). Both emulators treat `<label>.<their host>` as a
  virtual-hosted S3 bucket and default to `localhost`, so without it every
  routed request is read as bucket `aws`. If you change `AWS_ROUTER_HOST`,
  restart with `make aws-up` so both agree.
- Returned SQS queue URLs carry the emulator's internal port
  (`http://aws.{{PROJECT_NAME}}.localhost:4566/...`). The AWS CLI and SDKs send to
  `AWS_ENDPOINT_URL` when it is set and ignore that host, so pass queue URLs
  through unchanged; do not `curl` them.
- State is kept in a volume (`FLOCI_STORAGE_MODE=hybrid`) and survives
  `make aws-down`. `make aws-reset` recreates the container with an empty
  volume. A 12-digit access key is its own account, so tests can isolate
  themselves without a reset.

Emulators accept any credentials and do not evaluate IAM by default. A test
that passes here says nothing about your execution roles.

## Sandbox gate

Real credentials come from the same two scopes as every mx provider:

```bash
mx infra setup aws          # global: ~/.mech-crate/config/infra/aws.env
$EDITOR infra/aws/.env.aws  # project (gitignored); wins over the global file
mx infra link aws           # or: ignore the project file and use the global one
```

Set `AWS_SANDBOX_ACCOUNT_ID` in either file. It is the guard:

```bash
mx aws vars                           # what resolved, and from which scope (no secrets)
mx aws check-sandbox                  # refuses unless the caller is that account
mx aws sandbox terraform apply        # run a command against the sandbox account
```

`aws-check-sandbox` fails when `AWS_ENDPOINT_URL` is set (an emulator endpoint
must never ride along with real credentials), when no sandbox account id is
configured, and when `aws sts get-caller-identity` reports a different account.
Add `allowed_account_ids = ["<sandbox id>"]` to the Terraform provider block
you use for this lane as a second lock.

## CI

```yaml
services:
  aws:
    image: floci/floci:2.5.0   # same tag as infra/aws/compose.yml
    ports: ["4566:4566"]
env:
  AWS_ENDPOINT_URL: http://localhost:4566
  AWS_ACCESS_KEY_ID: "000000000000"
  AWS_SECRET_ACCESS_KEY: test
  AWS_REGION: us-east-1
  AWS_DEFAULT_REGION: us-east-1
  AWS_EC2_METADATA_DISABLED: "true"
```

Give the job no real AWS credentials.
