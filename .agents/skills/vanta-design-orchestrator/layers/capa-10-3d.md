# CAPA 10 — 3D AVANZADO [OPCIONAL]

> ⚠️ **Capa condicional.** Se activa SOLO si el proyecto incluye escenas Three.js complejas, shaders personalizados o geometría 3D interactiva. Preguntar al usuario al iniciar la tarea.

---

## `threejs-fundamentals` — Fundamentos de Three.js

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de fundamentos: setup de escena, cámaras (Perspective/Orthographic), renderer (WebGL/WebGPU), jerarquía Object3D, y sistemas de coordenadas. |
| **¿Para qué es?** | Establecer la base de cualquier proyecto Three.js: scene, camera, renderer, y el loop de renderizado. |
| **¿Para qué se usa?** | Inicializar una escena 3D, configurar la cámara (posición, fov, near/far planes), elegir el renderer (WebGL vs WebGPU), y manejar transforms. |
| **¿Cómo se usa?** | `const scene = new THREE.Scene()` → `const camera = new THREE.PerspectiveCamera(75, w/h, 0.1, 1000)` → `const renderer = new THREE.WebGLRenderer()` → `renderer.render(scene, camera)`. |
| **¿Cómo debería usarse?** | Primera skill de Three.js a consultar. Establece el scaffold básico. Las demás threejs-* skills se consultan según la necesidad específica. |
| **¿Cuándo debería usarse?** | **Fase 3** — Si el proyecto incluye elementos 3D. Esta es la puerta de entrada a todas las threejs-* skills. |
| **Dependencias** | `npx skills add cloudai-x/threejs-skills@threejs-fundamentals -g`. Proyecto: `npm install three`. |
| **Requerimientos** | Node 18+. Three.js r150+. WebGL 1/2 o WebGPU. GPU con soporte WebGL. |

## `threejs-geometry` — Geometría y BufferGeometry

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de creación de geometría: built-in shapes, BufferGeometry personalizada, geometría dinámica, e instancing (InstancedMesh). |
| **¿Para qué es?** | Crear y manipular mallas 3D desde formas simples hasta geometrías complejas con vértices personalizados. |
| **¿Cómo se usa?** | `new THREE.BoxGeometry(1,1,1)` para formas simples. `new THREE.BufferGeometry()` + `setAttribute('position', ...)` para geometría custom. |
| **¿Cómo debería usarse?** | Después de `threejs-fundamentals`. Para geometrías simples usa built-ins; para orgánicas usa BufferGeometry. |
| **¿Cuándo debería usarse?** | **Fase 3** — Al crear elementos 3D: logos 3D, partículas, terrenos, o cualquier mesh personalizado. |
| **Dependencias** | `npx skills add cloudai-x/threejs-skills@threejs-geometry -g`. Proyecto: `npm install three`. |
| **Requerimientos** | Node 18+. Three.js r150+. WebGL 1/2. |

## `threejs-materials` — Materiales y Texturas

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de materiales Three.js: PBR (Standard, Physical), básicos (Basic, Lambert, Phong), shader materials, texturas, environment maps, y propiedades de material. |
| **¿Para qué es?** | Definir el aspecto visual de los objetos 3D: color, textura, rugosidad, metálico, emisión, transparencia. |
| **¿Cómo se usa?** | `new THREE.MeshStandardMaterial({color: 0xff6600, roughness: 0.3, metalness: 0.8})`. Cargar texturas con `TextureLoader`. |
| **¿Cómo debería usarse?** | Preferir `MeshStandardMaterial` por defecto (PBR físico). `MeshPhysicalMaterial` solo cuando se necesita transmission/clearcoat. |
| **¿Cuándo debería usarse?** | **Fase 3** — Al texturizar objetos 3D. |
| **Dependencias** | `npx skills add cloudai-x/threejs-skills@threejs-materials -g`. Proyecto: `npm install three`. |
| **Requerimientos** | Node 18+. Three.js r150+. WebGL 1/2. |

## `threejs-interaction` — Interacción 3D

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de interacción 3D: raycasting, controles (OrbitControls, FirstPerson, Fly), input mouse/touch, y selección de objetos. |
| **¿Para qué es?** | Hacer que la escena 3D responda al usuario: click en objetos, arrastrar, hover, zoom, y navegación por la escena. |
| **¿Cómo se usa?** | `Raycaster` para detección de clicks: `raycaster.setFromCamera(mouse, camera)` → `raycaster.intersectObjects(objects)`. `OrbitControls` para navegación. |
| **¿Cómo debería usarse?** | `OrbitControls` para escenas explorables. `Raycaster` + hover states para UI 3D interactiva. Siempre con throttle en eventos mouse. |
| **¿Cuándo debería usarse?** | **Fase 3** — Cuando el usuario necesita manipular o navegar la escena 3D. |
| **Dependencias** | `npx skills add cloudai-x/threejs-skills@threejs-interaction -g`. Proyecto: `npm install three`. |
| **Requerimientos** | Node 18+. Three.js r150+. WebGL 1/2. |

## `threejs-animation` — Animación en Three.js

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de animación Three.js: keyframe animation, skeletal animation, morph targets, y animation blending. |
| **¿Para qué es?** | Animar objetos 3D en el tiempo: rotaciones, posiciones, morphing de formas, huesos de personajes. |
| **¿Cómo se usa?** | `AnimationMixer` + `ClipAction` para animaciones desde GLTF. KeyframeTrack para animaciones manuales. |
| **¿Cómo debería usarse?** | Para animaciones importadas (GLTF), usar AnimationMixer. Para animaciones procedurales, animar propiedades directamente en el loop de render. |
| **¿Cuándo debería usarse?** | **Fase 3** — Cuando los objetos 3D necesitan movimiento. |
| **Dependencias** | `npx skills add cloudai-x/threejs-skills@threejs-animation -g`. Proyecto: `npm install three`. |
| **Requerimientos** | Node 18+. Three.js r150+. WebGL 1/2. |

## `threejs-shaders` — Shaders GLSL Personalizados

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de shaders Three.js: GLSL, ShaderMaterial, uniforms, atributos, varyings, y efectos visuales personalizados. |
| **¿Para qué es?** | Crear efectos visuales que no son posibles con materiales estándar: distortion waves, glitch, morphing shaders, hologramas, auroras. |
| **¿Cómo se usa?** | `new THREE.ShaderMaterial({ vertexShader, fragmentShader, uniforms })`. Escribir GLSL en template strings. |
| **¿Cómo debería usarse?** | Solo cuando MeshStandardMaterial/MeshPhysicalMaterial no pueden lograr el efecto. Mantener presupuesto de GPU. |
| **¿Cuándo debería usarse?** | **Fase 3** — Para efectos visuales avanzados. Comprobar primero si el efecto se puede lograr sin shaders. |
| **Dependencias** | `npx skills add cloudai-x/threejs-skills@threejs-shaders -g`. Proyecto: `npm install three`. |
| **Requerimientos** | Node 18+. Three.js r150+. WebGL 1/2 (WebGL2 recomendado para shaders complejos). |
