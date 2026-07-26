---
name: monitoring-expert
description: Observability — metrics, logs, traces, alerting. Prometheus, Grafana, OpenTelemetry, structured logging. Load when designing observability for a service or auditing existing instrumentation.
metadata:
  audience: fullstack-lt, developer, sre, sec-ops-expert
---

# Monitoring Expert

## When to Use

- Adding observability to a new service
- Auditing existing instrumentation
- Designing dashboards and alerts
- Investigating an incident
- Reducing alert noise

## Triggers

observability, monitoring, metrics, logs, traces, prometheus, grafana, otel, opentelemetry, jaeger, tempo, loki, alert, dashboard, slo, rum, apm

## The Three Pillars (+ events)

| Pillar | What | When |
|---|---|---|
| **Metrics** | Aggregated numerical observations | Always; basis for alerts/SLOs |
| **Logs** | Discrete events with context | Always; debugging, audit |
| **Traces** | Request flow across services | Distributed systems |
| **Events** | Discrete state changes (deploys, configs) | Annotate timelines |

## Metrics — the RED + USE Methods

### RED (per service / endpoint)
- **R**ate: requests per second
- **E**rrors: failed requests per second
- **D**uration: histogram of request times

### USE (per resource — CPU, memory, disk, network)
- **U**tilization: % time busy
- **S**aturation: queue depth / wait
- **E**rrors: count

Together: RED on services, USE on infra.

## Cardinality Discipline

- Labels with **bounded, low cardinality**: status code, method, env, service.
- **Never** label by user_id, request_id, full URL path. Use traces/logs for that.
- High cardinality kills your TSDB. Budget: tens of label combinations per metric.

## Structured Logging

```json
{
  "ts": "2024-01-15T10:23:45.123Z",
  "level": "error",
  "service": "orders",
  "env": "prod",
  "trace_id": "abc...",
  "span_id": "def...",
  "user_id": "u_123",
  "msg": "payment failed",
  "err": "insufficient_funds",
  "amount_cents": 5000
}
```

- JSON output.
- Consistent timestamp (RFC3339, UTC).
- Include `trace_id` / `span_id` for correlation.
- Log levels used consistently: error / warn / info / debug.
- No secrets, no PII. Redact at the logger.

## Tracing

- **OpenTelemetry** as the standard. Instrument once, export anywhere.
- **Sampling**: head-based for low traffic, tail-based for high (capture rare slow/error traces).
- **Context propagation**: W3C `traceparent` header across all hops.
- **Span attributes** for: status, error, db.statement (parameterized), http.route, peer.service.

## Alerting

### Alert on Symptoms, Not Causes

- ✅ "p95 latency > 500ms for 5min" (user-facing symptom)
- ❌ "CPU > 80%" (cause; may or may not impact users)

### Alert Hierarchy

- **Page** (wake someone up): SLO-burning conditions, customer impact.
- **Ticket**: degraded health, non-urgent issues.
- **Info**: telemetry, never paged on.

### Burn-Rate Alerts (Google SRE)

For a 99.9% SLO (0.1% error budget over 30d), page when:
- **Fast burn**: 14.4x normal error rate over 1h (burns 2% in 1h).
- **Slow burn**: 6x normal over 6h.

This gives both fast response to outages and catches slow leaks.

### Anti-Noise Rules

- Every alert must have a runbook link.
- Alerts that fire frequently with no action → delete or fix the cause.
- No alerts on transient single-sample spikes; require sustained windows.
- On-call review of every paged alert weekly.

## Dashboards

### Service Dashboard (canonical)

1. Top: SLO status + error budget remaining.
2. RED metrics with traffic, error, latency panels.
3. Saturation: queue depth, connection pool, memory.
4. Dependencies: latency/error to downstream services.
5. Deploy markers and incident annotations.

### Anti-Patterns

- 50-panel "wall of charts" with no priority.
- Different time ranges per panel (impossible to correlate).
- Dashboards that aren't linked from runbooks.

## OpenTelemetry Setup

```
App ──OTLP──→ Collector ──→ Metrics: Prometheus / Cortex / Mimir
                       ──→ Traces: Tempo / Jaeger
                       ──→ Logs: Loki / Elastic
```

Use the Collector as the buffer/router. Apps push to Collector, not to backends directly.

## SLO Implementation

```
SLI metric → recording rule (aggregate) → SLO target → burn-rate alert
```

Define SLOs in code (Sloth, OpenSLO) to keep them versioned.

## PII / Secrets Hygiene

- Redact at the logger boundary, not "later".
- Common offenders: full URLs (query params), request bodies, exception messages, stack traces.
- Use structured fields so redaction is mechanical.
- Authentication on `/metrics` endpoints if they expose business data.

## Output Template

```
## Service: <name>

## SLIs / SLOs
- <metric>: <target>% over <window>

## Metrics Emitted
- <name> {labels}: <type, unit>

## Logs
- Format: JSON
- Fields: <list>
- PII handling: <approach>

## Traces
- Library: <otel SDK>
- Sampling: <strategy>
- Key spans: <list>

## Alerts
| Severity | Condition | Runbook |

## Dashboards
- <name>: <URL or link>
```

## References

- *Observability Engineering* — Charity Majors et al.
- Google SRE alerting chapter
- OpenTelemetry: https://opentelemetry.io
- Prometheus best practices: https://prometheus.io/docs/practices/
