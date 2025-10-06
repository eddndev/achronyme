# Operator Precedence and Associativity - Mathematical Expressions Only

**Version:** 0.2.0 - SIMPLIFIED
**Status:** Draft
**Last Updated:** 2025-10-05

---

## Overview

Defines the **precedence** (order of operations) and **associativity** (left-to-right or right-to-left) for mathematical operators in Achronyme. **Much simpler** than full programming languages - only 5 precedence levels.

---

## Precedence Table

**Higher precedence = evaluated first**

| Level | Operator | Description | Associativity | Example |
|-------|----------|-------------|---------------|---------|
| **1** | `()` `[]` | Grouping, function calls, indexing | Left-to-right | `sin(x)` `a[0]` |
| **2** | `^` | Exponentiation | **Right-to-left** | `2^3^2` = `2^(3^2)` |
| **3** | `+` `-` (unary) | Unary plus, minus | Right-to-left | `-5` `+3` |
| **4** | `*` `/` | Multiplication, division | Left-to-right | `2 * 3 / 4` |
| **5** | `+` `-` (binary) | Addition, subtraction | Left-to-right | `2 + 3 - 4` |

**Note:** Comparison operators (`<`, `<=`, `>`, `>=`, `==`) are ONLY used in piecewise function conditions, not as general expressions.

**Removed (not needed for math):**
- ❌ Logical operators (`&&`, `||`, `!`)
- ❌ Assignment operators (`=`, `+=`, `-=`)
- ❌ Ternary conditional (`? :`)
- ❌ Modulo (`%`)
- ❌ Member access (`.`)

---

## Detailed Examples

### Level 1: Highest Precedence (Grouping)

```achronyme
# Function calls
sin(x)              # Always evaluated as sin(x), not (sin)(x)
dft([1, 2, 3])      # Function call before any arithmetic

# Indexing
matrix[1][2]        # Left-to-right: (matrix[1])[2]

# Parentheses override all
(2 + 3) * 4         # 20, not 14
```

### Level 2: Exponentiation (Right-Associative)

```achronyme
# Right-to-left
2^3^2               # Parsed as: 2^(3^2) = 2^9 = 512
                    # NOT: (2^3)^2 = 8^2 = 64

# Explicit grouping
(2^3)^2             # 64

# Mixed with unary
-2^2                # Parsed as: -(2^2) = -4
(-2)^2              # 4
```

### Level 3: Unary Operators

```achronyme
# Unary minus
-5 + 3              # Parsed as: (-5) + 3 = -2

# Unary plus
+5                  # Parsed as: (+5) = 5

# Multiple unary
--5                 # Parsed as: -(-5) = 5
-+5                 # Parsed as: -(+5) = -5
```

### Level 4: Multiplicative Operators

```achronyme
# Left-to-right
2 * 3 / 4           # Parsed as: (2 * 3) / 4 = 1.5
                    # NOT: 2 * (3 / 4) = 1.5 (same result but different tree)

6 / 2 * 3           # Parsed as: (6 / 2) * 3 = 9

# With vectors (broadcasting)
2 * [1, 2, 3]       # [2, 4, 6]
```

### Level 5: Additive Operators

```achronyme
# Left-to-right
2 + 3 - 4           # Parsed as: (2 + 3) - 4 = 1

5 - 2 + 3           # Parsed as: (5 - 2) + 3 = 6

# Mixed with multiplication (higher precedence)
2 + 3 * 4           # Parsed as: 2 + (3 * 4) = 14
```

### Comparison Operators (Piecewise Only)

**Note:** Comparison operators (`<`, `<=`, `>`, `>=`, `==`) are **NOT** general operators in Achronyme. They are **only** used inside piecewise function conditions.

```achronyme
# ✓ Valid (inside piecewise)
{ t^2 if t < 0, exp(-t) if t >= 0 }

# ✗ Invalid (as standalone expression)
x < y               # NOT SUPPORTED as general boolean expression
```

**Why removed?** Achronyme is a math engine, not a logic engine. Use JavaScript/host language for conditionals.

---

## Complex Examples

### Example 1: Mixed Operators

```achronyme
2 + 3 * 4^2
```

**Parse order:**
1. `4^2` → 16 (exponentiation, highest)
2. `3 * 16` → 48 (multiplication)
3. `2 + 48` → 50 (addition)

**Parse tree:**
```
    +
   / \
  2   *
     / \
    3   ^
       / \
      4   2
```

### Example 2: DFT Expression

```achronyme
1/2 * dft([1, 2, 3, 4])
```

**Parse order:**
1. `[1, 2, 3, 4]` → Array literal
2. `dft(...)` → Function call (highest precedence)
3. `1/2` → 0.5 (division, left-to-right)
4. `0.5 * result` → Multiplication

**Parse tree:**
```
       *
      / \
     /   dft
    / \    |
   1   2   [1,2,3,4]
```

### Example 3: Composition Chain

```achronyme
magnitude(dft([1, 2, 3])) / sum(magnitude(dft([1, 2, 3])))
```

