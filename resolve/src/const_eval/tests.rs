#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_env(vars: &[(&str, i64)]) -> (EvalCtx<'static>, ConstValues) {
        // Leak the ConstValues so we can return both ctx and result.
        // Only for tests; the leaked map is tiny.
        let result = Box::leak(Box::new(ConstValues::new()));
        let mut scopes = vec![HashMap::new()];
        for &(name, val) in vars {
            scopes[0].insert(name.to_string(), val);
        }
        (
            EvalCtx {
                module_id: crate::module_graph::ModuleId::from_raw(0),
                scopes,
                result,
            },
            ConstValues::new(),
        )
    }

    fn zero_span() -> achronyme_parser::ast::Span {
        achronyme_parser::ast::Span {
            byte_start: 0,
            byte_end: 0,
            line_start: 0,
            col_start: 0,
            line_end: 0,
            col_end: 0,
        }
    }

    fn num(value: i64) -> Expr {
        use achronyme_parser::ast::ExprId;
        Expr::Number {
            id: ExprId::SYNTHETIC,
            value: value.to_string(),
            span: zero_span(),
        }
    }

    fn ident(name: &str) -> Expr {
        use achronyme_parser::ast::ExprId;
        Expr::Ident {
            id: ExprId::SYNTHETIC,
            name: name.to_string(),
            span: zero_span(),
        }
    }

    fn binop(op: BinOp, lhs: Expr, rhs: Expr) -> Expr {
        use achronyme_parser::ast::ExprId;
        Expr::BinOp {
            id: ExprId::SYNTHETIC,
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            span: zero_span(),
        }
    }

    fn unary(op: UnaryOp, operand: Expr) -> Expr {
        use achronyme_parser::ast::ExprId;
        Expr::UnaryOp {
            id: ExprId::SYNTHETIC,
            op,
            operand: Box::new(operand),
            span: zero_span(),
        }
    }

    #[test]
    fn number_literal() {
        let (ctx, _) = ctx_with_env(&[]);
        assert_eq!(try_eval(&ctx, &num(42)), Some(42));
    }

    #[test]
    fn negative_number() {
        let (ctx, _) = ctx_with_env(&[]);
        assert_eq!(try_eval(&ctx, &num(-7)), Some(-7));
    }

    #[test]
    fn ident_lookup() {
        let (ctx, _) = ctx_with_env(&[("n", 8)]);
        assert_eq!(try_eval(&ctx, &ident("n")), Some(8));
    }

    #[test]
    fn unknown_ident() {
        let (ctx, _) = ctx_with_env(&[]);
        assert_eq!(try_eval(&ctx, &ident("x")), None);
    }

    #[test]
    fn add() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = binop(BinOp::Add, num(3), num(4));
        assert_eq!(try_eval(&ctx, &e), Some(7));
    }

    #[test]
    fn sub() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = binop(BinOp::Sub, num(10), num(3));
        assert_eq!(try_eval(&ctx, &e), Some(7));
    }

    #[test]
    fn mul() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = binop(BinOp::Mul, num(6), num(7));
        assert_eq!(try_eval(&ctx, &e), Some(42));
    }

    #[test]
    fn div_by_zero() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = binop(BinOp::Div, num(10), num(0));
        assert_eq!(try_eval(&ctx, &e), None);
    }

    #[test]
    fn pow() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = binop(BinOp::Pow, num(2), num(10));
        assert_eq!(try_eval(&ctx, &e), Some(1024));
    }

    #[test]
    fn nested_binop_with_ident() {
        let (ctx, _) = ctx_with_env(&[("n", 4)]);
        let e = binop(BinOp::Mul, ident("n"), num(2));
        assert_eq!(try_eval(&ctx, &e), Some(8));
    }

    #[test]
    fn comparison() {
        let (ctx, _) = ctx_with_env(&[]);
        assert_eq!(try_eval(&ctx, &binop(BinOp::Lt, num(3), num(5))), Some(1));
        assert_eq!(try_eval(&ctx, &binop(BinOp::Gt, num(3), num(5))), Some(0));
    }

    #[test]
    fn negation() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = unary(UnaryOp::Neg, num(42));
        assert_eq!(try_eval(&ctx, &e), Some(-42));
    }

    #[test]
    fn not_op() {
        let (ctx, _) = ctx_with_env(&[]);
        assert_eq!(try_eval(&ctx, &unary(UnaryOp::Not, num(0))), Some(1));
        assert_eq!(try_eval(&ctx, &unary(UnaryOp::Not, num(5))), Some(0));
    }

    #[test]
    fn partial_unknown_returns_none() {
        let (ctx, _) = ctx_with_env(&[]);
        let e = binop(BinOp::Add, num(1), ident("unknown"));
        assert_eq!(try_eval(&ctx, &e), None);
    }
}
