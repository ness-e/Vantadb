---
description: analisis-impacto
---

Trigger: /analisis-impacto

Pasos del proceso:

1. Identificación: Localizar componentes afectados (Storage, HNSW, SDK) según todo.md.

2. Deducción FODA: Generar matriz con:

- Fortalezas: Ganancias en performance/latencia.

- Oportunidades: Nuevas capacidades indexación/búsqueda.

- Debilidades: Incremento de complejidad o consumo de RAM.

- Amenazas: Riesgo de corrupción de datos o inestabilidad en el linker MSVC.

1. Simulación de Fallos: Identificar los 3 puntos de ruptura más probables (ej. race conditions, desalineación de memoria).

2. Plan de Mitigación: Proponer pruebas unitarias o guardias de seguridad para los riesgos detectados.

3. Veredicto: Recomendar proceder, rediseñar o abortar.
