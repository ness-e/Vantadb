# Gestión de Estado — Frontend Web

> **Contexto:** Next.js 16 + React 19. Sin React Server Components (todo `"use client"`).
> Sin bibliotecas de estado externas en uso activo — zustand, next-intl y react-query están en
> `package.json` pero no se importan en ningún módulo de la aplicación.

---

## 1. Arquitectura de estado

El frontend de VantaDB maneja cuatro tipos de estado, cada uno resuelto con un mecanismo diferente:

| Tipo | Mecanismo | ¿Reactivo? | Persistente |
|------|-----------|------------|-------------|
| i18n (idioma) | React Context (`LanguageProvider`) | Sí | `localStorage` |
| Tema (claro/oscuro) | `next-themes` (ThemeProvider) | Sí | No efectivo |
| UI global (modales, overlays) | Eventos sintéticos del DOM | No | No |
| Datos de contenido | Importaciones directas (`vanta-data.ts`) | No | No |

No existe estado global mutable compartido entre páginas. Cada página importa lo que necesita
y los componentes de UI global se orquestan desde `SiteShell`.

---

## 2. LanguageProvider — Contexto de i18n

**Archivo:** `src/lib/language-provider.tsx`

`LanguageProvider` envuelve la aplicación con un React Context que expone:

```tsx
const { t, lang, setLang, mounted } = useLanguage()
```

### Mecanismo

- `t(key)` busca una clave en un `Record<string, string>` plano (no hay anidamiento de objetos).
  Si la clave no existe, devuelve la misma key como fallback.
- `tt(key, fallback)` wrapper que devuelve `fallback` si `t(key)` no encontró traducción. El
  contenido base (`vanta-data.ts`) está en **español**, y los diccionarios EN en `dictionaries.ts`
  sobrescriben valores. El fallback efectivo de `tt()` es el texto en español de `vanta-data.ts`.
- Los diccionarios se importan estáticamente desde `dictionaries.ts` — no hay carga lazy ni
  chunk splitting por idioma.

### Persistencia

En cada cambio de idioma se ejecuta `localStorage.setItem("lang", lang)`. Al montar, el
contexto lee `localStorage.getItem("lang")` para restaurar la preferencia. Si no hay valor
guardado, el idioma por defecto es `"es"`.

### SSR / Doble render

Al ser un Context personalizado (no `next-intl`), el servidor no tiene acceso al dato de
`localStorage`. Esto produce un doble render en el primer paint: el servidor renderiza con el
default y el cliente re-renderiza al leer `localStorage`. La prop `mounted` permite a los
componentes hijos ocultar contenido hasta que el contexto se haya hidratado.

### html lang

El atributo `<html lang="es">` está hardcodeado en el layout raíz. No se actualiza cuando
el usuario cambia de idioma vía `setLang`.

---

## 3. Sistema de temas — next-themes (stub)

**Archivo:** `src/components/vanta/theme-toggle.tsx`

### Configuración

```tsx
<ThemeProvider attribute="class" defaultTheme="light" enableSystem={false}>
```

- `ThemeProvider` de `next-themes` envuelve la app.
- `ThemeToggle` renderiza un botón que llama `setTheme("dark")` / `setTheme("light")`.
- El toggle agrega/remueve la clase `dark` en `<html>`.

### No funcional

No hay reglas CSS definidas para `.dark` en `globals.css`. El toggle existe, el atributo
`class` se modifica, pero ningún elemento cambia visualmente. Dark mode es una feature
declarada pero no implementada — la infraestructura está lista, falta el diseño visual.

---

## 4. Sistema de eventos sintéticos

**Archivo:** `src/components/vanta/site-shell.tsx`

La comunicación entre componentes que no comparten un ancestro común se resuelve con
eventos custom del DOM, no con Context ni estado global.

### Atajos de teclado globales

`SiteShell` registra un `document.addEventListener("keydown", handler)` en un `useEffect`.
El handler intercepta tres combinaciones:

| Tecla | Acción | Componente |
|-------|--------|------------|
| `⌘K` / `Ctrl+K` | Abre paleta de comandos | `CommandPalette` |
| `?` (`Shift+/`) | Muestra overlays de atajos | `ShortcutOverlay` |
| Escribir "vanta" | Activa easter egg | `EasterEgg` |

### Comunicación hijo → padre

Los componentes hijos (`CommandPalette`, `ShortcutOverlay`, etc.) no reciben props de
apertura/cierre. En lugar de eso:

```tsx
document.dispatchEvent(new CustomEvent("close-command-palette"))
```

