use std::collections::HashMap;

use thiserror::Error;
use torel_diagnostics::{Diagnostic, Label, Span};
use torel_ir::{
    Mutability, ResolvedValue, TypeId, TypedBlock, TypedExpr, TypedExprKind, TypedModule,
    TypedProc, TypedStmt, TypedStmtKind, TypedTypeRef,
};

use crate::{
    MirBinaryOp, MirBlock, MirBlockId, MirFunction, MirLocal, MirLowerError::InvalidMir, MirModule,
    MirOperand, MirPlace, MirRvalue, MirStatement, MirTemp, MirTempId, MirTerminator,
    MirTerminatorKind, MirType, MirUnaryOp, validate_module,
};

#[derive(Debug, Error)]
pub enum MirLowerError {
    #[error("cannot lower procedure value to MIR operand")]
    ProcedureValue { span: Span },

    #[error("`break` reached MIR lowering without an enclosing loop target")]
    BreakWithoutLoop { span: Span },

    #[error("`continue` reached MIR lowering without an enclosing loop target")]
    ContinueWithoutLoop { span: Span },

    #[error("cannot append MIR statement after a terminator")]
    AppendAfterTerminator { span: Span },

    #[error("lowered MIR failed validation: {message}")]
    InvalidMir { message: String },
}

impl MirLowerError {
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            Self::ProcedureValue { span } => Diagnostic::error(self.to_string()).with_label(
                Label::primary(*span, "procedure cannot be used as a MIR value"),
            ),
            Self::BreakWithoutLoop { span } => Diagnostic::error(self.to_string())
                .with_label(Label::primary(*span, "break has no MIR loop target")),
            Self::ContinueWithoutLoop { span } => Diagnostic::error(self.to_string())
                .with_label(Label::primary(*span, "continue has no MIR loop target")),
            Self::AppendAfterTerminator { span } => Diagnostic::error(self.to_string())
                .with_label(Label::primary(*span, "block is already terminated")),
            Self::InvalidMir { .. } => Diagnostic::error(self.to_string()),
        }
    }
}

pub fn lower_to_mir(module: &TypedModule) -> Result<MirModule, MirLowerError> {
    let mut type_names = builtin_type_names();
    collect_module_type_names(module, &mut type_names);

    let functions = module
        .procs
        .iter()
        .map(|proc| FunctionLowerer::new(proc, type_names.clone()).lower())
        .collect::<Result<Vec<_>, _>>()?;
    let mir = MirModule {
        unit_path: module.unit_path.clone(),
        functions,
    };

    validate_module(&mir).map_err(|err| InvalidMir {
        message: err.to_string(),
    })?;

    Ok(mir)
}

struct FunctionLowerer<'a> {
    proc: &'a TypedProc,
    type_names: HashMap<TypeId, String>,
    locals: Vec<MirLocal>,
    temps: Vec<MirTemp>,
    blocks: Vec<MirBlock>,
    current: MirBlockId,
    next_block: u32,
    next_temp: u32,
    break_targets: Vec<MirBlockId>,
    continue_targets: Vec<MirBlockId>,
}

impl<'a> FunctionLowerer<'a> {
    fn new(proc: &'a TypedProc, type_names: HashMap<TypeId, String>) -> Self {
        let entry = MirBlockId(0);

        Self {
            proc,
            type_names,
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: vec![MirBlock {
                id: entry,
                statements: Vec::new(),
                terminator: None,
                span: proc.body.span,
            }],
            current: entry,
            next_block: 1,
            next_temp: 0,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
        }
    }

