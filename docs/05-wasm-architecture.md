# WebAssembly Architecture - Achronyme

**Versión:** 1.0
**Fecha:** 2025-10-05
**Mantenido por:** @eddndev

---

## 1. Objetivos de la Migración a WASM

### ¿Por qué WASM?
- **Rendimiento**: 10-100x más rápido que JavaScript para cálculos matemáticos
- **Portabilidad**: Mismo código corre en navegador, Node.js, y otros runtimes
- **Tamaño**: Binarios comprimidos más pequeños que librerías JS equivalentes
- **Seguridad**: Sandboxed execution, sin acceso directo al sistema
- **Reutilización**: Código C++ existente puede ser portado

### Scope de la Migración
- ✅ **Reemplazar completamente Math.js** para todos los cálculos
- ✅ **Parser matemático propio** (expresiones, funciones, constantes)
- ✅ **Módulos especializados**: DSP, Álgebra Lineal, Cálculo Numérico
- ✅ **Optimización de tamaño**: Compresión Brotli/Gzip, tree-shaking
- ✅ **Build pipeline integrado**: Emscripten + Vite

---

## 2. Estructura de Directorios

```
achronyme/
├── wasm/                           # Todo el código WASM/C++
│   ├── src/                        # Código fuente C++
│   │   ├── core/                   # Núcleo matemático
│   │   │   ├── types.hpp           # Tipos base (Complex, Matrix, Vector)
│   │   │   ├── constants.hpp       # Constantes matemáticas (PI, E, etc)
│   │   │   └── utils.hpp           # Utilidades generales
│   │   │
│   │   ├── parser/                 # Parser de expresiones matemáticas
│   │   │   ├── lexer.cpp           # Tokenización
│   │   │   ├── parser.cpp          # Análisis sintáctico
│   │   │   ├── evaluator.cpp       # Evaluación de expresiones
│   │   │   └── functions.cpp       # Funciones matemáticas (sin, cos, etc)
│   │   │
│   │   ├── dsp/                    # Digital Signal Processing
│   │   │   ├── fourier.cpp         # Series y Transform de Fourier
│   │   │   ├── fft.cpp             # Fast Fourier Transform (Cooley-Tukey)
│   │   │   ├── convolution.cpp     # Convolución de señales
│   │   │   ├── filters.cpp         # Filtros (FIR, IIR)
│   │   │   └── windows.cpp         # Funciones ventana (Hamming, Hann, etc)
│   │   │
│   │   ├── linalg/                 # Álgebra Lineal
│   │   │   ├── matrix.cpp          # Operaciones matriciales
│   │   │   ├── vector.cpp          # Operaciones vectoriales
│   │   │   ├── decomposition.cpp   # LU, QR, SVD, Cholesky
│   │   │   ├── eigenvalues.cpp     # Cálculo de eigenvalues/eigenvectors
│   │   │   └── solver.cpp          # Sistemas lineales (Gauss, Jacobi)
│   │   │
│   │   ├── numerical/              # Métodos Numéricos
│   │   │   ├── integration.cpp     # Integración (Simpson, Trapecio)
│   │   │   ├── differentiation.cpp # Derivación numérica
│   │   │   ├── rootfinding.cpp     # Raíces (Newton, Bisección, Secante)
│   │   │   ├── interpolation.cpp   # Interpolación (Lagrange, Spline)
│   │   │   └── ode.cpp             # EDOs (Euler, Runge-Kutta)
│   │   │
│   │   ├── optimization/           # Optimización
│   │   │   ├── simplex.cpp         # Método Simplex (LP)
│   │   │   ├── gradient.cpp        # Descenso de gradiente
│   │   │   └── genetic.cpp         # Algoritmos genéticos (futuro)
│   │   │
│   │   └── bindings/               # Bindings para JavaScript
│   │       ├── dsp_bindings.cpp    # Exportar funciones DSP
│   │       ├── linalg_bindings.cpp # Exportar funciones Álgebra
│   │       ├── parser_bindings.cpp # Exportar parser
│   │       └── main.cpp            # Entry point principal
│   │
│   ├── include/                    # Headers públicos
│   │   └── achronyme/
│   │       ├── dsp.h
│   │       ├── linalg.h
│   │       ├── numerical.h
│   │       └── parser.h
│   │
│   ├── build/                      # Output de compilación (gitignored)
│   │   ├── debug/                  # Build debug (sin optimizar)
│   │   └── release/                # Build release (optimizado)
│   │
│   ├── dist/                       # Módulos WASM finales (gitignored)
│   │   ├── achronyme-core.wasm     # Módulo principal
│   │   ├── achronyme-core.js       # Glue code
│   │   ├── achronyme-dsp.wasm      # Módulo DSP (lazy load)
│   │   ├── achronyme-linalg.wasm   # Módulo Álgebra (lazy load)
│   │   └── *.wasm.br               # Versiones comprimidas Brotli
│   │
│   ├── tests/                      # Tests C++ (Google Test)
│   │   ├── test_parser.cpp
│   │   ├── test_fourier.cpp
│   │   ├── test_matrix.cpp
│   │   └── benchmark.cpp           # Benchmarks de rendimiento
│   │
│   ├── CMakeLists.txt              # Build config principal
│   ├── emscripten.cmake            # Config específica de Emscripten
│   └── README.md                   # Docs de desarrollo WASM
│
├── resources/js/                   # Frontend JavaScript/TypeScript
│   ├── wasm/                       # Wrappers JS para WASM
│   │   ├── loader.ts               # Carga dinámica de módulos WASM
│   │   ├── core.ts                 # Wrapper del módulo core
│   │   ├── dsp.ts                  # Wrapper DSP (lazy)
│   │   ├── linalg.ts               # Wrapper Álgebra (lazy)
│   │   └── fallback.ts             # Fallback a Math.js si WASM falla
│   │
│   ├── math/                       # API unificada (abstracción)
│   │   ├── index.ts                # Export principal
│   │   ├── parser.ts               # Parser interface
│   │   ├── fourier.ts              # Fourier interface
│   │   └── matrix.ts               # Matrix interface
│   │
│   └── utils/
│       └── wasm-detect.ts          # Detección de soporte WASM
│
├── public/wasm/                    # WASM servido estáticamente
│   └── (copiado desde wasm/dist/)
│
└── scripts/                        # Scripts de build
    ├── build-wasm.sh               # Build completo de WASM
    ├── optimize-wasm.sh            # Optimización (wasm-opt, brotli)
    └── generate-bindings.py        # Auto-generar bindings (futuro)
```

