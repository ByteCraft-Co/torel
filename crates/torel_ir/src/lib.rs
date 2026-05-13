use torel_ast::SourceFile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirModule {
    pub unit_path: Option<Vec<String>>,
    pub item_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedModule {
    pub unit_path: Option<Vec<String>>,
    pub item_count: usize,
}

#[must_use]
pub fn lower_ast(source_file: &SourceFile) -> HirModule {
    HirModule {
        unit_path: source_file.unit.as_ref().map(|unit| unit.path.clone()),
        item_count: source_file.items.len(),
    }
}