    fn lower(mut self) -> Result<MirFunction, MirLowerError> {
        self.lower_block(&self.proc.body)?;

        if self.current_is_open() {
            let terminator = if self.proc.return_type.display_name == "Void" {
                MirTerminatorKind::Return { value: None }
            } else {
                MirTerminatorKind::Unreachable
            };
            self.terminate_current(terminator, self.proc.body.span)?;
        }

        Ok(MirFunction {
            id: self.proc.id,
            name: self.proc.name.clone(),
            return_type: mir_type(&self.proc.return_type),
            params: self
                .proc
                .params
                .iter()
                .map(|param| MirLocal {
                    id: param.id,
                    name: param.name.clone(),
                    ty: mir_type(&param.ty),
                    mutability: Mutability::Immutable,
                    span: param.name_span,
                })
                .collect(),
            locals: self.locals,
            temps: self.temps,
            blocks: self.blocks,
            entry: MirBlockId(0),
            span: self.proc.span,
        })
    }

    fn lower_block(&mut self, block: &TypedBlock) -> Result<(), MirLowerError> {
        for stmt in &block.stmts {
            if !self.current_is_open() {
                return Err(MirLowerError::AppendAfterTerminator { span: stmt.span });
            }

            self.lower_stmt(stmt)?;
        }

        if let Some(tail) = &block.tail {
            if !self.current_is_open() {
                return Err(MirLowerError::AppendAfterTerminator { span: tail.span });
            }

            let value = self.lower_expr(tail)?;
            self.terminate_current(MirTerminatorKind::Return { value: Some(value) }, tail.span)?;
        }

        Ok(())
    }

    fn lower_stmt(&mut self, stmt: &TypedStmt) -> Result<(), MirLowerError> {
        match &stmt.kind {
            TypedStmtKind::Local {
                id,
                mutability,
                name,
                ty,
                value,
            } => {
                self.locals.push(MirLocal {
                    id: *id,
                    name: name.clone(),
                    ty: mir_type(ty),
                    mutability: *mutability,
                    span: stmt.span,
                });
                let value = self.lower_expr(value)?;
                self.emit_assign(MirPlace::Local(*id), MirRvalue::Use(value), stmt.span)
            }
            TypedStmtKind::Assign { target, value } => {
                let value = self.lower_expr(value)?;
                self.emit_assign(MirPlace::Local(*target), MirRvalue::Use(value), stmt.span)
            }
            TypedStmtKind::If {
                condition,
                then_block,
                else_block,
            } => self.lower_if(condition, then_block, else_block.as_ref(), stmt.span),
            TypedStmtKind::While { condition, body } => {
                self.lower_while(condition, body, stmt.span)
            }
            TypedStmtKind::Loop { body } => self.lower_loop(body, stmt.span),
            TypedStmtKind::Break => {
                let Some(target) = self.break_targets.last().copied() else {
                    return Err(MirLowerError::BreakWithoutLoop { span: stmt.span });
                };
                self.terminate_current(MirTerminatorKind::Jump(target), stmt.span)
            }
            TypedStmtKind::Continue => {
                let Some(target) = self.continue_targets.last().copied() else {
                    return Err(MirLowerError::ContinueWithoutLoop { span: stmt.span });
                };
                self.terminate_current(MirTerminatorKind::Jump(target), stmt.span)
            }
            TypedStmtKind::Return(expr) => {
                let value = self.lower_expr(expr)?;
                self.terminate_current(MirTerminatorKind::Return { value: Some(value) }, stmt.span)
            }
        }
    }

    fn lower_if(
        &mut self,
        condition: &TypedExpr,
        then_block: &TypedBlock,
        else_block: Option<&TypedBlock>,
        span: Span,
    ) -> Result<(), MirLowerError> {
        let condition = self.lower_expr(condition)?;
        let then_id = self.new_block(then_block.span);
        let else_id = self.new_block(else_block.map_or(span, |block| block.span));
        let join_id = self.new_block(span);

        self.terminate_current(
            MirTerminatorKind::Branch {
                condition,
                then_block: then_id,
                else_block: else_id,
            },
            span,
        )?;

        self.current = then_id;
        self.lower_block(then_block)?;
        let then_terminated = !self.current_is_open();
        if !then_terminated {
            self.terminate_current(MirTerminatorKind::Jump(join_id), then_block.span)?;
        }

        self.current = else_id;
        if let Some(else_block) = else_block {
            self.lower_block(else_block)?;
        }
        let else_terminated = !self.current_is_open();
        if !else_terminated {
            self.terminate_current(MirTerminatorKind::Jump(join_id), span)?;
        }

        self.current = join_id;
        if then_terminated && else_terminated {
            self.terminate_current(MirTerminatorKind::Unreachable, span)?;
        }

        Ok(())
    }

