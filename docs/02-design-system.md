# Design System - Achronyme

**Versión:** 1.0
**Fecha:** 2025-10-05
**Mantenido por:** @eddndev

---

## 1. Principios de Diseño

### Accesibilidad Primero
- **WCAG 2.1 Level AA** compliance mínimo
- Contraste de color ≥ 4.5:1 para texto normal
- Contraste de color ≥ 3:1 para texto grande
- Navegación completa por teclado
- Screen reader friendly

### Rendimiento
- First Contentful Paint (FCP) < 1.5s
- Largest Contentful Paint (LCP) < 2.5s
- Time to Interactive (TTI) < 3.5s
- Cumulative Layout Shift (CLS) < 0.1

### Responsive Design
- Mobile First approach
- Breakpoints: 320px, 768px, 1024px, 1280px, 1536px
- Touch-friendly (mínimo 44x44px para botones)

---

## 2. Paleta de Colores

### Colores Primarios

```css
/* Purple - Brand Color */
--purple-50:  #faf5ff;
--purple-100: #f3e8ff;
--purple-200: #e9d5ff;
--purple-300: #d8b4fe;
--purple-400: #c084fc;
--purple-500: #a855f7;  /* Primary */
--purple-600: #9333ea;
--purple-700: #7e22ce;
--purple-800: #6b21a8;
--purple-900: #581c87;

/* Violet - Secondary */
--violet-50:  #f5f3ff;
--violet-100: #ede9fe;
--violet-200: #ddd6fe;
--violet-300: #c4b5fd;
--violet-400: #a78bfa;
--violet-500: #8b5cf6;  /* Secondary */
--violet-600: #7c3aed;
--violet-700: #6d28d9;
--violet-800: #5b21b6;
--violet-900: #4c1d95;
```

### Colores Neutros

```css
/* Gray Scale */
--gray-50:  #f9fafb;
--gray-100: #f3f4f6;
--gray-200: #e5e7eb;
--gray-300: #d1d5db;
--gray-400: #9ca3af;
--gray-500: #6b7280;
--gray-600: #4b5563;
--gray-700: #374151;
--gray-800: #1f2937;
--gray-900: #111827;
```

### Colores Semánticos

```css
/* Success */
--success: #10b981;     /* green-500 */
--success-light: #d1fae5;
--success-dark: #047857;

/* Warning */
--warning: #f59e0b;     /* amber-500 */
--warning-light: #fef3c7;
--warning-dark: #d97706;

/* Error */
--error: #ef4444;       /* red-500 */
--error-light: #fee2e2;
--error-dark: #dc2626;

/* Info */
--info: #3b82f6;        /* blue-500 */
--info-light: #dbeafe;
--info-dark: #2563eb;
```

---

## 3. Tipografía

### Familia de Fuentes

```css
/* Primary Font Stack */
--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI',
             Roboto, 'Helvetica Neue', Arial, sans-serif;

/* Monospace (para código) */
--font-mono: 'JetBrains Mono', 'Fira Code', 'Courier New', monospace;

/* Math (para fórmulas) */
--font-math: 'Latin Modern Math', 'STIX Two Math', serif;
```

### Escala Tipográfica

```css
/* Font Sizes */
--text-xs:   0.75rem;   /* 12px */
--text-sm:   0.875rem;  /* 14px */
--text-base: 1rem;      /* 16px */
--text-lg:   1.125rem;  /* 18px */
--text-xl:   1.25rem;   /* 20px */
--text-2xl:  1.5rem;    /* 24px */
--text-3xl:  1.875rem;  /* 30px */
--text-4xl:  2.25rem;   /* 36px */
--text-5xl:  3rem;      /* 48px */
--text-6xl:  3.75rem;   /* 60px */

/* Font Weights */
--font-light:     300;
--font-normal:    400;
--font-medium:    500;
--font-semibold:  600;
--font-bold:      700;
--font-extrabold: 800;

/* Line Heights */
--leading-tight:   1.25;
--leading-snug:    1.375;
--leading-normal:  1.5;
--leading-relaxed: 1.625;
--leading-loose:   2;
```

