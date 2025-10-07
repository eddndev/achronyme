# Lexical Analysis - Mathematical Tokens Only

**Version:** 0.2.0 - SIMPLIFIED
**Status:** Draft
**Last Updated:** 2025-10-05

---

## Overview

Token specification for Achronyme mathematical expression evaluator. **~30 token types** (vs 50+ in full programming languages).

---

## 1. Keywords and Constants

### Mathematical Constants

```
PI          → 3.141592653589793
E           → 2.718281828459045
PHI         → 1.618033988749895
```

### Special Values

```
Infinity    → ∞
-Infinity   → -∞
NaN         → Not a Number
```

### Reserved Keywords (Minimal)

```
if          # Only for piecewise conditions
```

**Note:** No `then`, `else`, `for`, `while`, `function`, etc. - Not needed for math expressions.

---

## 2. Identifiers

### Pattern

```
Identifier: Letter (Letter | Digit | Underscore)*

Letter:     a..z | A..Z
Digit:      0..9
Underscore: _
```

### Examples

```
x
y
t
omega
sin
dft
magnitude
inverse
```

### Case Sensitivity

- `sin` ≠ `Sin` ≠ `SIN`
- `PI` ≠ `pi`

---

## 3. Numbers

### Integer Literals

```
42
0
1000
```

### Decimal Literals

```
3.14
0.5
.5          # Same as 0.5
2.          # Same as 2.0
```

### Scientific Notation

```
1e10
1.5e-8
6.022e23
-4.5E-10
```

**Pattern:**
```
Number: Digit+ (. Digit+)? ((e | E) (+ | -)? Digit+)?
```

---

## 4. Operators

### Arithmetic Operators

```
+       Plus
-       Minus
*       Star (multiplication)
/       Slash (division)
^       Caret (exponentiation)
```

### Comparison Operators (Piecewise Only)

```
<       Less
<=      LessEqual
>       Greater
>=      GreaterEqual
==      Equal
```

**Note:** These are ONLY used in piecewise function conditions, not for general boolean logic.

---

## 5. Delimiters

```
(       LeftParen
)       RightParen
[       LeftBracket
]       RightBracket
{       LeftBrace
}       RightBrace
,       Comma
```

**Note:** No semicolons, no assignment operators, no logical operators.

---

## 6. Comments

### Single-Line Comments

```
# This is a comment
x + 5  # Inline comment
```

**Pattern:** `#` followed by anything until newline

---

## 7. Whitespace (Ignored)

```
Space:      ' '
Tab:        '\t'
Newline:    '\n'
```

Whitespace ignored except inside tokens.

---

## 8. Complete Token List

| Token | Lexeme | Category | Example |
|-------|--------|----------|---------|
| **NUMBER** | `42`, `3.14`, `1e10` | Literal | `123.456` |
| **IDENTIFIER** | `x`, `sin`, `dft` | Identifier | `magnitude` |
| **PI** | `PI` | Constant | `PI` |
| **E** | `E` | Constant | `E` |
| **PHI** | `PHI` | Constant | `PHI` |
| **INFINITY** | `Infinity` | Special | `Infinity` |
| **NAN** | `NaN` | Special | `NaN` |
| **IF** | `if` | Keyword | `if` |
| **PLUS** | `+` | Operator | `+` |
| **MINUS** | `-` | Operator | `-` |
| **STAR** | `*` | Operator | `*` |
| **SLASH** | `/` | Operator | `/` |
| **CARET** | `^` | Operator | `^` |
| **LESS** | `<` | Comparison | `<` |
| **LESS_EQUAL** | `<=` | Comparison | `<=` |
| **GREATER** | `>` | Comparison | `>` |
| **GREATER_EQUAL** | `>=` | Comparison | `>=` |
| **EQUAL** | `==` | Comparison | `==` |
| **LEFT_PAREN** | `(` | Delimiter | `(` |
| **RIGHT_PAREN** | `)` | Delimiter | `)` |
| **LEFT_BRACKET** | `[` | Delimiter | `[` |
| **RIGHT_BRACKET** | `]` | Delimiter | `]` |
| **LEFT_BRACE** | `{` | Delimiter | `{` |
| **RIGHT_BRACE** | `}` | Delimiter | `}` |
| **COMMA** | `,` | Delimiter | `,` |
| **EOF** | (end of input) | Special | - |

**Total: ~27 token types**

---

## 9. Token Examples

### Simple Expression

```
Input:  2 + 3 * 4

Tokens:
NUMBER(2)
PLUS
NUMBER(3)
STAR
NUMBER(4)
EOF
```

### Function Call

```
Input:  sin(PI / 2)

Tokens:
IDENTIFIER("sin")
LEFT_PAREN
IDENTIFIER("PI")
SLASH
NUMBER(2)
RIGHT_PAREN
EOF
```

### DFT Expression

