# Function Composition and Chaining - Mathematical Expressions

**Version:** 0.2.0 - SIMPLIFIED
**Status:** Draft
**Last Updated:** 2025-10-05

---

## Overview

This document defines how mathematical functions compose in Achronyme, including:
- Function composition rules (mathematical only)
- Type compatibility
- Automatic broadcasting
- Chain evaluation order
- Optimization opportunities

**Note:** This is for a **math engine**, not a programming language. No variables, no assignments, no control flow.

---

## 1. Basic Composition

### Definition

Function composition `f(g(x))` evaluates `g(x)` first, then applies `f` to the result.

```achronyme
# Basic composition
sin(cos(x))              # Evaluate cos(x), then sin of result

# Multiple levels
abs(sqrt(sin(x)))        # sin → sqrt → abs

# With operators
2 * sin(x) + 1           # sin(x) → multiply by 2 → add 1
```

### Evaluation Order

**Right-to-left for function calls:**

```achronyme
f(g(h(x)))
# 1. Evaluate h(x) → result1
# 2. Evaluate g(result1) → result2
# 3. Evaluate f(result2) → final result
```

**Left-to-right for operators (respecting precedence):**

```achronyme
2 + 3 * 4
# 1. 3 * 4 → 12
# 2. 2 + 12 → 14
```

---

## 2. Type-Based Composition Rules

### Rule 1: Scalar Functions on Collections (Broadcasting)

**When a scalar function receives a collection, apply element-wise:**

```achronyme
# Example: sin on vector
sin([0, PI/2, PI])       → [0, 1, 0]

# Example: sqrt on vector
sqrt([1, 4, 9, 16])      → [1, 2, 3, 4]

# Example: abs on complex vector
abs([3+4i, 5+12i])       → [5, 13]

# Example: Nested composition
sin(sqrt([0, 1, 4]))     → sin([0, 1, 2]) → [0, 0.841, 0.909]
```

**Type signature:**
```typescript
sin: Number → Number
sin: Vector<Number> → Vector<Number>  // Automatic broadcasting
sin: Matrix<Number> → Matrix<Number>  // Element-wise
```

### Rule 2: Collection Functions on Collections

**Collections return collections:**

```achronyme
# DFT returns complex vector
dft([1, 2, 3, 4])        → Vector<Complex>

# FFT returns complex vector
fft([1, 2, 3, 4, 5, 6, 7, 8])  → Vector<Complex>

# Eigenvalues return vector
eigenvalues([[1,2],[3,4]])  → Vector<Number>
```

### Rule 3: Reduction Functions

**Reduce collection to scalar:**

```achronyme
# Sum
sum([1, 2, 3, 4])        → 10

# Product
product([1, 2, 3, 4])    → 24

# Mean
mean([1, 2, 3, 4, 5])    → 3

# Norm
norm([3, 4])             → 5
```

---

## 3. DFT Composition Examples

### Your Specific Question: `1/2 * dft(...)`

```achronyme
# Case 1: Discrete DFT
result1 = dft([1, 2, 3, 4])
# Type: Vector<Complex>
# Value: [10+0i, -2+2i, -2+0i, -2-2i]

scaled1 = 1/2 * result1
# Type: Vector<Complex>
# Value: [5+0i, -1+1i, -1+0i, -1-1i]

# Composed:
1/2 * dft([1, 2, 3, 4])  → [5+0i, -1+1i, -1+0i, -1-1i]
```

```achronyme
# Case 2: Continuous DFT
result2 = dft(exp(-abs(t)), -5, 5, 100)
# Type: Vector<Complex>
# Value: [array of 100 complex numbers]

scaled2 = 1/2 * result2
# Type: Vector<Complex>
# Broadcasting: each element multiplied by 0.5
```

### Magnitude Composition: `magnitude(dft(...))`

```achronyme
# Step 1: DFT
dftResult = dft([1, 2, 3, 4])
# Type: Vector<Complex>

# Step 2: Magnitude (element-wise on vector)
magnitudes = magnitude(dftResult)
# Type: Vector<Number>
# Value: [10, 2.828, 2, 2.828]

# Composed:
magnitude(dft([1, 2, 3, 4]))  → [10, 2.828, 2, 2.828]
```

**Type signatures:**
```typescript
dft: Vector<Number> → Vector<Complex>
magnitude: Complex → Number
magnitude: Vector<Complex> → Vector<Number>  // Broadcasting
```

### Multi-level Composition

