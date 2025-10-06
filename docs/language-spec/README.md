# Achronyme Mathematical Expression Language

**Version:** 0.2.0 - SIMPLIFIED
**Status:** In Development
**Authors:** @eddndev
**Last Updated:** 2025-10-05

---

## 📚 Overview

Achronyme is a **mathematical expression evaluator and symbolic computation engine** - NOT a general-purpose programming language.

**Think of it as:** A powerful scientific calculator that can handle symbolic math, vectors, matrices, and Fourier transforms.

**Compare to:**
- ✅ Wolfram Alpha (query engine)
- ✅ Math.js (expression parser)
- ✅ SymPy (symbolic computation)
- ✅ NumPy/SciPy (numerical methods)

**NOT like:**
- ❌ Python (general programming)
- ❌ JavaScript (scripting)
- ❌ MATLAB programming (loops, if/else)

---

## 🎯 What Achronyme Does

### ✅ IN SCOPE

```achronyme
# Arithmetic expressions
2 + 3 * 4
(2 + 3) ^ 2

# Mathematical functions
sin(PI / 2)
exp(-abs(t))
sqrt(16)

# Symbolic calculus
diff(x^2, x)                    → 2*x
integrate(sin(x), x)            → -cos(x)
simplify(x + x)                 → 2*x

# DSP (Digital Signal Processing)
dft([1, 2, 3, 4])               → Vector<Complex>
dft(exp(-abs(t)), -5, 5, 500)   → Continuous Fourier Transform
magnitude(dft([1, 2, 3]))       → [10, 2.828, 2]

# Linear Algebra
inverse([[1, 2], [3, 4]])
eigenvalues(A)
det(matrix)

# Function composition
magnitude(dft([1, 2, 3]))
1/2 * dft([1, 2, 3])
sum(magnitude(dft([1, 2, 3])))
```

### ❌ OUT OF SCOPE

```achronyme
# ✗ String manipulation (NOT NEEDED)
"hello" + "world"

# ✗ Control flow (NOT NEEDED)
if x > 0 then { ... }
for i in 1..10 do { ... }

# ✗ Mutable variables (NOT NEEDED)
x = 5
x += 1

# ✗ Boolean logic (NOT NEEDED)
true && false
```

**Why removed?** These are programming constructs. Achronyme evaluates **mathematical expressions only**. If you need logic/loops, do it in JavaScript wrapper.

---

## 🗂️ Directory Structure

```
language-spec/
├── README.md                    # This file (overview)
├── INDEX.md                     # Navigation guide
│
├── grammar/                     # Language grammar
│   ├── 01-lexical.md           # Tokens (~30 types)
│   ├── 02-syntax.md            # BNF (expressions only)
│   └── 03-precedence.md        # Operator precedence
│
├── types/                       # Type system
│   ├── 01-primitive-types.md   # Number only
│   ├── 02-complex-types.md     # Complex numbers
│   └── 03-collections.md       # Vector, Matrix
│
├── functions/                   # Function catalog
│   ├── 01-elementary.md        # sin, cos, exp, log
│   ├── 03-calculus.md          # diff, integrate
│   ├── 04-linear-algebra.md    # inverse, det, eigenvalues
│   └── 05-dsp.md               # dft, fft, convolve
│
├── semantics/                   # Execution semantics
│   └── 02-composition.md       # Function composition
│
└── examples/                    # Use cases
    └── 07-composition.md       # DFT examples
```

---

## 🎯 Design Principles

### 1. **Expressions Only, No Statements**

```achronyme
# ✓ Valid (expression)
2 + 3 * 4

# ✓ Valid (function call)
sin(PI / 2)

# ✗ Invalid (assignment statement)
x = 5       # Not allowed (no mutable state)

# ✗ Invalid (control flow statement)
if x > 0 then ...  # Not allowed
```

### 2. **Pure Functions (No Side Effects)**

```achronyme
# Every expression returns a value
sin(x)              → Number
dft([1,2,3])        → Vector<Complex>
inverse(A)          → Matrix

# No mutation, no state changes
```

### 3. **Composability First**

```achronyme
# Functions compose naturally
magnitude(dft([1, 2, 3]))
1/2 * dft([1, 2, 3])
sum(magnitude(dft([1, 2, 3])))

# Type system ensures correctness
```

### 4. **Symbolic When Possible, Numeric When Needed**

```achronyme
# Symbolic
diff(x^2, x)        → 2*x (symbolic result)

# Numeric
integrate(exp(-x^2), x, 0, 1)  → 0.7468 (numeric result)
```

---

## 📖 Type System

```
Value (abstract)
├── Number          # 42, 3.14, 1e10 (f64)
├── Complex         # 3+4i
├── Vector          # [1, 2, 3]
├── Matrix          # [[1,2],[3,4]]
├── Function        # sin, dft, inverse
└── Symbolic        # x, x^2, 2*x+1
```

**Removed from full language spec:**
- ❌ Boolean (true/false) - Not needed for math
- ❌ String ("text") - Not a mathematical object
- ❌ Undefined - Not needed (functions always return)

**Note:** Comparisons (`<`, `>`, `==`) only used in piecewise function conditions, not general logic.

---

## 🔑 Key Concepts

### Mathematical Expressions

