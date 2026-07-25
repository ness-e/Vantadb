# Antigravity Technical Reference Checklist Suite

## 1. Security & Hardening (`security-checklist.md`)
- [ ] Sanitización de inputs en API pública y MCP (`DOMPurify`, validación de tipos).
- [ ] Manejo seguro de secrets y variables de entorno (`.env`, no hardcode).
- [ ] Verificación de bloques `unsafe` en Rust (`// SAFETY:` docs e invariantes).
- [ ] Seguridad en dependencias (`cargo deny check`, `cargo audit`).

## 2. Performance Optimization (`performance-checklist.md`)
- [ ] Evitar allocaciones innecesarias en hot paths (`smallvec`, `zerocopy`).
- [ ] Reducción de contention en locks (`DashMap`, `parking_lot`).
- [ ] Bundle size budget en Web y WASM (`<120KB gzip JS`).

## 3. Testing Patterns (`testing-patterns.md`)
- [ ] Red-Green-Refactor TDD.
- [ ] Cobertura de edge cases y caminos de error.
- [ ] Pruebas unitarias e integración en `tests/`.

## 4. Orchestration Patterns (`orchestration-patterns.md`)
- [ ] Patrones de subagentes Orquestador vs Leaf Nodes.
- [ ] Recitation pattern para memoria persistente.
- [ ] Triage gate y mitigación FMEA.

## 5. Reference Synthesis (`REFERENCE-SYNTHESIS.md`)
- Compendio unificado de los principios de ingeniería de VantaDB.
