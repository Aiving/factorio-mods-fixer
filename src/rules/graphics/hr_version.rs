use crate::rules::{FixRule, PrototypeKind};
use crate::{MaybeInto, Table, Value};
use full_moon::ast;
use owo_colors::OwoColorize;

pub const FIX_HIGH_RES_GRAPHICS: FixRule = FixRule {
    enabled: false,
    kind: PrototypeKind::None,
    filter: |_, _, table| table.contains_key("hr_version"),
    action: |mod_name, _, _, table| {
        // let mut graphics_set = Table::default();

        let high_res_version = table.remove("hr_version")?;

        let high_res_table: Table = if let ast::Expression::BinaryOperator {
            lhs,
            binop: ast::BinOp::Or(_),
            ..
        } = high_res_version
        {
            if let ast::Expression::BinaryOperator {
                binop: ast::BinOp::And(_),
                rhs,
                ..
            } = *lhs
            {
                MaybeInto::try_into(Value::from_raw(*rhs)?)
            } else {
                MaybeInto::try_into(Value::from_raw(*lhs)?)
            }
        } else {
            MaybeInto::try_into(Value::from_raw(high_res_version)?)
        }?;

        let filename: String = high_res_table.get_value("filename")?;

        println!(
            "[{}] Fixed graphics for {}",
            mod_name.bright_green(),
            filename.bright_blue()
        );
        // let filename = filename.replace("/hr-", "/");

        // let pos = high_res_table.index_of("filename")?;

        // high_res_table.remove("filename");
        // high_res_table.insert_at(
        //     pos,
        //     "filename",
        //     ast::Expression::String(tokenizer::TokenReference::new(
        //         vec![],
        //         tokenizer::Token::new(tokenizer::TokenType::StringLiteral {
        //             literal: ShortString::new(filename),
        //             multi_line_depth: 0,
        //             quote_type: tokenizer::StringLiteralQuoteType::Double,
        //         }),
        //         vec![],
        //     )),
        // );

        table.clear();
        table.extend(high_res_table);

        Some(())
    },
};