```achronyme
# Arithmetic
2 + 3 * 4           → 14
2 ^ 3 ^ 2           → 512 (right-associative)

# Functions
sin(PI / 2)         → 1
exp(0)              → 1

# Composition
sin(cos(x))
abs(sqrt(x))
```

### Function Signatures

```typescript
// Elementary
sin: Number → Number
sin: Complex → Complex

// DFT (overloaded)
dft: Vector<Number> → Vector<Complex>                    // Discrete
dft: (Function, Number, Number, Number) → Vector<Complex> // Continuous

// Linear Algebra
inverse: Matrix → Matrix
det: Matrix → Number
```

### Composition Rules

```achronyme
# Scalar operations broadcast over collections
2 * [1, 2, 3]                   → [2, 4, 6]
[1, 2, 3] + 5                   → [6, 7, 8]

# Functions compose naturally
f(g(x))                         → evaluate g first, then f

# Element-wise operations
[1, 2] * [3, 4]                 → [3, 8] (Hadamard product)
magnitude([3+4i, 5+12i])        → [5, 13]
```

---

## 🚀 Minimal Grammar Summary

```bnf
# Top-level: Just an expression
Program: Expression

# Mathematical expressions
Expression:
    Number                          # 42, 3.14
    | Identifier                    # x, PI, E
    | Expression + Expression       # Arithmetic
    | Expression - Expression
    | Expression * Expression
    | Expression / Expression
    | Expression ^ Expression
    | - Expression                  # Unary minus
    | FunctionCall                  # sin(x), dft([...])
    | ( Expression )                # Grouping
    | [ ElementList ]               # Vector
    | [[ RowList ]]                 # Matrix
    | { PiecewiseList }             # Piecewise

FunctionCall:
    Identifier ( ArgumentList )

# No assignments, no control flow, no variables!
```

---

## 🎯 Use Cases (Real Examples)

### DSP - Your Core Use Case

```achronyme
# Discrete DFT
dft([1, 2, 3, 4])
→ [10+0i, -2+2i, -2+0i, -2-2i]

# Continuous DFT (piecewise function)
f(t) = { t^2 if t < 0, exp(-t) if t >= 0 }
dft(f(t), -5, 5, 500)
→ Vector<Complex> with 500 frequency points

# Composition
magnitude(dft([1, 2, 3]))
→ [10, 2.828, 2]

# Scaling
1/2 * dft([1, 2, 3, 4])
→ [5+0i, -1+1i, -1+0i, -1-1i]
```

### Linear Algebra

```achronyme
# Matrix operations
A = [[1, 2], [3, 4]]
inverse(A)
→ [[-2, 1], [1.5, -0.5]]

# Eigenvalues
eigenvalues([[4, 1], [1, 3]])
→ [5, 2]

# System solving
solve(A * x == b, x)
```

### Symbolic Calculus

```achronyme
# Differentiation
diff(x^2 + 2*x, x)
→ 2*x + 2

# Integration
integrate(sin(x), x)
→ -cos(x) + C

# Simplification
simplify(sin(x)^2 + cos(x)^2)
→ 1
```

---

## 📊 Comparison: Before vs After

| Feature | Full Language | Math Engine | Removed |
|---------|--------------|-------------|---------|
| **Tokens** | 50+ | ~30 | Strings, booleans, keywords |
| **Grammar** | 6000+ lines | ~1000 lines | Control flow, assignments |
| **Types** | 7 types | 4 types | Boolean, String, Undefined |
| **Complexity** | High | Low | 70% reduction |
| **Impl Time** | 8 weeks | 4 weeks | 50% faster |

---

## ✅ What Makes This Practical

### 1. **Focused Scope**
- Only mathematical operations
- No programming constructs
- Clear, simple grammar

### 2. **Your Actual Needs**
- DFT/FFT (core feature)
- Matrix operations
- Symbolic calculus
- Function composition

### 3. **Implementation Simplicity**
- ~30 token types (vs 50+)
- Expression parser only (vs full language)
- No control flow complexity
- No variable scoping

### 4. **Performance**
- Pure expressions (easy to optimize)
- No side effects (parallelizable)
- WASM compilation straightforward

---

## 🎓 Mental Model

**Think of Achronyme as:**

```
INPUT:  "magnitude(dft([1, 2, 3]))"
          ↓
PARSE:   Build expression tree
          ↓
TYPE:    Infer Vector<Complex> → Vector<Number>
          ↓
EVAL:    Execute in WASM
          ↓
OUTPUT:  [10, 2.828, 2]
```

**No variables. No loops. No state. Just math.**

---

## 📝 Contributing

When adding features, ask:
- ✅ "Is this a mathematical operation?"
- ❌ "Is this a programming construct?"

If it's not math, it doesn't belong in Achronyme.

---

## 🔗 Related Documents

- **Mathematica Features:** [docs/MATHEMATICA_FEATURE_COMPLETE.md](../MATHEMATICA_FEATURE_COMPLETE.md)
- **WASM Architecture:** [docs/05-wasm-architecture.md](../05-wasm-architecture.md)
- **Sprint Planning:** [docs/sprints/01-wasm-foundation.md](../sprints/01-wasm-foundation.md)

---

**Next:** Read [grammar/02-syntax.md](grammar/02-syntax.md) for complete BNF grammar (expressions only).
