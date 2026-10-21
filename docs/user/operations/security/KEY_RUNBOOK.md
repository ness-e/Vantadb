---
title: "Runbook de claves: cifrado en reposo (encryption at rest)"
kind: runbook
status: active
description: "Donde se guarda la master key, como se rota, que se pierde si se pierde el registro, y los limites reales del cifrado en reposo de VantaDB (ADR-0057)."
tags: [vantadb, security, encryption, runbook, keys, operations]
created: "2026-10-21"
---

# Runbook de claves — cifrado en reposo (encryption at rest)

> **Que cubre este runbook:** la operacion del cifrado en reposo de VantaDB:
> donde vive la master key, como se rota, que se rompe si se pierde el
> registro, y que queda **fuera** del cifrado.
>
> **Diseno:** [`ADR-0057`](../../../dev/architecture/adr/ADR-0057-encryption-at-rest.md).
> **Claim publica:** [`SECURITY.md`](../../../../SECURITY.md).

---

## 1. Que se cifra (y que no)

El cifrado en reposo esta **encendido dos veces**: la feature `encryption` de
Cargo **no** esta en `default`, y sin master key el camino es plaintext
**byte a byte** (identico a un build sin la feature).

Con la feature compilada **y** una master key resuelta, se cifran los **cuatro
puntos de escritura independientes**:

| Artefacto | Ficheros | Granularidad del envelope | Version |
|---|---|---|---|
| **WAL** | `data/vanta.wal`, `data/vanta.shard{N}.wal` | por frame | `WAL_FORMAT_VERSION` 3 → **4** |
| **VantaFile** | `data/vstore_L{0..3}.vanta` | por nodo | `VFILE_VERSION` 2 → **3** |
| **HNSW image** | `data/vector_index.bin` | por fichero | `VECTOR_INDEX_VERSION` 8 → **9** |
| **backend KV** | journal y SSTs de Fjall / RocksDB | por valor | sin version propia (el registro es el marcador) |

Y **por transito**: el **indice de texto** (un keyspace del backend KV) y los
**snapshots** (un espejo de los cuatro).

### Lo que NO se cifra (limites declarados)

| Limite | Detalle |
|---|---|
| **Las claves del backend KV** | El namespace + el nombre de la clave quedan **en claro**: el indice de texto necesita prefix scans por namespace y el indice de namespace necesita lookups ordenados. Los **valores**, los postings y los metadatos si se cifran. |
| **Los tokens del indice de texto** | Son parte de la **clave** del keyspace, no del valor: quedan en claro. La clave del posting es `namespace ‖ campo ‖ string indexado ‖ clave`, asi que un **valor de campo de texto** (un valor de metadata, por ejemplo) se indexa **literalmente** en la clave y es legible en disco. |
| **El tamano de los valores** | El envelope es `plaintext + 28 bytes` (nonce de 12 + tag de 16): el tamano del valor se filtra. |
| **La estructura** | El numero de valores, su orden y su existencia dentro de un rango son observables; los ficheros (cabeceras, contadores, timestamps, numero de shards) siguen en claro. |
| **La master key en memoria** | Vive en la memoria del proceso mientras la base esta abierta: el cifrado protege **en reposo**, no contra un atacante con acceso a la memoria del proceso. |
| **El registro** | `data/encryption.registry` contiene las DEKs **envueltas** (cifradas con la KEK), el `db_salt` (publico) y el epoch de nonce actual. La master key **nunca** se persiste. |
| **El trafico de red** | `wal-shipping` saca registros **en claro** hacia la replica: la replica necesita la misma DEK y el transporte queda fuera del frontera "at rest". |
| **La migracion in-place** | No existe. Una base plaintext **nunca** se migra implicitamente (una key configurada en una base plaintext se ignora, con un warning). |

---

## 2. Donde se guarda la master key

Tres fuentes, en **precedencia fija** (la primera que resuelve gana; las otras
ni se leen):

| # | Fuente | Variable | Notas |
|---|---|---|---|
| 1 | **Entorno** | `VANTADB_ENCRYPTION_KEY` | Hex de 32 bytes. Es el contrato historico, pero **no** se recomienda en produccion: `/proc/<pid>/environ`, los core dumps y los logs de CI la exponen. |
| 2 | **Fichero** | `VANTADB_ENCRYPTION_KEY_FILE` | Ruta a un fichero con la key en hex. En Unix el motor **rechaza** un fichero con permisos de grupo/otros y un symlink (debe ser `0600`). En Windows el check es limitado a "fichero regular, no vacio". **La forma recomendada.** |
| 3 | **Comando** | `VANTADB_ENCRYPTION_KEY_CMD` | Un comando cuya stdout es la key en hex (shim de KMS: `aws kms decrypt ...`, `gcloud secrets versions access ...`). Se ejecuta **una vez al abrir**; la salida se captura en un buffer zeroizable y **nunca** se loguea. La key nunca toca el disco. |

Tambien se puede configurar **programaticamente** (`Config::encryption_key`):
esa es la via de los bindings (Python/TS/Node). El motor la resuelve **una vez
por proceso** y la reparte a los cuatro artefactos.

### Reglas de higiene