### Jerarquía de Encabezados

```css
h1 {
  font-size: var(--text-5xl);    /* 48px */
  font-weight: var(--font-bold);
  line-height: var(--leading-tight);
}

h2 {
  font-size: var(--text-4xl);    /* 36px */
  font-weight: var(--font-bold);
  line-height: var(--leading-tight);
}

h3 {
  font-size: var(--text-3xl);    /* 30px */
  font-weight: var(--font-semibold);
  line-height: var(--leading-snug);
}

h4 {
  font-size: var(--text-2xl);    /* 24px */
  font-weight: var(--font-semibold);
  line-height: var(--leading-snug);
}

h5 {
  font-size: var(--text-xl);     /* 20px */
  font-weight: var(--font-medium);
  line-height: var(--leading-normal);
}

h6 {
  font-size: var(--text-lg);     /* 18px */
  font-weight: var(--font-medium);
  line-height: var(--leading-normal);
}
```

---

## 4. Espaciado

### Sistema de Espaciado

```css
--space-0:  0;
--space-1:  0.25rem;   /* 4px */
--space-2:  0.5rem;    /* 8px */
--space-3:  0.75rem;   /* 12px */
--space-4:  1rem;      /* 16px */
--space-5:  1.25rem;   /* 20px */
--space-6:  1.5rem;    /* 24px */
--space-8:  2rem;      /* 32px */
--space-10: 2.5rem;    /* 40px */
--space-12: 3rem;      /* 48px */
--space-16: 4rem;      /* 64px */
--space-20: 5rem;      /* 80px */
--space-24: 6rem;      /* 96px */
```

### Márgenes y Padding

- **Componentes pequeños**: padding `space-2` a `space-4`
- **Componentes medianos**: padding `space-4` a `space-6`
- **Contenedores**: padding `space-6` a `space-12`
- **Secciones**: margin-bottom `space-8` a `space-16`

---

## 5. Componentes UI

### Botones

#### Variantes

**Primary Button**
```css
.btn-primary {
  background-color: var(--purple-500);
  color: white;
  padding: var(--space-3) var(--space-6);
  border-radius: 0.5rem;
  font-weight: var(--font-medium);
  transition: all 0.2s ease;
}

.btn-primary:hover {
  background-color: var(--purple-600);
  transform: translateY(-1px);
  box-shadow: 0 4px 6px -1px rgba(168, 85, 247, 0.3);
}
```

**Secondary Button**
```css
.btn-secondary {
  background-color: var(--violet-100);
  color: var(--violet-700);
  padding: var(--space-3) var(--space-6);
  border-radius: 0.5rem;
  font-weight: var(--font-medium);
  transition: all 0.2s ease;
}

.btn-secondary:hover {
  background-color: var(--violet-200);
}
```

**Outline Button**
```css
.btn-outline {
  background-color: transparent;
  color: var(--purple-500);
  border: 2px solid var(--purple-500);
  padding: var(--space-3) var(--space-6);
  border-radius: 0.5rem;
  font-weight: var(--font-medium);
  transition: all 0.2s ease;
}

.btn-outline:hover {
  background-color: var(--purple-500);
  color: white;
}
```

#### Tamaños

```css
.btn-sm  { padding: var(--space-2) var(--space-4); font-size: var(--text-sm); }
.btn-md  { padding: var(--space-3) var(--space-6); font-size: var(--text-base); }
.btn-lg  { padding: var(--space-4) var(--space-8); font-size: var(--text-lg); }
```

### Cards

```css
.card {
  background: white;
  border-radius: 1rem;
  padding: var(--space-6);
  box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.3s ease;
}

.card:hover {
  box-shadow: 0 10px 15px -3px rgba(168, 85, 247, 0.2);
}
```

### Inputs

```css
.input {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: 2px solid var(--gray-300);
  border-radius: 0.5rem;
  font-size: var(--text-base);
  transition: border-color 0.2s ease;
}

.input:focus {
  outline: none;
  border-color: var(--purple-500);
  box-shadow: 0 0 0 3px rgba(168, 85, 247, 0.1);
}

.input.error {
  border-color: var(--error);
}
```

