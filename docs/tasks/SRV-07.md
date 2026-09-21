# SRV-07 — Dockerfile unprivileged + wiring release

- **Plan:** docs/plans/2026-09-03-quality-gtm-wave.md (Task 5) | **Ruta:** vanta-worker | **Fecha:** 2026-09-03
- **Estado:** ✅ COMPLETED (commit `ci(docker): unprivileged + release build wiring (SRV-07)`)

## Impacto mapeado (Regla 0)

- **Leídos completos:** `Dockerfile` (raíz, 100L), `vantadb-server/Dockerfile` (112L), `docker-compose.yml` (21L, solo lectura — intocable), `docker-compose.dev.yml`, `.dockerignore`, `build.rs`, `release-binaries-63.yml` (150L), `vantadb-server/src/main.rs` (flags reales: `-h/--help/--mcp`), root `Cargo.toml` (members, 21 `[[bench]]`, 73 `[[test]]` path, `[[bin]]` path, `[profile.ci]`, MSRV 1.94.1 ≤ RUST_VERSION 1.95.0 ✅ regla release-ci.md §3).
- **Referencias entrantes a Dockerfile raíz:** `docker-compose.yml:3` (`build: .`), `docker-compose.dev.yml` (`target: builder` → el stage `builder` y `cargo-watch` DEBEN seguir existiendo ✅ preservados).
- **Referencias a workflow release:** ningún otro workflow referencia release-binaries-63.
- **Veredicto:** cambios acotados a build/runtime container + job aditivo en CI; cero símbolos Rust públicos; blast radius = imagen docker + lane release. `hardening.md` §5 referencia `vantadb-server/Dockerfile` (NO tocado — también roto, ver deuda).

## SDP

SDP: base-only (keywords: docker, unprivileged, release wiring; el contrato inline del orquestador cubrió discovery; ninguna skill de manifest aporta sobre Docker/CI infra además de `ci-cd-and-automation` conceptual).

## Steps

1. ✅ Rewritten root Dockerfile builder (skeleton layer eliminado — irrecuperable: manifest-load validation de [[bin]]/[[test]] paths + COPY desde cache-mount nunca commiteado; 2 `&&` sin `\` → build abortaba) + unprivileged: `chmod 777 /var/lib/vantadb` + `ARG VANTA_RUNAS_UID=1001`, `USER vantadb` preservado.
2. ✅ .dockerignore: re-incluir `tests/`+`benches/` (requisito manifest), agregar `data/` (guard contexto).
3. ✅ `release-binaries-63.yml`: job `docker-image` build-no-push + 2 smokes `--user 10001:10001` + docker save como release asset.
4. ✅ Docs: DEPLOYMENT_GUIDE §3 "Run unprivileged (arbitrary UID)"; CI_POLICY "Docker Image Publishing (SRV-07)" (decisión no-push documentada).
5. ✅ Verificación mecánica: `rg -n "^USER|runas|RUNAS" Dockerfile` = 5; `rg -ci docker release-binaries-63.yml` = 9; `actionlint` exit 0; continuation-lint Dockerfile OK.
6. ✅ Sin daemon local → `docker build/run` + `compose config -q` diferidos al job CI (nota explícita en CI_POLICY; no fake).

## Deuda abierta

- Verificación e2e del build en CI al primer dispatch/tag (gate real añadido por esta tarea).
- `vantadb-server/Dockerfile` roto (COPY a crate inexistente `vantadb/`, asset name del release-binary no coincide con lo que sube 63) → candidato FIND-*.
- Registry push: decisión de marca + credenciales.

## Recitation

contract: verificación arriba, claim-by-claim con comando+output · invariantes: stage `builder` + cargo-watch intactos (dev compose); volumen compose `/var/lib/vantadb` no cambió · deuda: ítems de arriba.