**Parse order:**
1. Both `[1, 2, 3]` array literals
2. Both `dft(...)` function calls
3. Both `magnitude(...)` function calls
4. `sum(...)` function call
5. `/` division

**Type flow:**
```
[1, 2, 3]                    → Vector<Number>
dft([1, 2, 3])               → Vector<Complex>
magnitude(dft(...))          → Vector<Number>
sum(magnitude(dft(...)))     → Number
magnitude(...) / sum(...)    → Vector<Number> (broadcasting)
```

### Example 4: Piecewise with Comparisons

```achronyme
{ t^2 if t < 0, exp(-t) if t >= 0 }
```

**Parse order:**
1. `t^2` → power expression
2. `exp(-t)` → function call
3. `t < 0` → comparison (inside piecewise condition)
4. `t >= 0` → comparison (inside piecewise condition)
5. Assemble piecewise expression

**Note:** Comparisons are ONLY valid inside piecewise `if` conditions.

---

## Associativity Rules

### Left-Associative (Most Operators)

Operators at the same precedence level evaluate **left-to-right**:

```achronyme
# Subtraction (left-associative)
10 - 5 - 2          # (10 - 5) - 2 = 3
                    # NOT: 10 - (5 - 2) = 7

# Division (left-associative)
100 / 10 / 2        # (100 / 10) / 2 = 5
                    # NOT: 100 / (10 / 2) = 20
```

### Right-Associative (Special Cases)

Only **two** operator types are right-associative:

#### 1. Exponentiation (`^`)

```achronyme
2^3^2               # 2^(3^2) = 512
                    # Mathematical convention
```

#### 2. Unary Operators (`+`, `-`)

```achronyme
--5                 # -(-5) = 5
-+5                 # -(+5) = -5
+-5                 # +(-5) = -5
```

**Removed (not needed for math):**
- ❌ Assignment operators (no mutable variables)
- ❌ Ternary conditional (use piecewise instead)
- ❌ Unary NOT (`!`) (no boolean logic)

---

## Comparison with Other Languages

### Python

```python
# Python
2 ** 3 ** 2         # 512 (right-associative, like Achronyme)
10 - 5 - 2          # 3 (left-associative, like Achronyme)
```

### JavaScript

```javascript
// JavaScript
2 ** 3 ** 2         // 512 (right-associative)
10 - 5 - 2          // 3 (left-associative)
```

### Mathematica

```mathematica
(* Mathematica *)
2^3^2               (* 512 - right-associative *)
10 - 5 - 2          (* 3 - left-associative *)
```

**Achronyme follows the same conventions** as these languages.

---

## Precedence Override with Parentheses

Always use **parentheses** to override default precedence:

```achronyme
# Default precedence
2 + 3 * 4           # 14

# Forced precedence
(2 + 3) * 4         # 20

# Complex override
(2 + 3) * (4 + 5)   # 45

# Nested parentheses
((2 + 3) * 4) + 5   # 25
```

---

## Common Mistakes

### Mistake 1: Exponentiation Associativity

```achronyme
# WRONG mental model
2^3^2 = (2^3)^2 = 64    # ✗ INCORRECT

# CORRECT
2^3^2 = 2^(3^2) = 512   # ✓ CORRECT
```

### Mistake 2: Unary Minus and Exponentiation

```achronyme
# Ambiguous
-2^2                # Could be (-2)^2 = 4 or -(2^2) = -4

# Achronyme: Exponentiation has higher precedence
-2^2 = -(2^2) = -4  # ✓ This is what happens

# To get 4, use parentheses
(-2)^2 = 4          # ✓ Explicit
```

### Mistake 3: Using Comparisons Outside Piecewise

```achronyme
# ✗ WRONG (comparisons not general expressions)
result = x < 10     # NOT SUPPORTED

# ✓ CORRECT (use in piecewise)
{ 0 if x < 10, 1 if x >= 10 }

# ✓ CORRECT (use host language for logic)
// JavaScript
const result = achronyme.eval("x") < 10 ? 0 : 1
```

---

## Implementation Notes

### Parser Implementation

```typescript
// Precedence climbing method
function parseExpression(minPrecedence: number): ASTNode {
    let left = parsePrimary()

    while (currentToken.precedence >= minPrecedence) {
        const op = currentToken
        advance()

        const right = parseExpression(
            op.precedence + (op.isLeftAssociative ? 1 : 0)
        )

        left = new BinaryOp(op, left, right)
    }

    return left
}
```

### Precedence Values (Implementation)

```typescript
const PRECEDENCE = {
    '+': 1, '-': 1,              // Additive (lowest)
    '*': 2, '/': 2,              // Multiplicative
    '^': 3,                      // Exponentiation (right-associative)
    'unary': 4,                  // Unary +, -
    'call': 5,                   // Function calls, indexing (highest)
}

// Comparison operators (<, <=, >, >=, ==) only in piecewise conditions
// No logical operators (&&, ||, !)
// No assignment operators (=, +=, -=)
```

---

## See Also

- [Syntax Grammar](02-syntax.md) - Complete BNF grammar
- [Lexical Analysis](01-lexical.md) - Token definitions
- [Composition Semantics](../semantics/02-composition.md) - Execution model