---

## 3. Build Pipeline

### 3.1 Herramientas Necesarias

```bash
# Emscripten (compilador C++ → WASM)
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh

# Binaryen (optimización WASM)
npm install -g binaryen

# Google Test (testing C++)
# Se descarga automáticamente con CMake
```

### 3.2 Proceso de Build

**Step 1: Compilar C++ a WASM**

```bash
cd wasm/
mkdir -p build/release
cd build/release

# Configurar con CMake + Emscripten
emcmake cmake ../.. \
  -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_SIMD=ON \
  -DENABLE_THREADS=OFF

# Compilar
emmake make -j$(nproc)

# Output: *.wasm + *.js en build/release/
```

**Step 2: Optimizar con wasm-opt**

```bash
# Optimización agresiva (Level 3)
wasm-opt -O3 \
  --strip-debug \
  --strip-producers \
  --vacuum \
  --flatten \
  --rereloop \
  achronyme-core.wasm \
  -o achronyme-core.opt.wasm

# Tree-shaking (eliminar código no usado)
wasm-opt --dce \
  --remove-unused-names \
  --remove-unused-module-elements \
  achronyme-core.opt.wasm \
  -o achronyme-core.final.wasm
```

**Step 3: Comprimir con Brotli**

```bash
# Compresión Brotli (mejor que gzip para WASM)
brotli -q 11 achronyme-core.final.wasm

# Output: achronyme-core.final.wasm.br (60-80% más pequeño)
```

