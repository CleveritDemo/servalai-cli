---
name: api-designer
description: API design heuristics for REST, gRPC, GraphQL, and event schemas. Versioning, error handling, pagination, idempotency, contract testing. Load when designing or reviewing service contracts.
metadata:
  audience: architect, fullstack-lt, developer, code-review
---

# API Designer

## When to Use

- Designing a new API (REST, gRPC, GraphQL, async event)
- Reviewing an API change for backward compatibility
- Defining error formats, pagination, idempotency
- Producing OpenAPI / Proto / GraphQL schema

## Triggers

api, rest, grpc, graphql, openapi, swagger, proto, versioning, pagination, idempotency, contract, schema, event schema, webhook

## REST Principles

- **Resources**, not actions. Nouns in URLs. Verbs in HTTP methods.
- **Plural** collection names: `/users/{id}`, not `/user/{id}`.
- **No verbs in path** except for non-CRUD actions: `POST /orders/{id}/cancel`.
- **Status codes carry meaning**: 2xx success, 4xx client error, 5xx server error.
- **HATEOAS where it adds value**, not dogmatically.

### Standard Methods

| Method | Idempotent | Safe | Use |
|---|---|---|---|
| GET | ✅ | ✅ | Read |
| HEAD | ✅ | ✅ | Existence/metadata |
| POST | ❌ | ❌ | Create / non-idempotent action |
| PUT | ✅ | ❌ | Full replace (idempotent) |
| PATCH | ❌ (usually) | ❌ | Partial update |
| DELETE | ✅ | ❌ | Remove |

### Status Codes (canonical)

- `200 OK`, `201 Created` (with `Location`), `202 Accepted` (async), `204 No Content` (delete).
- `400 Bad Request`, `401 Unauthorized`, `403 Forbidden`, `404 Not Found`, `409 Conflict`, `422 Unprocessable Entity`, `429 Too Many Requests`.
- `500 Internal Server Error`, `502 Bad Gateway`, `503 Service Unavailable`, `504 Gateway Timeout`.

### Error Format (RFC 7807 Problem Details)

```json
{
  "type": "https://api.example.com/errors/insufficient-funds",
  "title": "Insufficient funds",
  "status": 422,
  "detail": "Account 1234 has balance $10, requested $50",
  "instance": "/transfers/abc-123",
  "code": "INSUFFICIENT_FUNDS",
  "traceId": "..."
}
```

### Pagination

- **Cursor-based** (preferred): `?cursor=xyz&limit=50`. Returns `{ items, next_cursor }`. Stable under inserts.
- **Offset-based**: `?page=2&size=50`. Simpler but breaks under churn.
- Always cap `limit`. Default 50, max 200 typical.

### Idempotency

- All `POST` that creates a resource should accept `Idempotency-Key` header.
- Server stores key → response for N hours (e.g. 24h).
- Repeated requests with same key return cached response.

### Versioning

Pick one and be consistent:
- **URL**: `/v1/users` — most explicit, easiest for clients.
- **Header**: `Accept: application/vnd.example.v1+json` — purest REST.
- **Query**: `?api-version=1` — pragmatic for early stages.

Breaking changes require a new version. Additive changes (new optional field, new endpoint) do not.

## gRPC

- Define in `.proto`. Reserve field numbers when removing.
- Field number ranges: 1-15 (1 byte), 16-2047 (2 bytes). Use 1-15 for frequent fields.
- Never reuse a field number. Mark removed: `reserved 5; reserved "old_name";`.
- Use `oneof` for sum types, `optional` (proto3) for explicit nullability.
- Streaming: client-stream / server-stream / bidi when batch or live is real, not theoretical.

## GraphQL

- One schema, evolved additively. Deprecate with `@deprecated(reason: "...")`.
- **N+1 problem**: dataloader pattern, batch loads per request.
- **Query depth/complexity limits** enforced server-side.
- Avoid exposing DB shape directly. Map to domain concepts.

## Event Schemas

- **Schema registry** mandatory (Avro/JSON Schema/Protobuf).
- **Forward + backward compatible** evolution.
- Include: `event_id`, `event_type`, `occurred_at`, `producer`, `version`, `correlation_id`, `payload`.
- Events describe **what happened**, not **what to do**. `OrderPlaced`, not `SendEmail`.
- One topic per event type, or one per aggregate — pick policy and stick to it.

## Contract Testing

- **Provider** publishes contract (OpenAPI / proto / Pact).
- **Consumer** tests against contract.
- CI fails on breaking contract change without version bump.

## Anti-Patterns

- Verbs in REST URLs: `/getUsers`, `/createOrder`.
- HTTP 200 with `{ "error": "..." }` in body.
- Inconsistent error formats across endpoints.
- No idempotency on payment / creation endpoints.
- Exposing internal IDs that leak business info.
- Returning DB schema directly as API shape.
- Mutating `GET` requests.
- Pagination without stable sort.

## Output Template

```
## Endpoint
<METHOD> /path

## Request
<headers, body schema>

## Response
- 200: <schema>
- 4xx: <conditions>
- 5xx: <conditions>

## Idempotency
<yes/no, key, TTL>

## Pagination
<cursor/offset, limits>

## Versioning Plan
<additive vs breaking>

## Backward Compatibility
<what existing clients see>
```

## References

- RFC 7807 Problem Details
- *API Design Patterns* — JJ Geewax
- Google API Design Guide: https://cloud.google.com/apis/design
