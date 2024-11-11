use super::{FixRule, PrototypeKind};
use crate::{string_expr, Table};
use owo_colors::OwoColorize;

pub const FIX_RECIPE: FixRule = FixRule {
    enabled: false,
    kind: PrototypeKind::Single("recipe"),
    filter: |prototype_name, locales, table| {
        locales
            .find_category_by_key(prototype_name)
            .is_none_or(|category| category != "recipe-name")
            && (table.contains_key("main_product") || table.contains_key("results"))
            && !table.contains_key("localised_name")
    },
    action: |mod_name, prototype_name, locales, table| {
        let name: String = if let Some(product) = table.get_value("main_product") {
            product
        } else {
            let results: Table = table.get_value("results")?;

            if results.len() != 1 {
                println!(
                    "[{}] Failure at fixing recipe called {}: results contain more than 1 element, can't choose.",
                    mod_name.bright_red(),
                    prototype_name.bright_blue(),
                );

                return None;
            }

            let result: Table = results.get_value_at(0)?;

            result.get_value("name")?
        };

        if name == prototype_name
            && locales
                .find_category_by_key(prototype_name)
                .is_some_and(|category_name| category_name == "recipe-name")
        {
            return None;
        }

        if name.contains("void") {
            println!(
                "[{}] Failure at fixing recipe called {}: it's a void!",
                mod_name.bright_red(),
                prototype_name.bright_blue(),
            );

            return None;
        } else if name.contains("slag") {
            println!(
                "[{}] Failure at fixing recipe called {} (it's a slag!)",
                mod_name.bright_red(),
                prototype_name.bright_blue(),
            );

            return None;
        }

        if let Some(category) =
            locales.find_in_categories_by_key(&["item-name", "fluid-name", "entity-name"], &name)
        {
            let mut localised = Table::default();

            localised.push(string_expr(format!("{category}.{name}")));

            let name_pos = table.index_of("name")?;

            table.insert_after(name_pos, "localised_name", localised);

            println!(
                "[{}] Fixed recipe called {}",
                mod_name.bright_green(),
                prototype_name.bright_blue()
            );
        } else {
            println!(
                "[{}] Failure at fixing recipe called {} (there is no category for {})",
                mod_name.bright_red(),
                prototype_name.bright_blue(),
                name.bright_yellow()
            );
        }

        Some(())
    },
};
