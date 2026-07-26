---
name: microservices-architect
description: Microservice decomposition, boundary identification, inter-service communication patterns, data consistency, service mesh. Load when carving boundaries or evaluating sync vs async between services.
metadata:
  audience: architect, fullstack-lt
---

# Microservices Architect

## When to Use

- Decomposing a monolith (e.g. `llmapps-main`)
- Defining service boundaries
- Choosing sync vs async integration
- Handling distributed data consistency
- Service mesh / sidecar decisions

## Triggers

microservice, bounded context, decomposition, monolith, strangler, saga, choreography, orchestration, service mesh, istio, linkerd, sidecar, api gateway, bff, event-driven, eventual consistency

## Boundary Heuristics

A service boundary is correct when:

- It maps to one **bounded context** (DDD).
- It owns its data (no shared DB outside the boundary).
- It can be deployed independently without coordinated releases.
- Its API is stable enough that internal refactors don't leak.
- A single team can own it end-to-end.

If any of those is false, the boundary is wrong.

## Decomposition Patterns

| Pattern | When |
|---|---|
| **Decompose by business capability** | Default; align with domain teams |
| **Decompose by subdomain (DDD)** | Complex domain with clear contexts |
| **Strangler fig** | Migrating from monolith incrementally |
| **Branch by abstraction** | Internal refactor before extraction |

## Communication Patterns

| Pattern | Use | Avoid when |
|---|---|---|
| Sync REST/gRPC | Immediate response needed | Long chains; downstream unreliable |
| Async events | Decoupling, fan-out, retries | Strong consistency required |
| Request-reply over queue | Async with response | Complexity overhead unjustified |
| Saga (choreography) | Multi-service transactions | <3 services; team can grok orchestrator |
| Saga (orchestration) | Multi-service transactions | Tight coupling to orchestrator OK |

## Data Consistency

- **Strong**: stay in one service. Don't cross boundaries with 2PC.
- **Eventual**: outbox pattern + idempotent consumers.
- **Read models**: CQRS where read/write needs diverge.
- **Sagas**: long-running multi-service flows with compensation.

## Outbox Pattern (canonical)

```
TX: write business state + outbox row in same DB transaction
    │
    └─→ Relay reads outbox → publishes to broker → marks sent
```

Guarantees: at-least-once delivery, ordered per aggregate, no dual-write inconsistency.

## Service Mesh — When

Adopt a mesh (Istio/Linkerd) when **at least two** apply:

- mTLS between services required by compliance.
- Need fine-grained traffic control (canary, mirror, fault inject).
- Need uniform observability across polyglot services.
- > 10 services and growing.

Avoid for < 5 services unless mTLS is mandatory — operational cost is real.

## API Gateway — When

Adopt when:

- Multiple client types (web, mobile, partner).
- Cross-cutting needs: authn, rate limit, request shaping.
- Want to hide internal topology from clients.

Prefer **BFF** (Backend For Frontend) over a monolithic gateway when client needs diverge.

## Anti-Patterns

- **Distributed monolith**: services that must be deployed together.
- **Shared DB across services**: re-couples what you tried to decouple.
- **Chatty interfaces**: N round-trips for one user action.
- **Sync chain > 3 hops deep**: failure modes compound.
- **No idempotency**: retries cause duplicates.
- **No correlation IDs**: can't trace cross-service flows.
- **Service-per-team without domain alignment**: Conway's law applied wrong.

## Migration: Monolith → Services (Strangler Fig)

```
1. Identify seam (one bounded context).
2. Build new service alongside; route a small % of traffic.
3. Validate parity (shadow traffic, dual-write where safe).
4. Cut over; remove old code path.
5. Repeat per seam.
```

Never big-bang. Never rewrite while shipping features in parallel for >1 quarter.

## Output Template

```
## Boundary Proposal
- Service: <name>
- Bounded context: <description>
- Data owned: <list>
- API: <REST/gRPC/events>
- Team: <owner>

## Integration
- Inbound: <who calls, how>
- Outbound: <who they call, how>
- Events emitted: <list>
- Events consumed: <list>

## Consistency
- Strong/eventual per flow
- Outbox? Saga? Read model?

## Trade-offs
<honest>
```

## References

- *Building Microservices* — Sam Newman
- *Microservices Patterns* — Chris Richardson
- *Implementing Domain-Driven Design* — Vaughn Vernon
