# Advanced Composition Examples - Mathematical Expressions Only

**Version:** 0.2.0 - SIMPLIFIED
**Status:** Draft
**Last Updated:** 2025-10-05

---

## Overview

This document contains **real-world examples** of mathematical function composition in Achronyme, including edge cases, performance considerations, and best practices.

**Note:** Achronyme is a **math engine**, not a programming language. No variables, no loops, no assignments - only mathematical expressions.

---

## 1. Your Original Question: DFT Composition

### Q: Can you do `1/2 * dft(...)`?

**Answer: YES ✓**

```achronyme
# Direct expression (no variables):
1/2 * dft([1, 2, 3, 4])

# Type flow:
# [1, 2, 3, 4]       : Vector<Number>
# dft(...)           : Vector<Complex>
# 1/2                : Number
# 1/2 * Vector<C>    : Vector<Complex>  (broadcasts scalar)

# Actual values:
# dft([1,2,3,4]) = [10+0i, -2+2i, -2+0i, -2-2i]
# Result         = [5+0i, -1+1i, -1+0i, -1-1i]
```

### Q: Can you do `magnitude(dft(...))`?

**Answer: YES ✓**

```achronyme
# Direct expression (no variables):
magnitude(dft([1, 2, 3, 4]))

# Type flow:
# dft(...)           : Vector<Complex>
# magnitude(Vector)  : Vector<Number>  (element-wise)

# Actual values:
# dft([1,2,3,4])     = [10+0i, -2+2i, -2+0i, -2-2i]
# Result             = [10, 2.828, 2, 2.828]
```

### Q: Can you chain multiple operations?

**Answer: YES ✓**

```achronyme
# Direct expression (no variables):
sum(magnitude(dft([1, 2, 3, 4]))) / 4

# Step by step:
# 1. dft([1,2,3,4])           → [10+0i, -2+2i, -2+0i, -2-2i]
# 2. magnitude(...)           → [10, 2.828, 2, 2.828]
# 3. sum(...)                 → 17.656
# 4. sum(...) / 4             → 4.414
```

---

## 2. DSP Signal Processing Chains

### Example 1: Complete Fourier Analysis

```achronyme
# Compute Fourier Transform of piecewise function:
dft({ exp(-abs(t)) * sin(2*PI*t) if -10 < t < 10, 0 if t >= 10 }, -10, 10, 1000)
# Type: Vector<Complex>
# 1000 frequency points from -10 to 10 rad/s

# Extract magnitude spectrum:
magnitude(dft({ exp(-abs(t)) * sin(2*PI*t) if -10 < t < 10, 0 if t >= 10 }, -10, 10, 1000))
# Type: Vector<Number>

# Extract phase:
phase(dft({ exp(-abs(t)) * sin(2*PI*t) if -10 < t < 10, 0 if t >= 10 }, -10, 10, 1000))
# Type: Vector<Number>

# Energy spectral density:
sum(magnitude(dft({ exp(-abs(t)) * sin(2*PI*t) if -10 < t < 10, 0 if t >= 10 }, -10, 10, 1000))^2) * (20 / 1000)
# Type: Number

# Normalized spectrum:
dft({ exp(-abs(t)) * sin(2*PI*t) if -10 < t < 10, 0 if t >= 10 }, -10, 10, 1000) / max(magnitude(dft({ exp(-abs(t)) * sin(2*PI*t) if -10 < t < 10, 0 if t >= 10 }, -10, 10, 1000)))
# Type: Vector<Complex>
```

**Note:** Long expressions become verbose without variables. Use host language for intermediate results or wait for pipe operator (`|>`).

### Example 2: Filtering Pipeline

```achronyme
# Original signal (noisy)
signal = [1.2, 2.1, 2.9, 4.2, 5.1, 3.8, 2.9, 1.1]

# Apply windowing
window = hamming(length(signal))
windowed = signal * window

# Compute FFT
freq_domain = fft(windowed)

# Apply low-pass filter (keep DC to 3 bins)
filtered_freq = freq_domain * [1, 1, 1, 0, 0, 0, 0, 0]

# Inverse FFT
filtered_signal = real(ifft(filtered_freq))

# Composed:
filtered = real(ifft(
    fft(signal * hamming(length(signal))) *
    lowpass_mask(length(signal), 3)
))
```

### Example 3: Cross-Correlation

```achronyme
# Two signals
sig1 = [1, 2, 3, 4, 3, 2, 1]
sig2 = [0, 1, 2, 3, 2, 1, 0]

# Cross-correlation via FFT (faster)
xcorr = real(ifft(
    fft(sig1) * conjugate(fft(sig2))
))

# Find peak (time delay)
delay = argmax(xcorr)
```

---

## 3. Linear Algebra Compositions

### Example 1: Least Squares Regression

