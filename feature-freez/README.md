# feature-freez/

Carpeta de features congeladas: código que se deja de desarrollar, compilar
y testear en CI hasta nuevo aviso, pero se conserva intacto en el repo.

## Contenido

| Ruta | Estado desde | Motivo |
|---|---|---|
| `desktop/` | 2026-10-16 (decisión owner) | App Tauri congelada: sin feature work, fixes ni jobs de CI |

## Reglas del freeze

- NO editar nada bajo esta carpeta (ni código, ni docs internos, ni configs).
- NO referenciarla desde CI, scripts o gates (sus paths no existen para el pipeline).
- Los docs vivos (`docs/dev/desktop/`, `docs/user/desktop/`) apuntan aquí como
  ubicación del código; al hacer unfreeze hay que revertir esos refs también.

## Unfreeze (procedimiento)

1. `git mv feature-freez/desktop desktop` (o la subcarpeta que corresponda).
2. Revertir los refs: `rg -n "feature-freez/desktop" --glob "!target/**" --glob "!node_modules/**"`.
3. Re-activar CI/dependabot según el plan vigente y verificar gates en verde.
4. Registrar en `docs/dev/Backlog.md` + task file correspondiente.
