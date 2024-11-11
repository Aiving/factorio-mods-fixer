#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

mod locales;
mod rules;
mod value;

use full_moon::{ast::TableConstructor, node::Node, visitors::VisitorMut};
use locales::Locales;
use rules::{
    fluid_boxes::FIX_FLUID_BOXES,
    graphics::{
        beam::FIX_BEAM_GRAPHICS, hr_version::FIX_HIGH_RES_GRAPHICS, machine::FIX_MACHINE_GRAPHICS,
        offshore_pump::FIX_OFFSHORE_PUMP_GRAPHICS, turret::FIX_TURRET_GRAPHICS,
    },
    recipe::FIX_RECIPE,
    FixRule,
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
pub use value::*;

#[derive(Debug)]
pub struct LuaFixApplier {
    pub name: String,
    pub locales: Locales,
    pub rules: Vec<FixRule>,
}

impl LuaFixApplier {
    fn new<T: Into<String>>(name: T) -> Self {
        let rules = vec![
            FIX_RECIPE,
            FIX_BEAM_GRAPHICS,
            FIX_MACHINE_GRAPHICS,
            FIX_OFFSHORE_PUMP_GRAPHICS,
            FIX_TURRET_GRAPHICS,
            FIX_HIGH_RES_GRAPHICS,
            FIX_FLUID_BOXES,
        ];

        Self {
            name: name.into(),
            locales: Locales::default(),
            rules,
        }
    }

    fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = name.into();
    }

    fn visit_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();

        if path.extension().is_some_and(|ext| ext == "lua") {
            let file = fs::read_to_string(path)?;

            if let Ok(ast) = full_moon::parse(&file) {
                let prev_ast = ast.clone();
                let result_ast = self.visit_ast(ast);

                if !prev_ast.similar(&result_ast) {
                    let mut config = stylua_lib::Config::new();

                    config.indent_type = stylua_lib::IndentType::Spaces;
                    config.indent_width = 2;

                    fs::write(
                        path,
                        &stylua_lib::format_code(
                            &result_ast.to_string(),
                            config,
                            None,
                            stylua_lib::OutputVerification::Full,
                        )?,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn visit_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        let path = path.as_ref();

        for entry in path.read_dir()?.flatten() {
            let path = entry.path();

            if path.is_dir() && !path.ends_with("graphics") && !path.ends_with("locale") {
                self.visit_dir(path)?;
            } else {
                self.visit_file(path)?;
            }
        }

        Ok(())
    }

    fn try_visit_table(&self, mut node: Table) -> Option<Table> {
        for rule in &self.rules {
            if rule.enabled {
                if rule.kind.is_none() {
                    if rule.enabled && (rule.filter)("table", &self.locales, &node) {
                        (rule.action)(&self.name, "table", &self.locales, &mut node)?;
                    }
                } else {
                    let kind: String = node.get_value("type")?;
                    let prototype_name: String = node.get_value("name")?;

                    if rule.enabled
                        && rule.kind.verify(&kind)
                        && (rule.filter)(&prototype_name, &self.locales, &node)
                    {
                        (rule.action)(&self.name, &prototype_name, &self.locales, &mut node)?;
                    }
                }
            }
        }

        Some(node)
    }
}

impl VisitorMut for LuaFixApplier {
    fn visit_table_constructor(&mut self, node: TableConstructor) -> TableConstructor {
        if let Some(table) = self.try_visit_table(Table::new(&node)) {
            return table.into_constructor();
        }

        node
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from("/run/media/aiving/Drive/factorio");
    let data = root.join("data");
    let mods = root.join("mods");

    let mut visitor = LuaFixApplier::new("Factorio");

    visitor.locales.load_dir(data.join("base/locale"));
    visitor.locales.load_dir(data.join("core/locale"));
    visitor.locales.load_dir(data.join("quality/locale"));

    for entry in mods.read_dir()?.flatten() {
        let path = entry.path();

        if path.join("locale").is_dir() {
            visitor.locales.load_dir(path.join("locale"));
        }
    }

    for entry in PathBuf::from("/run/media/aiving/Drive/factorio-dev/AngelsMods")
        .read_dir()?
        .flatten()
    {
        let path = entry.path();

        if path.is_dir() && path.join("info.json").is_file() {
            visitor.set_name(
                path.file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap(),
            );

            visitor.visit_dir(path)?;
        }
    }

    Ok(())
}
