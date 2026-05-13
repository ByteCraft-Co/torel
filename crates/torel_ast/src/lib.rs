#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub unit: Option<UnitDecl>,
    pub items: Vec<Item>,
}

impl SourceFile {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            unit: None,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDecl {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Placeholder,
}
