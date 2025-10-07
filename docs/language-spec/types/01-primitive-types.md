# Primitive Types - Mathematical Values Only

**Version:** 0.2.0 - SIMPLIFIED
**Status:** Draft
**Last Updated:** 2025-10-05

---

## Overview

Primitive type for Achronyme mathematical engine. **Only one primitive type:** Number. No Boolean, no String, no Undefined - those are programming constructs, not mathematical objects.

---

## Type Hierarchy

```
Value (abstract base)
└── Number
```

**Removed from full language spec:**
- ❌ Boolean (true/false) - Not a mathematical value
- ❌ String ("text") - Not a mathematical value
- ❌ Undefined - Functions always return values

---

## 1. Number

### Description
Represents real numbers (double-precision floating point).

### Internal Representation
- **WASM:** `f64` (IEEE 754 double precision)
- **Range:** ±1.7976931348623157E+308
- **Precision:** ~15-17 decimal digits

### Literals

```achronyme
# Integer notation
42
-17
0

# Decimal notation
3.14159
-0.5
2.71828
.5          # Same as 0.5
2.          # Same as 2.0

# Scientific notation
6.022e23        # Avogadro's number
1.6e-19         # Elementary charge
-4.5E-10

# Special values
Infinity        # Positive infinity
-Infinity       # Negative infinity
NaN             # Not a Number
```

### Operations

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Addition | `2 + 3` | `5` |
| `-` | Subtraction | `5 - 2` | `3` |
| `*` | Multiplication | `4 * 3` | `12` |
| `/` | Division | `10 / 4` | `2.5` |
| `^` | Exponentiation | `2 ^ 3` | `8` |
| `-` (unary) | Negation | `-5` | `-5` |
| `+` (unary) | Identity | `+5` | `5` |

**Note:** No modulo (`%`) operator - use `mod(a, b)` function if needed.

### Built-in Functions

```achronyme
# Basic operations
abs(-5)          → 5
sqrt(16)         → 4
floor(3.7)       → 3
ceil(3.2)        → 4
round(3.5)       → 4
sign(-5)         → -1
min(3, 5)        → 3
max(3, 5)        → 5

# Trigonometry
sin(PI/2)        → 1
cos(0)           → 1
tan(PI/4)        → 1

# Exponential/Logarithmic
exp(1)           → E
log(E)           → 1
log10(100)       → 2

# Power functions
pow(2, 8)        → 256
```

### Type Coercion

```achronyme
# Automatic coercion to Complex when needed
1 + 2i           → 1.0 + 2.0i

# Number to Vector (broadcasting)
2 * [1, 2, 3]    → [2, 4, 6]
```

**Removed:**
- ❌ String to Number (`parseNumber("42")`) - No strings
- ❌ Boolean to Number (`toNumber(true)`) - No booleans

### Special Behavior

```achronyme
# Division by zero
5 / 0            → Infinity
-5 / 0           → -Infinity
0 / 0            → NaN

# NaN propagation
NaN + 5          → NaN
NaN * 2          → NaN

# Infinity arithmetic
Infinity + 5     → Infinity
Infinity * 2     → Infinity
Infinity / Infinity → NaN
```

---

## Mathematical Constants

Built-in mathematical constants (all of type Number):

```achronyme
PI         → 3.141592653589793
E          → 2.718281828459045
PHI        → 1.618033988749895    # Golden ratio
```

**Note:** These are reserved identifiers in the lexer (see [01-lexical.md](../grammar/01-lexical.md)).

---

## Comparison Operations (Piecewise Only)

Comparison operators (`<`, `<=`, `>`, `>=`, `==`) are **NOT** general operators - they are **only** used in piecewise function conditions:

```achronyme
# ✓ Valid (inside piecewise)
{ t^2 if t < 0, exp(-t) if t >= 0 }

# ✗ Invalid (as standalone expression)
x < y               # NOT SUPPORTED

# ✗ Invalid (no boolean result)
result = (x == y)   # NOT SUPPORTED
```

**Why removed?** Comparisons return boolean values, which are programming constructs. Achronyme evaluates mathematical expressions only.

---

## Type Checking Functions

```achronyme
isNumber(42)             → 1 (true)
isNumber([1, 2])         → 0 (false)

isComplex(3+4i)          → 1
isVector([1, 2, 3])      → 1
isMatrix([[1,2],[3,4]])  → 1
```

**Note:** These return `1` (true) or `0` (false), not boolean values.

---

## What's NOT Here

### ❌ Removed Types (vs Full Programming Language)

```
# NO Boolean type
true, false         # Not supported

# NO String type
"hello"             # Not supported

# NO Undefined type
Undefined           # Not supported

# NO type conversion from strings
parseNumber("42")   # Not supported
toString(42)        # Not supported

# NO truthiness/falsy values
if x then ...       # Use piecewise instead
```

**Result:** 1 primitive type instead of 4

---

## Implementation Notes

### Precision Considerations

```achronyme
# Floating point arithmetic can have rounding errors
0.1 + 0.2                → 0.30000000000000004

# Use epsilon comparison for near-equality
abs((0.1 + 0.2) - 0.3) < 1e-10  → 1 (true)
```

### Performance

- **Number operations:** Single CPU instruction (native f64)
- **All operations are pure:** No side effects, easy to optimize

### Memory Layout

```
Number:    8 bytes (f64)
```

---

## Examples

### Basic Arithmetic

```achronyme
# Simple expressions
2 + 3 * 4           → 14
(2 + 3) * 4         → 20
2^3^2               → 512 (right-associative)

# With functions
sin(PI / 2)         → 1
exp(-abs(t))        → (depends on t)
```

### Special Values

```achronyme
# Infinity
x = 1 / 0           → Infinity
y = -1 / 0          → -Infinity

# NaN
z = 0 / 0           → NaN
isNaN(z)            → 1 (true)
```

### Piecewise with Comparisons

```achronyme
# Unit step function
{ 0 if t < 0, 1 if t >= 0 }

# Absolute value (manual)
{ -x if x < 0, x if x >= 0 }

# Sign function
{ -1 if x < 0, 0 if x == 0, 1 if x > 0 }
```

---

## Comparison: Before vs After

| Feature | Full Language | Math Engine | Removed |
|---------|--------------|-------------|---------|
| **Primitive Types** | 4 (Number, Boolean, String, Undefined) | 1 (Number only) | Boolean, String, Undefined |
| **Comparison Operators** | General expressions | Piecewise only | As boolean expressions |
| **Type Conversion** | 9 functions | 1 (to Complex) | parseNumber, toString, toBoolean |
| **Type Checking** | Returns Boolean | Returns 1/0 | Boolean type |
| **Complexity** | High | Low | 75% reduction |

---

## See Also

- [Complex Types](02-complex-types.md) - Complex number representation (a + bi)
- [Collections](03-collections.md) - Vector, Matrix
- [Lexical Analysis](../grammar/01-lexical.md) - Token definitions for constants
