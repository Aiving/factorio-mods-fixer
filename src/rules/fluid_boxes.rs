use super::{FixRule, PrototypeKind};
use crate::Table;

pub const FIX_FLUID_BOXES: FixRule = FixRule {
    enabled: true,
    kind: PrototypeKind::None,
    filter: |_, _, table| {
        table.contains_key("name")
            && table.contains_key("collision_box")
            && (table.contains_key("fluid_box") || table.contains_key("fluid_boxes"))
    },
    action: |mod_name, _, _, table| {
        let name: String = table.get_value("name")?;

        let fix_fluid_box = |fluid_box: &mut Table| -> Option<()> {
            let (mut pipe_connections, pos) =
                fluid_box.remove_value_pos::<Vec<Table>>("pipe_connections")?;

            for pipe_connection in &mut pipe_connections {
                let ([mut x, mut y], pos) =
                    pipe_connection.remove_value_pos::<[f32; 2]>("position")?;

                if x % 0.5 != 0.0 {
                    x = x - (x % 0.5);
                }

                if y % 0.5 != 0.0 {
                    y = y - (y % 0.5);
                }

                pipe_connection.insert_at(pos, "position", [x, y]);
            }

            fluid_box.insert_at(pos, "pipe_connections", pipe_connections);

            Some(())
        };

        if let Some((mut fluid_box, pos)) = table.remove_value_pos::<Table>("fluid_box") {
            fix_fluid_box(&mut fluid_box);

            table.insert_at(pos, "fluid_box", fluid_box);

            println!("[{mod_name}] Fixed fluid box for {name}");
        } else {
            let (mut fluid_boxes, pos) = table.remove_value_pos::<Vec<Table>>("fluid_boxes")?;

            for fluid_box in &mut fluid_boxes {
                fix_fluid_box(fluid_box);
            }

            table.insert_at(pos, "fluid_boxes", fluid_boxes);

            println!("[{mod_name}] Fixed fluid boxes for {name}");
        }

        Some(())
    },
};
