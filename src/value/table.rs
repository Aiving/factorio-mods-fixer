use std::{collections::HashMap, vec};

use super::{IntoExpr, MaybeInto, Value};
use full_moon::{
    ast::{self, punctuated, span, Expression},
    node::Node,
    tokenizer, ShortString,
};

#[derive(Debug)]
pub struct Field {
    at: usize,
    value: punctuated::Pair<ast::Field>,
}

impl Field {
    #[must_use]
    pub fn is_key_value(&self) -> bool {
        matches!(self.value.value(), ast::Field::NameKey { .. })
    }

    #[must_use]
    pub fn is_element(&self) -> bool {
        matches!(self.value.value(), ast::Field::NoKey(..))
    }

    #[must_use]
    pub fn get_key(&self) -> Option<String> {
        match self.value.value() {
            ast::Field::NameKey { key, .. } => {
                let tokenizer::TokenType::Identifier { identifier } = key.token_type() else {
                    unreachable!()
                };

                Some(identifier.to_string())
            }
            ast::Field::NoKey(_) => Some(self.at.to_string()),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_value(&self) -> Option<&Expression> {
        match self.value.value() {
            ast::Field::NameKey { value, .. } | ast::Field::NoKey(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_key_value(&self) -> Option<(String, &Expression)> {
        match self.value.value() {
            ast::Field::NameKey { key, value, .. } => {
                let tokenizer::TokenType::Identifier { identifier } = key.token_type() else {
                    unreachable!()
                };

                Some((identifier.to_string(), value))
            }
            ast::Field::NoKey(value) => Some((self.at.to_string(), value)),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_trailing_trivia(&self) -> Vec<&tokenizer::Token> {
        self.value.value().surrounding_trivia().0
    }

    #[must_use]
    pub const fn from_raw(at: usize, value: punctuated::Pair<ast::Field>) -> Self {
        Self { at, value }
    }

    #[must_use]
    pub fn into_value(self) -> ast::Expression {
        match self.value.into_value() {
            ast::Field::NameKey { value, .. } | ast::Field::NoKey(value) => value,
            _ => unreachable!(),
        }
    }

    #[must_use]
    pub fn into_pair(self) -> punctuated::Pair<ast::Field> {
        self.value
    }

    #[must_use]
    pub fn into_raw(self) -> ast::Field {
        self.value.into_value()
    }
}

#[derive(Debug)]
pub struct Table {
    braces: span::ContainedSpan,
    fields: Vec<Field>,
}

impl Table {
    #[must_use]
    pub fn new(table: &ast::TableConstructor) -> Self {
        Self {
            braces: table.braces().clone(),
            fields: table
                .fields()
                .clone()
                .into_pairs()
                .enumerate()
                .map(|(at, field)| Field::from_raw(at, field))
                .collect(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn clear(&mut self) {
        self.fields.clear();
    }

    pub fn extend(&mut self, table: Self) {
        self.fields.extend(table.fields);
    }

    pub fn remove(&mut self, field: impl AsRef<str>) -> Option<ast::Expression> {
        let position = self
            .fields
            .iter()
            .position(|value| value.get_key().is_some_and(|key| key == field.as_ref()));

        position.map(|position| self.fields.remove(position).into_value())
    }

    pub fn remove_value<T>(&mut self, field: impl AsRef<str>) -> Option<T>
    where
        Value: MaybeInto<T>,
    {
        let position = self
            .fields
            .iter()
            .position(|value| value.get_key().is_some_and(|key| key == field.as_ref()));

        position.and_then(|position| {
            Value::from_raw(self.fields.remove(position).into_value()).and_then(MaybeInto::try_into)
        })
    }

    pub fn remove_value_pos<T>(&mut self, field: impl AsRef<str>) -> Option<(T, usize)>
    where
        Value: MaybeInto<T>,
    {
        let position = self
            .fields
            .iter()
            .position(|value| value.get_key().is_some_and(|key| key == field.as_ref()));

        position.and_then(|position| {
            Value::from_raw(self.fields.remove(position).into_value())
                .and_then(MaybeInto::try_into)
                .map(|v| (v, position))
        })
    }

    pub fn index_of(&self, name: impl AsRef<str>) -> Option<usize> {
        self.fields
            .iter()
            .position(|field| field.get_key().is_some_and(|key| key == name.as_ref()))
    }

    pub fn insert_after<T: IntoExpr>(&mut self, pos: usize, name: impl AsRef<str>, value: T) {
        self.insert_at(pos + 1, name, value);
    }

    pub fn insert_before<T: IntoExpr>(&mut self, pos: usize, name: impl AsRef<str>, value: T) {
        self.insert_at(pos - 1, name, value);
    }

    pub fn insert_at<T: IntoExpr>(&mut self, pos: usize, name: impl AsRef<str>, value: T) {
        let value = value.into_expr();
        let trivia = self.fields.last().map_or_else(Vec::new, |last_field| {
            last_field
                .get_trailing_trivia()
                .into_iter()
                .cloned()
                .collect()
        });

        let field = ast::Field::NameKey {
            key: tokenizer::TokenReference::new(
                trivia,
                tokenizer::Token::new(tokenizer::TokenType::Identifier {
                    identifier: ShortString::new(name.as_ref()),
                }),
                vec![],
            ),
            equal: tokenizer::TokenReference::symbol(" = ").unwrap(),
            value,
        };

        self.fields.insert(
            pos,
            Field::from_raw(
                self.fields.len(),
                punctuated::Pair::new(
                    field,
                    Some(tokenizer::TokenReference::symbol(",\n").unwrap()),
                ),
            ),
        );
    }

    pub fn insert<T: IntoExpr>(&mut self, name: impl AsRef<str>, value: T) {
        let value = value.into_expr();
        let trivia = self.fields.last().map_or_else(Vec::new, |last_field| {
            last_field
                .get_trailing_trivia()
                .into_iter()
                .cloned()
                .collect()
        });

        let field = ast::Field::NameKey {
            key: tokenizer::TokenReference::new(
                trivia,
                tokenizer::Token::new(tokenizer::TokenType::Identifier {
                    identifier: ShortString::new(name.as_ref()),
                }),
                vec![],
            ),
            equal: tokenizer::TokenReference::symbol(" = ").unwrap(),
            value,
        };

        self.fields.push(Field::from_raw(
            self.fields.len(),
            punctuated::Pair::new(
                field,
                Some(tokenizer::TokenReference::symbol(",\n").unwrap()),
            ),
        ));
    }

    pub fn push<T: IntoExpr>(&mut self, value: T) {
        self.fields.push(Field::from_raw(
            self.fields.len(),
            punctuated::Pair::new(
                ast::Field::NoKey(value.into_expr()),
                Some(tokenizer::TokenReference::symbol(",\n").unwrap()),
            ),
        ));
    }

    pub fn get_expr(&self, name: impl AsRef<str>) -> Option<&Expression> {
        self.fields.iter().find_map(|field| {
            let (key, value) = field.get_key_value()?;

            if key == name.as_ref() {
                Some(value)
            } else {
                None
            }
        })
    }

    pub fn get_value<T>(&self, name: impl AsRef<str>) -> Option<T>
    where
        Value: MaybeInto<T>,
    {
        self.fields.iter().find_map(|field| {
            let (key, value) = field.get_key_value()?;

            if key == name.as_ref() {
                MaybeInto::try_into(Value::from_raw(value.clone())?)
            } else {
                None
            }
        })
    }

    pub fn get_value_at<T>(&self, pos: usize) -> Option<T>
    where
        Value: MaybeInto<T>,
    {
        self.fields
            .get(pos)
            .and_then(Field::get_value)
            .and_then(|value| MaybeInto::try_into(Value::from_raw(value.clone())?))
    }

    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.fields
            .iter()
            .any(|field| field.get_key().is_some_and(|key| key == name.as_ref()))
    }

    pub fn contains_keys(&self, names: &[impl AsRef<str>]) -> bool {
        let mut names = names
            .iter()
            .map(|value| (value.as_ref().to_string(), false))
            .collect::<HashMap<_, _>>();

        for field in self.fields.iter().filter_map(Field::get_key) {
            if let Some(name) = names.get_mut(&field) {
                *name = true;
            }

            if names.values().all(|value| *value) {
                break;
            }
        }

        names.into_values().all(|value| value)
    }

    #[must_use]
    pub fn with_field<T: IntoExpr>(mut self, name: impl AsRef<str>, value: T) -> Self {
        self.insert(name, value);

        self
    }

    pub fn into_constructor(self) -> ast::TableConstructor {
        ast::TableConstructor::new()
            .with_braces(self.braces)
            .with_fields(self.fields.into_iter().map(Field::into_pair).collect())
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new(&ast::TableConstructor::new())
    }
}

impl IntoIterator for Table {
    type Item = Field;

    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl IntoExpr for Table {
    fn into_expr(self) -> ast::Expression {
        ast::Expression::TableConstructor(self.into_constructor())
    }
}

impl<T: IntoExpr> IntoExpr for Vec<T> {
    fn into_expr(self) -> ast::Expression {
        let mut table = Table::default();

        for value in self {
            table.push(value);
        }

        table.into_expr()
    }
}

impl<T: IntoExpr, const N: usize> IntoExpr for [T; N] {
    fn into_expr(self) -> ast::Expression {
        let mut table = Table::default();

        for value in self {
            table.push(value);
        }

        table.into_expr()
    }
}