    fn lower_while(
        &mut self,
        condition: &TypedExpr,
        body: &TypedBlock,
        span: Span,
    ) -> Result<(), MirLowerError> {
        let condition_id = self.new_block(condition.span);
        let body_id = self.new_block(body.span);
        let exit_id = self.new_block(span);

        self.terminate_current(MirTerminatorKind::Jump(condition_id), span)?;

        self.current = condition_id;
        let condition = self.lower_expr(condition)?;
        self.terminate_current(
            MirTerminatorKind::Branch {
                condition,
                then_block: body_id,
                else_block: exit_id,
            },
            span,
        )?;

        self.current = body_id;
        self.break_targets.push(exit_id);
        self.continue_targets.push(condition_id);
        self.lower_block(body)?;
        self.continue_targets.pop();
        self.break_targets.pop();

        if self.current_is_open() {
            self.terminate_current(MirTerminatorKind::Jump(condition_id), body.span)?;
        }

        self.current = exit_id;
        Ok(())
    }

    fn lower_loop(&mut self, body: &TypedBlock, span: Span) -> Result<(), MirLowerError> {
        let body_id = self.new_block(body.span);
        let exit_id = self.new_block(span);
        let has_direct_break = block_has_direct_break(body);

        self.terminate_current(MirTerminatorKind::Jump(body_id), span)?;

        self.current = body_id;
        self.break_targets.push(exit_id);
        self.continue_targets.push(body_id);
        self.lower_block(body)?;
        self.continue_targets.pop();
        self.break_targets.pop();

        if self.current_is_open() {
            self.terminate_current(MirTerminatorKind::Jump(body_id), body.span)?;
        }

        self.current = exit_id;
        if !has_direct_break {
            self.terminate_current(MirTerminatorKind::Unreachable, span)?;
        }

        Ok(())
    }

    fn lower_expr(&mut self, expr: &TypedExpr) -> Result<MirOperand, MirLowerError> {
        match &expr.kind {
            TypedExprKind::Path { path, resolved } => match resolved {
                ResolvedValue::BuiltinValue(id) => Ok(MirOperand::BuiltinValue {
                    id: *id,
                    name: path.join("."),
                    ty: self.type_from_id(expr.ty),
                }),
                ResolvedValue::Local(id) => Ok(MirOperand::Place(MirPlace::Local(*id))),
                ResolvedValue::Proc(_) => Err(MirLowerError::ProcedureValue { span: expr.span }),
            },
            TypedExprKind::Int { value } => Ok(MirOperand::Int32(value.clone())),
            TypedExprKind::Text { value } => Ok(MirOperand::Text(value.clone())),
            TypedExprKind::Bool { value } => Ok(MirOperand::Bool(*value)),
            TypedExprKind::Call { callee, args } => {
                let args = args
                    .iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                let temp = self.new_temp(expr.ty, expr.span);
                self.emit_assign(
                    MirPlace::Temp(temp),
                    MirRvalue::Call {
                        callee: *callee,
                        args,
                    },
                    expr.span,
                )?;
                Ok(MirOperand::Place(MirPlace::Temp(temp)))
            }
            TypedExprKind::Unary { op, expr: inner } => {
                let arg = self.lower_expr(inner)?;
                let temp = self.new_temp(expr.ty, expr.span);
                self.emit_assign(
                    MirPlace::Temp(temp),
                    MirRvalue::Unary {
                        op: MirUnaryOp::from(*op),
                        arg,
                    },
                    expr.span,
                )?;
                Ok(MirOperand::Place(MirPlace::Temp(temp)))
            }
            TypedExprKind::Binary { op, lhs, rhs } => {
                let lhs = self.lower_expr(lhs)?;
                let rhs = self.lower_expr(rhs)?;
                let temp = self.new_temp(expr.ty, expr.span);
                self.emit_assign(
                    MirPlace::Temp(temp),
                    MirRvalue::Binary {
                        op: MirBinaryOp::from(*op),
                        lhs,
                        rhs,
                    },
                    expr.span,
                )?;
                Ok(MirOperand::Place(MirPlace::Temp(temp)))
            }
        }
    }

