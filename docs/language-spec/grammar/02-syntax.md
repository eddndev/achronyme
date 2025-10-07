# Syntax Grammar - Mathematical Expressions Only

**Version:** 0.2.0 - SIMPLIFIED
**Status:** Draft
**Last Updated:** 2025-10-05

---

## Overview

Complete **BNF grammar** for Achronyme mathematical expressions. This is **NOT** a full programming language - only mathematical evaluation.

### Notation

```
Rule:           Name followed by colon
|               Alternatives (OR)
( )             Grouping
*               Zero or more
+               One or more
?               Optional (zero or one)
UPPERCASE       Terminals (tokens)
PascalCase      Nonterminals
```

---

## Complete Grammar

```bnf
# ============================================
# TOP LEVEL: Just evaluate an expression
# ============================================

Program:
    Expression

# ============================================
# EXPRESSIONS (by precedence, lowest to highest)
# ============================================

Expression:
    AdditiveExpression

AdditiveExpression:
    MultiplicativeExpression
    | AdditiveExpression PLUS MultiplicativeExpression
    | AdditiveExpression MINUS MultiplicativeExpression

MultiplicativeExpression:
    UnaryExpression
    | MultiplicativeExpression STAR UnaryExpression
    | MultiplicativeExpression SLASH UnaryExpression

UnaryExpression:
    PowerExpression
    | PLUS UnaryExpression
    | MINUS UnaryExpression

PowerExpression:
    PostfixExpression
    | PostfixExpression CARET PowerExpression

PostfixExpression:
    PrimaryExpression
    | PostfixExpression LEFT_BRACKET Expression RIGHT_BRACKET
    | PostfixExpression LEFT_PAREN ArgumentList? RIGHT_PAREN

PrimaryExpression:
    NUMBER
    | IDENTIFIER
    | LEFT_PAREN Expression RIGHT_PAREN
    | ArrayLiteral
    | MatrixLiteral
    | PiecewiseExpression

# ============================================
# COLLECTIONS
# ============================================

ArrayLiteral:
    LEFT_BRACKET ElementList? RIGHT_BRACKET

ElementList:
    Expression
    | ElementList COMMA Expression

MatrixLiteral:
    LEFT_BRACKET RowList RIGHT_BRACKET

RowList:
    ArrayLiteral
    | RowList COMMA ArrayLiteral

# ============================================
# FUNCTION CALLS
# ============================================

ArgumentList:
    Expression
    | ArgumentList COMMA Expression

# ============================================
# PIECEWISE FUNCTIONS
# ============================================

PiecewiseExpression:
    LEFT_BRACE PiecewiseList RIGHT_BRACE

PiecewiseList:
    PiecewisePiece
    | PiecewiseList COMMA PiecewisePiece

PiecewisePiece:
    Expression IF Condition

Condition:
    Expression LESS Expression
    | Expression LESS_EQUAL Expression
    | Expression GREATER Expression
    | Expression GREATER_EQUAL Expression
    | Expression EQUAL Expression

# ============================================
# END OF GRAMMAR
# ============================================
```

---

## Examples with Parse Trees

### Example 1: Simple Arithmetic

```achronyme
2 + 3 * 4
```

**Parse Tree:**
```
AdditiveExpression
├─ MultiplicativeExpression
│  └─ NUMBER(2)
├─ PLUS
└─ MultiplicativeExpression
   ├─ NUMBER(3)
   ├─ STAR
   └─ NUMBER(4)

Result: 14
```

### Example 2: Power Expression

```achronyme
2^3^2
```

**Parse Tree:**
```
PowerExpression
├─ NUMBER(2)
├─ CARET
└─ PowerExpression
   ├─ NUMBER(3)
   ├─ CARET
   └─ NUMBER(2)

Result: 2^(3^2) = 512 (right-associative)
```

### Example 3: Function Call

```achronyme
sin(PI / 2)
```

**Parse Tree:**
```
PostfixExpression
├─ IDENTIFIER(sin)
├─ LEFT_PAREN
├─ ArgumentList
│  └─ MultiplicativeExpression
│     ├─ IDENTIFIER(PI)
│     ├─ SLASH
│     └─ NUMBER(2)
└─ RIGHT_PAREN

Result: 1
```

### Example 4: DFT Composition

```achronyme
magnitude(dft([1, 2, 3, 4]))
```

**Parse Tree:**
```
PostfixExpression
├─ IDENTIFIER(magnitude)
├─ LEFT_PAREN
├─ ArgumentList
│  └─ PostfixExpression
│     ├─ IDENTIFIER(dft)
│     ├─ LEFT_PAREN
│     ├─ ArgumentList
│     │  └─ ArrayLiteral
│     │     ├─ LEFT_BRACKET
│     │     ├─ NUMBER(1), NUMBER(2), NUMBER(3), NUMBER(4)
│     │     └─ RIGHT_BRACKET
│     └─ RIGHT_PAREN
└─ RIGHT_PAREN

Result: [10, 2.828, 2, 2.828]
```