```
Input:  magnitude(dft([1, 2, 3]))

Tokens:
IDENTIFIER("magnitude")
LEFT_PAREN
IDENTIFIER("dft")
LEFT_PAREN
LEFT_BRACKET
NUMBER(1)
COMMA
NUMBER(2)
COMMA
NUMBER(3)
RIGHT_BRACKET
RIGHT_PAREN
RIGHT_PAREN
EOF
```

### Piecewise Function

```
Input:  { t^2 if t < 0, exp(-t) if t >= 0 }

Tokens:
LEFT_BRACE
IDENTIFIER("t")
CARET
NUMBER(2)
IF
IDENTIFIER("t")
LESS
NUMBER(0)
COMMA
IDENTIFIER("exp")
LEFT_PAREN
MINUS
IDENTIFIER("t")
RIGHT_PAREN
IF
IDENTIFIER("t")
GREATER_EQUAL
NUMBER(0)
RIGHT_BRACE
EOF
```

---

## 10. Lexer State Machine

### States

```
START       → Initial state
NUMBER      → Reading numeric literal
IDENTIFIER  → Reading identifier
COMMENT     → Reading comment
OPERATOR    → Reading operator (may be multi-char)
```

### Transitions

```
START:
    Digit           → NUMBER
    Letter          → IDENTIFIER
    #               → COMMENT
    Operator        → OPERATOR
    Whitespace      → START (skip)
    EOF             → END

NUMBER:
    Digit | . | e | E → NUMBER
    Other           → Emit NUMBER, return to START

IDENTIFIER:
    Letter | Digit | _  → IDENTIFIER
    Other           → Check if keyword/constant, emit, return to START

COMMENT:
    Newline         → START
    Other           → COMMENT (skip)

OPERATOR:
    < followed by = → LESS_EQUAL
    > followed by = → GREATER_EQUAL
    = followed by = → EQUAL
    Other           → Emit single-char operator, return to START
```

---

## 11. What's NOT Here

### ❌ Removed Tokens (vs Full Language)

```
# NO string literals
STRING              # Not needed

# NO boolean literals
TRUE, FALSE         # Not needed

# NO assignment operators
ASSIGN (=)          # Not needed
PLUS_ASSIGN (+=)    # Not needed
MINUS_ASSIGN (-=)   # Not needed

# NO logical operators
AND (&&)            # Not needed
OR (||)             # Not needed
NOT (!)             # Not needed

# NO control flow keywords
THEN, ELSE          # Not needed
FOR, WHILE, DO      # Not needed
FUNCTION, RETURN    # Not needed

# NO other punctuation
SEMICOLON (;)       # Not needed
COLON (:)           # Not needed
DOT (.)             # Not needed
ARROW (=>)          # Not needed
```

**Result:** 27 tokens instead of 50+

---

## 12. Ambiguity Resolution

### Minus Sign

```
Input: x - y

Context: Binary subtraction
Tokens: IDENTIFIER("x"), MINUS, IDENTIFIER("y")
```

```
Input: -y

Context: Unary negation
Tokens: MINUS, IDENTIFIER("y")
```

**Rule:** After operators/keywords: unary. After identifiers/literals/closing parens: binary.

### Comparison Operators

```
Input: <=

Longest match: LESS_EQUAL (not LESS followed by EQUAL)
```

---

## 13. Error Handling

### Invalid Character

```
Input:  @#$%

Error:  Unexpected character '@' at line 1, column 1
```

### Invalid Number

```
Input:  1.2.3

Error:  Invalid number literal at line 1, column 1
```

---

## 14. Token Implementation

```typescript
enum TokenType {
    // Literals
    NUMBER,

    // Identifiers and Keywords
    IDENTIFIER,
    IF,

    // Constants
    PI, E, PHI, INFINITY, NAN,

    // Operators
    PLUS, MINUS, STAR, SLASH, CARET,

    // Comparisons
    LESS, LESS_EQUAL, GREATER, GREATER_EQUAL, EQUAL,

    // Delimiters
    LEFT_PAREN, RIGHT_PAREN,
    LEFT_BRACKET, RIGHT_BRACKET,
    LEFT_BRACE, RIGHT_BRACE,
    COMMA,

    // Special
    EOF
}

interface Token {
    type: TokenType
    value: string | number
    line: number
    column: number
}
```

---

## 15. Comparison: Before vs After

| Category | Full Language | Math Engine |
|----------|--------------|-------------|
| **Keywords** | 15+ | 1 (`if` only) |
| **Operators** | 20+ | 10 |
| **Delimiters** | 10+ | 7 |
| **Literals** | 4 types | 1 type (NUMBER) |
| **Total Tokens** | 50+ | ~27 |
| **Complexity** | High | Low |

---

## See Also

- [Syntax Grammar](02-syntax.md) - BNF grammar (expressions only)
- [Operator Precedence](03-precedence.md) - Precedence table
- [Examples](../examples/07-composition.md) - Real use cases
