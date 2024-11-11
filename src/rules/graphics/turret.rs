use crate::rules::{FixRule, PrototypeKind};
use crate::Table;
use owo_colors::OwoColorize;

pub const FIX_TURRET_GRAPHICS: FixRule = FixRule {
    enabled: false,
    kind: PrototypeKind::Verify(|kind| {
        matches!(
            kind,
            "turret" | "electric-turret" | "ammo-turret" | "fluid-turret"
        )
    }),
    filter: |_, _, table| !table.contains_key("graphics_set") && table.contains_key("base_picture"),
    action: |mod_name, prototype_name, _, table| {
        let mut graphics_set = Table::default();

        let pos = table.index_of("base_picture")?;

        graphics_set.insert(
            "base_visualisation",
            Table::default().with_field("animation", table.remove("base_picture")?),
        );

        table.insert_at(pos, "graphics_set", graphics_set);

        println!(
            "[{}] Fixed graphics for turret prototype called {}",
            mod_name.bright_red(),
            prototype_name.bright_blue(),
        );

        Some(())
    },
};
