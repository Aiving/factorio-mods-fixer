use crate::rules::{FixRule, PrototypeKind};
use crate::Table;
use owo_colors::OwoColorize;

pub const FIX_MACHINE_GRAPHICS: FixRule = FixRule {
    enabled: false,
    kind: PrototypeKind::Verify(|kind| {
        matches!(
            kind,
            "assembling-machine" | "furnace" | "mining-drill" | "rocket-silo"
        )
    }),
    filter: |_, _, table| {
        !table.contains_key("graphics_set")
            && (table.contains_key("animation")
                || table.contains_key("idle_animation")
                || table.contains_key("working_visualisations"))
    },
    action: |mod_name, prototype_name, _, table| {
        let mut graphics_set = Table::default();

        let pos = table
            .index_of("animation")
            .or_else(|| table.index_of("idle_animation"))
            .or_else(|| table.index_of("working_visualisations"))?;

        if let Some(value) = table.remove("animation") {
            graphics_set.insert("animation", value);
        }

        if let Some(value) = table.remove("idle_animation") {
            graphics_set.insert("idle_animation", value);
        }

        if let Some(value) = table.remove("working_visualisations") {
            graphics_set.insert("working_visualisations", value);
        }

        table.insert_at(pos, "graphics_set", graphics_set);

        println!(
            "[{}] Fixed graphics for assembling machine prototype called {}",
            mod_name.bright_red(),
            prototype_name.bright_blue(),
        );

        Some(())
    },
};
