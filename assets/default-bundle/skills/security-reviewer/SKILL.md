---
name: security-reviewer
description: Application security audit. OWASP Top 10, authn/authz, injection, crypto, dependency CVEs, secrets, supply chain. Load for any security audit of code or configs.
metadata:
  audience: sec-ops-expert, code-review, developer
---

# Security Reviewer

## When to Use

- Auditing a diff or branch for security
- Reviewing authn/authz changes
- Triaging dependency CVE alerts
- Threat modeling a new feature
- Pre-release security gate

## Triggers

security, vulnerability, owasp, cve, authn, authz, oauth, jwt, sql injection, xss, csrf, ssrf, deserialization, crypto, secret, supply chain, dependency, sast, dast

## OWASP Top 10 Lens (2021)

For every diff, ask:

1. **A01 Broken Access Control** — does this enforce ownership / role checks?
2. **A02 Cryptographic Failures** — TLS? Strong algos? Key management?
3. **A03 Injection** — parameterized queries? Escaped templates? No `eval()`?
4. **A04 Insecure Design** — threat model done? Abuse cases considered?
5. **A05 Security Misconfiguration** — secure defaults? Disabled debug/admin?
6. **A06 Vulnerable Components** — pinned, scanned, up to date?
7. **A07 Identification/Authn Failures** — strong session mgmt? Rate-limited login?
8. **A08 Software/Data Integrity** — signed artifacts? Verified updates?
9. **A09 Logging/Monitoring Failures** — security events logged? Alerted?
10. **A10 SSRF** — outbound URLs validated? Metadata endpoints blocked?

## Authentication

- **MFA** for admin/sensitive accounts.
- **Argon2id / scrypt / bcrypt** for password hashing. Never MD5/SHA1/raw SHA256.
- **Session tokens** — random ≥128 bits, HttpOnly, Secure, SameSite=Lax/Strict, short TTL with refresh.
- **JWT pitfalls**: validate `alg`, reject `none`, verify signature, check `exp`/`nbf`/`iss`/`aud`, short TTL.
- **Login rate-limit + lockout**, account-level not just IP.

## Authorization

- **Default deny** for every resource.
- **Object-level checks** — never trust the client's ID. Always verify ownership.
- **Role + attribute** for complex cases (ABAC).
- **Don't trust the LLM/agent** to enforce permissions — enforce in the tool/service layer.
- **No `is_admin` flag** in user-controllable JSON.

## Input Validation

- **Allow-list** > deny-list.
- **At the boundary** (controller/handler), not deep in business logic.
- **Type + range + format** — every untrusted input.
- **For paths**: normalize, then check it stays within allowed root (`startswith` after `realpath`).
- **For URLs (SSRF)**: parse, resolve DNS, reject private/loopback/link-local/metadata.

## Injection Defenses

| Type | Defense |
|---|---|
| SQL | Parameterized queries / ORMs. Never string-build. |
| Command | `exec` with arg array; never shell concatenation. Allow-list binary path. |
| LDAP | Escape special chars; parameterized filters. |
| Template (SSTI) | Auto-escape; never `safe`/`raw` with user input. |
| XPath | Parameterized expressions. |
| NoSQL | Parameterized queries; reject operators in user input. |
| Header | Reject CR/LF. Use framework helpers. |

## Cryptography

- **TLS 1.2+ only**. Disable old ciphers. HSTS for web.
- **Symmetric**: AES-256-GCM. Never ECB.
- **Asymmetric**: RSA 2048+ or Ed25519/X25519. Never sign + encrypt with same key.
- **Hashing (not passwords)**: SHA-256+. Never MD5/SHA1 for security purposes.
- **Passwords**: argon2id (preferred), bcrypt, scrypt. With unique salt.
- **Randomness**: CSPRNG (`/dev/urandom`, `crypto.randomBytes`). Never `Math.random()` for secrets.
- **Don't roll your own crypto.** Use vetted libraries.

## Secrets

- **Never in source code.** Scan with gitleaks / trufflehog in CI.
- **Never in logs / errors / metrics.** Redact at the boundary.
- **Never in client-side code.** Browser bundles, mobile apps, CLI tools shipped to users.
- **Store** in secret manager (Vault, AWS Secrets Manager, GCP SM). Rotate.
- **Detect** leaks: monitor public registries, gitleaks pre-commit + CI, repo scanning.

## Dependencies / Supply Chain

- **Pin** to specific versions (`==` / `=` / lockfile).
- **SBOM** on every build (Syft, CycloneDX).
- **Scan** every commit (Snyk, Trivy, Grype, OSV-Scanner).
- **Triage** policy: CRITICAL → fix this sprint, HIGH → next sprint, MEDIUM → tracked.
- **Verify signatures** for tooling installs (Cosign, GPG).
- **Mirror** critical deps internally if supply-chain attacks are a concern.

## Web-Specific

- **CSP** — `default-src 'self'`, no `unsafe-inline`.
- **CSRF** — SameSite cookies + CSRF token for state-changing.
- **CORS** — explicit origins, not `*`, especially with credentials.
- **Security headers**: HSTS, X-Frame-Options/`frame-ancestors`, X-Content-Type-Options.
- **XSS**: framework-level escaping; never `innerHTML` with user input; sanitize HTML with DOMPurify if needed.

## Threat Modeling — STRIDE Lens

For each component:

- **S**poofing — can someone impersonate?
- **T**ampering — can data be modified in transit/rest?
- **R**epudiation — can actions be denied? Need audit log?
- **I**nformation disclosure — data leaks?
- **D**enial of Service — overload paths?
- **E**levation of Privilege — boundary crossing?

## Output Format

```
## Verdict: PASS | BLOCKED

## Summary
<2-3 lines>

## CRITICAL (merge-blocking)
- [file:line] <issue> — <impact> — <fix>

## HIGH
- [file:line] <issue> — <fix>

## MEDIUM
- [file:line] <issue>

## LOW / Hygiene
- [file:line] <issue>

## Out-of-Scope (escalate)
- <items needing arch change>
```

## References

- OWASP Top 10 (2021): https://owasp.org/Top10/
- OWASP Cheat Sheet Series: https://cheatsheetseries.owasp.org
- CWE Top 25
- NIST SP 800-63B (digital identity / passwords)
