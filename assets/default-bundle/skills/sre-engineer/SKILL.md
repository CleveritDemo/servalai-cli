---
name: sre-engineer
description: Site Reliability Engineering. SLI/SLO/error budget, incident response, postmortems, capacity planning, DR. Load for reliability discussions, on-call concerns, or SLO definitions.
metadata:
  audience: architect, fullstack-lt, sec-ops-expert
---

# SRE Engineer

## When to Use

- Defining SLIs/SLOs for a service
- Designing for failure (graceful degradation, circuit breakers)
- Incident response and postmortem
- Capacity planning
- Disaster recovery and chaos exercises

## Triggers

sre, sli, slo, sla, error budget, reliability, availability, incident, postmortem, blast radius, chaos, dr, disaster recovery, capacity, on-call

## SLI / SLO / SLA

| Term | Definition |
|---|---|
| **SLI** | A direct measurement of service quality (e.g. p95 latency, success rate). |
| **SLO** | The target for an SLI over a window (e.g. 99.9% success over 30 days). |
| **SLA** | A contractual commitment to a customer, usually weaker than SLO. |
| **Error budget** | `1 - SLO`. The amount of unreliability you can spend. |

### Good SLIs

- **Availability**: % of requests not returning 5xx (over window).
- **Latency**: p95/p99 of successful requests (over window).
- **Quality**: % of requests where business invariant held.
- **Freshness**: age of data served (for caches/replicas).
- **Correctness**: % of jobs that produced expected output.

Measured from **the user's perspective** — not at the load balancer if that doesn't match what they experience.

### SLO Targets by Tier

| Tier | SLO | Error budget / 30d |
|---|---|---|
| Tier 1 (revenue critical) | 99.95% | 21 min |
| Tier 2 (important) | 99.9% | 43 min |
| Tier 3 (internal tooling) | 99.5% | 3.6 hr |
| Tier 4 (experimental) | 99.0% | 7.2 hr |

Don't promise 99.99%+ unless your architecture and team genuinely support it.

## Error Budget Policy

When error budget is exhausted:
1. Freeze risky changes (feature deploys, migrations).
2. Prioritize reliability work.
3. Resume normal velocity when budget recovers.

This is the social contract between dev and SRE.

## Designing for Failure

- **Timeouts** on every outbound call. Cascade prevention.
- **Retries** with exponential backoff + jitter. Cap attempts. Idempotent only.
- **Circuit breakers** when downstream is repeatedly failing.
- **Bulkheads** — pool isolation so one slow dep doesn't drain everything.
- **Graceful degradation** — serve stale, hide non-critical features.
- **Backpressure** — bounded queues; shed load explicitly.
- **Health endpoints** distinct: liveness (am I alive?), readiness (can I take traffic?).

## Incident Response

```
DETECT → TRIAGE → MITIGATE → RESOLVE → REVIEW
```

- **Detect**: alerting on SLI burn rate, not just on individual errors.
- **Triage**: on-call assigns severity, declares incident if SEV1/2.
- **Mitigate**: stop the bleeding. Rollback > forward fix. Restore service first.
- **Resolve**: full fix; reopen if regressed.
- **Review**: blameless postmortem.

### Severity Levels

- **SEV1**: total outage / major data loss. Page everyone.
- **SEV2**: significant degradation. Page on-call.
- **SEV3**: minor impact. Ticket, fix during hours.
- **SEV4**: cosmetic or internal-only.

## Blameless Postmortem

Structure:
1. **Summary** — what happened, when, impact.
2. **Timeline** — minute by minute.
3. **Root cause(s)** — *what*, not *who*.
4. **Detection** — how/when did we notice? Could we have noticed sooner?
5. **Response** — what worked, what didn't.
6. **Action items** — owned, dated, tracked.

Blame is poison. Bad systems make good people make mistakes.

## Capacity Planning

- Measure: current load, growth rate, peak/sustained ratio.
- Project: 6-12 months out.
- Buffer: 30-50% headroom for spikes.
- Test: load test to 2-3x projected peak.
- Plan: bottleneck order (CPU/mem/IO/DB/network) — usually DB first.

## Disaster Recovery

- **RPO** (data loss tolerance) and **RTO** (downtime tolerance) per service.
- **Backups**: automated, cross-region, tested restore quarterly.
- **DR runbooks**: stepped, tested, recently rehearsed.
- **Chaos exercises**: kill instances, partition networks, inject latency.

## Anti-Patterns

- SLOs nobody agreed to.
- Alerts on causes (CPU > 80%) instead of symptoms (latency > 500ms).
- Pages with no runbook link.
- "Postmortems" that name-and-shame.
- Untested DR plan.
- Heroics rewarded over reliability investment.
- "We don't need monitoring; we have logs."

## Output Template

```
## Service: <name>

## SLIs
- <name>: <how measured, window>

## SLOs
- <SLI>: <target>% over <window>
- Error budget: <minutes/month>

## Failure Modes
| Failure | Detection | Mitigation | Blast radius |

## Health Endpoints
- Liveness: <path, semantic>
- Readiness: <path, semantic>

## Capacity
- Current peak: <RPS / users>
- Projected (6mo): <RPS / users>
- Bottleneck: <resource>

## DR
- RPO: <time>
- RTO: <time>
- Last tested: <date>
```

## References

- *Site Reliability Engineering* — Google (free online: https://sre.google/books/)
- *The SRE Workbook* — Google
- *Seeking SRE* — David Blank-Edelman