```achronyme
# Real part of DFT
real(dft([1, 2, 3, 4]))
# → [10, -2, -2, -2]

# Imaginary part
imag(dft([1, 2, 3, 4]))
# → [0, 2, 0, -2]

# Sum of magnitudes
sum(magnitude(dft([1, 2, 3, 4])))
# → 10 + 2.828 + 2 + 2.828 = 17.656

# Mean magnitude
mean(magnitude(dft([1, 2, 3, 4])))
# → 17.656 / 4 = 4.414

# Normalized DFT
normalized = dft([1, 2, 3, 4]) / sqrt(4)
# Each element divided by 2
```

---

## 4. Matrix Composition Examples

### Linear Algebra Chains

```achronyme
# Matrix inverse composition
inverse([[1, 2], [3, 4]] * [[5, 6], [7, 8]])
# Multiply first, then invert

# Composition: inverse(A) * inverse(B)
inverse([[1, 2], [3, 4]]) * inverse([[5, 6], [7, 8]])
# Mathematical property: equals inverse(B * A)

# Determinant of product
det([[1, 2], [3, 4]] * [[5, 6], [7, 8]])
# Equals det([[1, 2], [3, 4]]) * det([[5, 6], [7, 8]])
```

### Eigenvalue Chains

```achronyme
# Eigenvalues of inverse
eigenvalues(inverse([[1, 2], [3, 4]]))
# Should equal 1 / eigenvalues([[1, 2], [3, 4]])

# Trace
trace([[1, 2], [3, 4]] * [[5, 6], [7, 8]])
# Equals sum(eigenvalues([[1, 2], [3, 4]] * [[5, 6], [7, 8]]))
```

---

## 5. Operator Overloading by Type

### Scalar Operations

```achronyme
# Number + Number → Number
5 + 3                    → 8

# Complex + Complex → Complex
(3 + 4i) + (1 + 2i)      → 4 + 6i

# Number + Complex → Complex (auto-coercion)
5 + (3 + 4i)             → 8 + 4i
```

### Vector Operations

```achronyme
# Vector + Vector → Vector (element-wise)
[1, 2, 3] + [4, 5, 6]    → [5, 7, 9]

# Vector * Vector → Vector (element-wise, Hadamard product)
[1, 2, 3] * [4, 5, 6]    → [4, 10, 18]

# Scalar * Vector → Vector (broadcasting)
2 * [1, 2, 3]            → [2, 4, 6]

# Vector / Scalar → Vector
[2, 4, 6] / 2            → [1, 2, 3]
```

### Dot Product (Explicit)

```achronyme
# Use explicit function
dot([1, 2, 3], [4, 5, 6])  → 32

# Not: [1, 2, 3] * [4, 5, 6]  (that's Hadamard)
```

### Matrix Operations

```achronyme
# Matrix * Matrix → Matrix (matrix multiplication)
[[1, 2], [3, 4]] * [[5, 6], [7, 8]]
# → [[19, 22], [43, 50]]

# Matrix * Vector → Vector (matrix-vector product)
[[1, 2], [3, 4]] * [5, 6]
# → [17, 39]

# Element-wise: use hadamard() function
hadamard([[1, 2], [3, 4]], [[5, 6], [7, 8]])
# → [[5, 12], [21, 32]]
```

---

## 6. Pipe Operator (Future Feature)

### Proposed Syntax

```achronyme
# Traditional nested:
magnitude(dft([1, 2, 3, 4]))

# Pipe syntax (more readable):
[1, 2, 3, 4] |> dft |> magnitude

# Multi-line pipe:
[1, 2, 3, 4]
  |> dft
  |> magnitude
  |> sum
# → 17.656
```

### Benefits

- More readable for long chains
- Left-to-right reading (natural for composition)
- Easy to comment/debug individual steps

---

## 7. Automatic Optimization

### Optimization 1: Constant Folding

```achronyme
# User writes:
2 * PI * sin(0.5)

# Compiler optimizes:
# 1. 2 * PI → 6.283185307
# 2. sin(0.5) → 0.479425539
# 3. 6.283185307 * 0.479425539 → 3.012378481

# No runtime computation of 2 * PI
```

### Optimization 2: Common Subexpression Elimination

```achronyme
# Expression with repeated subexpressions:
sin(x) + cos(x) + sin(x) - cos(x)

# Compiler optimizes:
# temp1 = sin(x)
# temp2 = cos(x)
# result = temp1 + temp2 + temp1 - temp2

# sin(x) and cos(x) computed only once
```

**Note:** No variables in user code, but compiler can use temporaries internally for optimization.

---

## 8. Type Compatibility Matrix

