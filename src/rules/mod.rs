use crate::{locales::Locales, Table};

pub mod fluid_boxes;
pub mod graphics;
pub mod recipe;

#[derive(Debug)]
pub enum PrototypeKind {
    Single(&'static str),
    Verify(fn(&str) -> bool),
    /// For tables where there is no `type` field
    None,
}

impl PrototypeKind {
    pub fn verify(&self, kind: &str) -> bool {
        match self {
            Self::Single(value) => value == &kind,
            Self::Verify(func) => func(kind),
            Self::None => false,
        }
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug)]
pub struct FixRule {
    pub enabled: bool,
    pub kind: PrototypeKind,
    pub filter: fn(&str, &Locales, &Table) -> bool,
    pub action: fn(&str, &str, &Locales, &mut Table) -> Option<()>,
}

