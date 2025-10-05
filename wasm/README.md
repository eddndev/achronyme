# Achronyme WASM Development

This directory contains the C++ source code that gets compiled to WebAssembly for high-performance mathematical computations.

## 🚀 Quick Start

### Prerequisites

1. **Emscripten SDK**
   ```bash
   git clone https://github.com/emscripten-core/emsdk.git
   cd emsdk
   ./emsdk install latest
   ./emsdk activate latest
   source ./emsdk_env.sh  # Add to your shell profile
   ```

2. **CMake** (≥ 3.20)
   ```bash
   # macOS
   brew install cmake

   # Ubuntu/Debian
   sudo apt install cmake

   # Windows
   # Download from: https://cmake.org/download/
   ```

3. **Binaryen** (for wasm-opt)
   ```bash
   npm install -g binaryen
   ```

4. **Brotli** (for compression)
   ```bash
   npm install -g brotli-cli
   ```

### Build Commands

```bash
# From project root
cd /path/to/achronyme

# Build for production (optimized + compressed)
bash scripts/build-wasm.sh Release ON

# Build for development (debug symbols)
bash scripts/build-wasm.sh Debug OFF

# Or use npm scripts (recommended)
npm run wasm:build        # Production build
npm run wasm:dev          # Development build
npm run wasm:clean        # Clean build artifacts
```

## 📁 Directory Structure

```
wasm/
├── src/                    # C++ source code
│   ├── core/              # Base types (Complex, Vector, Matrix)
│   ├── parser/            # Mathematical expression parser
│   ├── dsp/               # Digital Signal Processing
│   ├── linalg/            # Linear Algebra
│   ├── numerical/         # Numerical methods
│   ├── optimization/      # Optimization algorithms
│   └── bindings/          # Emscripten bindings (C++ → JS)
│
├── include/               # Public headers
├── tests/                 # C++ tests (Google Test)
├── build/                 # Build output (gitignored)
├── dist/                  # Final WASM modules (gitignored)
├── CMakeLists.txt         # Build configuration
└── README.md             # This file
```

## 🔧 Development Workflow

### 1. Create a New Feature

```bash
# Example: Adding a new DSP function
touch wasm/src/dsp/windowing.cpp
touch wasm/include/achronyme/windowing.h
```

### 2. Update CMakeLists.txt

Add your new file to the appropriate library:

```cmake
add_library(achronyme_dsp STATIC
    src/dsp/fourier.cpp
    src/dsp/fft.cpp
    src/dsp/windowing.cpp  # ← Add here
)
```

### 3. Write C++ Code

```cpp
// wasm/src/dsp/windowing.cpp
#include "achronyme/windowing.h"
#include <cmath>

std::vector<double> hammingWindow(size_t N) {
    std::vector<double> window(N);
    for (size_t n = 0; n < N; n++) {
        window[n] = 0.54 - 0.46 * std::cos(2.0 * M_PI * n / (N - 1));
    }
    return window;
}
```

### 4. Create Bindings

```cpp
// wasm/src/bindings/dsp_bindings.cpp
#include <emscripten/bind.h>
#include "achronyme/windowing.h"

using namespace emscripten;

EMSCRIPTEN_BINDINGS(dsp) {
    function("hammingWindow", &hammingWindow);
    register_vector<double>("VectorDouble");
}
```

### 5. Build & Test

```bash
# Build
npm run wasm:build

# Test in browser
npm run dev

# Check console for:
# ✅ Using WASM engine
```

## 🧪 Testing

### Unit Tests (C++)

```bash
# Build tests (native, not WASM)
mkdir -p wasm/build/test
cd wasm/build/test
cmake ../.. -DBUILD_TESTS=ON -DBUILD_BENCHMARKS=ON
make -j$(nproc)

# Run tests
./test_parser
./test_fourier
./test_matrix

# Run benchmarks
./benchmark_all
```

### Integration Tests (TypeScript)

```bash
# In project root
npm run test
```

## ⚡ Optimization Guide

### Compiler Flags (CMakeLists.txt)

- **`-O3`**: Maximum optimization
- **`-ffast-math`**: Aggressive math optimizations
- **`-flto`**: Link-time optimization
- **`-msimd128`**: Enable SIMD instructions

### wasm-opt Passes

- **`-O3`**: Code optimization
- **`--strip-debug`**: Remove debug info
- **`--dce`**: Dead code elimination
- **`--vacuum`**: Remove unused code
- **`--flatten`**: Flatten control flow

### Brotli Compression

- **Quality 11**: Maximum compression (~70% size reduction)
- Served with `Content-Encoding: br` header

## 📊 Performance Benchmarks

Run benchmarks to verify performance:

```bash
cd wasm/build/test
./benchmark_all --benchmark_format=console
```

Expected results (compared to Math.js):

| Operation | Math.js | WASM | Speedup |
|-----------|---------|------|---------|
| Parse expression | 50μs | 5μs | **10x** |
| FFT 1024 | 5ms | 0.3ms | **16x** |
| Matrix mult 100x100 | 80ms | 3ms | **26x** |

## 🐛 Debugging

### Enable Debug Build

```bash
bash scripts/build-wasm.sh Debug OFF
```

Debug builds include:
- DWARF debug symbols
- Assertions (`SAFE_HEAP`)
- No minification

### Browser DevTools

1. Open DevTools → Sources
2. Enable "WebAssembly debugging" in settings
3. Set breakpoints in WASM code
4. Step through C++ source

### Memory Debugging

```javascript
// In browser console
Module.HEAP8    // Raw memory view
Module.ccall    // Call C++ functions directly
Module.cwrap    // Wrap C++ functions
```

## 📦 Module Structure

### Core Module (achronyme-core.wasm)

- Parser
- Base types (Complex, Vector, Matrix)
- Constants (PI, E, etc.)
- **~50KB compressed**

### DSP Module (achronyme-dsp.wasm) [Lazy loaded]

- Fourier Series/Transform
- FFT
- Convolution
- Filters
- **~80KB compressed**

### LinAlg Module (achronyme-linalg.wasm) [Lazy loaded]

- Matrix operations
- Linear solvers
- Decompositions
- **~120KB compressed**

## 🔗 Useful Links

- [Emscripten Documentation](https://emscripten.org/docs/)
- [WebAssembly Spec](https://webassembly.github.io/spec/)
- [Embind (C++ ↔ JS bindings)](https://emscripten.org/docs/porting/connecting_cpp_and_javascript/embind.html)
- [wasm-opt Options](https://github.com/WebAssembly/binaryen#wasm-opt)
- [Google Test Primer](https://google.github.io/googletest/primer.html)

## 🚨 Common Issues

### Issue: `emcc not found`

**Solution:**
```bash
source ~/emsdk/emsdk_env.sh
# Or add to ~/.bashrc / ~/.zshrc
```

### Issue: `Module is not defined`

**Solution:** Check that Vite is configured to handle `.wasm` files:
```javascript
// vite.config.js
assetsInclude: ['**/*.wasm']
```

### Issue: Memory errors in WASM

**Solution:** Increase `INITIAL_MEMORY` in CMakeLists.txt:
```cmake
"-s INITIAL_MEMORY=67108864"  # 64MB
```

### Issue: Slow build times

**Solution:** Use ccache:
```bash
export CC="ccache emcc"
export CXX="ccache em++"
```

## 📝 Contributing

When adding new C++ code:

1. ✅ Follow C++20 standard
2. ✅ Add unit tests
3. ✅ Document public APIs with Doxygen comments
4. ✅ Update this README if adding new modules
5. ✅ Run benchmarks to verify performance

---

**Happy WASM coding! 🚀**
