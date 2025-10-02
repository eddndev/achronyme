# Guía de Construcción de Herramientas - Achronyme

**Documentación para desarrolladores que crean nuevas herramientas de Procesamiento Digital de Señales**

---

## 📐 Arquitectura de Componentes

### Jerarquía de Layouts

```
<x-app-layout>                      ← Base global (HTML, navegación, scripts)
  └─ <x-tool-layout>                ← Layout de herramienta (breadcrumbs, header, fondo)
       └─ <x-three-column-tool>     ← Grid de 3 columnas responsive
            └─ Alpine Component      ← Estado reactivo (x-data, x-init)
                 └─ UI Components    ← Inputs, botones, gráficas
```

---

## 🛠️ Componentes de Layout Disponibles

### 1. `<x-tool-layout>`

**Propósito:** Envuelve todas las herramientas y provee estructura común.

**Props:**
```blade
@props([
    'title',              // Título de la herramienta (requerido)
    'breadcrumbs' => [],  // Array de navegación
    'icon' => null,       // Identificador del ícono SVG
    'scripts' => null     // Scripts adicionales (slot)
])
```

**Uso:**
```blade
@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'PDS', 'url' => '#'],
    ['name' => 'Mi Herramienta', 'url' => '#']
];
@endphp

<x-tool-layout title="Mi Herramienta" :breadcrumbs="$breadcrumbs" icon="mi-icono">
    <x-slot:actions>
        <x-secondary-button>Volver</x-secondary-button>
        <x-primary-button class="ml-3">Guardar</x-primary-button>
    </x-slot>

    <!-- Contenido de la herramienta -->

    <x-slot:scripts>
        @vite('resources/js/mi-herramienta/app.ts')
    </x-slot>
</x-tool-layout>
```

**Incluye automáticamente:**
- ✅ Breadcrumbs responsive
- ✅ Header con ícono y título
- ✅ Background decorativo
- ✅ Container centralizado

---

### 2. `<x-three-column-tool>`

**Propósito:** Grid de 3 columnas para herramientas interactivas.

**Props:**
```blade
@props([
    'leftSidebar',   // Slot para columna izquierda
    'rightSidebar'   // Slot para columna derecha
])
```

**Uso:**
```blade
<x-three-column-tool>
    {{-- Columna izquierda: Controles --}}
    <x-slot:leftSidebar>
        <div class="space-y-6">
            <x-app-ui.radio-list ... />
            <x-app-ui.slider ... />
        </div>
    </x-slot>

    {{-- Centro: Gráfica principal --}}
    <div class="mb-8">
        <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
            <canvas id="myChart"></canvas>
        </div>
    </div>

    {{-- Columna derecha: Inputs --}}
    <x-slot:rightSidebar>
        <div class="space-y-6">
            <x-app-ui.input-text ... />
            <x-app-ui.button>Calcular</x-app-ui.button>
        </div>
    </x-slot>
</x-three-column-tool>
```

**Características:**
- 📱 Responsive (columnas colapsan en mobile)
- 📏 Columnas laterales: `w-96` fijo
- 🎨 Centro: `flex-1` (se expande)

---

### 3. Componentes Auxiliares

#### `<x-breadcrumbs>`

```blade
<x-breadcrumbs :items="[
    ['name' => 'Home', 'url' => route('home')],
    ['name' => 'Tools', 'url' => '#'],
    ['name' => 'Current', 'url' => '#']
]" />
```

#### `<x-tool-header>`

```blade
<x-tool-header title="Mi Herramienta" icon="icono-id">
    <x-slot:actions>
        <button>Acción 1</button>
        <button>Acción 2</button>
    </x-slot>
</x-tool-header>
```

#### `<x-background-blur>`

```blade
<x-background-blur /> {{-- Efectos decorativos de fondo --}}
```

---

## 🎨 Componentes UI Disponibles

### Inputs

#### `<x-app-ui.input-text>`

```blade
<x-app-ui.input-text
    label="Frecuencia"
    name="frequency"
    placeholder="Ej: 440"
    x-model="frequency"
    error-model="frequency_error"  {{-- Validación Alpine reactiva --}}
/>
```

**Props:**
- `label` - Etiqueta del campo
- `name` - Atributo name
- `type` - Tipo de input (default: 'text')
- `placeholder` - Texto de ayuda
- `optional` - Muestra "(Opcional)" al lado del label
- `error-model` - Variable Alpine para errores reactivos

---

#### `<x-app-ui.input-addon>`

