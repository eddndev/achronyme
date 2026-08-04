use std::fmt::Write;

use crate::ast::{Block, InputDecl, Stmt, TypedParam};

use super::{FormatError, Formatter};

impl Formatter {
    pub(super) fn write_stmt(&mut self, statement: &Stmt) -> Result<(), FormatError> {
        match statement {
            Stmt::LetDecl {
                name,
                type_ann,
                value,
                ..
            } => {
                write!(self.output, "let {name}").unwrap();
                if let Some(annotation) = type_ann {
                    write!(self.output, ": {annotation}").unwrap();
                }
                self.output.push_str(" = ");
                self.write_expr(value, 0)?;
            }
            Stmt::MutDecl {
                name,
                type_ann,
                value,
                ..
            } => {
                write!(self.output, "mut {name}").unwrap();
                if let Some(annotation) = type_ann {
                    write!(self.output, ": {annotation}").unwrap();
                }
                self.output.push_str(" = ");
                self.write_expr(value, 0)?;
            }
            Stmt::Assignment { target, value, .. } => {
                self.write_expr(target, 0)?;
                self.output.push_str(" = ");
                self.write_expr(value, 0)?;
            }
            Stmt::PublicDecl { names, .. } => {
                self.output.push_str("public ");
                self.write_input_decls(names);
            }
            Stmt::WitnessDecl { names, .. } => {
                self.output.push_str("witness ");
                self.write_input_decls(names);
            }
            Stmt::FnDecl {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                write!(self.output, "fn {name}(").unwrap();
                self.write_params(params);
                self.output.push(')');
                if let Some(annotation) = return_type {
                    write!(self.output, " -> {annotation}").unwrap();
                }
                self.output.push(' ');
                self.write_block(body)?;
            }
            Stmt::Print { value, .. } => {
                self.output.push_str("print(");
                self.write_expr(value, 0)?;
                self.output.push(')');
            }
            Stmt::Return { value, .. } => {
                self.output.push_str("return");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.write_expr(value, 0)?;
                } else {
                    self.output.push(';');
                }
            }
            Stmt::Break { .. } => self.output.push_str("break"),
            Stmt::Continue { .. } => self.output.push_str("continue"),
            Stmt::Import { path, alias, .. } => {
                write!(self.output, "import {:?} as {alias}", path).unwrap();
            }
            Stmt::Export { inner, .. } => {
                self.output.push_str("export ");
                self.write_stmt(inner)?;
            }
            Stmt::SelectiveImport { names, path, .. } => {
                write!(
                    self.output,
                    "import {{ {} }} from {:?}",
                    names.join(", "),
                    path
                )
                .unwrap();
            }
            Stmt::ExportList { names, .. } => {
                write!(self.output, "export {{ {} }}", names.join(", ")).unwrap();
            }
            Stmt::CircuitDecl {
                name, params, body, ..
            } => {
                write!(self.output, "circuit {name}(").unwrap();
                self.write_params(params);
                self.output.push_str(") ");
                self.write_block(body)?;
            }
            Stmt::ImportCircuit { path, alias, .. } => {
                write!(self.output, "import circuit {:?} as {alias}", path).unwrap();
            }
            Stmt::Expr(expression) => self.write_expr(expression, 0)?,
            Stmt::Error { .. } => return Err(FormatError::recovered_node("statement")),
        }
        Ok(())
    }

    pub(super) fn write_block(&mut self, block: &Block) -> Result<(), FormatError> {
        if block.stmts.is_empty() {
            self.output.push_str("{}");
            return Ok(());
        }

        self.output.push_str("{\n");
        self.indent += 1;
        for statement in &block.stmts {
            self.write_indent();
            self.write_stmt(statement)?;
            self.output.push('\n');
        }
        self.indent -= 1;
        self.write_indent();
        self.output.push('}');
        Ok(())
    }

    pub(super) fn write_params(&mut self, params: &[TypedParam]) {
        for (index, parameter) in params.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&parameter.name);
            if let Some(annotation) = &parameter.type_ann {
                write!(self.output, ": {annotation}").unwrap();
            }
        }
    }

    fn write_input_decls(&mut self, declarations: &[InputDecl]) {
        for (index, declaration) in declarations.iter().enumerate() {
            if index > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(&declaration.name);
            if let Some(size) = declaration.array_size {
                write!(self.output, "[{size}]").unwrap();
            }
            if let Some(annotation) = &declaration.type_ann {
                write!(self.output, ": {annotation}").unwrap();
            }
        }
    }
}
