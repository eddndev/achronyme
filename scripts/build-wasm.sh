#!/bin/bash

# Build script for Achronyme WASM modules
# Requires: Emscripten, CMake, Binaryen (wasm-opt), Brotli

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🔨 Building Achronyme WASM Modules${NC}"

# Check if Emscripten is available
if ! command -v emcc &> /dev/null; then
    echo -e "${RED}❌ Emscripten not found!${NC}"
    echo "Please install: https://emscripten.org/docs/getting_started/downloads.html"
    exit 1
fi

# Check if wasm-opt is available
if ! command -v wasm-opt &> /dev/null; then
    echo -e "${YELLOW}⚠️  wasm-opt not found. Installing via npm...${NC}"
    npm install -g binaryen
fi

# Check if brotli is available
if ! command -v brotli &> /dev/null; then
    echo -e "${YELLOW}⚠️  brotli not found. Installing...${NC}"
    npm install -g brotli-cli
fi

# Configuration
BUILD_TYPE=${1:-Release}  # Debug or Release
ENABLE_SIMD=${2:-ON}      # ON or OFF
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_DIR="${PROJECT_ROOT}/wasm"
BUILD_DIR="${WASM_DIR}/build/${BUILD_TYPE,,}"
DIST_DIR="${WASM_DIR}/dist"
PUBLIC_DIR="${PROJECT_ROOT}/public/wasm"

echo -e "${GREEN}📁 Project root: ${PROJECT_ROOT}${NC}"
echo -e "${GREEN}🔧 Build type: ${BUILD_TYPE}${NC}"
echo -e "${GREEN}⚡ SIMD: ${ENABLE_SIMD}${NC}"

# Clean previous build
echo -e "${YELLOW}🧹 Cleaning previous build...${NC}"
rm -rf "${BUILD_DIR}"
rm -rf "${DIST_DIR}"
mkdir -p "${BUILD_DIR}"
mkdir -p "${DIST_DIR}"
mkdir -p "${PUBLIC_DIR}"

# Step 1: Configure with CMake
echo -e "${GREEN}⚙️  Step 1/5: Configuring CMake...${NC}"
cd "${BUILD_DIR}"
emcmake cmake "${WASM_DIR}" \
    -DCMAKE_BUILD_TYPE="${BUILD_TYPE}" \
    -DENABLE_SIMD="${ENABLE_SIMD}" \
    -G "Unix Makefiles"

# Step 2: Build
echo -e "${GREEN}🔨 Step 2/5: Compiling C++ to WASM...${NC}"
emmake make -j$(nproc)

# Step 3: Optimize with wasm-opt
echo -e "${GREEN}⚡ Step 3/5: Optimizing WASM modules...${NC}"

if [ "${BUILD_TYPE}" = "Release" ]; then
    for wasm_file in "${BUILD_DIR}"/*.wasm; do
        if [ -f "$wasm_file" ]; then
            filename=$(basename "$wasm_file" .wasm)
            echo "  Optimizing ${filename}.wasm..."

            # Optimization passes
            wasm-opt -O3 \
                --strip-debug \
                --strip-producers \
                --vacuum \
                --flatten \
                --rereloop \
                --enable-simd \
                "$wasm_file" \
                -o "${DIST_DIR}/${filename}.opt.wasm"

            # Tree-shaking
            wasm-opt --dce \
                --remove-unused-names \
                --remove-unused-module-elements \
                "${DIST_DIR}/${filename}.opt.wasm" \
                -o "${DIST_DIR}/${filename}.wasm"

            # Remove intermediate file
            rm "${DIST_DIR}/${filename}.opt.wasm"

            # Get size
            size=$(du -h "${DIST_DIR}/${filename}.wasm" | cut -f1)
            echo -e "  ✅ ${filename}.wasm optimized (${size})"
        fi
    done
else
    # Debug: just copy
    cp "${BUILD_DIR}"/*.wasm "${DIST_DIR}/"
fi

# Copy JS glue code
cp "${BUILD_DIR}"/*.js "${DIST_DIR}/"

# Step 4: Compress with Brotli
echo -e "${GREEN}📦 Step 4/5: Compressing with Brotli...${NC}"

for wasm_file in "${DIST_DIR}"/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        echo "  Compressing ${filename}..."

        # Brotli compression (quality 11 = max)
        brotli -q 11 -f "$wasm_file" -o "${wasm_file}.br"

        # Calculate compression ratio
        original_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file")
        compressed_size=$(stat -f%z "${wasm_file}.br" 2>/dev/null || stat -c%s "${wasm_file}.br")
        ratio=$(awk "BEGIN {printf \"%.1f\", ($original_size - $compressed_size) * 100 / $original_size}")

        original_human=$(du -h "$wasm_file" | cut -f1)
        compressed_human=$(du -h "${wasm_file}.br" | cut -f1)

        echo -e "  ✅ ${filename}.br created (${original_human} → ${compressed_human}, ${ratio}% reduction)"
    fi
done

# Step 5: Copy to public/
echo -e "${GREEN}📂 Step 5/5: Copying to public/wasm/...${NC}"
cp "${DIST_DIR}"/*.wasm "${PUBLIC_DIR}/"
cp "${DIST_DIR}"/*.wasm.br "${PUBLIC_DIR}/"
cp "${DIST_DIR}"/*.js "${PUBLIC_DIR}/"

# Summary
echo ""
echo -e "${GREEN}✅ Build completed successfully!${NC}"
echo ""
echo -e "${GREEN}📊 Build Summary:${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for wasm_file in "${DIST_DIR}"/*.wasm; do
    if [ -f "$wasm_file" ]; then
        filename=$(basename "$wasm_file")
        wasm_size=$(du -h "$wasm_file" | cut -f1)
        br_size=$(du -h "${wasm_file}.br" | cut -f1)
        echo "  📦 ${filename}: ${wasm_size} (${br_size} compressed)"
    fi
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${GREEN}🚀 Files ready in: ${PUBLIC_DIR}/${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Run: npm run dev"
echo "  2. Test WASM modules in browser"
echo "  3. Check DevTools console for 'Using WASM engine' message"
echo ""