```blade
<x-app-ui.input-addon
    label="Amplitud"
    addon="dB"
    position="right"  {{-- 'left' o 'right' --}}
    name="amplitude"
    x-model="amplitude"
    error-model="amplitude_error"
/>
```

**Uso:** Inputs con unidades (Hz, dB, V, etc.)

---

#### `<x-app-ui.function-domain>`

```blade
<x-app-ui.function-domain
    x-model-function="func.definition"
    x-model-domain-start="func.domainStart"
    x-model-domain-end="func.domainEnd"
    function-placeholder="Ej: sin(2*pi*t)"
    domain-start-placeholder="-pi"
    domain-end-placeholder="pi"
    :function-error-model="'func.definitionError'"
    :domain-start-error-model="'func.domainStartError'"
    :domain-end-error-model="'func.domainEndError'"
    ::index="index"
/>
```

**Uso:** Definir funciones matemáticas con dominio `[a, b]`

---

#### `<x-app-ui.slider>`

```blade
<x-app-ui.slider
    label="Número de muestras"
    name="samples"
    min="1"
    max="100"
    step="1"
    value="50"
    x-model="numSamples"
/>
```

---

#### `<x-app-ui.checkbox>`

```blade
<x-app-ui.checkbox
    id="show_grid"
    value="grid"
    x-model="renderOptions"  {{-- Array de valores --}}
/>
```

---

#### `<x-app-ui.radio-list>`

```blade
@php
$options = [
    ['value' => 'mode1', 'title' => 'Modo 1', 'description' => 'Descripción...'],
    ['value' => 'mode2', 'title' => 'Modo 2', 'description' => 'Descripción...']
];
@endphp

<x-app-ui.radio-list
    legend="Selecciona modo"
    name="mode"
    :options="$options"
    checkedValue="mode1"
    x-model="selectedMode"
/>
```

---

### Botones

#### `<x-app-ui.button>` (Primario)

```blade
<x-app-ui.button
    type="button"
    @click="calculate()"
    loading-text="Calculando..."
>
    Calcular
</x-app-ui.button>
```

#### `<x-app-ui.secondary-button>`

```blade
<x-app-ui.secondary-button @click="reset()">
    Resetear
</x-app-ui.secondary-button>
```

#### `<x-app-ui.danger-circular-button>`

```blade
<x-app-ui.danger-circular-button
    icon="trash"
    @click="deleteItem(id)"
    title="Eliminar"
/>
```

---

## 🧪 Patrón Alpine.js para Herramientas

### Estructura de Estado

**Archivo:** `resources/js/mi-herramienta/tool-state.ts`

```typescript
import * as math from 'mathjs';
import { validateConstant, validateFunction } from '../utils/validation';

interface ToolState {
    // UI State
    isLoading: boolean;
    errorMessage: string;

    // Data Model
    inputValue: string;
    inputValue_error: string | null;

    // Methods
    init(): void;
    validate(): boolean;
    calculate(): void;
    $watch: (property: string, callback: (value: any) => void) => void;
}

function toolState(): ToolState {
    return {
        isLoading: false,
        errorMessage: '',
        inputValue: '0',
        inputValue_error: null,

        init() {
            console.log('[Alpine] Initializing toolState...');

            // Watchers reactivos
            this.$watch('inputValue', () => {
                this.validate();
            });
        },

        validate() {
            this.inputValue_error = null;

            const validation = validateConstant(this.inputValue);
            if (!validation.isValid) {
                this.inputValue_error = validation.error!;
                return false;
            }

            return true;
        },

        calculate() {
            if (!this.validate()) return;

            this.isLoading = true;
            try {
                // Lógica de cálculo
                const result = math.evaluate(this.inputValue);
                console.log('Resultado:', result);
            } catch (error: any) {
                this.errorMessage = error.message;
            } finally {
                this.isLoading = false;
            }
        }
    } as unknown as ToolState;
}

declare global {
    interface Window {
        toolState: () => ToolState;
    }
}

window.toolState = toolState;
```

---

## ✅ Validación de Inputs

**Utilidades disponibles:** `resources/js/utils/validation.ts`

### `validateConstant(str: string)`

Valida expresiones matemáticas constantes:

```typescript
import { validateConstant } from '../utils/validation';

const result = validateConstant('2*pi');
// { isValid: true }

const result = validateConstant('xyz');
// { isValid: false, error: 'Símbolo desconocido: xyz' }
```

**Uso:** Validar dominios, amplitudes, frecuencias, etc.

---

### `validateFunction(str: string, domainVar: string = 't')`

Valida funciones matemáticas:

