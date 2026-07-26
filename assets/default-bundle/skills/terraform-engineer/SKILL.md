---
name: terraform-engineer
description: Terraform/IaC patterns. Module design, state management, drift, providers, secrets, plan/apply workflow. Load when authoring or reviewing Terraform.
metadata:
  audience: architect, fullstack-lt, developer, sec-ops-expert
---

# Terraform Engineer

## When to Use

- Authoring or reviewing Terraform (`.tf`, `.tfvars`)
- Designing module structure
- State backend / locking strategy
- Drift detection and remediation
- IaC security (open ports, public buckets, IAM wildcards)

## Triggers

terraform, tf, iac, hcl, module, state, backend, provider, workspace, atlantis, terragrunt, opentofu, plan, apply, drift

## Core Principles

- **Infrastructure as code, not as console clicks.** Console = read-only.
- **Plan before apply.** Always. CI shows plan; humans approve.
- **Small, composable modules.** Reusable, parameterized, tested.
- **State is precious.** Remote backend, locked, encrypted, versioned.
- **No secrets in `.tf` or `.tfvars`.** Use providers' secret refs.

## Repository Layout

```
infra/
├── modules/                  # reusable building blocks
│   ├── eks-cluster/
│   ├── rds-postgres/
│   └── ...
├── environments/
│   ├── dev/
│   │   ├── main.tf           # composes modules
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── backend.tf
│   ├── stage/
│   └── prod/
└── README.md
```

## State Backend

- **Remote**: S3 + DynamoDB lock, GCS, Azure Blob, or Terraform Cloud.
- **Encryption** at rest.
- **Versioning** enabled so you can roll back state.
- **Separate state per environment**. Never share dev/prod state.
- **No local state** in any repo committed to git.

## Module Design

A good module:

- Has a single, clear purpose.
- Inputs: minimal required, sane defaults.
- Outputs: everything callers need to compose.
- Pinned provider versions in `required_providers`.
- README with example usage.
- Optional `examples/` directory.
- Tests (Terratest or `terraform test`).

## Provider Versioning

```hcl
terraform {
  required_version = ">= 1.6.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"   # pessimistic, allow patch
    }
  }
  backend "s3" {
    bucket         = "tfstate-acme"
    key            = "prod/eks/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "tfstate-lock"
    encrypt        = true
  }
}
```

## Secrets

- **Read** from secret manager (AWS Secrets Manager, Vault, GCP Secret Manager) via data sources.
- **Write** secret values to those managers out-of-band; reference in TF.
- **Never** put credentials in `.tfvars` checked into git.
- Use `sensitive = true` on variables and outputs that carry secrets.

## Plan / Apply Workflow

```
git push → CI runs `terraform plan` → posts plan to PR
human reviews plan → approves PR → CI runs `terraform apply`
```

Tooling: Atlantis, Terraform Cloud, env0, Spacelift. Pick one. No local applies in prod.

## Drift

- **Detect** weekly via scheduled `terraform plan` in CI; alert on non-empty plan.
- **Reconcile**: either bring infra back to state (apply) or bring state to infra (`terraform import` / `state rm`).
- **Root-cause** drift — manual change indicates a process gap.

## Security Anti-Patterns to Catch

- Security groups with `0.0.0.0/0` ingress on non-LB.
- S3 buckets without `block_public_access`.
- IAM policies with `Action: *` or `Resource: *` (review carefully).
- RDS / EBS without encryption.
- Plaintext secrets in resources.
- Public RDS / Redshift / DocumentDB.
- Lambdas with overly broad execution roles.
- No tags (kills cost allocation).

## Style

- `terraform fmt` enforced in CI.
- `tflint` + `checkov` / `tfsec` in CI.
- Consistent naming: `snake_case` for resources, `kebab-case` for tags/names exposed externally.
- Use `for_each` over `count` for stable identities.
- Avoid `null_resource` + `local-exec` — escape hatch only.

## Testing

- **Unit (module)**: `terraform validate`, `terraform plan` against fixture vars.
- **Integration**: Terratest spins real infra in a sandbox, asserts, tears down.
- **Policy**: OPA / Sentinel / Checkov gates in CI.

## Output Template

```
## Module
- Path: <relative>
- Purpose: <one line>
- Inputs (required): <list>
- Outputs: <list>

## State
- Backend: <type, location>
- Locking: <mechanism>

## Plan Summary
+ N to add
~ M to change
- K to destroy

## Risks
- <e.g., destroys a stateful resource>
- <e.g., changes IAM scope>

## Validation
- `terraform fmt -check`: <pass/fail>
- `tflint`: <pass/fail>
- `tfsec`/`checkov`: <findings>
```

## References

- Terraform docs: https://developer.hashicorp.com/terraform/docs
- *Terraform: Up & Running* — Yevgeniy Brikman
- Checkov: https://www.checkov.io
- Terratest: https://terratest.gruntwork.io
