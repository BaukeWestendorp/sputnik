use crate::css::tokenizer::Token;

/// https://drafts.csswg.org/css-syntax-3/#css-stylesheet
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    pub location: Option<String>,
    pub rules: Vec<QualifiedRule>,
}

impl StyleSheet {
    pub fn new(location: Option<&str>) -> Self {
        Self {
            location: location.map(|s| s.to_string()),
            rules: vec![],
        }
    }
}

/// https://drafts.csswg.org/css-syntax-3/#css-rule
#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    AtRule(AtRule),
    QualifiedRule(QualifiedRule),
}

/// https://drafts.csswg.org/css-syntax-3/#at-rule
#[derive(Debug, Clone, PartialEq)]
pub struct AtRule {
    // FIXME: Implement members
}

/// https://www.w3.org/TR/css-syntax-3/#qualified-rule
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    pub prelude: Vec<ComponentValue>,
    pub declarations: Vec<Declaration>,
    pub child_rules: Vec<QualifiedRule>,
}

/// https://drafts.csswg.org/css-syntax-3/#declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: Vec<ComponentValue>,
    pub important: bool,
    pub original_text: Option<String>,
}

/// https://www.w3.org/TR/css-syntax-3/#component-value
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// https://www.w3.org/TR/css-syntax-3/#preserved-tokens
    PreservedToken(Token),
    /// https://www.w3.org/TR/css-syntax-3/#function
    Function(Function),
    SimpleBlock(SimpleBlock),
}

/// https://drafts.csswg.org/css-syntax-3/#function
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub value: Vec<ComponentValue>,
}

/// https://www.w3.org/TR/css-syntax-3/#simple-block
#[derive(Debug, Clone, PartialEq)]
pub struct SimpleBlock {
    pub associated_token: Token,
    pub values: Vec<ComponentValue>,
}