```typescript
import { validateFunction } from '../utils/validation';

// Función de tiempo
const result = validateFunction('sin(2*pi*t)', 't');
// { isValid: true }

// Función de frecuencia
const result = validateFunction('1/(1 + omega^2)', 'omega');
// { isValid: true }

// Función discreta
const result = validateFunction('n^2 + 1', 'n');
// { isValid: true }
```

**Parámetro `domainVar`:**
- `'t'` → Tiempo continuo (default)
- `'n'` → Tiempo discreto
- `'omega'` → Frecuencia angular
- `'f'` → Frecuencia en Hz
- `'x'`, `'y'`, etc.

---

## 📊 Integración con Chart.js

### Estructura básica

**Archivo:** `resources/js/mi-herramienta/app.ts`

```typescript
import { Chart, registerables } from 'chart.js';
import * as math from 'mathjs';

Chart.register(...registerables);

window.MyToolChart = {
    chart: null as Chart | null,

    init() {
        const ctx = document.getElementById('myChart') as HTMLCanvasElement;
        if (!ctx) return;

        if (this.chart) {
            this.chart.destroy();
        }

        this.chart = new Chart(ctx, {
            type: 'line',
            data: { labels: [], datasets: [] },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    x: { title: { display: true, text: 't' } },
                    y: { title: { display: true, text: 'f(t)' } }
                }
            }
        });
    },

    redraw(data: number[]) {
        if (!this.chart) return;

        this.chart.data.labels = data.map((_, i) => i.toString());
        this.chart.data.datasets = [{
            label: 'Señal',
            data: data,
            borderColor: 'rgb(59, 130, 246)',
            borderWidth: 2
        }];

        this.chart.update('none');
    }
};

declare global {
    interface Window {
        MyToolChart: {
            chart: Chart | null;
            init: () => void;
            redraw: (data: number[]) => void;
        };
    }
}
```

---

## 📁 Estructura de Archivos para Nueva Herramienta

```
resources/
├── js/
│   ├── utils/
│   │   ├── validation.ts          ← Reutilizable
│   │   ├── numerical-integration.ts
│   │   └── types.ts
│   │
│   └── mi-herramienta/
│       ├── app.ts                  ← Inicialización Chart.js
│       ├── tool-state.ts           ← Estado Alpine
│       └── calculations.ts         ← Lógica matemática
│
└── views/
    └── tools/
        └── categoria/
            └── mi-herramienta/
                ├── index.blade.php        ← Layout + metadata
                └── tool-content.blade.php ← Columnas + Alpine
```

---

## 🎯 Checklist para Crear una Herramienta

### 1. **Vista Principal** (`index.blade.php`)

```blade
@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'Categoría', 'url' => '#'],
    ['name' => 'Mi Herramienta', 'url' => '#']
];
@endphp

<x-tool-layout title="Mi Herramienta" :breadcrumbs="$breadcrumbs" icon="icono-id">
    <x-slot:actions>
        <x-secondary-button>Volver</x-secondary-button>
        <x-primary-button class="ml-3">Guardar</x-primary-button>
    </x-slot>

    @include('tools.categoria.mi-herramienta.tool-content')

    <x-slot:scripts>
        @vite('resources/js/mi-herramienta/app.ts')
    </x-slot>
</x-tool-layout>
```

---

### 2. **Contenido de la Herramienta** (`tool-content.blade.php`)

```blade
<div x-data="myToolState()" x-init="init()">
    <x-three-column-tool>
        {{-- Columna izquierda: Controles --}}
        <x-slot:leftSidebar>
            <div class="space-y-6">
                <!-- Radio lists, checkboxes, sliders -->
            </div>
        </x-slot>

        {{-- Centro: Gráfica --}}
        <div class="mb-8">
            <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                <div class="relative w-full" style="height: 500px;" id="chartContainer">
                    <canvas id="myChart"></canvas>
                </div>
            </div>
        </div>

        {{-- Columna derecha: Inputs --}}
        <x-slot:rightSidebar>
            <div class="space-y-6">
                <!-- Inputs con validación -->
                <x-app-ui.button @click="calculate()">
                    Calcular
                </x-app-ui.button>
            </div>
        </x-slot>
    </x-three-column-tool>
</div>
```

---

### 3. **Estado Alpine** (`tool-state.ts`)

- ✅ Importar validadores de `utils/validation`
- ✅ Definir interface TypeScript
- ✅ Inicializar campos con errores (`_error` suffix)
- ✅ Implementar método `validate()`
- ✅ Agregar watchers reactivos en `init()`
- ✅ Exportar a `window.myToolState`