**Step 4: Copiar a public/**

```bash
cp ../dist/*.wasm public/wasm/
cp ../dist/*.wasm.br public/wasm/
```

### 3.3 Integración con Vite

**vite.config.js**

```javascript
import { defineConfig } from 'vite';
import laravel from 'laravel-vite-plugin';
import { execSync } from 'child_process';

export default defineConfig({
  plugins: [
    laravel({
      input: ['resources/css/app.css', 'resources/js/app.js'],
      refresh: true,
    }),

    // Plugin custom para build WASM
    {
      name: 'wasm-builder',
      buildStart() {
        console.log('🔨 Building WASM modules...');
        execSync('bash scripts/build-wasm.sh', { stdio: 'inherit' });
      }
    }
  ],

  // Configuración para servir WASM
  assetsInclude: ['**/*.wasm', '**/*.wasm.br'],

  server: {
    headers: {
      // Headers necesarios para SharedArrayBuffer (threading futuro)
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    }
  },

  build: {
    rollupOptions: {
      output: {
        // Optimizar chunking para lazy load
        manualChunks(id) {
          if (id.includes('wasm/dsp')) return 'wasm-dsp';
          if (id.includes('wasm/linalg')) return 'wasm-linalg';
        }
      }
    }
  }
});
```

---

## 4. Estrategia de Optimización de Tamaño

### 4.1 Técnicas de Reducción

| Técnica | Reducción | Descripción |
|---------|-----------|-------------|
| **wasm-opt -O3** | ~30% | Optimización de código |
| **Strip debug info** | ~15% | Eliminar símbolos de debug |
| **Tree-shaking** | ~20% | Eliminar código no usado |
| **Brotli compression** | ~70% | Compresión HTTP |
| **Lazy loading** | N/A | Cargar módulos solo cuando se necesitan |
| **Code splitting** | N/A | Dividir en módulos pequeños |

### 4.2 Módulos Especializados

En lugar de un WASM monolítico, crear módulos pequeños:

```
achronyme-core.wasm     →  ~50KB  (parser, tipos base)
achronyme-dsp.wasm      →  ~80KB  (Fourier, FFT, convolución)
achronyme-linalg.wasm   →  ~120KB (matrices, sistemas lineales)
achronyme-numerical.wasm→  ~60KB  (integración, derivación, raíces)
```

**Total comprimido:** ~150-200KB (vs. Math.js ~500KB sin comprimir)

### 4.3 Lazy Loading con Detección

```typescript
// resources/js/wasm/loader.ts
export class WasmLoader {
  private loadedModules = new Map<string, WebAssembly.Module>();

  async loadModule(name: string): Promise<WebAssembly.Instance> {
    // Verificar si ya está cargado
    if (this.loadedModules.has(name)) {
      return this.loadedModules.get(name)!;
    }

    // Detectar soporte Brotli
    const supportsBrotli = await this.detectBrotli();
    const ext = supportsBrotli ? '.wasm.br' : '.wasm';

    // Cargar módulo
    const response = await fetch(`/wasm/${name}${ext}`);
    const bytes = supportsBrotli
      ? await this.decompressBrotli(response)
      : await response.arrayBuffer();

    const module = await WebAssembly.compile(bytes);
    const instance = await WebAssembly.instantiate(module, {
      env: {
        memory: new WebAssembly.Memory({ initial: 256, maximum: 512 }),
        // Imports de funciones JS si son necesarias
      }
    });

    this.loadedModules.set(name, instance);
    return instance;
  }

  private async detectBrotli(): Promise<boolean> {
    return 'DecompressionStream' in window;
  }

  private async decompressBrotli(response: Response): Promise<ArrayBuffer> {
    const stream = response.body!
      .pipeThrough(new DecompressionStream('br'));

    const reader = stream.getReader();
    const chunks: Uint8Array[] = [];

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
    }

    const totalLength = chunks.reduce((acc, chunk) => acc + chunk.length, 0);
    const result = new Uint8Array(totalLength);
    let offset = 0;

    for (const chunk of chunks) {
      result.set(chunk, offset);
      offset += chunk.length;
    }

    return result.buffer;
  }
}
```

---

## 5. API Design

### 5.1 Parser Matemático

**C++ (wasm/src/parser/parser.cpp)**

```cpp
#include <emscripten/bind.h>
#include "parser.hpp"

using namespace emscripten;

class MathParser {
public:
  double evaluate(const std::string& expression) {
    return evaluator_.eval(expression);
  }

  val evaluateComplex(const std::string& expression) {
    Complex result = evaluator_.evalComplex(expression);
    return val::object()
      .set("re", result.real())
      .set("im", result.imag());
  }

private:
  Evaluator evaluator_;
};

EMSCRIPTEN_BINDINGS(parser) {
  class_<MathParser>("MathParser")
    .constructor<>()
    .function("evaluate", &MathParser::evaluate)
    .function("evaluateComplex", &MathParser::evaluateComplex);
}
```

**TypeScript Wrapper (resources/js/wasm/parser.ts)**

```typescript
import { WasmLoader } from './loader';

export class WasmParser {
  private instance: any;

  async init() {
    const loader = new WasmLoader();
    this.instance = await loader.loadModule('achronyme-core');
  }

  evaluate(expression: string): number {
    if (!this.instance) throw new Error('Parser not initialized');
    return this.instance.MathParser.evaluate(expression);
  }

  evaluateComplex(expression: string): { re: number; im: number } {
    if (!this.instance) throw new Error('Parser not initialized');
    return this.instance.MathParser.evaluateComplex(expression);
  }
}
```

### 5.2 API Unificada (Abstracción)

```typescript
// resources/js/math/index.ts
import { WasmParser } from '../wasm/parser';
import { detectWasmSupport } from '../utils/wasm-detect';
import * as mathjs from 'mathjs'; // Fallback

class MathEngine {
  private parser: WasmParser | null = null;
  private useWasm = false;

  async init() {
    this.useWasm = await detectWasmSupport();

    if (this.useWasm) {
      this.parser = new WasmParser();
      await this.parser.init();
      console.log('✅ Using WASM engine');
    } else {
      console.warn('⚠️ WASM not supported, using Math.js fallback');
    }
  }

  evaluate(expression: string): number {
    if (this.useWasm && this.parser) {
      return this.parser.evaluate(expression);
    }
    return mathjs.evaluate(expression);
  }
}

export const math = new MathEngine();
```

---

## 6. Testing Strategy

### 6.1 Tests C++ (Google Test)

```cpp
// wasm/tests/test_parser.cpp
#include <gtest/gtest.h>
#include "parser.hpp"

TEST(ParserTest, BasicArithmetic) {
  MathParser parser;
  EXPECT_DOUBLE_EQ(parser.evaluate("2 + 2"), 4.0);
  EXPECT_DOUBLE_EQ(parser.evaluate("10 * 5"), 50.0);
}

TEST(ParserTest, Functions) {
  MathParser parser;
  EXPECT_NEAR(parser.evaluate("sin(0)"), 0.0, 1e-10);
  EXPECT_NEAR(parser.evaluate("cos(0)"), 1.0, 1e-10);
}
```

### 6.2 Benchmarks

```cpp
// wasm/tests/benchmark.cpp
#include <benchmark/benchmark.h>
#include "fourier.hpp"

static void BM_FFT_1024(benchmark::State& state) {
  std::vector<Complex> signal(1024);
  for (auto _ : state) {
    fft(signal);
  }
}
BENCHMARK(BM_FFT_1024);
```

### 6.3 Integration Tests (Vitest)

```typescript
// resources/js/__tests__/wasm-integration.test.ts
import { describe, it, expect } from 'vitest';
import { math } from '../math';

describe('WASM Integration', () => {
  beforeAll(async () => {
    await math.init();
  });

  it('should evaluate basic expressions', () => {
    expect(math.evaluate('2 + 2')).toBe(4);
  });

  it('should match Math.js results', () => {
    const expr = 'sin(pi/2) + cos(0)';
    expect(math.evaluate(expr)).toBeCloseTo(2, 10);
  });
});
```

---

## 7. Performance Targets

| Operación | Math.js | WASM Target | Mejora |
|-----------|---------|-------------|--------|
| Parse expression | 50μs | 5μs | **10x** |
| FFT 1024 points | 5ms | 0.3ms | **16x** |
| Matrix multiply 100x100 | 80ms | 3ms | **26x** |
| Fourier series (n=50) | 120ms | 8ms | **15x** |
| Solve linear system 50x50 | 200ms | 12ms | **16x** |

**Target global:** 10-20x más rápido que Math.js

---

## 8. Roadmap de Implementación

### Sprint 1: Core + Parser (2 semanas)
- Setup build pipeline (CMake + Emscripten)
- Implementar tipos base (Complex, Vector, Matrix)
- Parser matemático completo
- Bindings JavaScript
- Tests unitarios

### Sprint 2: DSP Module (2 semanas)
- Fourier Series
- FFT (Cooley-Tukey)
- Convolution
- Lazy loading setup

### Sprint 3: Linear Algebra (2 semanas)
- Operaciones matriciales
- Solvers de sistemas lineales
- Descomposiciones (LU, QR)

### Sprint 4: Optimization (1 semana)
- wasm-opt pipeline
- Brotli compression
- Code splitting
- Performance benchmarks

### Sprint 5: Migration (1 semana)
- Reemplazar Math.js en frontend
- Fallback testing
- Production deployment

---

**Total estimado:** 8 semanas para migración completa

**Documento vivo - Actualizado con decisiones técnicas de WASM**
