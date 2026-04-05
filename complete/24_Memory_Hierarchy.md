# Fase 24: Jerarquía de Memoria Bio-Técnica

ConnectomeDB optimiza el rendimiento y la durabilidad mediante una estructura de memoria de dos niveles, inspirada en la formación de recuerdos humanos.

## 1. Short-Term Memory (STNeuron)
Corresponde a la **Memoria de Trabajo** o Fase de Ingesta Activa.
- **Ubicación**: HashMap atómico en RAM (`cortex_ram`).
- **Propósito**: Alojar nodos con alta frecuencia de acceso o mutaciones recientes que aún no han sido consolidadas.
- **Acceso**: Latencia sub-microsegundo. Sin serialización.
- **Persistencia**: Protegida temporalmente por el **Axon** (WAL).

## 2. Long-Term Memory (LTNeuron)
Corresponde a la **Memoria Permanente**.
- **Ubicación**: RocksDB (SST Files en SSD/NVMe).
- **Propósito**: Almacenamiento masivo de conocimiento histórico y relaciones estables.
- **Acceso**: ~20ms (según I/O). Optimizado mediante el BlockCache.
- **Estructura**: Nodos serializados en `bincode`.

## Estrategia de Swapping (Mantenimiento Circadiano)
El paso de STN a LTN no es binario, sino que depende de la "energía" o relevancia del nodo:
- **Calentamiento**: Al consultar un LTNeuron con un `hits` alto, el motor puede decidir "subirlo" a RAM (STNeuron) para acelerar futuras inferencias.
- **Enfriamiento**: El `SleepWorker` degrada periódicamente los nodos en RAM hacia el disco si su frecuencia de uso cae por debajo del umbral bayesiano.

---

### Optimización "Survival Mode" (mmap)
En hardware limitado (16GB RAM), ConnectomeDB utiliza **Memory-Mapped Files** para el acceso directo a los vectores del Neural Index:
1. Los descriptores vectoriales se mapean desde el disco al espacio de direcciones virtual.
2. El sistema operativo gestiona el paginado de memoria según la demanda.
3. Esto permite navegar grafos de 1M+ de nodos sin saturar la RAM física, manteniendo la ilusión de una base de datos 100% in-memory.