---

### 4. **Chart.js** (`app.ts`)

- ✅ Registrar `Chart.js`
- ✅ Crear objeto global `window.MyToolChart`
- ✅ Implementar `init()` y `redraw()`
- ✅ Manejar responsive con `ResizeObserver`
- ✅ Importar `tool-state.ts`

---

### 5. **Lógica Matemática** (`calculations.ts`)

- ✅ Usar `mathjs` para evaluación de expresiones
- ✅ Reutilizar `utils/numerical-integration` si es necesario
- ✅ Retornar resultados tipados

---

## 🎨 Convenciones de Estilos

### Espaciado

```blade
<div class="space-y-6">      <!-- Espacio vertical entre elementos -->
<div class="space-x-3">      <!-- Espacio horizontal -->
<div class="p-6">            <!-- Padding interno -->
<div class="mb-8">           <!-- Margin bottom -->
```

### Contenedores de gráficas

```blade
<div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
    <div class="relative w-full" style="height: 500px;" id="chartContainer">
        <canvas id="myChart"></canvas>
    </div>
</div>
```

### Dark mode

Siempre agregar variantes dark:

```blade
<p class="text-slate-900 dark:text-white">
<div class="bg-white dark:bg-slate-800">
```

---

## 🔗 Referencias

- **Validación:** `resources/js/utils/validation.ts`
- **Ejemplo completo:** `resources/views/tools/fourier/fs/`
- **Componentes UI:** `resources/views/components/app-ui/`
- **Layouts:** `resources/views/components/`

---

## 🚀 Ejemplo Completo: Herramienta de Convolución

```blade
{{-- tools/convolution/index.blade.php --}}
@php
$breadcrumbs = [
    ['name' => 'Herramientas', 'url' => route('home')],
    ['name' => 'PDS', 'url' => '#'],
    ['name' => 'Convolución', 'url' => '#']
];
@endphp

<x-tool-layout title="Convolución" :breadcrumbs="$breadcrumbs" icon="convolution">
    <x-slot:actions>
        <x-secondary-button>Volver</x-secondary-button>
        <x-primary-button class="ml-3">Exportar</x-primary-button>
    </x-slot>

    <div x-data="convolutionState()" x-init="init()">
        <x-three-column-tool>
            <x-slot:leftSidebar>
                <div class="space-y-6">
                    <x-app-ui.radio-list
                        legend="Tipo de convolución"
                        name="conv_type"
                        :options="[
                            ['value' => 'discrete', 'title' => 'Discreta', 'description' => 'x[n] * h[n]'],
                            ['value' => 'continuous', 'title' => 'Continua', 'description' => 'x(t) * h(t)']
                        ]"
                        checkedValue="discrete"
                        x-model="convType"
                    />
                </div>
            </x-slot>

            <div class="mb-8">
                <div class="relative bg-slate-50 dark:bg-slate-800/50 p-4 rounded-lg">
                    <canvas id="convChart" style="height: 500px;"></canvas>
                </div>
            </div>

            <x-slot:rightSidebar>
                <div class="space-y-6">
                    <x-app-ui.function-domain
                        x-model-function="signal1"
                        x-model-domain-start="start1"
                        x-model-domain-end="end1"
                        function-placeholder="x[n]"
                        :function-error-model="'signal1_error'"
                    />

                    <x-app-ui.function-domain
                        x-model-function="signal2"
                        x-model-domain-start="start2"
                        x-model-domain-end="end2"
                        function-placeholder="h[n]"
                        :function-error-model="'signal2_error'"
                    />

                    <x-app-ui.button @click="convolve()">
                        Convolucionar
                    </x-app-ui.button>
                </div>
            </x-slot>
        </x-three-column-tool>
    </div>

    <x-slot:scripts>
        @vite('resources/js/convolution/app.ts')
    </x-slot>
</x-tool-layout>
```

---

## 📝 Notas Finales

- ✅ **Siempre usa `validateFunction` y `validateConstant`** para inputs matemáticos
- ✅ **Sufijo `_error`** en campos de error Alpine (ej: `frequency_error`)
- ✅ **ResizeObserver** para gráficas responsive
- ✅ **`x-transition`** para animaciones suaves al cambiar modos
- ✅ **Typed interfaces** en TypeScript para estados Alpine
- ✅ **Reutiliza componentes** antes de crear nuevos

---

**Autor:** Equipo Achronyme
**Última actualización:** 2025-10-01
**Versión:** 1.0.0
