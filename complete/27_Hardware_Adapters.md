# Fase 27: Adaptadores de Hardware (Modo Camaleón)

ConnectomeDB está diseñado para ser "Hardware-Agile", ajustando sus parámetros de gobernanza y persistencia según la infraestructura donde se despliega.

## 1. Perfil "Survival" (Edge/Laptop Mode)
Optimizado para entornos con recursos limitados (ej. 16GB RAM, CPU de consumo).
- **BlockCache**: Limitado estrictamente a 2GB.
- **Poda Sináptica**: Agresiva. El `SleepWorker` corre con una cadencia alta (cada 5s).
- **Jerarquía de Vectores**: Uso intensivo de `I8 Quantization` para reducir la huella de memoria del índice HNSW en un 75%.
- **Checkpoints**: Mantiene solo los últimos 3 snapshots de "Seguro de Vida".

## 2. Perfil "Enterprise" (Server Mode)
Optimizado para servidores dedicados y clusters distribuidos.
- **BlockCache**: Escala proporcionalmente a la RAM disponible.
- **Poda Sináptica**: Diferida. Se prioriza la retención total y solo se comprime si hay presión de disco.
- **Jerarquía de Vectores**: FP32 (Full Precision) nativo. Navegación del grafo 100% en RAM si es posible.
- **Checkpoints**: Historial completo de cambios (Audit Trail) habilitado de forma predeterminada.

---

## 3. Auto-Detección de Entorno
En el arranque (`main.rs`), el motor encuesta el sistema:
1. **CPU Check**: ¿Soporta instrucciones AVX-512/NEON? (Activa aceleración SIMD).
2. **RAM Check**: Si RAM < 16GB, inyecta automáticamente el `SurvivalProfile`.
3. **I/O Check**: Mide la latencia de escritura en la carpeta de datos para ajustar el tamaño de las `MemTables` de RocksDB.

## 4. Gobernanza del Techo Térmico
Para laptops en entornos de alta temperatura, el motor implementa un **Throttling Cognitivo**: si la CPU reporta sobrecalentamiento, se incrementa la latencia artificial entre operaciones de inferencia pesadas para permitir el enfriamiento pasivo, evitando el `Thermal Throttling` del sistema operativo.