---

## 6. Animaciones

### Transiciones

```css
/* Duraciones */
--duration-fast:   150ms;
--duration-normal: 300ms;
--duration-slow:   500ms;

/* Easing */
--ease-in:     cubic-bezier(0.4, 0, 1, 1);
--ease-out:    cubic-bezier(0, 0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
```

### Animaciones con GSAP

```javascript
// Fade In
gsap.from('.element', {
  opacity: 0,
  y: 20,
  duration: 0.6,
  ease: 'power2.out'
});

// Slide In
gsap.from('.element', {
  x: -50,
  opacity: 0,
  duration: 0.8,
  ease: 'power3.out'
});

// Scale In
gsap.from('.element', {
  scale: 0.8,
  opacity: 0,
  duration: 0.5,
  ease: 'back.out(1.7)'
});
```

---

## 7. Gráficas (Chart.js)

### Configuración por Defecto

```javascript
const chartDefaults = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      display: true,
      position: 'top',
      labels: {
        font: {
          family: 'Inter',
          size: 14
        },
        color: '#374151' // gray-700
      }
    }
  },
  scales: {
    x: {
      grid: {
        color: '#e5e7eb' // gray-200
      },
      ticks: {
        font: {
          family: 'Inter',
          size: 12
        },
        color: '#6b7280' // gray-500
      }
    },
    y: {
      grid: {
        color: '#e5e7eb'
      },
      ticks: {
        font: {
          family: 'Inter',
          size: 12
        },
        color: '#6b7280'
      }
    }
  }
};

// Color palette para datasets
const chartColors = [
  '#a855f7', // purple-500
  '#8b5cf6', // violet-500
  '#3b82f6', // blue-500
  '#10b981', // green-500
  '#f59e0b', // amber-500
  '#ef4444'  // red-500
];
```

---

## 8. Dark Mode

### Estrategia
- Usar `prefers-color-scheme` para detección automática
- Toggle manual para override
- Persistir preferencia en localStorage

### Paleta Dark Mode

```css
@media (prefers-color-scheme: dark) {
  :root {
    --bg-primary:   #111827;  /* gray-900 */
    --bg-secondary: #1f2937;  /* gray-800 */
    --bg-tertiary:  #374151;  /* gray-700 */

    --text-primary:   #f9fafb;  /* gray-50 */
    --text-secondary: #d1d5db;  /* gray-300 */
    --text-tertiary:  #9ca3af;  /* gray-400 */

    --border-color: #4b5563;  /* gray-600 */
  }
}
```

---

## 9. Iconografía

### Sistema de Iconos
- **Heroicons** para UI general
- **Simple Icons** para logos de tecnologías
- SVG inline para máximo control

### Tamaños

```css
.icon-xs { width: 16px; height: 16px; }
.icon-sm { width: 20px; height: 20px; }
.icon-md { width: 24px; height: 24px; }
.icon-lg { width: 32px; height: 32px; }
.icon-xl { width: 48px; height: 48px; }
```

---

## 10. Layout

### Container

```css
.container {
  width: 100%;
  max-width: 1280px;
  margin: 0 auto;
  padding: 0 var(--space-4);
}

@media (min-width: 768px) {
  .container {
    padding: 0 var(--space-6);
  }
}

@media (min-width: 1024px) {
  .container {
    padding: 0 var(--space-8);
  }
}
```

### Grid System

```css
.grid {
  display: grid;
  gap: var(--space-6);
}

.grid-cols-1  { grid-template-columns: repeat(1, 1fr); }
.grid-cols-2  { grid-template-columns: repeat(2, 1fr); }
.grid-cols-3  { grid-template-columns: repeat(3, 1fr); }
.grid-cols-4  { grid-template-columns: repeat(4, 1fr); }
.grid-cols-12 { grid-template-columns: repeat(12, 1fr); }
```

---

**Documento vivo - Actualizado con cada cambio de diseño mayor**