```achronyme
# Data points
X = [[1, 1], [1, 2], [1, 3], [1, 4], [1, 5]]  # Design matrix
y = [2.1, 3.9, 6.2, 8.1, 9.8]                 # Observations

# Normal equation: β = (X'X)^(-1) X'y
beta = inverse(transpose(X) * X) * transpose(X) * y

# Predicted values
y_pred = X * beta

# Residuals
residuals = y - y_pred

# R-squared
ss_tot = sum((y - mean(y))^2)
ss_res = sum(residuals^2)
r_squared = 1 - ss_res / ss_tot
```

### Example 2: Eigenvalue Decomposition

```achronyme
# Symmetric matrix
A = [[4, 1, 0],
     [1, 3, 1],
     [0, 1, 2]]

# Eigendecomposition: A = V Λ V'
eigenvals = eigenvalues(A)
eigenvecs = eigenvectors(A)

# Reconstruct matrix
Lambda = diag(eigenvals)
reconstructed = eigenvecs * Lambda * transpose(eigenvecs)

# Verify
error = norm(A - reconstructed)  # Should be ~0
```

### Example 3: Matrix Power via Diagonalization

```achronyme
# Compute A^100 efficiently
A = [[0.8, 0.3],
     [0.2, 0.7]]

# Method 1: Slow (iterative)
result_slow = A^100  # O(n^3 * 100)

# Method 2: Fast (eigendecomposition)
V = eigenvectors(A)
D = diag(eigenvalues(A))
result_fast = V * (D^100) * inverse(V)  # O(n^3)

# For large matrices, this is 100x faster
```

---

## 4. Calculus Compositions

### Example 1: Numerical Integration

```achronyme
# Integrate sin(x^2) from 0 to PI
f(x) = sin(x^2)
result = integrate(f(x), x, 0, PI)

# With adaptive resolution
result_adaptive = integrate(f(x), x, 0, PI, method="adaptive", tol=1e-10)
```

### Example 2: Derivative Verification

```achronyme
# Define function
f(x) = exp(x) * sin(x)

# Symbolic derivative
df = diff(f(x), x)
# → exp(x)*sin(x) + exp(x)*cos(x)

# Verify at x=1
x_val = 1
analytic = exp(x_val) * (sin(x_val) + cos(x_val))
numeric = (f(x_val + 1e-8) - f(x_val)) / 1e-8

error = abs(analytic - numeric)  # Should be ~1e-8
```

### Example 3: Finding Minimum (Symbolic)

```achronyme
# Find minimum of f(x) = x^2 + 4*x + 3
diff(x^2 + 4*x + 3, x)
# → 2*x + 4

# Solve for critical point: 2*x + 4 = 0
solve(2*x + 4 == 0, x)
# → x = -2

# Verify it's a minimum (second derivative):
diff(diff(x^2 + 4*x + 3, x), x)
# → 2 (positive, so it's a minimum)
```

**Note:** Numerical optimization (gradient descent) would be done in host language (JavaScript/Python) using Achronyme for gradient evaluation.

---

## 5. Piecewise Function Compositions

### Example 1: Fourier Transform of Piecewise Function

```achronyme
# Define piecewise function
f(t) = {
    t^2          if t < 0,
    exp(-t)      if 0 <= t < 2,
    0            if t >= 2
}

# Compute DFT
spectrum = dft(f(t), -5, 5, 500)

# Extract magnitude
mag_spectrum = magnitude(spectrum)

# Find dominant frequency
dominant_freq = omega[argmax(mag_spectrum)]
```

### Example 2: Heaviside Step Compositions

```achronyme
# Rectangular pulse
pulse(t) = heaviside(t) - heaviside(t - 1)

# Fourier transform of pulse (sinc function)
spectrum = dft(pulse(t), -10, 10, 1000)

# Verify sinc shape
# F{pulse}(ω) = (e^(-iω) - 1) / (iω)
```

---

## 6. Optimization Compositions

### Example 1: Linear Programming with Constraints

```achronyme
# Maximize: 3x + 4y
# Subject to:
#   2x + y <= 10
#   x + 3y <= 12
#   x, y >= 0

result = maximize(
    3*x + 4*y,
    constraints = [
        2*x + y <= 10,
        x + 3*y <= 12,
        x >= 0,
        y >= 0
    ]
)

# result = {x: 2, y: 6, value: 30}
```

### Example 2: Nonlinear Optimization

```achronyme
# Minimize: (x-3)^2 + (y-4)^2
# Subject to: x + y >= 5

result = minimize(
    (x - 3)^2 + (y - 4)^2,
    constraints = [x + y >= 5]
)

# Lagrange multiplier method
L(x, y, λ) = (x-3)^2 + (y-4)^2 - λ*(x + y - 5)
```

---

## 7. Edge Cases and Gotchas