| Operation | Number | Complex | Vector | Matrix | Function |
|-----------|--------|---------|--------|--------|----------|
| **Number +** | Number | Complex | Vector | Matrix | Error |
| **Complex +** | Complex | Complex | Vector<C> | Matrix<C> | Error |
| **Vector +** | Vector | Vector<C> | Vector | Error | Error |
| **Matrix +** | Matrix | Matrix<C> | Error | Matrix | Error |

| Operation | Number | Complex | Vector | Matrix |
|-----------|--------|---------|--------|--------|
| **Number *** | Number | Complex | Vector | Matrix |
| **Complex *** | Complex | Complex | Vector<C> | Matrix<C> |
| **Vector *** | Vector | Vector<C> | Vector (Hadamard) | Error |
| **Matrix *** | Matrix | Matrix<C> | Vector (M*v) | Matrix (M*M) |

---

## 9. Error Handling in Composition

### Type Mismatch Errors

```achronyme
# Invalid: Can't add matrix and number
[[1, 2], [3, 4]] + 5
# Error: Cannot broadcast scalar to matrix
# Suggestion: Use element-wise: map(x => x + 5, matrix)

# Invalid: Dimension mismatch
[1, 2, 3] + [4, 5]
# Error: Vector dimensions must match (3 vs 2)

# Invalid: Matrix multiplication dimensions
[[1, 2]] * [[3], [4], [5]]
# Error: Matrix dimensions incompatible (1x2) * (3x1)
```

### Domain Errors

```achronyme
# sqrt of negative (real mode)
sqrt(-4)
# Error: sqrt of negative number (use complex mode or abs())

# Division by zero
5 / 0
# Result: Infinity (not error, follows IEEE 754)

# log of non-positive
log(-5)
# Error: log of non-positive number
```

### Propagation

```achronyme
# NaN propagates through composition
sin(sqrt(-1))
# → NaN (sqrt returns NaN, sin(NaN) = NaN)

# Infinity propagates
exp(Infinity)
# → Infinity
```

---

## 10. Examples: Complex Compositions

### Example 1: Signal Processing Pipeline

```achronyme
# Direct composition (no variables):
magnitude(fft([1, 2, 3, 4, 5, 6, 7, 8] * hamming(8))) / max(magnitude(fft([1, 2, 3, 4, 5, 6, 7, 8] * hamming(8))))

# Better with pipe (future):
[1, 2, 3, 4, 5, 6, 7, 8]
  |> (* hamming(8))
  |> fft
  |> magnitude
  |> (/ max($))
```

**Note:** Host language (JavaScript) can use variables:
```javascript
// JavaScript wrapper
const signal = [1, 2, 3, 4, 5, 6, 7, 8]
const windowed = achronyme.eval(`${signal} * hamming(8)`)
const spectrum = achronyme.eval(`fft(${windowed})`)
// etc.
```

### Example 2: Linear System Solver

```achronyme
# Solve Ax = b using least squares (direct composition):
inverse(transpose([[1, 2], [3, 4], [5, 6]]) * [[1, 2], [3, 4], [5, 6]]) * transpose([[1, 2], [3, 4], [5, 6]]) * [7, 8, 9]

# Or use built-in:
leastSquares([[1, 2], [3, 4], [5, 6]], [7, 8, 9])

# Verify solution residual:
norm([[1, 2], [3, 4], [5, 6]] * leastSquares([[1, 2], [3, 4], [5, 6]], [7, 8, 9]) - [7, 8, 9])
```

### Example 3: Symbolic Calculus

```achronyme
# Symbolic differentiation:
diff(x^2 + 2*x, x)
# → 2*x + 2

# Symbolic integration:
integrate(sin(x), x)
# → -cos(x) + C

# Simplification:
simplify(sin(x)^2 + cos(x)^2)
# → 1
```

---

## Implementation Notes

### Type Inference

The compiler must infer types through composition:

```typescript
// User code:
magnitude(dft([1, 2, 3, 4]))

// Type inference:
[1, 2, 3, 4]          : Vector<Number>
dft(...)              : Vector<Complex>
magnitude(...)        : Vector<Number>
```

### Performance Considerations

- **Lazy evaluation:** Only compute what's needed
- **Fusion:** Combine multiple passes into one (e.g., `map(f, map(g, xs))` → `map(compose(f, g), xs)`)
- **Parallel execution:** Vector operations can use SIMD
- **Caching:** Memoize expensive pure functions

---

## See Also

- [Type System](../types/01-primitive-types.md)
- [Function Overloading](03-overloading.md)
- [Operator Precedence](../grammar/03-precedence.md)
- [DSP Functions](../functions/05-dsp.md)
