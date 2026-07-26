---
name: cloud-architect
description: Cloud architecture patterns. Multi-tier, multi-region, cost, networking, identity, data residency. Load when proposing cloud topology or evaluating cloud-native trade-offs.
metadata:
  audience: architect, fullstack-lt
---

# Cloud Architect

## When to Use

- New deployment topology (single region → multi region)
- Cloud vendor decision (AWS/GCP/Azure/hybrid)
- Cost / capacity planning
- Network architecture (VPC, peering, transit gateway, private endpoints)
- Identity and data residency

## Triggers

cloud, aws, gcp, azure, vpc, region, az, availability zone, multi-region, dr, disaster recovery, cost, billing, iam, sso, data residency, gdpr

## Pillars (AWS Well-Architected analogs work for all clouds)

1. **Operational excellence** — runbooks, observability, change management.
2. **Security** — least privilege, encryption, defense in depth.
3. **Reliability** — SLOs, multi-AZ, blast radius limits.
4. **Performance efficiency** — right-sized compute, caching, async where appropriate.
5. **Cost optimization** — visibility, tagging, reserved/spot, right-sizing.
6. **Sustainability** — region choice, efficient resource use.

## Topology Decisions

### Single Region, Multi-AZ
**When**: most products, RPO/RTO measured in minutes is acceptable.
**Spread**: 3 AZs minimum for stateful, 2 OK for stateless behind LB.

### Multi-Region
**When**: regulatory data residency, global latency requirements, or RTO < minutes.
**Cost**: 2-3x infra + complex data sync. Don't choose lightly.
**Strategy**: active-passive (warm standby) is the common pragmatic start; active-active for read-heavy globals.

### Hybrid / On-Prem
**When**: existing data center investment, ultra-low latency to on-prem systems, regulatory.
**Connectivity**: DirectConnect / ExpressRoute / Interconnect with redundant circuits.

## Networking

- **VPC per environment** (dev/stage/prod), not shared.
- **Subnets**: public (LB), private (app), isolated (DB).
- **No 0.0.0.0/0 ingress** except via LB.
- **Egress controlled** — NAT gateway or proxy; deny by default if compliance.
- **Private endpoints** for managed services (S3 gateway, etc.) to avoid public hops.
- **Transit hub** (Transit Gateway / Hub-and-spoke) for >3 VPCs.

## Identity

- **No IAM users for workloads** — use IAM roles + workload identity (IRSA on EKS, Workload Identity on GKE).
- **SSO/IdP** for humans, federated. MFA mandatory.
- **Least privilege** — start with deny-all, grant as needed. Use IAM Access Analyzer / equivalents.
- **Break-glass accounts** with strict logging.

## Data

- **Encryption at rest** by default. KMS / Cloud KMS with key rotation.
- **Encryption in transit** — TLS 1.2+ everywhere; mTLS for service-to-service in regulated paths.
- **Backups** with separate account / region. Test restores quarterly.
- **Data classification** — know what's PII/PHI/PCI; map to storage decisions.

## Cost

- **Tag everything** — `env`, `service`, `owner`, `cost-center`.
- **Budgets + alerts** per service/team.
- **Right-size before commit** — observe 2-4 weeks then commit (RIs/Savings Plans).
- **Spot** for fault-tolerant batch (50-90% savings).
- **Lifecycle policies** on object storage (hot → cool → archive).
- **Idle resource hunt** monthly: orphaned EBS, unattached EIPs, idle LBs.

## Disaster Recovery Tiers

| Tier | RTO | RPO | Approach |
|---|---|---|---|
| 1 — Backup & Restore | hours | hours | Backups in another region |
| 2 — Pilot Light | tens of min | minutes | Minimal infra always running |
| 3 — Warm Standby | minutes | seconds | Scaled-down full stack |
| 4 — Multi-Site Active | seconds | ~0 | Active-active, true HA |

Pick per service. Most services don't need tier 4.

## Anti-Patterns

- One AWS account for everything.
- IAM users with long-lived keys for workloads.
- Public S3 buckets / blob containers.
- No tagging strategy → can't allocate cost.
- "We'll add monitoring later".
- DR plan that's never been tested.
- Multi-region for vanity, not requirements.

## Output Template

```
## Topology Recommendation
- Regions: <list, justification>
- AZs: <count per region>
- VPC layout: <sketch>

## Identity
- Human access: <SSO/IdP>
- Workload identity: <approach>

## Data
- Storage classes: <hot/cool/archive policy>
- Encryption: <KMS strategy>
- Backups: <where, RPO/RTO>

## Cost Projection
- Monthly estimate: <breakdown by service category>
- Tagging: <strategy>

## DR Tier
- Target: <tier 1-4>
- Justification: <SLO mapping>

## Trade-offs
<honest>
```

## References

- AWS Well-Architected: https://aws.amazon.com/architecture/well-architected/
- Google Cloud Architecture Framework
- Azure Well-Architected Framework