### Example 5: Vector Operations

```achronyme
1/2 * dft([1, 2, 3])
```

**Parse Tree:**
```
MultiplicativeExpression
├─ MultiplicativeExpression
│  ├─ NUMBER(1)
│  ├─ SLASH
│  └─ NUMBER(2)
├─ STAR
└─ PostfixExpression
   ├─ IDENTIFIER(dft)
   ├─ LEFT_PAREN
   ├─ ArrayLiteral([1, 2, 3])
   └─ RIGHT_PAREN

Result: Vector<Complex>
```

### Example 6: Matrix Literal

```achronyme
[[1, 2], [3, 4]]
```

**Parse Tree:**
```
MatrixLiteral
├─ LEFT_BRACKET
├─ ArrayLiteral([1, 2])
├─ COMMA
├─ ArrayLiteral([3, 4])
└─ RIGHT_BRACKET

Result: Matrix 2x2
```

### Example 7: Piecewise Function

```achronyme
f(t) = { t^2 if t < 0, exp(-t) if t >= 0 }
```

**Parse Tree:**
```
PiecewiseExpression
├─ LEFT_BRACE
├─ PiecewisePiece
│  ├─ PowerExpression(t^2)
│  ├─ IF
│  └─ Condition(t < 0)
├─ COMMA
├─ PiecewisePiece
│  ├─ FunctionCall(exp(-t))
│  ├─ IF
│  └─ Condition(t >= 0)
└─ RIGHT_BRACE

Usage: dft(f(t), -5, 5, 500)
```

---

## Precedence Summary

| Level | Operator | Associativity | Example |
|-------|----------|---------------|---------|
| 1 (high) | `()` `[]` | Left | `f(x)` `a[0]` |
| 2 | `^` | **Right** | `2^3^2 = 512` |
| 3 | `+` `-` (unary) | Right | `-5` |
| 4 | `*` `/` | Left | `2*3/4` |
| 5 (low) | `+` `-` | Left | `2+3-4` |

**Note:** No assignment operators, no logical operators, no comparison operators (except in piecewise conditions).

---

## What's NOT in This Grammar

### ❌ Removed (Too Complex for Math Engine)

```bnf
# NO assignments
x = 5                       # NOT SUPPORTED

# NO control flow
if x > 0 then { ... }       # NOT SUPPORTED
for i in 1..10 do { ... }   # NOT SUPPORTED
while x > 0 do { ... }      # NOT SUPPORTED

# NO mutable variables
x += 1                      # NOT SUPPORTED

# NO boolean logic
true && false               # NOT SUPPORTED
!condition                  # NOT SUPPORTED

# NO strings
"hello" + "world"           # NOT SUPPORTED

# NO function declarations with bodies
function f(x) { ... }       # NOT SUPPORTED
```

### ✅ What's Supported Instead

```achronyme
# Just expressions
2 + 3 * 4                           ✓
sin(PI / 2)                         ✓

# Function calls
dft([1, 2, 3])                      ✓
magnitude(dft([1, 2, 3]))           ✓

# Piecewise (only for math functions)
f(t) = { t^2 if t < 0, ... }        ✓

# Symbolic variables (not assignments)
x, y, t                             ✓

# Collections
[1, 2, 3]                           ✓
[[1, 2], [3, 4]]                    ✓
```

---

## Grammar Properties

### Left-Recursive (for left-associativity)

```bnf
AdditiveExpression:
    MultiplicativeExpression
    | AdditiveExpression PLUS MultiplicativeExpression
    # ↑ Left-recursive: a + b + c = (a + b) + c
```

### Right-Recursive (for right-associativity)

```bnf
PowerExpression:
    PostfixExpression
    | PostfixExpression CARET PowerExpression
    # ↑ Right-recursive: a^b^c = a^(b^c)
```

### Parsing Strategy

This grammar is:
- ✅ **LR(1)** parseable (handles left-recursion)
- ✅ **LL(k)** parseable after left-recursion elimination
- ✅ **Precedence climbing** parseable (recommended)

---

## Implementation Notes

### Precedence Climbing Algorithm

```typescript
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

### Token Precedence Values

```typescript
const PRECEDENCE = {
    '+': 1, '-': 1,              // Lowest
    '*': 2, '/': 2,
    '^': 3,                      // Highest (right-associative)
    'unary': 4,
    'call': 5,
}
```

---

## Valid Programs

```achronyme
# All valid programs:
42
2 + 3
sin(x)
dft([1, 2, 3])
magnitude(dft([1, 2, 3]))
[[1, 2], [3, 4]]
{ x^2 if x < 0, x if x >= 0 }
```

---

## See Also

- [Lexical Analysis](01-lexical.md) - Token definitions (~30 tokens)
- [Operator Precedence](03-precedence.md) - Detailed precedence table
- [Composition Semantics](../semantics/02-composition.md) - How functions compose
- [Examples](../examples/07-composition.md) - Real-world use cases
