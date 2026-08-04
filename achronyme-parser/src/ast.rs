// Owned AST types for the Achronyme language.
//
// These types represent the parsed structure of an Achronyme program,
// independent of the pest parser. All types are `Clone + Debug`.

// Re-export Span from the shared diagnostics crate.
pub use diagnostics::Span;

mod expr;
mod types;

pub use expr::{AwaitMode, Expr};
pub use types::{BaseType, TypeAnnotation, TypedParam, Visibility};

/// Dense, unique identifier assigned to every `Expr` at parse time.
///
/// `ExprId` is the key used by the resolver pass to attach a
/// `SymbolId` to each call site and identifier via a parallel
/// `HashMap<ExprId, SymbolId>` inside `resolve::SymbolTable`. Every
/// parser-allocated id is unique within one `Program`; clones of an
/// `Expr` preserve the original id (cloning is never a source of new
/// parse-time state).
///
/// The reserved value [`ExprId::SYNTHETIC`] marks `Expr` nodes constructed
/// outside the parser (e.g. by the IR or circom compilers for internal
/// lowering). Synthetic nodes are not resolved, so the resolver pass
/// skips them.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExprId(u32);

impl ExprId {
    /// Sentinel id for `Expr` nodes constructed outside the parser.
    ///
    /// Never collides with a parser-allocated id because the parser's
    /// counter starts at 1 (see `Parser::alloc_expr_id`).
    pub const SYNTHETIC: Self = Self(0);

    /// Construct an id from a raw `u32`. `0` is reserved for
    /// [`SYNTHETIC`](Self::SYNTHETIC); callers that need a parse-time
    /// id should use the parser's allocator instead.
    pub const fn from_raw(n: u32) -> Self {
        Self(n)
    }

    /// Raw underlying `u32`, suitable for stable hashing or indexing.
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns `true` if this id is the reserved synthetic sentinel.
    pub const fn is_synthetic(self) -> bool {
        self.0 == Self::SYNTHETIC.0
    }
}

/// A complete program: a sequence of statements.
#[derive(Clone, Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// Statement variants.
#[derive(Clone, Debug)]
pub enum Stmt {
    LetDecl {
        name: String,
        type_ann: Option<TypeAnnotation>,
        value: Expr,
        span: Span,
    },
    MutDecl {
        name: String,
        type_ann: Option<TypeAnnotation>,
        value: Expr,
        span: Span,
    },
    Assignment {
        target: Expr,
        value: Expr,
        span: Span,
    },
    PublicDecl {
        names: Vec<InputDecl>,
        span: Span,
    },
    WitnessDecl {
        names: Vec<InputDecl>,
        span: Span,
    },
    FnDecl {
        name: String,
        params: Vec<TypedParam>,
        return_type: Option<TypeAnnotation>,
        body: Block,
        span: Span,
    },
    Print {
        value: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    Import {
        path: String,
        alias: String,
        span: Span,
    },
    Export {
        inner: Box<Stmt>,
        span: Span,
    },
    SelectiveImport {
        names: Vec<String>,
        path: String,
        span: Span,
    },
    ExportList {
        names: Vec<String>,
        span: Span,
    },
    /// Reusable circuit definition: `circuit name(root: Public, secret: Witness) { body }`
    CircuitDecl {
        name: String,
        params: Vec<TypedParam>,
        body: Block,
        span: Span,
    },
    /// Circuit import: `import circuit "path" as name`
    ImportCircuit {
        path: String,
        alias: String,
        span: Span,
    },
    Expr(Expr),
    /// Placeholder for a statement that failed to parse (error recovery).
    Error {
        span: Span,
    },
}

/// A call argument — positional or keyword.
#[derive(Clone, Debug)]
pub struct CallArg {
    /// `None` = positional, `Some("x")` = keyword (`x: expr`).
    pub name: Option<String>,
    pub value: Expr,
}

/// A public/witness input declaration with optional array size.
#[derive(Clone, Debug)]
pub struct InputDecl {
    pub name: String,
    pub array_size: Option<usize>,
    pub type_ann: Option<TypeAnnotation>,
}

/// A block of statements (e.g., `{ ... }`).
#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// Radix for field element literals (`0p` prefix).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldRadix {
    Decimal,
    Hex,
    Binary,
}

/// Radix for BigInt literals (`0i` prefix).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BigIntRadix {
    Hex,
    Decimal,
    Binary,
}

/// Map key: either an identifier or a string literal.
#[derive(Clone, Debug)]
pub enum MapKey {
    Ident(String),
    StringLit(String),
}

/// Else branch: either a block or a chained `if`.
#[derive(Clone, Debug)]
pub enum ElseBranch {
    Block(Block),
    If(Box<Expr>),
}

/// For-loop iterable: either a range or an expression.
#[derive(Clone, Debug)]
pub enum ForIterable {
    Range {
        start: u64,
        end: u64,
    },
    /// Dynamic end bound: `0..n` or `0..(n+1)`.
    /// Start is a literal, end is an expression resolved at instantiation.
    /// Only valid in circuit/prove contexts; VM mode rejects this variant.
    ExprRange {
        start: u64,
        end: Box<Expr>,
    },
    Expr(Box<Expr>),
}

/// Binary operators.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

/// Unary operators.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}
