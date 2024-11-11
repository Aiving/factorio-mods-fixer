use crate::rules::{FixRule, PrototypeKind};
use crate::Table;

pub const FIX_OFFSHORE_PUMP_GRAPHICS: FixRule = FixRule {
    enabled: false,
    kind: PrototypeKind::Single("offshore-pump"),
    filter: |_, _, table| !table.contains_key("graphics_set"),
    action: |_, _, _, table| {
        let mut graphics_set = Table::default();

        let pos = table.index_of("picture");

        if let Some(value) = table.remove("picture") {
            graphics_set.insert("base_pictures", value);
        }

        if let Some(pos) = pos {
            table.insert_at(pos, "graphics_set", graphics_set);
        } else {
            table.insert("graphics_set", graphics_set);
        }

        Some(())
    },
};
