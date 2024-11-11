use full_moon::{ast, tokenizer, ShortString};

mod table;

pub use table::{Field, Table};

#[derive(Debug)]
pub enum Value {
    Bool(bool),
    String(String),
    Number(f32),
    Table(Box<Table>),
    Null,
}

impl Value {
    /// Returns `true` if the value is [`Null`].
    ///
    /// [`Null`]: Value::Null
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    #[must_use]
    pub fn from_raw(value: ast::Expression) -> Option<Self> {
        match value {
            ast::Expression::TableConstructor(table_constructor) => {
                Some(Self::Table(Box::new(Table::new(&table_constructor))))
            }
            ast::Expression::Number(number) => {
                if let tokenizer::TokenType::Number { text } = number.token_type() {
                    Some(Self::Number(text.parse().unwrap()))
                } else {
                    None
                }
            }
            ast::Expression::String(string) => {
                if let tokenizer::TokenType::StringLiteral { literal, .. } = string.token_type() {
                    Some(Self::String(literal.to_string()))
                } else {
                    None
                }
            }
            ast::Expression::Symbol(symbol) => {
                if let tokenizer::TokenType::Symbol { symbol } = symbol.token_type() {
                    match symbol {
                        tokenizer::Symbol::True => Some(Self::Bool(true)),
                        tokenizer::Symbol::False => Some(Self::Bool(false)),
                        tokenizer::Symbol::Nil => Some(Self::Null),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            ast::Expression::UnaryOperator {
                unop: ast::UnOp::Minus(_),
                expression,
            } => {
                if let ast::Expression::Number(number) = *expression {
                    if let tokenizer::TokenType::Number { text } = number.token_type() {
                        Some(Self::Number(-text.parse::<f32>().unwrap()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub trait MaybeInto<T> {
    fn try_into(self) -> Option<T>;
}

impl MaybeInto<bool> for Value {
    fn try_into(self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl MaybeInto<String> for Value {
    fn try_into(self) -> Option<String> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl MaybeInto<f32> for Value {
    fn try_into(self) -> Option<f32> {
        if let Self::Number(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl MaybeInto<Table> for Value {
    fn try_into(self) -> Option<Table> {
        if let Self::Table(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

impl<T> MaybeInto<Vec<T>> for Value
where
    Self: MaybeInto<T>,
{
    fn try_into(self) -> Option<Vec<T>> {
        if let Self::Table(v) = self {
            Some(
                v.into_iter()
                    .filter_map(|field| {
                        if field.is_element() {
                            let value = Self::from_raw(field.into_value())?;

                            MaybeInto::try_into(value)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
    }
}

impl<T, const N: usize> MaybeInto<[T; N]> for Value
where
    Self: MaybeInto<T>,
{
    fn try_into(self) -> Option<[T; N]> {
        if let Self::Table(v) = self {
            let values = v
                .into_iter()
                .filter_map(|field| {
                    if field.is_element() {
                        let value = Self::from_raw(field.into_value())?;

                        MaybeInto::try_into(value)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .try_into()
                .ok()?;

            Some(values)
        } else {
            None
        }
    }
}

pub trait IntoExpr {
    fn into_expr(self) -> ast::Expression;
}

impl IntoExpr for Value {
    fn into_expr(self) -> ast::Expression {
        match self {
            Self::Bool(v) => v.into_expr(),
            Self::String(v) => v.into_expr(),
            Self::Number(v) => v.into_expr(),
            Self::Table(table) => table.into_expr(),
            Self::Null => ().into_expr(),
        }
    }
}

impl IntoExpr for bool {
    fn into_expr(self) -> ast::Expression {
        ast::Expression::Symbol(tokenizer::TokenReference::new(
            vec![],
            tokenizer::Token::new(tokenizer::TokenType::Symbol {
                symbol: if self {
                    tokenizer::Symbol::True
                } else {
                    tokenizer::Symbol::False
                },
            }),
            vec![],
        ))
    }
}

impl IntoExpr for String {
    fn into_expr(self) -> ast::Expression {
        ast::Expression::Number(tokenizer::TokenReference::new(
            vec![],
            tokenizer::Token::new(tokenizer::TokenType::StringLiteral {
                literal: ShortString::new(self),
                multi_line_depth: 0,
                quote_type: tokenizer::StringLiteralQuoteType::Double,
            }),
            vec![],
        ))
    }
}

impl IntoExpr for f32 {
    fn into_expr(self) -> ast::Expression {
        ast::Expression::Symbol(tokenizer::TokenReference::new(
            vec![],
            tokenizer::Token::new(tokenizer::TokenType::Number {
                text: ShortString::new(self.to_string()),
            }),
            vec![],
        ))
    }
}

impl<T: IntoExpr> IntoExpr for Option<T> {
    fn into_expr(self) -> ast::Expression {
        self.map_or_else(|| ().into_expr(), IntoExpr::into_expr)
    }
}

impl IntoExpr for () {
    fn into_expr(self) -> ast::Expression {
        ast::Expression::Symbol(tokenizer::TokenReference::new(
            vec![],
            tokenizer::Token::new(tokenizer::TokenType::Symbol {
                symbol: tokenizer::Symbol::Nil,
            }),
            vec![],
        ))
    }
}

impl IntoExpr for ast::Expression {
    fn into_expr(self) -> ast::Expression {
        self
    }
}

pub fn string_expr<T: Into<String> + AsRef<str>>(value: T) -> ast::Expression {
    ast::Expression::String(tokenizer::TokenReference::new(
        vec![],
        tokenizer::Token::new(tokenizer::TokenType::StringLiteral {
            literal: ShortString::new(value),
            multi_line_depth: 0,
            quote_type: tokenizer::StringLiteralQuoteType::Double,
        }),
        vec![],
    ))
}
