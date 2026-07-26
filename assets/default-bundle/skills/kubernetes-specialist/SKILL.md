---
name: kubernetes-specialist
description: Kubernetes manifest design and review. Workloads, networking, RBAC, NetworkPolicy, security context, resource limits, probes, Helm, ArgoCD GitOps. Load for any K8s manifest, Helm chart, or cluster posture task.
metadata:
  audience: architect, fullstack-lt, developer, sec-ops-expert
---

# Kubernetes Specialist

## When to Use

- Writing or reviewing K8s manifests, Helm charts
- Auditing cluster security posture
- Troubleshooting pods, services, ingress
- ArgoCD / GitOps configuration (`nodrize-argocd-pulzen`)
- RBAC, NetworkPolicy, Pod Security Standards

## Triggers

kubernetes, k8s, kubectl, helm, argocd, flux, gitops, deployment, statefulset, daemonset, configmap, secret, ingress, networkpolicy, rbac, serviceaccount, crd, operator, istio, linkerd

## Core Workflow

1. Analyze workload requirements (stateless/stateful, scaling, persistence, secrets).
2. Choose workload type (Deployment / StatefulSet / DaemonSet / Job / CronJob).
3. Define networking (Service, Ingress, NetworkPolicy).
4. Configure security (SA, RBAC, securityContext, PSS).
5. Add observability (probes, metrics annotations, log format).
6. Validate (`kubectl --dry-run=server`, kubeconform, polaris/kubesec).

## MUST DO

- Declarative YAML only. No imperative `kubectl create/run` in production.
- **Resource requests AND limits** on every container.
- **Liveness AND readiness probes** on every container.
- **Secrets via `Secret` or external secret manager** (Vault). Never in ConfigMap or env literal.
- **Named ServiceAccount** per workload (never `default`).
- **RBAC least privilege** — Role + RoleBinding scoped to namespace.
- **NetworkPolicy default-deny** + explicit allow rules.
- **securityContext**: `runAsNonRoot: true`, `readOnlyRootFilesystem: true`, `allowPrivilegeEscalation: false`, drop ALL capabilities.
- **Pinned image tags** (semver or digest). Never `latest`.
- **Labels** consistently: `app.kubernetes.io/{name,version,part-of,managed-by}`.

## MUST NOT DO

- Run as root without justification.
- Use `latest` image tag in any environment.
- Store secrets in ConfigMaps or hardcoded env.
- Default ServiceAccount with cluster-admin.
- Allow-all NetworkPolicy or no NetworkPolicy.
- Skip probes — orchestrator can't recover what it can't observe.
- `hostNetwork`, `hostPID`, `privileged: true` without explicit ADR justifying.

## Canonical Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  namespace: my-ns
  labels:
    app.kubernetes.io/name: my-app
    app.kubernetes.io/version: "1.2.3"
spec:
  replicas: 3
  selector:
    matchLabels:
      app.kubernetes.io/name: my-app
  template:
    metadata:
      labels:
        app.kubernetes.io/name: my-app
        app.kubernetes.io/version: "1.2.3"
    spec:
      serviceAccountName: my-app-sa
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 2000
        seccompProfile: { type: RuntimeDefault }
      containers:
      - name: app
        image: registry/my-app:1.2.3
        ports: [{ containerPort: 8080 }]
        resources:
          requests: { cpu: 100m, memory: 128Mi }
          limits:   { cpu: 500m, memory: 512Mi }
        livenessProbe:
          httpGet: { path: /healthz, port: 8080 }
          initialDelaySeconds: 15
          periodSeconds: 20
        readinessProbe:
          httpGet: { path: /ready, port: 8080 }
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities: { drop: ["ALL"] }
        envFrom:
        - secretRef: { name: my-app-secret }
```

## Minimal RBAC

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: my-app-sa
  namespace: my-ns
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: my-app-role
  namespace: my-ns
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: my-app-rb
  namespace: my-ns
subjects:
- kind: ServiceAccount
  name: my-app-sa
  namespace: my-ns
roleRef:
  kind: Role
  name: my-app-role
  apiGroup: rbac.authorization.k8s.io
```

## NetworkPolicy (default-deny + allow)

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny
  namespace: my-ns
spec:
  podSelector: {}
  policyTypes: ["Ingress", "Egress"]
---
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-from-frontend
  namespace: my-ns
spec:
  podSelector: { matchLabels: { app.kubernetes.io/name: my-app } }
  policyTypes: ["Ingress"]
  ingress:
  - from:
    - podSelector: { matchLabels: { app.kubernetes.io/name: frontend } }
    ports:
    - { protocol: TCP, port: 8080 }
```

## Helm

- Values: `values.yaml` defaults + `values-{env}.yaml` overlays.
- Templates use `include` for reusable blocks; avoid `printf` chains.
- `_helpers.tpl` for naming, labels, selectorLabels.
- `Chart.yaml` versioning: bump `version` on any template change; `appVersion` tracks app.
- Use `--atomic` and `--wait` in CI.

## ArgoCD / GitOps

- One `Application` per service per env, or `ApplicationSet` for fleets.
- Sync waves for ordering (CRDs before workloads).
- Auto-sync with `prune: true, selfHeal: true` in non-prod; manual in prod.
- Sensitive values via SealedSecrets / ESO / SOPS — never plaintext in git.

## Validation Commands

```bash
kubectl rollout status deployment/<name> -n <ns>
kubectl get pods -n <ns> -w
kubectl describe pod <pod> -n <ns>
kubectl logs <pod> -n <ns> --previous
kubectl top pods -n <ns>
kubectl auth can-i --list --as=system:serviceaccount:<ns>:<sa>
kubectl rollout undo deployment/<name> -n <ns>
```

## Pulzen Cluster Contexts

- `cadi-k3s` — `~/.kube/cadi-k3s.yaml`
- `nodrik3s1-k3s` — `~/.kube/nodrik3s1-k3s.yaml`

Both available read-only via kubernetes MCP servers.

## References

- Kubernetes docs: https://kubernetes.io/docs/
- Pod Security Standards: https://kubernetes.io/docs/concepts/security/pod-security-standards/
- ArgoCD: https://argo-cd.readthedocs.io
