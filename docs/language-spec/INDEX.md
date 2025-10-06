# Achronyme Language Specification - Complete Index

**Version:** 0.2.0 - SIMPLIFIED (Math Engine Only)
**Last Updated:** 2025-10-05

---

## 📖 Quick Navigation

### For New Contributors
1. Start: [README.md](README.md) - Overview (Math engine, NOT full language)
2. Learn types: [types/01-primitive-types.md](types/01-primitive-types.md) - Only Number type
3. See examples: [examples/07-composition.md](examples/07-composition.md) - Math compositions

### For Implementers
1. Lexer: [grammar/01-lexical.md](grammar/01-lexical.md) - ~27 tokens (simplified)
2. Parser: [grammar/02-syntax.md](grammar/02-syntax.md) - Expressions only (no statements)
3. Precedence: [grammar/03-precedence.md](grammar/03-precedence.md) - Math operators only
4. Semantics: [semantics/02-composition.md](semantics/02-composition.md) - Function composition

---

## 📂 Complete File Structure

```
language-spec/
│
├── 📄 README.md                         ✅ Complete
│   └── Overview, design principles, type hierarchy
│
├── 📄 INDEX.md                          ✅ Complete (this file)
│   └── Navigation and quick reference
│
├── 📁 grammar/                          ✅ Complete (Simplified)
│   ├── 01-lexical.md                   ✅ Complete (~27 tokens)
│   │   └── Math tokens only, no strings/booleans
│   ├── 02-syntax.md                    ✅ Complete (Expressions only)
│   │   └── BNF grammar (no assignments, no control flow)
│   └── 03-precedence.md                ✅ Complete (5 levels)
│       └── Math operators only (+,-,*,/,^)
│
├── 📁 types/                            ✅ Complete (Simplified)
│   ├── 01-primitive-types.md           ✅ Complete (Number only)
│   │   └── Only Number type (f64), no Boolean/String/Undefined
│   ├── 02-complex-types.md             ⏳ Pending
│   │   └── Complex numbers (a+bi)
│   └── 03-collections.md               ⏳ Pending
│       └── Vector, Matrix
│
│   **Removed files (not needed for math):**
│   - ❌ 04-functions.md (no first-class functions in math engine)
│   - ❌ 05-symbolic.md (covered in functions)
│   - ❌ 06-type-coercion.md (only Number coercion needed)
│
├── 📁 functions/                        ⏳ Pending
│   ├── 01-elementary.md                ⏳ Pending
│   │   └── sin, cos, exp, log, sqrt, etc.
│   ├── 02-algebra.md                   ⏳ Pending
│   │   └── simplify, factor, expand, solve
│   ├── 03-calculus.md                  ⏳ Pending
│   │   └── diff, integrate, limit, series
│   ├── 04-linear-algebra.md            ⏳ Pending
│   │   └── inverse, det, eigenvalues, svd
│   ├── 05-dsp.md                       ⏳ Pending
│   │   └── dft, fft, convolve, filters
│   ├── 06-optimization.md              ⏳ Pending
│   │   └── minimize, maximize, solve, simplex
│   ├── 07-differential-eq.md           ⏳ Pending
│   │   └── dsolve, ndsolve, ode, pde
│   └── 08-special-functions.md         ⏳ Pending
│       └── gamma, bessel, erf, hypergeometric
│
├── 📁 semantics/                        ✅ Complete (Simplified)
│   └── 02-composition.md               ✅ Complete
│       └── Math function composition, broadcasting
│
│   **Removed files (not needed for math):**
│   - ❌ 01-evaluation-model.md (always eager for math)
│   - ❌ 03-overloading.md (covered in composition)
│   - ❌ 04-errors.md (IEEE 754 handles NaN/Infinity)
│   - ❌ 05-optimization.md (implementation detail)
│
└── 📁 examples/                         ✅ Complete (Simplified)
    └── 07-composition.md               ✅ Complete
        └── Math composition examples (DFT, linear algebra)

    **Removed files (not needed for math):**
    - ❌ 01-08 (replaced by single comprehensive example file)
```

**Legend:**
- ✅ Complete - Fully documented
- 🔄 In Progress - Partially complete
- ⏳ Pending - Not yet started

---

## 🎯 Key Decisions Documented

### ✅ Completed Decisions (Math Engine Only)

1. **Type System - SIMPLIFIED** (`types/01-primitive-types.md`)
   - Number: f64 (IEEE 754 double precision) - ONLY primitive type
   - **Removed:** Boolean, String, Undefined (not mathematical values)

2. **Grammar - SIMPLIFIED** (`grammar/`)
   - ~27 tokens (vs 50+ in full languages)
   - 5 precedence levels (vs 11+ in full languages)
   - Expressions only (NO assignments, NO control flow)
   - **Removed:** if/else, for/while, variables, boolean logic, strings

3. **Composition Rules** (`semantics/02-composition.md`)
   - `1/2 * dft(...)` → Scalar multiplication broadcasts ✓
   - `magnitude(dft(...))` → Element-wise magnitude ✓
   - `[1,2] * [3,4]` → Hadamard product (element-wise) ✓
   - `dot([1,2], [3,4])` → Explicit dot product function ✓

