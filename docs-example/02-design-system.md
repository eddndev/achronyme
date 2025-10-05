# Sistema de Diseño: [NOMBRE DEL PROYECTO]

**Versión:** 1.0
**Fecha:** [Fecha de creación]
**Enlace al Proyecto de Figma/Diseño:** [URL del proyecto de diseño]

---

## 1. Paleta de Colores

[Describe la filosofía de tu paleta de colores. Por ejemplo: si funciona en modo claro y oscuro, qué representa cada color, etc.]

### Colores Primarios ([Nombre de la paleta])
[Descripción del uso principal de estos colores]

```
Ejemplo de cómo documentar colores:
- Color-50: #HEXCODE
- Color-100: #HEXCODE
- Color-200: #HEXCODE
... hasta Color-950
```

[Opcional: Agregar badges visuales usando shields.io]
![color-50](https://img.shields.io/badge/--HEXCODE?style=for-the-badge&logoColor=black&color=HEXCODE)

### Colores de Acento ([Nombre de la paleta])
[Descripción de cuándo usar estos colores de acento]

```
- Accent-200: #HEXCODE
- Accent-400: #HEXCODE
- Accent-600: #HEXCODE
```

### Colores Neutrales ([Nombre de la paleta])
[Descripción: Típicamente usados para textos, fondos, bordes, etc.]

```
- Neutral-50: #HEXCODE
- Neutral-100: #HEXCODE
... hasta Neutral-950
```

### Colores Semánticos
[Colores para estados y feedback del usuario]

* **Éxito (`success`):** ![Success](https://img.shields.io/badge/--HEXCODE?style=for-the-badge&logoColor=white&color=HEXCODE)
* **Peligro (`danger`):** ![Danger](https://img.shields.io/badge/--HEXCODE?style=for-the-badge&logoColor=white&color=HEXCODE)
* **Advertencia (`warning`):** ![Warning](https://img.shields.io/badge/--HEXCODE?style=for-the-badge&logoColor=white&color=HEXCODE)
* **Información (`info`):** ![Info](https://img.shields.io/badge/--HEXCODE?style=for-the-badge&logoColor=white&color=HEXCODE)

### Uso en Tema Claro vs. Oscuro

[Si tu proyecto soporta temas, documenta cómo cambian los colores]

| Uso Semántico        | Modo Claro (`light`)                               | Modo Oscuro (`dark`)                               |
| -------------------- | -------------------------------------------------- | -------------------------------------------------- |
| **Fondo Principal** | `bg-[color-name]-[shade]`                          | `bg-[color-name]-[shade]`                          |
| **Fondo Tarjetas** | `bg-[color-name]`                                  | `bg-[color-name]-[shade]`                          |
| **Texto Principal** | `text-[color-name]-[shade]`                        | `text-[color-name]-[shade]`                        |
| **Texto Secundario** | `text-[color-name]-[shade]`                        | `text-[color-name]-[shade]`                        |
| **Botón Primario** | `bg-[color] text-white hover:bg-[color-darker]`    | `bg-[color] text-white hover:bg-[color-darker]`    |
| **Bordes** | `border-[color-name]-[shade]`                      | `border-[color-name]-[shade]`                      |

---

## 2. Tipografía

[Define las fuentes y tamaños tipográficos del proyecto]

* **Fuente Principal:** [Nombre de la fuente] ([Descripción, ej: Moderna, legible, etc. Fuente desde Google Fonts o sistema])
* **Fuente Secundaria (opcional):** [Nombre de la fuente]

### Escala Tipográfica

[Define los tamaños y pesos para cada nivel de encabezado y texto]

* **h1:** `text-[size] font-[weight]` ([Tamaño en px])
* **h2:** `text-[size] font-[weight]` ([Tamaño en px])
* **h3:** `text-[size] font-[weight]` ([Tamaño en px])
* **h4:** `text-[size] font-[weight]` ([Tamaño en px])
* **p (body):** `text-[size] font-[weight]` ([Tamaño en px])
* **small:** `text-[size] font-[weight]` ([Tamaño en px])
* **caption:** `text-[size] font-[weight]` ([Tamaño en px])

---

## 3. Espaciado y Rejilla (Grid)

[Describe el sistema de espaciado que usarás. Ejemplo: escala de Tailwind, sistema personalizado, etc.]

* `p-1`, `m-1`: [valor]px
* `p-2`, `m-2`: [valor]px
* `p-3`, `m-3`: [valor]px
* `p-4`, `m-4`: [valor]px
* `p-6`, `m-6`: [valor]px
* `p-8`, `m-8`: [valor]px
* `p-12`, `m-12`: [valor]px

### Sistema de Grid

[Opcional: Define cómo se estructura el layout con columnas]

* **Columnas:** [Número de columnas en el grid, ej: 12 columnas]
* **Gutters:** [Espaciado entre columnas]
* **Breakpoints:**
  * `sm`: [valor]px
  * `md`: [valor]px
  * `lg`: [valor]px
  * `xl`: [valor]px
  * `2xl`: [valor]px

---

## 4. Componentes Clave

[Define las directrices de diseño para los componentes reutilizables más importantes]

### Botones

[Todos los botones deben seguir estas directrices]

* **Primario:**
    * **Uso:** [Cuándo usar este tipo de botón]
    * **Estilo:** `[clases de CSS/Tailwind]`
    * **Ejemplo:**
    ```html
    <button class="[clases aquí]">Texto del botón</button>
    ```

* **Secundario:**
    * **Uso:** [Cuándo usar este tipo de botón]
    * **Estilo:** `[clases de CSS/Tailwind]`

* **Terciario/Texto:**
    * **Uso:** [Cuándo usar este tipo de botón]
    * **Estilo:** `[clases de CSS/Tailwind]`

* **Destructivo/Peligro:**
    * **Uso:** [Cuándo usar este tipo de botón]
    * **Estilo:** `[clases de CSS/Tailwind]`

### Inputs de Formulario

[Define el estilo de los campos de entrada]

* **Estilo General:**
    * `[clases de CSS/Tailwind para inputs normales]`
    * **Ejemplo:**
    ```html
    <input type="text" class="[clases aquí]" placeholder="Texto de ejemplo">
    ```

* **Estado de Enfoque (Focus):**
    * `[clases adicionales para el estado focus]`

* **Estado de Error:**
    * `[clases adicionales para mostrar errores]`

* **Estado Deshabilitado:**
    * `[clases adicionales para campos deshabilitados]`

### Tarjetas (Cards)

[El contenedor principal para la mayoría del contenido]

* **Estilo General:**
    * `[clases de CSS/Tailwind]`
    * **Ejemplo:**
    ```html
    <div class="[clases aquí]">
      <h3>Título de la tarjeta</h3>
      <p>Contenido de la tarjeta</p>
    </div>
    ```

* **Padding:**
    * Interior: `p-[valor]`
    * Margen exterior: `m-[valor]`

* **Variantes:**
    * **Con borde:** `[clases]`
    * **Con sombra:** `[clases]`
    * **Interactiva (hover):** `[clases]`

### Badges/Etiquetas

[Pequeños elementos para mostrar estados, categorías, etc.]

* **Estilo Base:**
    * `[clases de CSS/Tailwind]`

* **Variantes por Color:**
    * **Primario:** `[clases]`
    * **Éxito:** `[clases]`
    * **Advertencia:** `[clases]`
    * **Peligro:** `[clases]`
    * **Neutral:** `[clases]`

### Modales/Diálogos

[Ventanas emergentes para información importante o formularios]

* **Overlay (Fondo oscuro):**
    * `[clases de CSS/Tailwind]`

* **Contenedor del Modal:**
    * `[clases de CSS/Tailwind]`

* **Encabezado:**
    * `[clases de CSS/Tailwind]`

* **Cuerpo:**
    * `[clases de CSS/Tailwind]`

* **Footer (Acciones):**
    * `[clases de CSS/Tailwind]`

### Notificaciones/Alertas

[Mensajes de feedback para el usuario]

* **Tipo Éxito:**
    * `[clases de CSS/Tailwind]`

* **Tipo Error:**
    * `[clases de CSS/Tailwind]`

* **Tipo Advertencia:**
    * `[clases de CSS/Tailwind]`

* **Tipo Información:**
    * `[clases de CSS/Tailwind]`

---

## 5. Iconografía

[Define el sistema de iconos que usarás]

* **Librería de Iconos:** [Nombre de la librería, ej: Heroicons, Font Awesome, Feather Icons, etc.]
* **Tamaño por defecto:** `[valor]px` o `w-[valor] h-[valor]`
* **Estilo:** [Outline, Solid, Duotone, etc.]
* **Color:** [Hereda del texto padre o colores específicos]

### Iconos Comunes

| Acción/Contexto | Icono | Uso |
|-----------------|-------|-----|
| Cerrar | `[nombre del icono]` | Cerrar modales, notificaciones |
| Búsqueda | `[nombre del icono]` | Campo de búsqueda |
| Usuario | `[nombre del icono]` | Perfil de usuario |
| Configuración | `[nombre del icono]` | Acceso a ajustes |
| [Otro] | `[nombre del icono]` | [Uso] |

---

## 6. Sombras (Shadows)

[Define los niveles de elevación/sombra]

* **Ninguna:** `shadow-none`
* **Sutil:** `shadow-sm` - [Uso: elementos ligeramente elevados]
* **Normal:** `shadow` o `shadow-md` - [Uso: tarjetas, botones elevados]
* **Media:** `shadow-lg` - [Uso: modales, dropdowns]
* **Alta:** `shadow-xl` - [Uso: elementos muy prominentes]
* **Extrema:** `shadow-2xl` - [Uso: overlays importantes]

---

## 7. Bordes y Esquinas Redondeadas

[Define el estilo de bordes y border-radius]

* **Grosor de Borde:**
    * `border` ([valor]px)
    * `border-2` ([valor]px)

* **Radio de Esquinas:**
    * `rounded-none` (0px)
    * `rounded-sm` ([valor]px)
    * `rounded` o `rounded-md` ([valor]px)
    * `rounded-lg` ([valor]px)
    * `rounded-xl` ([valor]px)
    * `rounded-full` (100% - círculos/píldoras)

---

## 8. Animaciones y Transiciones

[Define las animaciones y transiciones estándar]

* **Duración:**
    * Rápida: `duration-150` (150ms)
    * Normal: `duration-200` o `duration-300` (200-300ms)
    * Lenta: `duration-500` (500ms)

* **Ease:**
    * `ease-in`
    * `ease-out`
    * `ease-in-out`
    * `linear`

* **Propiedades comunes a animar:**
    * Colores: `transition-colors`
    * Opacidad: `transition-opacity`
    * Transform: `transition-transform`
    * Todo: `transition-all` (usar con precaución)

### Animaciones Específicas

[Define animaciones personalizadas si aplica]

* **[Nombre de animación]:**
    * **Uso:** [Cuándo se usa]
    * **Keyframes/Código:**
    ```css
    @keyframes [nombre] {
      /* ... */
    }
    ```

---

## 9. Accesibilidad (A11y)

[Directrices de accesibilidad que deben seguirse]

* **Contraste de Colores:** Los textos deben cumplir con WCAG 2.1 AA mínimo (ratio 4.5:1 para texto normal)
* **Focus States:** Todos los elementos interactivos deben tener un indicador de foco visible
* **Etiquetas ARIA:** Usar atributos ARIA cuando sea necesario para mejorar la experiencia de lectores de pantalla
* **Tamaños de Toque:** Los elementos interactivos móviles deben tener mínimo 44x44px
* **[Otra directriz]:** [Descripción]

---

## 10. Responsive Design

[Define cómo se adapta el diseño a diferentes tamaños de pantalla]

### Breakpoints

| Breakpoint | Tamaño | Uso |
|------------|--------|-----|
| `xs` (opcional) | < [valor]px | Teléfonos pequeños |
| `sm` | ≥ [valor]px | Teléfonos |
| `md` | ≥ [valor]px | Tablets |
| `lg` | ≥ [valor]px | Desktop pequeño |
| `xl` | ≥ [valor]px | Desktop grande |
| `2xl` | ≥ [valor]px | Pantallas extra grandes |

### Estrategia

[Mobile-first, Desktop-first, o híbrido]

* **Enfoque:** [Descripción del enfoque responsivo]
* **Consideraciones especiales:**
    * [Punto 1]
    * [Punto 2]

---

## 11. Tokens de Diseño (Design Tokens)

[Opcional: Si usas un sistema de tokens centralizado]

```json
{
  "color": {
    "primary": {
      "500": "#HEXCODE"
    }
  },
  "spacing": {
    "base": "16px"
  },
  "typography": {
    "fontFamily": {
      "primary": "Font Name"
    }
  }
}
```

---

## 12. Recursos y Referencias

[Enlaces útiles relacionados con el diseño]

* **Figma/Sketch:** [URL del proyecto]
* **Guía de estilo de código:** [URL si aplica]
* **Librería de componentes:** [URL si aplica, ej: Storybook]
* **Paleta de colores interactiva:** [URL si aplica]

---

**Notas Finales:**
[Cualquier información adicional sobre el sistema de diseño, convenciones especiales, o instrucciones para el equipo]