- La master key es de **32 bytes aleatorios** (`openssl rand -hex 32`). Una
  passphrase mas corta se estira con PBKDF2-HMAC-SHA256 (210 000 iteraciones)
  y el registro de la base, pero la forma recomendada sigue siendo la key
  aleatoria.
- **Nunca** la pases por la linea de comandos (`--new-key` la deja en el
  historial del shell y en la lista de procesos): usa `--new-key-file`.
- La master key **no** se persiste en ningun sitio de VantaDB. Si se pierde,
  la base cifrada **no** se puede abrir (ver §4).

---

## 3. Rotacion de la master key

La rotacion **re-envuelve** las DEKs: **no** las re-deriva. Ningun ciphertext
existente se invalida (`FIND-194`) y la operacion es O(artefactos) = 4, no
O(datos).

```powershell
# La base debe estar CERRADA (el registro lo reescribe el motor en vivo).
$env:VANTADB_ENCRYPTION_KEY_FILE = "C:\keys\vanta-old.key"
vanta-cli encryption rotate --db ./mi-db --new-key-file "C:\keys\vanta-new.key"
```

Que hace:

1. Copia el registro actual a `data/encryption.registry.bak` (**rollback
   manual**).
2. Sube el `key_version` y re-envuelve cada DEK con la KEK derivada de la
   nueva master key (el `db_salt` no cambia).
3. Reescribe el registro de forma atomica (tmp → rename → fsync del dir).

Deshacer:

```powershell
# Mientras la master key VIEJA siga existiendo:
Copy-Item ./mi-db/data/encryption.registry.bak ./mi-db/data/encryption.registry
# Y se vuelve a configurar la master key vieja.
```

La rotacion es **O(artefactos)** y no toca los datos: no hace falta
re-cifrar ni reindexar.

---

## 4. Que se pierde si se pierde el registro

> **REGLA DURA:** `data/encryption.registry` **nunca** se separa de la base.

- Las DEKs viven **solo** en el registro (envueltas). Sin el registro, la
  KEK no se puede derivar y **la base no se puede descifrar**: los artefactos
  devuelven `EncryptionKeyRequired` (sin clave) o `EncryptionKeyRejected`
  (con una clave que no es la suya) — **nunca** "corrupto".
- **El registro no es un fichero de datos**: es material de clave. Backupealo
  **con** la base y con los mismos permisos (0600 en Unix).
- Un snapshot lleva su propia copia del registro
  (`data/snapshots/<n>/data/encryption.registry`): el snapshot se **copia**,
  nunca se hard-linkea (si se compartiera el inode, una escritura posterior
  del motor mutaria el snapshot).

Si se pierde la master key pero se conserva el registro: la base sigue
abrible con una master key nueva **si** se ha hecho una rotacion previa (las
DEKs siguen siendo las mismas). Si no hay rotacion previa, la base esta
perdida.

---

## 5. Procedimiento: cifrar una base existente

No hay migracion in-place (ADR-0057 D7). El procedimiento es el de
export/import:

```powershell
# 1. Exportar (la base sigue en claro durante este paso).
vanta-cli export --db ./mi-db --out ./export

# 2. Configurar la clave.
$env:VANTADB_ENCRYPTION_KEY_FILE = "C:\keys\vanta.key"

# 3. Crear una base NUEVA con la clave configurada -> cifrada desde el primer byte.
vanta-cli import --db ./mi-db-cifrada --from ./export

# 4. Verificar: ningun payload en claro en los 6 ficheros.
```

El probe de canario (un test de integracion del repo,
`tests/encryption_canary.rs`) comprueba exactamente eso.

---

## 6. Diagnostico

| Sintoma | Causa | Que hacer |
|---|---|---|
| `EncryptionKeyRequired` al abrir | La base esta cifrada y no hay master key (o el entorno no la exporta) | Configurar `VANTADB_ENCRYPTION_KEY_FILE` (o `_CMD`) |
| `EncryptionKeyRejected` al abrir | La master key no es la de esta base (o el registro fue manipulado) | Comprobar que la key es la del `encryption.registry`; si se rotó, usar la key nueva |
| `EncryptionCorrupt` al abrir | El registro esta malformado (CRC, magic) | Restaurar `encryption.registry` del backup; si no hay, la base esta perdida |
| Warning "the database is PLAINTEXT" | La base es plaintext y hay una key configurada: VantaDB **no** migra implicitamente | Exportar e importar en una base nueva (seccion 5) |
| Un proceso no ve la key | El proceso tiene **varias** bases abiertas con claves distintas: la decision por proceso es ambigua | Un proceso por base, o usa `VANTADB_ENCRYPTION_KEY_FILE` en todos |

---

## 7. Referencias

- ADR: [`ADR-0057`](../../../dev/architecture/adr/ADR-0057-encryption-at-rest.md)
- Plan de cableado: [`2026-10-20-wiring-encryption-feature.md`](../../../dev/plans/2026-10-20-wiring-encryption-feature.md)
- Claim publica: [`SECURITY.md`](../../../../SECURITY.md)
- Configuracion: [`CONFIGURATION.md`](../CONFIGURATION.md)
- Backup y restauracion: [`BACKUP_POLICY.md`](../BACKUP_POLICY.md),
  [`BACKUP_RESTORE.md`](../BACKUP_RESTORE.md)