    fn emit_assign(
        &mut self,
        target: MirPlace,
        value: MirRvalue,
        span: Span,
    ) -> Result<(), MirLowerError> {
        if !self.current_is_open() {
            return Err(MirLowerError::AppendAfterTerminator { span });
        }

        self.current_block_mut()
            .statements
            .push(MirStatement::assign(target, value, span));
        Ok(())
    }

    fn terminate_current(
        &mut self,
        kind: MirTerminatorKind,
        span: Span,
    ) -> Result<(), MirLowerError> {
        let block = self.current_block_mut();
        if block.terminator.is_some() {
            return Err(MirLowerError::AppendAfterTerminator { span });
        }

        block.terminator = Some(MirTerminator { kind, span });
        Ok(())
    }

    fn new_block(&mut self, span: Span) -> MirBlockId {
        let id = MirBlockId(self.next_block);
        self.next_block += 1;
        self.blocks.push(MirBlock {
            id,
            statements: Vec::new(),
            terminator: None,
            span,
        });
        id
    }

    fn new_temp(&mut self, ty: TypeId, span: Span) -> MirTempId {
        let id = MirTempId(self.next_temp);
        self.next_temp += 1;
        self.temps.push(MirTemp {
            id,
            ty: self.type_from_id(ty),
            span,
        });
        id
    }

    fn current_is_open(&self) -> bool {
        self.block(self.current).terminator.is_none()
    }

    fn current_block_mut(&mut self) -> &mut MirBlock {
        self.block_mut(self.current)
    }

    fn block(&self, id: MirBlockId) -> &MirBlock {
        self.blocks
            .get(id.0 as usize)
            .expect("MIR block IDs are allocated sequentially")
    }

    fn block_mut(&mut self, id: MirBlockId) -> &mut MirBlock {
        self.blocks
            .get_mut(id.0 as usize)
            .expect("MIR block IDs are allocated sequentially")
    }

    fn type_from_id(&self, id: TypeId) -> MirType {
        MirType {
            id,
            display_name: self
                .type_names
                .get(&id)
                .cloned()
                .unwrap_or_else(|| format!("<type {}>", id.0)),
        }
    }
}

fn collect_module_type_names(module: &TypedModule, names: &mut HashMap<TypeId, String>) {
    for proc in &module.procs {
        record_type(names, &proc.return_type);

        for param in &proc.params {
            record_type(names, &param.ty);
        }

        collect_block_type_names(&proc.body, names);
    }
}

fn collect_block_type_names(block: &TypedBlock, names: &mut HashMap<TypeId, String>) {
    for stmt in &block.stmts {
        match &stmt.kind {
            TypedStmtKind::Local { ty, value, .. } => {
                record_type(names, ty);
                collect_expr_type_names(value, names);
            }
            TypedStmtKind::Assign { value, .. } | TypedStmtKind::Return(value) => {
                collect_expr_type_names(value, names);
            }
            TypedStmtKind::If {
                condition,
                then_block,
                else_block,
            } => {
                collect_expr_type_names(condition, names);
                collect_block_type_names(then_block, names);
                if let Some(else_block) = else_block {
                    collect_block_type_names(else_block, names);
                }
            }
            TypedStmtKind::While { condition, body } => {
                collect_expr_type_names(condition, names);
                collect_block_type_names(body, names);
            }
            TypedStmtKind::Loop { body } => collect_block_type_names(body, names),
            TypedStmtKind::Break | TypedStmtKind::Continue => {}
        }
    }

    if let Some(tail) = &block.tail {
        collect_expr_type_names(tail, names);
    }
}

