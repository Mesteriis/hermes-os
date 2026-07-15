use sqlx::{Postgres, QueryBuilder};

use super::models::MessageSearchQuery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSearchField {
    Subject,
    Body,
    Sender,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSearchPredicateOperator {
    Contains,
    Equals,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageSearchPredicate {
    PlainTerm(String),
    Rule {
        field: MessageSearchField,
        operator: MessageSearchPredicateOperator,
        value: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageSearchBoolean {
    And,
    Or,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageSearchExpression {
    Predicate(MessageSearchPredicate),
    Group {
        boolean: MessageSearchBoolean,
        children: Vec<MessageSearchExpression>,
    },
}

impl MessageSearchExpression {
    pub fn term_count(&self) -> usize {
        match self {
            Self::Predicate(_) => 1,
            Self::Group { children, .. } => children.iter().map(Self::term_count).sum(),
        }
    }
}

pub fn append_message_search_filter<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    alias: &str,
    search: &MessageSearchQuery,
) {
    let Some(expression) = effective_message_search_expression(search) else {
        return;
    };

    builder.push(" AND ");
    push_expression(builder, alias, &expression);
}

pub fn effective_message_search_expression(
    search: &MessageSearchQuery,
) -> Option<MessageSearchExpression> {
    if let Some(expression) = search.expression.clone() {
        return Some(expression);
    }

    let boolean = if search.match_mode.is_any() {
        MessageSearchBoolean::Or
    } else {
        MessageSearchBoolean::And
    };
    let mut children = Vec::new();
    children.extend(search.plain_terms.iter().map(|value| {
        MessageSearchExpression::Predicate(MessageSearchPredicate::PlainTerm(value.clone()))
    }));
    extend_rule_terms(
        &mut children,
        MessageSearchField::Subject,
        MessageSearchPredicateOperator::Contains,
        &search.subject_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Subject,
        MessageSearchPredicateOperator::Equals,
        &search.subject_equals,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Body,
        MessageSearchPredicateOperator::Contains,
        &search.body_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Body,
        MessageSearchPredicateOperator::Equals,
        &search.body_equals,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Sender,
        MessageSearchPredicateOperator::Contains,
        &search.sender_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::Sender,
        MessageSearchPredicateOperator::Equals,
        &search.sender_equals,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::All,
        MessageSearchPredicateOperator::Contains,
        &search.all_contains,
    );
    extend_rule_terms(
        &mut children,
        MessageSearchField::All,
        MessageSearchPredicateOperator::Equals,
        &search.all_equals,
    );

    match children.len() {
        0 => None,
        1 => children.into_iter().next(),
        _ => Some(MessageSearchExpression::Group { boolean, children }),
    }
}

fn extend_rule_terms(
    children: &mut Vec<MessageSearchExpression>,
    field: MessageSearchField,
    operator: MessageSearchPredicateOperator,
    values: &[String],
) {
    children.extend(values.iter().map(|value| {
        MessageSearchExpression::Predicate(MessageSearchPredicate::Rule {
            field,
            operator,
            value: value.clone(),
        })
    }));
}

fn push_expression<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    alias: &str,
    expression: &MessageSearchExpression,
) {
    match expression {
        MessageSearchExpression::Predicate(predicate) => push_predicate(builder, alias, predicate),
        MessageSearchExpression::Group { boolean, children } => {
            builder.push("(");
            for (index, child) in children.iter().enumerate() {
                if index > 0 {
                    builder.push(match boolean {
                        MessageSearchBoolean::And => " AND ",
                        MessageSearchBoolean::Or => " OR ",
                    });
                }
                push_expression(builder, alias, child);
            }
            builder.push(")");
        }
    }
}

fn push_predicate<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    alias: &str,
    predicate: &MessageSearchPredicate,
) {
    match predicate {
        MessageSearchPredicate::PlainTerm(value) => {
            push_contains_predicate(builder, &combined_search_target(alias), value)
        }
        MessageSearchPredicate::Rule {
            field,
            operator,
            value,
        } => match (field, operator) {
            (MessageSearchField::Subject, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &coalesce_column(alias, "subject"), value)
            }
            (MessageSearchField::Subject, MessageSearchPredicateOperator::Equals) => {
                push_equals_predicate(builder, &coalesce_column(alias, "subject"), value)
            }
            (MessageSearchField::Body, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &coalesce_column(alias, "body_text"), value)
            }
            (MessageSearchField::Body, MessageSearchPredicateOperator::Equals) => {
                push_equals_predicate(builder, &coalesce_column(alias, "body_text"), value)
            }
            (MessageSearchField::Sender, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &coalesce_column(alias, "sender"), value)
            }
            (MessageSearchField::Sender, MessageSearchPredicateOperator::Equals) => {
                push_equals_predicate(builder, &coalesce_column(alias, "sender"), value)
            }
            (MessageSearchField::All, MessageSearchPredicateOperator::Contains) => {
                push_contains_predicate(builder, &combined_search_target(alias), value)
            }
            (MessageSearchField::All, MessageSearchPredicateOperator::Equals) => {
                builder.push("(");
                push_equals_predicate(builder, &coalesce_column(alias, "subject"), value);
                builder.push(" OR ");
                push_equals_predicate(builder, &coalesce_column(alias, "sender"), value);
                builder.push(" OR ");
                push_equals_predicate(builder, &coalesce_column(alias, "body_text"), value);
                builder.push(" OR ");
                push_equals_predicate(
                    builder,
                    &coalesce_column(alias, "provider_record_id"),
                    value,
                );
                builder.push(" OR ");
                push_equals_predicate(
                    builder,
                    &coalesce_column(alias, "sender_display_name"),
                    value,
                );
                builder.push(")");
            }
        },
    }
}

fn push_contains_predicate<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    target_sql: &str,
    value: &str,
) {
    builder.push("lower(");
    builder.push(target_sql);
    builder.push(") LIKE '%' || lower(");
    builder.push_bind(value.to_owned());
    builder.push(") || '%'");
}

fn push_equals_predicate<'args>(
    builder: &mut QueryBuilder<'args, Postgres>,
    target_sql: &str,
    value: &str,
) {
    builder.push("lower(");
    builder.push(target_sql);
    builder.push(") = lower(");
    builder.push_bind(value.to_owned());
    builder.push(")");
}

fn coalesce_column(alias: &str, column: &str) -> String {
    format!("coalesce({alias}.{column}, '')")
}

fn combined_search_target(alias: &str) -> String {
    format!(
        "concat_ws(' ', {}, {}, {}, {}, {})",
        coalesce_column(alias, "subject"),
        coalesce_column(alias, "sender"),
        coalesce_column(alias, "body_text"),
        coalesce_column(alias, "provider_record_id"),
        coalesce_column(alias, "sender_display_name")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_expression_builds_legacy_any_group() {
        let query = MessageSearchQuery {
            plain_terms: vec!["invoice".to_owned()],
            sender_contains: vec!["alex".to_owned()],
            match_mode: MessageSearchMatchMode::Any,
            ..MessageSearchQuery::default()
        };

        assert_eq!(
            effective_message_search_expression(&query),
            Some(MessageSearchExpression::Group {
                boolean: MessageSearchBoolean::Or,
                children: vec![
                    MessageSearchExpression::Predicate(MessageSearchPredicate::PlainTerm(
                        "invoice".to_owned()
                    )),
                    MessageSearchExpression::Predicate(MessageSearchPredicate::Rule {
                        field: MessageSearchField::Sender,
                        operator: MessageSearchPredicateOperator::Contains,
                        value: "alex".to_owned(),
                    }),
                ],
            })
        );
    }
}