### Case 1: Dimension Mismatch

```achronyme
# ERROR: Vector length mismatch
[1, 2, 3] + [4, 5]
# Error: Cannot add vectors of different lengths (3 vs 2)

# FIX: Pad or truncate
[1, 2, 3] + [4, 5, 0]  # ✓ OK
```

### Case 2: Type Confusion

```achronyme
# AMBIGUOUS: What is [1, 2] * [3, 4]?

# In Achronyme: Element-wise (Hadamard)
[1, 2] * [3, 4]  → [3, 8]

# For dot product, use explicit function
dot([1, 2], [3, 4])  → 11
```

### Case 3: DFT on Empty or Small Arrays

```achronyme
# Empty array
dft([])
# Error: DFT requires at least 1 element

# Single element
dft([5])
# → [5+0i]

# Two elements
dft([1, 2])
# → [3+0i, -1+0i]
```

### Case 4: Division by Zero in Vector

```achronyme
# Vector with zero element
v = [1, 2, 0, 4]

# Division
v / 0
# → [Infinity, Infinity, NaN, Infinity]

# Safe division
v / (v + 1e-10)
# → [0.999..., 1.999..., 0, 3.999...]
```

---

## 8. Performance Comparisons

### Benchmark 1: DFT Discrete vs Continuous

```achronyme
# Discrete: Fast (O(n log n) with FFT)
discrete_result = dft([...1000 samples...])
# Time: ~1ms

# Continuous: Slower (numerical integration)
continuous_result = dft(f(t), -10, 10, 1000)
# Time: ~500ms (1000 frequencies × 1000 integration points)
```

### Benchmark 2: Matrix Operations

```achronyme
# Small matrices: Direct computation
A = [[1, 2], [3, 4]]
inv_A = inverse(A)
# Time: <1μs

# Large matrices: Optimized algorithms
B = random_matrix(1000, 1000)
inv_B = inverse(B)
# Time: ~100ms (LU decomposition)
```

---

## 9. Best Practices

### Practice 1: Avoid Redundant Computation

```achronyme
# INEFFICIENT: Computes DFT twice
magnitude(dft([1,2,3,4])) / max(magnitude(dft([1,2,3,4])))

# BETTER: Use host language to cache
```

**Note:** Since Achronyme has no variables, use the host language (JavaScript/Python) to cache expensive computations:

```javascript
// JavaScript
const spectrum = achronyme.eval('dft([1,2,3,4])')
const mags = achronyme.eval(`magnitude(${spectrum})`)
const result = achronyme.eval(`${mags} / max(${mags})`)
```

### Practice 2: Use Specialized Functions

```achronyme
# SLOW: Naive matrix power
[[0.8, 0.3], [0.2, 0.7]]^100

# FAST: Specialized algorithm
matrixPower([[0.8, 0.3], [0.2, 0.7]], 100)
```

### Practice 3: Leverage Broadcasting

```achronyme
# GOOD: Automatic vectorization
sin([1, 2, 3, 4])
# Applies sin element-wise automatically
```

**Note:** No loops needed - broadcasting is automatic!

---

## 10. Future Pipe Operator Examples

```achronyme
# Traditional nested (hard to read)
result = sum(magnitude(dft(signal * hamming(length(signal)))))

# With pipe (easy to read)
result = signal
    |> (* hamming(length($)))
    |> dft
    |> magnitude
    |> sum

# Multi-step analysis
analysis = raw_data
    |> removeOutliers
    |> normalize
    |> fft
    |> magnitude
    |> (/ max($))
    |> plot
```

---

## Summary Table

| Composition Pattern | Valid? | Type Result | Example |
|---------------------|--------|-------------|---------|
| `scalar * dft(...)` | ✓ Yes | Vector<Complex> | `1/2 * dft([1,2,3])` |
| `magnitude(dft(...))` | ✓ Yes | Vector<Number> | `magnitude(dft([1,2,3]))` |
| `sum(magnitude(...))` | ✓ Yes | Number | `sum(magnitude(dft(...)))` |
| `dft(...) + dft(...)` | ✓ Yes | Vector<Complex> | Element-wise addition |
| `inverse(A * B)` | ✓ Yes | Matrix | `inverse([[1,2],[3,4]] * B)` |
| `diff(integrate(...))` | ✓ Yes | Expression | Should cancel out |
| `[1,2] * [3,4]` | ⚠️ Ambiguous | Vector | Hadamard (not dot!) |
| `[1,2,3] + [4,5]` | ✗ Error | - | Dimension mismatch |

---

## See Also

- [Type System](../types/01-primitive-types.md)
- [Composition Semantics](../semantics/02-composition.md)
- [DSP Functions](../functions/05-dsp.md)
- [Linear Algebra](../functions/04-linear-algebra.md)
