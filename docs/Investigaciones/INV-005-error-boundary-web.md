# Reporte de Auditoría y Propuesta — INV-005: ErrorBoundary en Web Frontend

> **ID:** `INV-005`  
> **Categoría:** Phase 4 — Engineering Health & Architecture  
> **Fecha:** 2026-07-31  
> **Estado:** ✅ Auditoría Completada — Propuesta Lista  

---

## 1. Contexto y Objetivos

`react-error-boundary` aparece listado en el archivo de dependencias de la aplicación web (`web/package-lock.json`), pero es necesario verificar si se está utilizando o si la aplicación Next.js App Router ya cubre la captura de errores a través de sus convenciones nativas (`error.tsx` / `global-error.tsx`).

---

## 2. Hallazgos del Análisis de la Aplicación Web (`web/`)

1. **Estructura de la Aplicación (Next.js App Router):**
   - El sitio en `web/` utiliza **Next.js 15 App Router**.
   - El archivo principal `web/src/app/layout.tsx` configura el shell global (`SiteShell`), fuentes, metadatos y proveedores de contexto (`LanguageProvider`).
   - No se encontró ningún archivo `error.tsx` o `global-error.tsx` en `web/src/app/`.

2. **Uso de `react-error-boundary`:**
   - No se encontraron importaciones activas de `react-error-boundary` en ningún componente UI dentro de `web/src/`.
   - La dependencia existe en `package-lock.json` pero no se aprovecha ni en layout ni en componentes individuales.

3. **Naturaleza del Sitio:**
   - La aplicación es primordialmente un portal de documentación, aterrizaje y demos interactivas simples.
   - Sin embargo, componentes dinámicos como el playground o los visualizadores de benchmarks podrían beneficiarse de una barrera de contención ante excepciones en tiempo de ejecución.

---

## 3. Propuesta de Implementación / Limpieza

Existen dos alternativas claras para resolver esta inconsistencia:

### Opción A: Adoptar la Convención Nativa de Next.js App Router (Recomendada)
Crear el archivo `web/src/app/error.tsx` para capturar cualquier error no controlado en componentes hijo sin necesidad de paquetes externos:

```tsx
'use client';

import { useEffect } from 'react';

export default function Error({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error('Unhandled app error:', error);
  }, [error]);

  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] p-6 text-center">
      <h2 className="text-xl font-bold mb-2">Algo salió mal</h2>
      <p className="text-sm text-gray-400 mb-4">Ocurrió un error inesperado al cargar esta sección.</p>
      <button
        onClick={() => reset()}
        className="px-4 py-2 bg-neutral-800 hover:bg-neutral-700 rounded-md text-sm"
      >
        Reintentar
      </button>
    </div>
  );
}
```

Luego, **desinstalar `react-error-boundary`** para reducir el footprint de dependencias no utilizadas.

### Opción B: Utilizar `react-error-boundary` en `layout.tsx`
Si se prefiere mantener la librería, envolver el contenido dentro de `layout.tsx` con `<ErrorBoundary>` especificando una interfaz de respaldo (*fallback UI*).

---

## 4. Conclusión

Se recomienda aplicar la **Opción A**: remover la dependencia redundante `react-error-boundary` e implementar `error.tsx` nativo en el App Router de Next.js.

---
*Reporte generado automáticamente como parte de INV-005.*