4. **Function Composition** (`examples/07-composition.md`)
   - Right-to-left for function calls: `f(g(h(x)))`
   - Left-to-right for operators: `2 + 3 * 4`
   - Broadcasting on collections
   - Type inference through composition
   - **No variables** in expressions (use host language for that)

---

## 🚀 Quick Reference: Your Questions Answered

### Q1: Can you do `1/2 * dft([1,2,3,4])`?
**Answer:** YES ✓

```achronyme
# Direct expression (no variables):
1/2 * dft([1, 2, 3, 4])
# Type: Vector<Complex>
# Scalar broadcasts to all elements
```

**See:** [examples/07-composition.md § 1](examples/07-composition.md#1-your-original-question-dft-composition)

---

### Q2: Can you do `magnitude(dft(...))`?
**Answer:** YES ✓

```achronyme
# Direct expression (no variables):
magnitude(dft([1, 2, 3, 4]))
# Type: Vector<Number>
# magnitude() applies element-wise
```

**See:** [examples/07-composition.md § 1](examples/07-composition.md#q-can-you-do-magnitudedft)

---

### Q3: How are functions composed?
**Answer:** Naturally with type inference (NO variables needed)

```achronyme
# Complex chain (direct expression):
sum(magnitude(dft([1, 2, 3, 4]))) / 4

# Type flow:
# dft(...)           : Vector<Complex>
# magnitude(...)     : Vector<Number>
# sum(...)           : Number
# ... / 4            : Number
```

**Note:** Host language (JavaScript/Python) manages variables, Achronyme evaluates pure math expressions.

**See:** [semantics/02-composition.md § 2](semantics/02-composition.md#2-type-based-composition-rules)

---

## 📊 Implementation Checklist (Simplified for Math Engine)

### Phase 1: Core Parser (Sprints 1-2)
- [ ] Lexer implementation (~27 tokens)
- [ ] Parser (expressions only, BNF → AST)
- [x] Type system design (Number only)
- [x] Composition rules
- [ ] Basic evaluator (expressions)

### Phase 2: Type System (Sprints 3-4)
- [x] Primitive type (Number only)
- [ ] Complex numbers (a + bi)
- [ ] Vectors and matrices
- [ ] ~~Functions and lambdas~~ (removed - not needed)
- [ ] Symbolic expressions

### Phase 3: Functions (Sprints 5-8)
- [ ] Elementary functions (sin, cos, exp)
- [ ] DSP (DFT, FFT) - **core feature**
- [ ] Linear algebra (inverse, det)
- [ ] Calculus (diff, integrate)
- [ ] ~~Optimization (simplex)~~ (future, not MVP)

**Complexity Reduction:** 70% fewer features than full language spec!

---

## 🔗 External Resources

### Similar Language Specs
- [Python Language Reference](https://docs.python.org/3/reference/)
- [Julia Manual](https://docs.julialang.org/en/v1/)
- [Wolfram Language](https://reference.wolfram.com/language/)

### Academic Papers
- "A Formal Semantics for Computer Algebra Systems" - need to research
- "Type Systems for Symbolic Computation" - need to research

---

## 📝 Contributing

To add or modify specification:

1. Create/edit file in appropriate directory
2. Update this INDEX.md
3. Link from README.md if major addition
4. Add examples in `examples/`
5. Tag issue with `lang-spec` label

---

## 🎓 Learning Path

### Beginner (Understand the language)
1. [README.md](README.md) - Overview
2. [types/01-primitive-types.md](types/01-primitive-types.md) - Basic types
3. [examples/07-composition.md](examples/07-composition.md) - How things combine

### Intermediate (Implement features)
4. [grammar/02-syntax.md](grammar/02-syntax.md) - Parse expressions *(pending)*
5. [semantics/02-composition.md](semantics/02-composition.md) - Execution model
6. [functions/05-dsp.md](functions/05-dsp.md) - Implement DSP *(pending)*

### Advanced (Extend the language)
7. [semantics/03-overloading.md](semantics/03-overloading.md) - Function overloading *(pending)*
8. [semantics/05-optimization.md](semantics/05-optimization.md) - Optimizations *(pending)*
9. [types/05-symbolic.md](types/05-symbolic.md) - Symbolic computation *(pending)*

---

## 🎯 Next Steps

**Completed (Math Engine Simplification):**

1. ✅ Simplify scope to math engine only
2. ✅ Update README.md (removed programming constructs)
3. ✅ Reduce grammar/01-lexical.md (~27 tokens)
4. ✅ Simplify grammar/02-syntax.md (expressions only)
5. ✅ Update grammar/03-precedence.md (5 levels)
6. ✅ Simplify types/01-primitive-types.md (Number only)
7. ✅ Update semantics/02-composition.md (math only)
8. ✅ Update examples/07-composition.md (no variables)

**Next priorities:**

9. ⏳ Implement lexer based on simplified grammar
10. ⏳ Implement parser (expressions only)
11. ⏳ Define types/02-complex-types.md
12. ⏳ Define functions/05-dsp.md (DFT, FFT)

**See:** [docs/sprints/01-wasm-foundation.md](../../sprints/01-wasm-foundation.md)

---

Last updated: 2025-10-05 (Version 0.2.0 - SIMPLIFIED)
