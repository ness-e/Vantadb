---
title: "ADR-039: JWT HS256 offline para SRV-06 (OIDC discovery DEFER)"
type: adr
status: accepted
tags: [vantadb, architecture, adr, auth, server]
created: 2026-09-10
last_reviewed: 2026-09-10
---

# ADR-039: JWT HS256 offline para SRV-06 (OIDC discovery DEFER)

## Context

SRV-06 exige autenticación enterprise ("equipos"): el mercado de facto es
weaviate OIDC nativo vs qdrant JWT RBAC HS256 offline
(`docs/api/HTTP_API.md:620`, `docs/operations/hardening.md:24`).
El servidor actual solo acepta Bearer opaco (`api_key`/`alt_api_key`,
`src/server/middleware.rs:42`) — sin expiración, sin claims, sin federación.
Restricciones: CI offline (pre-mortem del plan: OIDC discovery con fetch JWKS
rompe Fast Gate), appetite max 3d, scope enterprise creep documentado.

## Decision

MVP **HS256 offline** con crate `jsonwebtoken` v11 (backend `rust_crypto`
puro-Rust, `default-features = false`, opcional tras feature `server`):

- `VantaConfig.jwt_secret: Option<String>` vía `VANTADB_JWT_SECRET` (opt-in;
  `None` = comportamiento actual byte-idéntico).
- JWT válido (`HS256` + `exp` vigente + `sub` presente) → identidad
  `Transport` (L1-equivalente; reusa el path RBAC existente sin acoplarse a
  `EntityStore`/L3).
- Fallo JWT → mismo 401 genérico del path api-key (sin oráculo).
- **OIDC discovery (JWKS fetch, issuer/aud, rotación) → DEFER** explícito.

## Consequences

- Pros: sin red en runtime ni CI; HS256 verificado en µs fuera del hot path;
  aditivo (`Option` + default, sin migración, sin major bump); secreto
  HS256 compatible con tooling qdrant-like.
- Cons: secreto simétrico compartido (rotación manual: cambiar env +
  restart; documentado); sin federación SSO hasta OIDC; `Transport` no porta
  `sub` a RBAC por-token (el `sub` queda en audit via extensión futura —
  fuera del MVP).
- Deuda asumida: OIDC slice futuro añade `jwt_issuer`/`jwks_url` opcionales
  sobre el mismo módulo (One-Version Rule, sin forkear); flag CLI
  `--jwt-secret` DEFER (env-only en MVP).
