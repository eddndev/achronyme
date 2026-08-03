/// Token types for the Achronyme lexer.
use crate::ast::Span;

/// A single token produced by the lexer.
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

/// All token variants recognized by the lexer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // Literals
    Integer,
    FieldLit,
    BigIntLit,
    StringLit,

    // Keywords
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    In,
    Fn,
    Return,
    Break,
    Continue,
    Print,
    Nil,
    True,
    False,
    Public,
    Witness,
    Prove,
    Circuit,
    Forever,
    Import,
    Export,
    As,
    Concurrent,
    Spawn,
    Await,

    // Identifier
    Ident,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    Assign,
    Arrow,
    DotDot,
    Dot,

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    ColonColon,
    Semicolon,

    // End of file
    Eof,
}

/// Canonical source keyword table shared by the lexer and editor consumers.
pub const KEYWORDS: &[(&str, TokenKind)] = &[
    ("let", TokenKind::Let),
    ("mut", TokenKind::Mut),
    ("if", TokenKind::If),
    ("else", TokenKind::Else),
    ("while", TokenKind::While),
    ("for", TokenKind::For),
    ("in", TokenKind::In),
    ("fn", TokenKind::Fn),
    ("return", TokenKind::Return),
    ("break", TokenKind::Break),
    ("continue", TokenKind::Continue),
    ("print", TokenKind::Print),
    ("nil", TokenKind::Nil),
    ("true", TokenKind::True),
    ("false", TokenKind::False),
    ("public", TokenKind::Public),
    ("witness", TokenKind::Witness),
    ("prove", TokenKind::Prove),
    ("circuit", TokenKind::Circuit),
    ("forever", TokenKind::Forever),
    ("import", TokenKind::Import),
    ("export", TokenKind::Export),
    ("as", TokenKind::As),
    ("concurrent", TokenKind::Concurrent),
    ("spawn", TokenKind::Spawn),
    ("await", TokenKind::Await),
];

pub fn keyword_kind(source: &str) -> Option<TokenKind> {
    KEYWORDS
        .iter()
        .find_map(|(keyword, kind)| (*keyword == source).then_some(*kind))
}
