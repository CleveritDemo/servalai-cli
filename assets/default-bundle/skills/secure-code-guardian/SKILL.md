---
name: secure-code-guardian
description: Secure coding practices applied while writing code. Defensive patterns, safe defaults, input validation, error handling, secret hygiene. Load when writing new code to bake in security from the start.
metadata:
  audience: developer, code-review, sec-ops-expert
---

# Secure Code Guardian

## When to Use

- Writing new code (before review)
- Modifying authn/authz, crypto, file handling, external I/O
- Refactoring legacy code with known bad patterns

## Triggers

secure coding, defensive programming, input validation, error handling, safe defaults, least privilege, fail closed, secret hygiene

## Top Defensive Patterns

### 1. Fail closed, not open

```python
# bad — defaults to allow
if not check_permission(user, resource):
    log.warn("permission check error")
return resource.data

# good — defaults to deny
try:
    allowed = check_permission(user, resource)
except Exception:
    raise PermissionDenied("permission check failed")
if not allowed:
    raise PermissionDenied()
return resource.data
```

### 2. Validate at the boundary

Every entrypoint (HTTP handler, queue consumer, CLI arg, deserializer) validates input before business logic runs.

### 3. Parameterize, never concatenate

```python
# bad
db.execute(f"SELECT * FROM users WHERE id = {user_id}")

# good
db.execute("SELECT * FROM users WHERE id = $1", [user_id])
```

Same for `exec`, template rendering, LDAP, NoSQL queries.

### 4. Least privilege everywhere

- Code runs as non-root.
- DB user has only the grants it needs.
- API tokens scoped to single purpose.
- File handles closed promptly; no extra perms.

### 5. Time-of-check / time-of-use safety

```python
# bad — race condition
if file.exists(path):
    open(path).read()

# good — try/except handles disappearance
try:
    with open(path) as f:
        f.read()
except FileNotFoundError:
    ...
```

### 6. Constant-time comparison for secrets

```python
import hmac
hmac.compare_digest(provided, expected)   # not `==`
```

For tokens, signatures, password hashes. `==` short-circuits and leaks via timing.

## Input Validation Checklist

For every external input:

- [ ] Type checked
- [ ] Length / range bounded
- [ ] Format matched (regex, schema)
- [ ] Charset / encoding normalized
- [ ] Allow-listed against expected values where finite
- [ ] If a path: resolved + contained within allowed root
- [ ] If a URL: scheme allow-listed, host not private/loopback/metadata (SSRF)
- [ ] If JSON: schema validated (no extra fields if strict)

## Error Handling

- **Catch specific exceptions**, not bare `except`.
- **Log with context** (correlation ID, user ID if safe), not just the message.
- **Don't leak internals** to clients — return generic 4xx/5xx; full detail in logs.
- **Map exceptions to status codes** at the boundary.
- **Don't swallow errors silently** unless explicitly intended; document why.

```python
try:
    result = service.call()
except SpecificError as e:
    log.warn("call failed", correlation_id=cid, err=str(e))
    raise ServiceUnavailable("temporary failure")
```

## Secret Hygiene

- **Never** log secrets. Mask at the logger boundary.
- **Never** include secrets in exception messages.
- **Never** in error responses to clients.
- **Never** in URL query parameters (logged by everything).
- **Always** read from secret manager at runtime; never bake into images/code.
- **Always** rotate on suspected leak. Have a documented rotation procedure.

## File Handling

- **Never trust filename or path from user**. Validate and constrain.
- **Resolve to absolute path** and verify it stays inside allowed directory:

```python
import os
base = os.path.realpath("/var/uploads")
full = os.path.realpath(os.path.join(base, user_path))
if not full.startswith(base + os.sep):
    raise BadRequest("path traversal")
```

- **Random filenames** for user-uploaded content; preserve original in metadata.
- **MIME / magic-byte check**, not just extension.
- **Size limits** enforced at the streaming layer, not after buffering.

## Deserialization

- **Never** use language-native untrusted deserialization (Python `pickle`, Java `ObjectInputStream`, PHP `unserialize`, Ruby `Marshal`).
- **Prefer** schema-validated formats: JSON Schema, Protobuf, Avro, MessagePack with strict mode.
- **YAML**: use `safe_load`, not `load`.

## Concurrency

- **Identify shared state** and protect it (locks, channels, atomics).
- **Prefer immutable data** when sharing across boundaries.
- **Bounded queues** for backpressure — never unbounded channels.
- **Time-out everything** that crosses a process or network boundary.

## Crypto Choices (cheat sheet)

| Need | Use |
|---|---|
| Password hash | argon2id |
| Symmetric encryption | AES-256-GCM |
| Authenticated msgs | HMAC-SHA256 / AES-GCM |
| Public-key signing | Ed25519 |
| Key exchange | X25519 |
| TLS | 1.2+ (1.3 preferred) |
| Random for security | OS CSPRNG (`secrets`, `crypto.randomBytes`) |
| Token IDs | UUIDv7 or 128-bit random |

## API Boundaries

- **Idempotency keys** on POST that mutates.
- **Rate limit** per identity + per IP at the edge.
- **Pagination caps** to prevent enumeration.
- **No internal IDs** that reveal counts/structure when unnecessary.
- **CORS strict**: explicit origins, no `*` with credentials.

## Container / Process

- Non-root user.
- `readOnlyRootFilesystem: true`.
- `allowPrivilegeEscalation: false`.
- Drop all capabilities, add back only what's needed.
- Minimal base image.

## Self-Review Before Requesting Review

Before tagging `@code-review` or `@sec-ops-expert`, walk through:

- [ ] Did I validate every external input?
- [ ] Did I parameterize every query and command?
- [ ] Did I avoid logging anything sensitive?
- [ ] Did I check authorization on every protected resource access?
- [ ] Did I handle errors specifically and avoid leaking internals?
- [ ] Did I add or update tests covering the new code paths?
- [ ] Did I avoid `TODO: fix later` for anything security-relevant?

## References

- OWASP Secure Coding Practices Quick Reference
- CWE Top 25
- *Secure by Design* — Johnsson, Deogun, Sawano
