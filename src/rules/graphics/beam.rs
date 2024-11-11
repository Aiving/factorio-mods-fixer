use crate::rules::{FixRule, PrototypeKind};
use crate::Table;
use owo_colors::OwoColorize;

pub const FIX_BEAM_GRAPHICS: FixRule = FixRule {
    enabled: false,
    kind: PrototypeKind::Single("beam"),
    filter: |_, _, table| !table.contains_key("graphics_set"),
    action: |mod_name, prototype_name, _, table| {
        let mut animation = Table::default();

        let pos = table
            .index_of("start")
            .or_else(|| table.index_of("ending"))
            .or_else(|| table.index_of("head"))
            .or_else(|| table.index_of("tail"))
            .or_else(|| table.index_of("body"))?;

        if let Some(value) = table.remove("start") {
            animation.insert("start", value);
        }

        if let Some(value) = table.remove("ending") {
            animation.insert("ending", value);
        }

        if let Some(value) = table.remove("head") {
            animation.insert("head", value);
        }

        if let Some(value) = table.remove("tail") {
            animation.insert("tail", value);
        }

        if let Some(value) = table.remove("body") {
            animation.insert("body", value);
        }

        table.insert_at(
            pos,
            "graphics_set",
            Table::default().with_field("beam", animation),
        );

        println!(
            "[{}] Fixed graphics for beam prototype called {}",
            mod_name.bright_red(),
            prototype_name.bright_blue(),
        );

        Some(())
    },
};
