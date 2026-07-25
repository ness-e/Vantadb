# Antigravity Task System Rules (RULES.md)

1. **Sin Comandos de Prueba Autónomos:** Los comandos de prueba (`npm test`, `pytest`, `cargo test`, `docker run`) **no se ejecutan de forma autónoma**. Se deben mostrar en bloques de código para ejecución manual del usuario.
2. **Confirmación de Imágenes:** Solicitar confirmación previa al usuario antes de usar `generate_image`.
3. **Uso Obligatorio de CodeGraph:** Usar `codegraph_explore` para consultas de código antes de realizar `grep` o búsquedas ciegas.
4. **Commits Semánticos:** Todo commit debe seguir la convención de Commits Semánticos (`feat:`, `fix:`, `docs:`, `test:`, `perf:`).
5. **Respuesta en Español:** Formato Markdown técnico de alta densidad en español.
