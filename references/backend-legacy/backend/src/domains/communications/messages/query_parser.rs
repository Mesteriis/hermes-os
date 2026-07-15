use super::models::{MessageSearchMatchMode, MessageSearchQuery};
use super::search::{
    MessageSearchBoolean, MessageSearchExpression, MessageSearchField, MessageSearchPredicate,
    MessageSearchPredicateOperator,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct MessageSearchRule {
    field: MessageSearchField,
    operator: MessageSearchPredicateOperator,
    value: String,
}

pub(crate) fn parse_communication_message_search_query(
    raw_query: Option<&str>,
) -> MessageSearchQuery {
    let Some(query) = raw_query else {
        return MessageSearchQuery {
            match_mode: MessageSearchMatchMode::All,
            ..MessageSearchQuery::default()
        };
    };
    let query = query.trim();
    if query.is_empty() {
        return MessageSearchQuery {
            match_mode: MessageSearchMatchMode::All,
            ..MessageSearchQuery::default()
        };
    }
    if let Some(expression) = parse_explicit_search_expression(query) {
        return MessageSearchQuery {
            match_mode: MessageSearchMatchMode::All,
            expression: Some(expression),
            ..MessageSearchQuery::default()
        };
    }

    let mut parsed = MessageSearchQuery {
        match_mode: MessageSearchMatchMode::All,
        ..MessageSearchQuery::default()
    };
    let mut explicit_match_mode_seen = false;

    for token in tokenize_query_terms(query) {
        if let Some(match_mode) = parse_match_mode_token(&token) {
            if !explicit_match_mode_seen {
                parsed.match_mode = match_mode;
                explicit_match_mode_seen = true;
            }
            continue;
        }

        if let Some(rule) = parse_query_rule(&token) {
            add_rule_to_query(&mut parsed, rule);
            continue;
        }

        let normalized = strip_outer_quotes(&token);
        if !normalized.is_empty() {
            parsed.plain_terms.push(normalized);
        }
    }

    parsed
}

fn add_rule_to_query(parsed: &mut MessageSearchQuery, rule: MessageSearchRule) {
    if rule.value.trim().is_empty() {
        return;
    }

    match (rule.field, rule.operator) {
        (MessageSearchField::Subject, MessageSearchPredicateOperator::Contains) => {
            parsed.subject_contains.push(rule.value)
        }
        (MessageSearchField::Subject, MessageSearchPredicateOperator::Equals) => {
            parsed.subject_equals.push(rule.value)
        }
        (MessageSearchField::Body, MessageSearchPredicateOperator::Contains) => {
            parsed.body_contains.push(rule.value)
        }
        (MessageSearchField::Body, MessageSearchPredicateOperator::Equals) => {
            parsed.body_equals.push(rule.value)
        }
        (MessageSearchField::Sender, MessageSearchPredicateOperator::Contains) => {
            parsed.sender_contains.push(rule.value)
        }
        (MessageSearchField::Sender, MessageSearchPredicateOperator::Equals) => {
            parsed.sender_equals.push(rule.value)
        }
        (MessageSearchField::All, MessageSearchPredicateOperator::Contains) => {
            parsed.all_contains.push(rule.value)
        }
        (MessageSearchField::All, MessageSearchPredicateOperator::Equals) => {
            parsed.all_equals.push(rule.value)
        }
    }
}

fn parse_query_rule(token: &str) -> Option<MessageSearchRule> {
    if parse_match_mode_token(token).is_some() {
        return None;
    }

    let (field_name, operator, raw_value) = parse_rule_expression(token)?;
    let field = parse_search_field(field_name)?;
    let operator = parse_search_operator(operator)?;
    let value = strip_outer_quotes(raw_value);
    if value.is_empty() {
        return None;
    }

    Some(MessageSearchRule {
        field,
        operator,
        value,
    })
}

fn parse_match_mode_token(token: &str) -> Option<MessageSearchMatchMode> {
    let normalized = token.trim();
    if normalized.is_empty() {
        return None;
    }

    let mode_separator = normalized.find(':')?;
    if mode_separator == 0 {
        return None;
    }

    let field = normalized[..mode_separator].trim().to_lowercase();
    if field != "mode" {
        return None;
    }

    let value = normalized[(mode_separator + 1)..].trim().to_lowercase();
    match value.as_str() {
        "any" => Some(MessageSearchMatchMode::Any),
        "all" => Some(MessageSearchMatchMode::All),
        _ => None,
    }
}

fn parse_rule_expression(token: &str) -> Option<(&str, &str, &str)> {
    if let Some(index) = token.find("==")
        && index > 0
    {
        return Some((&token[0..index], "==", &token[index + 2..]));
    }
    if let Some(index) = token.find('=')
        && index > 0
    {
        return Some((&token[0..index], "=", &token[index + 1..]));
    }
    if let Some(index) = token.find(':')
        && index > 0
    {
        return Some((&token[0..index], ":", &token[index + 1..]));
    }

    None
}

fn parse_search_field(input: &str) -> Option<MessageSearchField> {
    match input.trim().to_lowercase().as_str() {
        "subject" => Some(MessageSearchField::Subject),
        "body" => Some(MessageSearchField::Body),
        "sender" | "from" => Some(MessageSearchField::Sender),
        "all" => Some(MessageSearchField::All),
        _ => None,
    }
}

fn parse_search_operator(value: &str) -> Option<MessageSearchPredicateOperator> {
    match value {
        ":" => Some(MessageSearchPredicateOperator::Contains),
        "=" | "==" => Some(MessageSearchPredicateOperator::Equals),
        _ => None,
    }
}

fn parse_explicit_search_expression(query: &str) -> Option<MessageSearchExpression> {
    let tokens = tokenize_search_expression(query);
    if !tokens.iter().any(|token| {
        matches!(
            token,
            SearchToken::OpenParen | SearchToken::CloseParen | SearchToken::And | SearchToken::Or
        )
    }) {
        return None;
    }

    let mut parser = ExplicitSearchParser::new(tokens);
    let expression = parser.parse_expression()?;
    if parser.has_remaining_tokens() {
        return None;
    }

    Some(expression)
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SearchToken {
    OpenParen,
    CloseParen,
    And,
    Or,
    Term(String),
}

fn tokenize_search_expression(query: &str) -> Vec<SearchToken> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote: Option<char> = None;

    let push_current = |tokens: &mut Vec<SearchToken>, current: &mut String| {
        let trimmed = current.trim();
        if trimmed.is_empty() {
            current.clear();
            return;
        }
        tokens.push(match trimmed {
            "AND" => SearchToken::And,
            "OR" => SearchToken::Or,
            _ => SearchToken::Term(trimmed.to_owned()),
        });
        current.clear();
    };

    for value in query.chars() {
        if matches!(value, '"' | '\'') {
            if !in_quotes {
                in_quotes = true;
                quote = Some(value);
                current.push(value);
                continue;
            }

            if Some(value) == quote {
                in_quotes = false;
                quote = None;
                current.push(value);
                continue;
            }
        }

        if !in_quotes && matches!(value, '(' | ')') {
            push_current(&mut tokens, &mut current);
            tokens.push(if value == '(' {
                SearchToken::OpenParen
            } else {
                SearchToken::CloseParen
            });
            continue;
        }

        if value.is_whitespace() && !in_quotes {
            push_current(&mut tokens, &mut current);
            continue;
        }

        current.push(value);
    }

    push_current(&mut tokens, &mut current);
    tokens
}

struct ExplicitSearchParser {
    tokens: Vec<SearchToken>,
    index: usize,
}

impl ExplicitSearchParser {
    fn new(tokens: Vec<SearchToken>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse_expression(&mut self) -> Option<MessageSearchExpression> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Option<MessageSearchExpression> {
        let mut children = vec![self.parse_and_expression()?];
        while matches!(self.peek(), Some(SearchToken::Or)) {
            self.index += 1;
            children.push(self.parse_and_expression()?);
        }
        Some(collapse_expression_group(
            MessageSearchBoolean::Or,
            children,
        ))
    }

    fn parse_and_expression(&mut self) -> Option<MessageSearchExpression> {
        let mut children = vec![self.parse_primary()?];
        while matches!(self.peek(), Some(SearchToken::And)) {
            self.index += 1;
            children.push(self.parse_primary()?);
        }
        Some(collapse_expression_group(
            MessageSearchBoolean::And,
            children,
        ))
    }

    fn parse_primary(&mut self) -> Option<MessageSearchExpression> {
        match self.peek()? {
            SearchToken::OpenParen => {
                self.index += 1;
                let expression = self.parse_expression()?;
                if !matches!(self.peek(), Some(SearchToken::CloseParen)) {
                    return None;
                }
                self.index += 1;
                Some(expression)
            }
            SearchToken::Term(_) => self.parse_term_predicate(),
            SearchToken::CloseParen | SearchToken::And | SearchToken::Or => None,
        }
    }

    fn parse_term_predicate(&mut self) -> Option<MessageSearchExpression> {
        let SearchToken::Term(raw_term) = self.tokens.get(self.index)?.clone() else {
            return None;
        };
        self.index += 1;

        if let Some(rule) = parse_query_rule(&raw_term) {
            return Some(MessageSearchExpression::Predicate(
                MessageSearchPredicate::Rule {
                    field: rule.field,
                    operator: rule.operator,
                    value: rule.value,
                },
            ));
        }

        let normalized = strip_outer_quotes(&raw_term);
        if normalized.is_empty() {
            return None;
        }
        Some(MessageSearchExpression::Predicate(
            MessageSearchPredicate::PlainTerm(normalized),
        ))
    }

    fn peek(&self) -> Option<&SearchToken> {
        self.tokens.get(self.index)
    }

    fn has_remaining_tokens(&self) -> bool {
        self.index < self.tokens.len()
    }
}

fn collapse_expression_group(
    boolean: MessageSearchBoolean,
    mut children: Vec<MessageSearchExpression>,
) -> MessageSearchExpression {
    if children.len() == 1 {
        return children.remove(0);
    }

    MessageSearchExpression::Group { boolean, children }
}

fn tokenize_query_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote: Option<char> = None;

    for value in query.chars() {
        if matches!(value, '"' | '\'') {
            if !in_quotes {
                in_quotes = true;
                quote = Some(value);
                current.push(value);
                continue;
            }

            if Some(value) == quote {
                in_quotes = false;
                quote = None;
                current.push(value);
                continue;
            }
        }

        if value.is_whitespace() && !in_quotes {
            if !current.trim().is_empty() {
                terms.push(current.trim().to_owned());
            }
            current.clear();
            continue;
        }

        current.push(value);
    }

    if !current.trim().is_empty() {
        terms.push(current.trim().to_owned());
    }

    terms
}

fn strip_outer_quotes(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let is_double = trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2;
    let is_single = trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2;
    if !(is_double || is_single) {
        return trimmed.to_owned();
    }

    let start = 1;
    let end = trimmed.len() - 1;
    trimmed[start..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_query_with_plain_terms_and_quoted_phrase() {
        let parsed =
            parse_communication_message_search_query(Some("quarterly invoice \"alpha beta\""));

        assert_eq!(
            parsed.plain_terms,
            vec![
                "quarterly".to_owned(),
                "invoice".to_owned(),
                "alpha beta".to_owned()
            ]
        );
        assert!(parsed.subject_contains.is_empty());
    }

    #[test]
    fn parse_query_with_rules_and_remaining_terms() {
        let parsed = parse_communication_message_search_query(Some(
            "from:alice subject:\"quarterly report\" open\"\" all==foo body:payment",
        ));

        assert_eq!(parsed.plain_terms, vec!["open\"\"".to_owned()]);
        assert_eq!(parsed.sender_contains, vec!["alice".to_owned()]);
        assert_eq!(parsed.subject_contains, vec!["quarterly report".to_owned()]);
        assert_eq!(parsed.all_equals, vec!["foo".to_owned()]);
        assert_eq!(parsed.body_contains, vec!["payment".to_owned()]);
    }

    #[test]
    fn parse_query_with_match_mode_and_plain_terms() {
        let parsed = parse_communication_message_search_query(Some(
            "mode:any invoice payment mode:all invoice",
        ));

        assert_eq!(parsed.match_mode, MessageSearchMatchMode::Any);
        assert_eq!(
            parsed.plain_terms,
            vec![
                "invoice".to_owned(),
                "payment".to_owned(),
                "invoice".to_owned()
            ]
        );
        assert_eq!(parsed.subject_contains, Vec::<String>::new());
    }

    #[test]
    fn parse_query_with_nested_boolean_groups() {
        let parsed = parse_communication_message_search_query(Some(
            "(subject:quarterly OR body:invoice) AND sender:alex",
        ));

        assert!(parsed.plain_terms.is_empty());
        assert!(parsed.subject_contains.is_empty());
        assert_eq!(
            parsed.expression,
            Some(MessageSearchExpression::Group {
                boolean: MessageSearchBoolean::And,
                children: vec![
                    MessageSearchExpression::Group {
                        boolean: MessageSearchBoolean::Or,
                        children: vec![
                            MessageSearchExpression::Predicate(MessageSearchPredicate::Rule {
                                field: MessageSearchField::Subject,
                                operator: MessageSearchPredicateOperator::Contains,
                                value: "quarterly".to_owned(),
                            }),
                            MessageSearchExpression::Predicate(MessageSearchPredicate::Rule {
                                field: MessageSearchField::Body,
                                operator: MessageSearchPredicateOperator::Contains,
                                value: "invoice".to_owned(),
                            }),
                        ],
                    },
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
