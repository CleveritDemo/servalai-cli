---
name: devops-engineer
description: CI/CD pipelines, GitOps, release engineering, deployment strategies, build caching, artifact management. Load for any pipeline, build, or release task.
metadata:
  audience: fullstack-lt, developer, sec-ops-expert
---

# DevOps Engineer

## When to Use

- Designing or reviewing CI/CD pipelines (GitHub Actions, GitLab CI, Jenkins, Argo Workflows)
- Release strategies (blue/green, canary, rolling)
- Artifact / container registry decisions
- GitOps with ArgoCD / Flux
- Build performance and caching

## Triggers

ci, cd, pipeline, github actions, gitlab ci, jenkins, argo, flux, gitops, release, deployment, blue green, canary, rolling, helm, kustomize, registry, artifact

## Pipeline Principles

- **One pipeline, gated stages.** Lint → test → build → security scan → publish → deploy.
- **Fail fast.** Cheap checks first; expensive last.
- **Reproducible builds.** Same input → same output. Pin everything.
- **Immutable artifacts.** Build once, promote across envs.
- **Secrets via secret manager** mounted at runtime — never echoed in logs.
- **Logs and SBOM** for every release.

## Stage Order

```
1. fmt + lint        (seconds)
2. unit tests        (seconds-minute)
3. build artifact    (varies)
4. integration tests (minutes)
5. security scan     (SAST, dep scan, container scan)
6. publish artifact  (registry)
7. deploy to dev     (automatic)
8. e2e tests         (against dev)
9. promote to stage  (gated)
10. promote to prod  (gated, often manual)
```

## Trunk-Based vs GitFlow

- **Trunk-based** is the default. Short-lived branches (< 2 days). Feature flags for incomplete work.
- **GitFlow** only when you need long-lived release branches (regulated industries, on-prem ship cycles).

## Deployment Strategies

| Strategy | Rollback | Cost | When |
|---|---|---|---|
| **Rolling** | slow (must redeploy) | low | Default; stateless services |
| **Blue/Green** | instant (swap traffic) | 2x | Critical services; quick rollback needed |
| **Canary** | gradual (shift traffic %) | low-medium | High-traffic; risk-controlled rollout |
| **Shadow** | n/a (no production impact) | medium | Validation before cutover |

## Feature Flags

- Decouple deploy from release.
- Flag types: release, ops (kill switch), experiment, permission.
- TTL on flags — delete when no longer needed.
- One source of truth (LaunchDarkly, Unleash, ConfigCat, in-house).

## Container Build

- **Distroless or minimal** base (`gcr.io/distroless/`, `chainguard/`, Alpine for non-glibc).
- **Multi-stage** builds — discard build deps in final image.
- **Pin base by digest** (`@sha256:...`) for reproducibility in prod.
- **Non-root user**, `USER 1000`.
- **No secrets in layers** — use BuildKit secrets, not `ARG`/`ENV`.
- **`.dockerignore`** to keep context small.
- Scan with Trivy / Grype / Snyk in CI.

## Caching

- **Layer cache** (Docker BuildKit, BuildX).
- **Dep cache** (npm/pip/gem/cargo) keyed by lockfile hash.
- **Test cache** (Bazel, Nx, Turborepo) for monorepos.
- Caches versioned; bust on toolchain upgrade.

## GitOps (ArgoCD / Flux)

- **Repo of truth** for cluster state — never `kubectl apply` directly in prod.
- **App-of-apps** pattern for many services.
- **Sync waves** to order CRDs before consumers.
- **Auto-sync** in non-prod; manual in prod for visibility.
- **Drift alerts** — anyone changing cluster outside git triggers alarm.
- Sensitive values via SealedSecrets / External Secrets Operator / SOPS.

## SBOM & Provenance

- Generate SBOM (Syft, CycloneDX) on every build.
- Sign artifacts (Cosign, Sigstore) — admission controllers verify.
- Attestations (in-toto, SLSA) for supply-chain integrity.

## Pulzen Specifics

- **GitOps target**: `nodrize-argocd-pulzen`.
- **Clusters**: `cadi-k3s`, `nodrik3s1-k3s`.
- **Promotion path**: typically dev → stage → prod, gated.

## Output Template

```
## Pipeline Summary
- Trigger: <push to branch | PR | tag | schedule>
- Stages: <ordered list>
- Total runtime budget: <minutes>

## Build
- Base image: <pinned by digest>
- SBOM: <yes/no>
- Signing: <yes/no>

## Deploy
- Strategy: <rolling/blue-green/canary>
- Rollback procedure: <documented>
- Health gate: <how/when>

## Risks
- <what could break>
```

## References

- *Continuous Delivery* — Humble & Farley
- *Accelerate* — Forsgren, Humble, Kim
- DORA metrics: deploy frequency, lead time, MTTR, change failure rate
- SLSA framework: https://slsa.dev
