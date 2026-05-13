use std::collections::{HashMap, HashSet};

use thiserror::Error;
use torel_diagnostics::Span;
use torel_ir::{
    IntOverflowMode, LocalId, Mutability, ProcId, TypeId, TypedBinaryOp, TypedUnaryOp, ValueId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirModule {
    pub unit_path: Option<Vec<String>>,
    pub functions: Vec<MirFunction>,
}

impl MirModule {
    #[must_use]
    pub fn unit_name(&self) -> String {
        self.unit_path
            .as_ref()
            .map(|path| path.join("."))
            .unwrap_or_else(|| "<anonymous>".to_owned())
    }

    #[must_use]
    pub fn pretty(&self) -> String {
        let mut output = format!("mir module {}\n", self.unit_name());

        for function in &self.functions {
            output.push('\n');
            output.push_str(&function.pretty());
        }

        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFunction {
    pub id: ProcId,
    pub name: String,
    pub return_type: MirType,
    pub params: Vec<MirLocal>,
    pub locals: Vec<MirLocal>,
    pub temps: Vec<MirTemp>,
    pub blocks: Vec<MirBlock>,
    pub entry: MirBlockId,
    pub span: Span,
}

impl MirFunction {
    #[must_use]
    pub fn pretty(&self) -> String {
        let params = self
            .params
            .iter()
            .map(|param| format!("{}: {}", param.name, param.ty.display_name))
            .collect::<Vec<_>>()
            .join(", ");
        let mut output = format!(
            "fn {}({}) -> {} {{\n",
            self.name, params, self.return_type.display_name
        );

        for local in &self.locals {
            output.push_str(&format!(
                "  local %{} {}: {} ({})\n",
                local.id.0,
                local.name,
                local.ty.display_name,
                local.mutability.label()
            ));
        }

        for temp in &self.temps {
            output.push_str(&format!(
                "  temp _t{}: {}\n",
                temp.id.0, temp.ty.display_name
            ));
        }

        for block in &self.blocks {
            output.push_str(&format!("  bb{}:\n", block.id.0));

            for statement in &block.statements {
                output.push_str(&format!("    {}\n", statement.pretty()));
            }

            match &block.terminator {
                Some(terminator) => output.push_str(&format!("    {}\n", terminator.pretty())),
                None => output.push_str("    <missing terminator>\n"),
            }
        }

        output.push_str("}\n");
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirType {
    pub id: TypeId,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirLocal {
    pub id: LocalId,
    pub name: String,
    pub ty: MirType,
    pub mutability: Mutability,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirTemp {
    pub id: MirTempId,
    pub ty: MirType,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirBlockId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MirTempId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirPlace {
    Local(LocalId),
    Temp(MirTempId),
}

impl MirPlace {
    #[must_use]
    pub fn pretty(self) -> String {
        match self {
            Self::Local(id) => format!("%{}", id.0),
            Self::Temp(id) => format!("_t{}", id.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirOperand {
    Place(MirPlace),
    Int32(String),
    Bool(bool),
    BuiltinValue {
        id: ValueId,
        name: String,
        ty: MirType,
    },
    Unit,
}

impl MirOperand {
    #[must_use]
    pub fn pretty(&self) -> String {
        match self {
            Self::Place(place) => place.pretty(),
            Self::Int32(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::BuiltinValue { name, .. } => name.clone(),
            Self::Unit => "()".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirRvalue {
    Use(MirOperand),
    Unary {
        op: MirUnaryOp,
        arg: MirOperand,
    },
    Binary {
        op: MirBinaryOp,
        lhs: MirOperand,
        rhs: MirOperand,
    },
    Call {
        callee: ProcId,
        args: Vec<MirOperand>,
    },
}

impl MirRvalue {
    #[must_use]
    pub fn pretty(&self) -> String {
        match self {
            Self::Use(operand) => operand.pretty(),
            Self::Unary { op, arg } => format!("{}{}", op.symbol(), arg.pretty()),
            Self::Binary { op, lhs, rhs } => {
                format!("{} {} {}", lhs.pretty(), op.symbol(), rhs.pretty())
            }
            Self::Call { callee, args } => {
                let args = args
                    .iter()
                    .map(MirOperand::pretty)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("proc{}({args})", callee.0)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirUnaryOp {
    BoolNot,
    IntNeg { overflow: IntOverflowMode },
}

impl MirUnaryOp {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::BoolNot => "!",
            Self::IntNeg { .. } => "-",
        }
    }
}

impl From<TypedUnaryOp> for MirUnaryOp {
    fn from(value: TypedUnaryOp) -> Self {
        match value {
            TypedUnaryOp::BoolNot => Self::BoolNot,
            TypedUnaryOp::IntNeg { overflow } => Self::IntNeg { overflow },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBinaryOp {
    IntAdd { overflow: IntOverflowMode },
    IntSub { overflow: IntOverflowMode },
    IntMul { overflow: IntOverflowMode },
    IntDiv,
    IntRem,
    IntEq,
    IntNotEq,
    IntLt,
    IntLtEq,
    IntGt,
    IntGtEq,
    BoolAnd,
    BoolOr,
    SameTypeEq,
    SameTypeNotEq,
}

impl MirBinaryOp {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::IntAdd { .. } => "+",
            Self::IntSub { .. } => "-",
            Self::IntMul { .. } => "*",
            Self::IntDiv => "/",
            Self::IntRem => "%",
            Self::IntEq | Self::SameTypeEq => "==",
            Self::IntNotEq | Self::SameTypeNotEq => "!=",
            Self::IntLt => "<",
            Self::IntLtEq => "<=",
            Self::IntGt => ">",
            Self::IntGtEq => ">=",
            Self::BoolAnd => "&&",
            Self::BoolOr => "||",
        }
    }
}

impl From<TypedBinaryOp> for MirBinaryOp {
    fn from(value: TypedBinaryOp) -> Self {
        match value {
            TypedBinaryOp::IntAdd { overflow } => Self::IntAdd { overflow },
            TypedBinaryOp::IntSub { overflow } => Self::IntSub { overflow },
            TypedBinaryOp::IntMul { overflow } => Self::IntMul { overflow },
            TypedBinaryOp::IntDiv => Self::IntDiv,
            TypedBinaryOp::IntRem => Self::IntRem,
            TypedBinaryOp::IntEq => Self::IntEq,
            TypedBinaryOp::IntNotEq => Self::IntNotEq,
            TypedBinaryOp::IntLt => Self::IntLt,
            TypedBinaryOp::IntLtEq => Self::IntLtEq,
            TypedBinaryOp::IntGt => Self::IntGt,
            TypedBinaryOp::IntGtEq => Self::IntGtEq,
            TypedBinaryOp::BoolAnd => Self::BoolAnd,
            TypedBinaryOp::BoolOr => Self::BoolOr,
            TypedBinaryOp::SameTypeEq => Self::SameTypeEq,
            TypedBinaryOp::SameTypeNotEq => Self::SameTypeNotEq,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirStatement {
    pub kind: MirStatementKind,
    pub span: Span,
}

impl MirStatement {
    #[must_use]
    pub fn assign(target: MirPlace, value: MirRvalue, span: Span) -> Self {
        Self {
            kind: MirStatementKind::Assign { target, value },
            span,
        }
    }

    #[must_use]
    pub fn pretty(&self) -> String {
        match &self.kind {
            MirStatementKind::Assign { target, value } => {
                format!("{} = {}", target.pretty(), value.pretty())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirStatementKind {
    Assign { target: MirPlace, value: MirRvalue },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirTerminator {
    pub kind: MirTerminatorKind,
    pub span: Span,
}

impl MirTerminator {
    #[must_use]
    pub fn pretty(&self) -> String {
        match &self.kind {
            MirTerminatorKind::Jump(target) => format!("jump bb{}", target.0),
            MirTerminatorKind::Branch {
                condition,
                then_block,
                else_block,
            } => format!(
                "branch {} ? bb{} : bb{}",
                condition.pretty(),
                then_block.0,
                else_block.0
            ),
            MirTerminatorKind::Return { value: Some(value) } => {
                format!("return {}", value.pretty())
            }
            MirTerminatorKind::Return { value: None } => "return".to_owned(),
            MirTerminatorKind::Unreachable => "unreachable".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirTerminatorKind {
    Jump(MirBlockId),
    Branch {
        condition: MirOperand,
        then_block: MirBlockId,
        else_block: MirBlockId,
    },
    Return {
        value: Option<MirOperand>,
    },
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirBlock {
    pub id: MirBlockId,
    pub statements: Vec<MirStatement>,
    pub terminator: Option<MirTerminator>,
    pub span: Span,
}

#[derive(Debug, Error)]
pub enum MirValidationError {
    #[error("MIR module has no functions")]
    EmptyModule,

    #[error("MIR function `{function}` entry block bb{entry} does not exist")]
    MissingEntryBlock { function: String, entry: u32 },

    #[error("MIR function `{function}` has duplicate block bb{block}")]
    DuplicateBlock { function: String, block: u32 },

    #[error("MIR block bb{block} in `{function}` is missing a terminator")]
    MissingTerminator { function: String, block: u32 },

    #[error("MIR terminator in `{function}` targets missing block bb{target}")]
    MissingTarget { function: String, target: u32 },

    #[error("MIR branch in `{function}` requires `Bool`, found `{found}`")]
    BranchConditionType { function: String, found: String },

    #[error("MIR return in `{function}` expected `{expected}`, found `{found}`")]
    ReturnTypeMismatch {
        function: String,
        expected: String,
        found: String,
    },

    #[error("MIR return in `{function}` is missing value `{expected}`")]
    MissingReturnValue { function: String, expected: String },

    #[error("MIR return in `{function}` returned a value from `Void` function")]
    UnexpectedReturnValue { function: String },

    #[error("MIR place in `{function}` references unknown local %{local}")]
    UnknownLocal { function: String, local: u32 },

    #[error("MIR place in `{function}` references unknown temp _t{temp}")]
    UnknownTemp { function: String, temp: u32 },

    #[error("MIR temp _t{temp} in `{function}` is used before definition")]
    TempUsedBeforeDefinition { function: String, temp: u32 },

    #[error("MIR temp _t{temp} in `{function}` is assigned more than once")]
    TempAssignedMoreThanOnce { function: String, temp: u32 },

    #[error("MIR assignment in `{function}` writes immutable local %{local}")]
    ImmutableLocalAssigned { function: String, local: u32 },

    #[error("MIR call in `{function}` targets unknown procedure proc{proc}")]
    UnknownProc { function: String, proc: u32 },
}

pub fn validate_module(module: &MirModule) -> Result<(), MirValidationError> {
    if module.functions.is_empty() {
        return Err(MirValidationError::EmptyModule);
    }

    let procs = module
        .functions
        .iter()
        .map(|function| function.id)
        .collect::<HashSet<_>>();

    for function in &module.functions {
        validate_function(function, &procs)?;
    }

    Ok(())
}

fn validate_function(
    function: &MirFunction,
    procs: &HashSet<ProcId>,
) -> Result<(), MirValidationError> {
    let mut blocks = HashSet::new();

    for block in &function.blocks {
        if !blocks.insert(block.id) {
            return Err(MirValidationError::DuplicateBlock {
                function: function.name.clone(),
                block: block.id.0,
            });
        }
    }

    if !blocks.contains(&function.entry) {
        return Err(MirValidationError::MissingEntryBlock {
            function: function.name.clone(),
            entry: function.entry.0,
        });
    }

    let mut local_types = HashMap::new();
    let mut local_mutability = HashMap::new();
    let mut assigned_locals = HashSet::new();

    for param in &function.params {
        local_types.insert(param.id, param.ty.clone());
        local_mutability.insert(param.id, param.mutability);
        assigned_locals.insert(param.id);
    }

    for local in &function.locals {
        local_types.insert(local.id, local.ty.clone());
        local_mutability.insert(local.id, local.mutability);
    }

    let temp_types = function
        .temps
        .iter()
        .map(|temp| (temp.id, temp.ty.clone()))
        .collect::<HashMap<_, _>>();
    let mut defined_temps = HashSet::new();
    let mut state = ValidationState {
        local_types: &local_types,
        local_mutability: &local_mutability,
        assigned_locals: &mut assigned_locals,
        temp_types: &temp_types,
        defined_temps: &mut defined_temps,
        procs,
    };

    for block in &function.blocks {
        for statement in &block.statements {
            validate_statement(function, statement, &mut state)?;
        }

        let Some(terminator) = &block.terminator else {
            return Err(MirValidationError::MissingTerminator {
                function: function.name.clone(),
                block: block.id.0,
            });
        };

        validate_terminator(
            function,
            terminator,
            &blocks,
            state.local_types,
            state.temp_types,
            state.defined_temps,
        )?;
    }

    Ok(())
}

struct ValidationState<'a> {
    local_types: &'a HashMap<LocalId, MirType>,
    local_mutability: &'a HashMap<LocalId, Mutability>,
    assigned_locals: &'a mut HashSet<LocalId>,
    temp_types: &'a HashMap<MirTempId, MirType>,
    defined_temps: &'a mut HashSet<MirTempId>,
    procs: &'a HashSet<ProcId>,
}

fn validate_statement(
    function: &MirFunction,
    statement: &MirStatement,
    state: &mut ValidationState<'_>,
) -> Result<(), MirValidationError> {
    match &statement.kind {
        MirStatementKind::Assign { target, value } => {
            validate_rvalue(
                function,
                value,
                state.local_types,
                state.temp_types,
                state.defined_temps,
                state.procs,
            )?;

            match target {
                MirPlace::Local(id) => {
                    if !state.local_types.contains_key(id) {
                        return Err(MirValidationError::UnknownLocal {
                            function: function.name.clone(),
                            local: id.0,
                        });
                    }

                    if state.local_mutability.get(id) == Some(&Mutability::Immutable)
                        && state.assigned_locals.contains(id)
                    {
                        return Err(MirValidationError::ImmutableLocalAssigned {
                            function: function.name.clone(),
                            local: id.0,
                        });
                    }

                    state.assigned_locals.insert(*id);
                }
                MirPlace::Temp(id) => {
                    if !state.temp_types.contains_key(id) {
                        return Err(MirValidationError::UnknownTemp {
                            function: function.name.clone(),
                            temp: id.0,
                        });
                    }

                    if !state.defined_temps.insert(*id) {
                        return Err(MirValidationError::TempAssignedMoreThanOnce {
                            function: function.name.clone(),
                            temp: id.0,
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_rvalue(
    function: &MirFunction,
    value: &MirRvalue,
    local_types: &HashMap<LocalId, MirType>,
    temp_types: &HashMap<MirTempId, MirType>,
    defined_temps: &HashSet<MirTempId>,
    procs: &HashSet<ProcId>,
) -> Result<(), MirValidationError> {
    match value {
        MirRvalue::Use(operand) | MirRvalue::Unary { arg: operand, .. } => {
            validate_operand(function, operand, local_types, temp_types, defined_temps)
        }
        MirRvalue::Binary { lhs, rhs, .. } => {
            validate_operand(function, lhs, local_types, temp_types, defined_temps)?;
            validate_operand(function, rhs, local_types, temp_types, defined_temps)
        }
        MirRvalue::Call { callee, args } => {
            if !procs.contains(callee) {
                return Err(MirValidationError::UnknownProc {
                    function: function.name.clone(),
                    proc: callee.0,
                });
            }

            for arg in args {
                validate_operand(function, arg, local_types, temp_types, defined_temps)?;
            }

            Ok(())
        }
    }
}

fn validate_terminator(
    function: &MirFunction,
    terminator: &MirTerminator,
    blocks: &HashSet<MirBlockId>,
    local_types: &HashMap<LocalId, MirType>,
    temp_types: &HashMap<MirTempId, MirType>,
    defined_temps: &HashSet<MirTempId>,
) -> Result<(), MirValidationError> {
    match &terminator.kind {
        MirTerminatorKind::Jump(target) => validate_target(function, *target, blocks),
        MirTerminatorKind::Branch {
            condition,
            then_block,
            else_block,
        } => {
            let found = operand_type(function, condition, local_types, temp_types, defined_temps)?;
            if found.display_name != "Bool" {
                return Err(MirValidationError::BranchConditionType {
                    function: function.name.clone(),
                    found: found.display_name,
                });
            }

            validate_target(function, *then_block, blocks)?;
            validate_target(function, *else_block, blocks)
        }
        MirTerminatorKind::Return { value: Some(value) } => {
            if function.return_type.display_name == "Void" {
                return Err(MirValidationError::UnexpectedReturnValue {
                    function: function.name.clone(),
                });
            }

            let found = operand_type(function, value, local_types, temp_types, defined_temps)?;
            if found.id != function.return_type.id {
                return Err(MirValidationError::ReturnTypeMismatch {
                    function: function.name.clone(),
                    expected: function.return_type.display_name.clone(),
                    found: found.display_name,
                });
            }

            Ok(())
        }
        MirTerminatorKind::Return { value: None } => {
            if function.return_type.display_name != "Void" {
                return Err(MirValidationError::MissingReturnValue {
                    function: function.name.clone(),
                    expected: function.return_type.display_name.clone(),
                });
            }

            Ok(())
        }
        MirTerminatorKind::Unreachable => Ok(()),
    }
}

fn validate_target(
    function: &MirFunction,
    target: MirBlockId,
    blocks: &HashSet<MirBlockId>,
) -> Result<(), MirValidationError> {
    if blocks.contains(&target) {
        return Ok(());
    }

    Err(MirValidationError::MissingTarget {
        function: function.name.clone(),
        target: target.0,
    })
}

fn validate_operand(
    function: &MirFunction,
    operand: &MirOperand,
    local_types: &HashMap<LocalId, MirType>,
    temp_types: &HashMap<MirTempId, MirType>,
    defined_temps: &HashSet<MirTempId>,
) -> Result<(), MirValidationError> {
    operand_type(function, operand, local_types, temp_types, defined_temps).map(drop)
}

fn operand_type(
    function: &MirFunction,
    operand: &MirOperand,
    local_types: &HashMap<LocalId, MirType>,
    temp_types: &HashMap<MirTempId, MirType>,
    defined_temps: &HashSet<MirTempId>,
) -> Result<MirType, MirValidationError> {
    match operand {
        MirOperand::Place(MirPlace::Local(id)) => {
            local_types
                .get(id)
                .cloned()
                .ok_or_else(|| MirValidationError::UnknownLocal {
                    function: function.name.clone(),
                    local: id.0,
                })
        }
        MirOperand::Place(MirPlace::Temp(id)) => {
            if !defined_temps.contains(id) {
                return Err(MirValidationError::TempUsedBeforeDefinition {
                    function: function.name.clone(),
                    temp: id.0,
                });
            }

            temp_types
                .get(id)
                .cloned()
                .ok_or_else(|| MirValidationError::UnknownTemp {
                    function: function.name.clone(),
                    temp: id.0,
                })
        }
        MirOperand::Int32(_) => Ok(MirType {
            id: TypeId(3),
            display_name: "Int32".to_owned(),
        }),
        MirOperand::Bool(_) => Ok(MirType {
            id: TypeId(2),
            display_name: "Bool".to_owned(),
        }),
        MirOperand::BuiltinValue { ty, .. } => Ok(ty.clone()),
        MirOperand::Unit => Ok(MirType {
            id: TypeId(1),
            display_name: "Void".to_owned(),
        }),
    }
}

trait MutabilityExt {
    fn label(self) -> &'static str;
}

impl MutabilityExt for Mutability {
    fn label(self) -> &'static str {
        match self {
            Self::Immutable => "immutable",
            Self::Mutable => "mutable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_minimal_returning_function() {
        let span = Span::new(0, 1);
        let int32 = MirType {
            id: TypeId(3),
            display_name: "Int32".to_owned(),
        };
        let module = MirModule {
            unit_path: Some(vec!["tests".to_owned(), "mir".to_owned()]),
            functions: vec![MirFunction {
                id: ProcId(0),
                name: "main".to_owned(),
                return_type: int32.clone(),
                params: Vec::new(),
                locals: Vec::new(),
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: Some(MirTerminator {
                        kind: MirTerminatorKind::Return {
                            value: Some(MirOperand::Int32("42".to_owned())),
                        },
                        span,
                    }),
                    span,
                }],
                entry: MirBlockId(0),
                span,
            }],
        };

        validate_module(&module).expect("MIR should validate");
        assert!(module.pretty().contains("return 42"));
    }

    #[test]
    fn rejects_missing_terminator() {
        let span = Span::new(0, 1);
        let module = MirModule {
            unit_path: None,
            functions: vec![MirFunction {
                id: ProcId(0),
                name: "main".to_owned(),
                return_type: MirType {
                    id: TypeId(1),
                    display_name: "Void".to_owned(),
                },
                params: Vec::new(),
                locals: Vec::new(),
                temps: Vec::new(),
                blocks: vec![MirBlock {
                    id: MirBlockId(0),
                    statements: Vec::new(),
                    terminator: None,
                    span,
                }],
                entry: MirBlockId(0),
                span,
            }],
        };

        assert!(matches!(
            validate_module(&module),
            Err(MirValidationError::MissingTerminator { .. })
        ));
    }
}