`SiteShell` escucha estos eventos y maneja el estado de visibilidad internamente.

### Fragilidad conocida

Este patrón es frágil por varias razones:

- No hay tipado para los nombres de eventos ni sus payloads — un typo en el string name
  silencia el evento sin advertencia.
- La depuración requiere inspeccionar `document` en DevTools; no hay un registro central.
- Si `SiteShell` se desmonta (p.ej., en una página sin layout compartido), los atajos
  globales y la comunicación entre hijos dejan de funcionar.
- No hay `AbortController` en el `addEventListener` — el efecto limpia con `removeEventListener`
  explícito, pero es fácil olvidar el cleanup en futuras extensiones.

---

## 5. SiteShell — Orquestación de UI global

**Archivo:** `src/components/vanta/site-shell.tsx`

El layout raíz renderiza `SiteShell`, que monta todos los componentes de UI global en un
orden fijo:

```
ScrollProgress
  → SiteNavbar
    → <main> + PageTransition (children)
      → Footer
        → BackToTop
          → CommandPalette
            → ShortcutOverlay
              → EasterEgg
```

`SiteShell` es el único lugar donde se gestiona el estado de visibilidad de los overlays
globales. Los hijos notifican su intención de abrirse/cerrarse mediante los eventos
sintéticos descritos en la sección anterior.

---

## 6. Datos estáticos — vanta-data.ts

**Archivo:** `src/lib/vanta-data.ts`

25 exportaciones, todas `const as const`. No hay indirección de estado, ni contextos, ni
carga asíncrona. Los datos son literales TypeScript que se importan directamente donde se
necesitan:

```ts
import { FEATURES, METRICS, TESTIMONIALS } from "@/lib/vanta-data"
```

Los componentes consumidores son puramente presentacionales: reciben datos estáticos y
renderizan. No hay fetching, caché, ni estado de carga.

### Por qué funciona

La totalidad del contenido del sitio es conocida en build time. No hay contenido generado
por usuarios, CMS, ni datos externos. vanta-data.ts equivale a un "static store" sin
indirección — y para este modelo de datos, esa simplicidad es correcta.

---

## 7. Decisiones — por qué NO se usan bibliotecas instaladas

Tres bibliotecas de estado/fetching están presentes en `package.json` pero tienen **cero
imports** en la aplicación:

### zustand

Aparece en `package.json` (posiblemente de un template inicial o un experimento anterior).
No hay stores definidos, ni `create()`, ni `useStore()`. La app no tiene estado global
mutable que justifique zustand: el único estado compartido es el idioma (resuelto con
Context) y la visibilidad de overlays (resuelto con eventos sintéticos).

### next-intl

Instalado pero no usado. VantaDB implementó su propio `LanguageProvider` en lugar de
usar la solución del ecosistema Next.js. La razón no está documentada, pero el resultado
es un sistema más simple (sin configuración de rutas i18n, sin carga lazy de
diccionarios, sin server-side detection) a costa de SSR imperfecto.

### @tanstack/react-query

Instalado pero sin `QueryClient` ni hooks. No hay endpoints REST, GraphQL, ni fuentes de
datos asíncronas. Todo el contenido es estático y se importa directamente.

---

## 8. Anti-patrones conocidos

1. **Eventos sintéticos frágiles.** Comunicación hijo→padre mediante `CustomEvent` sin
   tipado, sin registro central, sin verificación en tiempo de compilación. Un cambio de
   nombre o un typo pasa desapercibido hasta runtime. Alternativa recomendada: contexto de
   UI con un reducer o callbacks explícitos en props (mientras el número de overlays siga
   siendo pequeño).

2. **ThemeProvider stub.** `next-themes` está integrado pero dark mode no existe visualmente.
   El toggle modifica `document.documentElement.classList` sin efecto visible. Esto envía
   señales contradictorias al usuario: la UI sugiere una funcionalidad que no funciona.

3. **Doble render por i18n.** El `LanguageProvider` custom produce un flash de contenido en
   el idioma incorrecto en SSR. `mounted` atenúa el síntoma pero no lo resuelve.

4. **`<html lang>` hardcodeado.** El atributo `lang` no se actualiza al cambiar de idioma,
   lo que afecta accesibilidad y semántica.

5. **`tt()` como fallback duplicado.** Cada componente que usa `tt(key, fallback)` hardcodea
   el texto en inglés como segundo argumento. Si el diccionario se actualiza, los fallbacks
   quedan obsoletos. Un solo `t()` que devuelva el texto en inglés por defecto eliminaría
   esta duplicación.