fn collect_expr_type_names(expr: &TypedExpr, names: &mut HashMap<TypeId, String>) {
    names
        .entry(expr.ty)
        .or_insert_with(|| builtin_type_name(expr.ty).to_owned());

    match &expr.kind {
        TypedExprKind::Call { args, .. } => {
            for arg in args {
                collect_expr_type_names(arg, names);
            }
        }
        TypedExprKind::Unary { expr, .. } => collect_expr_type_names(expr, names),
        TypedExprKind::Binary { lhs, rhs, .. } => {
            collect_expr_type_names(lhs, names);
            collect_expr_type_names(rhs, names);
        }
        TypedExprKind::Path { .. }
        | TypedExprKind::Int { .. }
        | TypedExprKind::Text { .. }
        | TypedExprKind::Bool { .. } => {}
    }
}

fn record_type(names: &mut HashMap<TypeId, String>, ty: &TypedTypeRef) {
    names.insert(ty.id, ty.display_name.clone());
}

fn mir_type(ty: &TypedTypeRef) -> MirType {
    MirType {
        id: ty.id,
        display_name: ty.display_name.clone(),
    }
}

fn builtin_type_names() -> HashMap<TypeId, String> {
    [
        (TypeId(0), "Exit"),
        (TypeId(1), "Void"),
        (TypeId(2), "Bool"),
        (TypeId(3), "Int32"),
        (TypeId(4), "UInt64"),
        (TypeId(5), "Text"),
        (TypeId(6), "Never"),
    ]
    .into_iter()
    .map(|(id, name)| (id, name.to_owned()))
    .collect()
}

fn builtin_type_name(id: TypeId) -> &'static str {
    match id.0 {
        0 => "Exit",
        1 => "Void",
        2 => "Bool",
        3 => "Int32",
        4 => "UInt64",
        5 => "Text",
        6 => "Never",
        _ => "<unknown>",
    }
}

fn block_has_direct_break(block: &TypedBlock) -> bool {
    block.stmts.iter().any(stmt_has_direct_break)
}

fn stmt_has_direct_break(stmt: &TypedStmt) -> bool {
    match &stmt.kind {
        TypedStmtKind::Break => true,
        TypedStmtKind::If {
            then_block,
            else_block,
            ..
        } => {
            block_has_direct_break(then_block)
                || else_block.as_ref().is_some_and(block_has_direct_break)
        }
        TypedStmtKind::Local { .. }
        | TypedStmtKind::Assign { .. }
        | TypedStmtKind::While { .. }
        | TypedStmtKind::Loop { .. }
        | TypedStmtKind::Continue
        | TypedStmtKind::Return(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use torel_ir::lower_ast;

    #[test]
    fn lowers_loop_break_to_explicit_jump() {
        let mir = lower_source("Int32", "loop { break; } 42");
        let pretty = mir.pretty();

        assert!(!pretty.contains("break"));
        assert!(pretty.contains("jump bb"));
        assert!(pretty.contains("return 42"));
    }

    #[test]
    fn lowers_while_continue_without_structured_loop_terms() {
        let mir = lower_source(
            "Int32",
            "slot answer: Int32 = 0; while answer < 1 { answer = answer + 1; continue; } answer",
        );
        let pretty = mir.pretty();

        assert!(!pretty.contains("continue"));
        assert!(pretty.contains("branch"));
        assert!(pretty.contains("jump bb"));
    }

    fn lower_source(return_type: &str, body: &str) -> MirModule {
        let source = format!("unit tests.mir; export proc main() -> {return_type} {{ {body} }}");
        let tokens = torel_lexer::lex(&source);
        let ast = torel_parse::parse_source_file(&tokens).expect("source should parse");
        let hir = lower_ast(&ast);
        let typed = torel_typeck::check_types(&hir).expect("source should type-check");
        lower_to_mir(&typed).expect("MIR should lower and validate")
    }
}
