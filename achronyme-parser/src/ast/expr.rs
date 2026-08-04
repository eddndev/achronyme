use super::*;

/// Expression variants.
///
/// Every variant carries an [`ExprId`] assigned by the parser (or
/// [`ExprId::SYNTHETIC`] for nodes constructed outside the parser).
/// The resolver pass uses this id to attach a `SymbolId` via a
/// parallel `HashMap<ExprId, SymbolId>`.
#[derive(Clone, Debug)]
pub enum Expr {
    Number {
        id: ExprId,
        value: String,
        span: Span,
    },
    FieldLit {
        id: ExprId,
        value: String,
        radix: FieldRadix,
        span: Span,
    },
    BigIntLit {
        id: ExprId,
        value: String,
        width: u16,
        radix: BigIntRadix,
        span: Span,
    },
    Bool {
        id: ExprId,
        value: bool,
        span: Span,
    },
    StringLit {
        id: ExprId,
        value: String,
        span: Span,
    },
    Nil {
        id: ExprId,
        span: Span,
    },
    Ident {
        id: ExprId,
        name: String,
        span: Span,
    },
    BinOp {
        id: ExprId,
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
    UnaryOp {
        id: ExprId,
        op: UnaryOp,
        operand: Box<Expr>,
        span: Span,
    },
    Call {
        id: ExprId,
        callee: Box<Expr>,
        args: Vec<CallArg>,
        span: Span,
    },
    Index {
        id: ExprId,
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    DotAccess {
        id: ExprId,
        object: Box<Expr>,
        field: String,
        span: Span,
    },
    If {
        id: ExprId,
        condition: Box<Expr>,
        then_block: Block,
        else_branch: Option<ElseBranch>,
        span: Span,
    },
    For {
        id: ExprId,
        var: String,
        iterable: ForIterable,
        body: Block,
        span: Span,
    },
    While {
        id: ExprId,
        condition: Box<Expr>,
        body: Block,
        span: Span,
    },
    Forever {
        id: ExprId,
        body: Block,
        span: Span,
    },
    /// Structured-concurrency scope: `concurrent { ... }`.
    Concurrent {
        id: ExprId,
        body: Block,
        span: Span,
    },
    /// Start a child task owned by the nearest concurrent scope.
    Spawn {
        id: ExprId,
        call: Box<Expr>,
        span: Span,
    },
    /// Explicit suspension while waiting for a task or async operation.
    Await {
        id: ExprId,
        task: Box<Expr>,
        /// Whether failure is returned as an explicit outcome value instead
        /// of being propagated through the owning concurrent scope.
        mode: AwaitMode,
        span: Span,
    },
    /// Block expression. The [`ExprId`] is attached to the expression
    /// wrapper; the inner [`Block`] carries its own span but no id.
    Block {
        id: ExprId,
        block: Block,
    },
    FnExpr {
        id: ExprId,
        name: Option<String>,
        params: Vec<TypedParam>,
        return_type: Option<TypeAnnotation>,
        body: Block,
        span: Span,
    },
    Prove {
        id: ExprId,
        /// Optional name: `prove eligibility(hash: Public) { ... }`
        name: Option<String>,
        body: Block,
        /// Typed params with visibility: `prove(hash: Public, flag: Public Bool) { ... }`
        /// When non-empty, witnesses are auto-inferred from outer scope.
        /// Also supports deprecated `prove(public: [x, y])` (converted to params).
        params: Vec<TypedParam>,
        span: Span,
    },
    // CircuitCall was removed; Call now carries keyword CallArgs.
    Array {
        id: ExprId,
        elements: Vec<Expr>,
        span: Span,
    },
    Map {
        id: ExprId,
        pairs: Vec<(MapKey, Expr)>,
        span: Span,
    },
    /// Static access: `Type::MEMBER` (e.g., `Int::MAX`, `Field::ORDER`).
    StaticAccess {
        id: ExprId,
        type_name: String,
        member: String,
        span: Span,
    },
    /// Placeholder for an expression that failed to parse (error recovery).
    Error {
        id: ExprId,
        span: Span,
    },
}

/// Failure handling selected at an explicit `await` suspension point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AwaitMode {
    /// Propagate a child failure through the owning concurrent scope.
    #[default]
    Propagate,
    /// Return `{ ok, value }` or `{ ok, error }` to the awaiting task.
    Outcome,
    /// Return the first terminal task from a bounded list and cancel the rest.
    Race,
}

impl Expr {
    /// Borrow the source span covering this expression.
    pub fn span(&self) -> &Span {
        match self {
            Expr::Number { span, .. }
            | Expr::FieldLit { span, .. }
            | Expr::BigIntLit { span, .. }
            | Expr::Bool { span, .. }
            | Expr::StringLit { span, .. }
            | Expr::Nil { span, .. }
            | Expr::Ident { span, .. }
            | Expr::BinOp { span, .. }
            | Expr::UnaryOp { span, .. }
            | Expr::Call { span, .. }
            | Expr::Index { span, .. }
            | Expr::DotAccess { span, .. }
            | Expr::If { span, .. }
            | Expr::For { span, .. }
            | Expr::While { span, .. }
            | Expr::Forever { span, .. }
            | Expr::Concurrent { span, .. }
            | Expr::Spawn { span, .. }
            | Expr::Await { span, .. }
            | Expr::FnExpr { span, .. }
            | Expr::Prove { span, .. }
            | Expr::Array { span, .. }
            | Expr::Map { span, .. }
            | Expr::StaticAccess { span, .. }
            | Expr::Error { span, .. } => span,
            Expr::Block { block, .. } => &block.span,
        }
    }

    /// Return the [`ExprId`] assigned to this expression.
    ///
    /// For parser-produced nodes this id is dense and unique within the
    /// enclosing `Program`. Nodes constructed outside the parser carry
    /// [`ExprId::SYNTHETIC`].
    pub fn id(&self) -> ExprId {
        match self {
            Expr::Number { id, .. }
            | Expr::FieldLit { id, .. }
            | Expr::BigIntLit { id, .. }
            | Expr::Bool { id, .. }
            | Expr::StringLit { id, .. }
            | Expr::Nil { id, .. }
            | Expr::Ident { id, .. }
            | Expr::BinOp { id, .. }
            | Expr::UnaryOp { id, .. }
            | Expr::Call { id, .. }
            | Expr::Index { id, .. }
            | Expr::DotAccess { id, .. }
            | Expr::If { id, .. }
            | Expr::For { id, .. }
            | Expr::While { id, .. }
            | Expr::Forever { id, .. }
            | Expr::Concurrent { id, .. }
            | Expr::Spawn { id, .. }
            | Expr::Await { id, .. }
            | Expr::Block { id, .. }
            | Expr::FnExpr { id, .. }
            | Expr::Prove { id, .. }
            | Expr::Array { id, .. }
            | Expr::Map { id, .. }
            | Expr::StaticAccess { id, .. }
            | Expr::Error { id, .. } => *id,
        }
    }
}
