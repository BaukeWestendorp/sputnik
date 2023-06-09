use crate::css::parser::types::{ComponentValue, Function};
use crate::css::tokenizer::Token;
use crate::html::parser::log_parser_error;

use super::token_streams::{ProcessResult, TokenStream};
use super::types::{Declaration, QualifiedRule, SimpleBlock};

/// 5.5.1. Consume a stylesheet's contents
///
/// https://drafts.csswg.org/css-syntax-3/#consume-a-stylesheets-contents
pub(crate) fn consume_a_stylesheets_content(input: &TokenStream) -> Vec<QualifiedRule> {
    // Let rules be an initially empty list of rules.
    let mut rules = vec![];

    // Process input:
    input.process(|token| match token {
        Token::Whitespace => {
            // Discard a token from input.
            input.discard_a_token();
            ProcessResult::Continue
        }
        Token::EndOfFile => {
            // Return rules.
            ProcessResult::Return(rules.clone())
        }
        Token::Cdo | Token::Cdc => {
            // Discard a token from input.
            input.discard_a_token();
            ProcessResult::Continue
        }
        Token::AtKeyword { .. } => todo!(),
        _ => {
            // Consume a qualified rule from input.
            let qualified_rule = consume_a_qualified_rule(input, None, false);
            // If anything is returned, append it to rules.
            if let Some(qualified_rule) = qualified_rule {
                rules.push(qualified_rule);
            }
            ProcessResult::Continue
        }
    })
}

/// 5.5.3. Consume a qualified rule
///
/// https://drafts.csswg.org/css-syntax-3/#consume-qualified-rule
pub(crate) fn consume_a_qualified_rule(
    input: &TokenStream,
    stop_token: Option<&Token>,
    nested: bool,
) -> Option<QualifiedRule> {
    // Let rule be a new qualified rule with its prelude, declarations, and child rules all initially set to empty lists.
    let mut rule = QualifiedRule {
        prelude: vec![],
        declarations: vec![],
        child_rules: vec![],
    };

    input.process(|token| match token {
        token if token == &Token::EndOfFile || Some(token) == stop_token => {
            // This is a parse error.
            log_parser_error!("Unexpected EOF while parsing a qualified rule.");

            // Return nothing.
            ProcessResult::Return(None)
        }
        Token::RightCurlyBracket => {
            // This is a parse error.
            log_parser_error!("Unexpected '{{' while parsing a qualified rule.");

            // If nested is true, return nothing.
            if nested {
                return ProcessResult::Return(None);
            }

            // Otherwise, consume a token and append the result to rule’s prelude.
            rule.prelude.push(ComponentValue::PreservedToken(
                input.consume_a_token().clone(),
            ));

            ProcessResult::Continue
        }
        Token::LeftCurlyBracket => {
            // FIXME: If the first two non-<whitespace-token> values of rule’s prelude are an <ident-token>
            //        whose value starts with "--" followed by a <colon-token>,
            //        consume the remnants of a bad declaration from input, with nested,
            //        and return nothing.

            // Otherwise, consume a block from input, and assign the results to rule’s lists of declarations and child rules.
            let (decls, rules) = consume_a_block(input);
            rule.declarations = decls;
            rule.child_rules = rules;

            // If rule is valid in the current context, return it; otherwise return nothing.
            // FIXME: Check if rule is valid in the current context.
            ProcessResult::Return(Some(rule.clone()))
        }
        _ => {
            // Consume a component value from input
            let component_value = consume_a_component_value(input);
            // and append the result to rule’s prelude.
            rule.prelude.push(component_value);

            ProcessResult::Continue
        }
    })
}

/// 5.5.4. Consume a block
///
/// https://drafts.csswg.org/css-syntax-3/#consume-block
pub(crate) fn consume_a_block(input: &TokenStream) -> (Vec<Declaration>, Vec<QualifiedRule>) {
    // Assert: The next token is a <{-token>.
    assert_eq!(input.next_token(), &Token::LeftCurlyBracket);

    // Let decls be an empty list of declarations, and rules be an empty list of rules.
    // NOTE: We create them from the return value of consume_a_blocks_contents instead.

    // Discard a token from input.
    input.discard_a_token();
    // Consume a block’s contents from input and assign the results to decls and rules.
    let (decls, rules) = consume_a_blocks_contents(input);

    // Discard a token from input.
    input.discard_a_token();

    // Return decls and rules.
    (decls, rules)
}

/// 5.5.5. Consume a block's contents
///
/// https://drafts.csswg.org/css-syntax-3/#consume-block-contents
pub(crate) fn consume_a_blocks_contents(
    input: &TokenStream,
) -> (Vec<Declaration>, Vec<QualifiedRule>) {
    let mut decls = vec![];
    let mut rules = vec![];

    input.process(|token| match token {
        Token::Whitespace | Token::Semicolon => {
            // Discard a token from input.
            input.discard_a_token();
            ProcessResult::Continue
        }
        Token::EndOfFile | Token::RightCurlyBracket => {
            // Return decls and rules.
            ProcessResult::Return((decls.clone(), rules.clone()))
        }
        Token::AtKeyword { .. } => todo!(),
        _ => {
            // Mark input.
            input.mark();

            // Consume a declaration from input, with nested set to true.
            let declaration = consume_a_declaration(input, true);
            // If a declaration was returned, append it to decls, and discard a mark from input.
            if let Some(declaration) = declaration {
                decls.push(declaration);
                input.discard_a_mark();

                ProcessResult::Continue
            } else {
                // Otherwise, restore a mark from input,
                input.restore_a_mark();
                // then consume a qualified rule from input, with nested set to true, and <semicolon-token> as the stop token.
                let qualified_rule = consume_a_qualified_rule(input, Some(&Token::Semicolon), true);
                // If a rule was returned, append it to rules.
                if let Some(qualified_rule) = qualified_rule {
                    rules.push(qualified_rule);
                }

                ProcessResult::Continue
            }
        }
    })
}

/// 5.5.6. Consume a declaration
///
/// https://drafts.csswg.org/css-syntax-3/#consume-declaration
pub(crate) fn consume_a_declaration(input: &TokenStream, nested: bool) -> Option<Declaration> {
    let mut decl = Declaration {
        name: String::new(),
        value: vec![],
        important: false,
        original_text: None,
    };

    // 1. If the next token is an <ident-token>,
    if let Token::Ident { value } = input.next_token() {
        // consume a token from input and set decl’s name to the token’s value.
        input.consume_a_token();
        decl.name = value.clone();
    } else {
        // Otherwise, consume the remnants of a bad declaration from input, with nested,
        // and return nothing.
        todo!()
    }

    // 2. Discard whitespace from input.
    input.discard_whitespace();

    // 3. If the next token is a <colon-token>, discard a token from input.
    if input.next_token() == &Token::Colon {
        input.discard_a_token();
    } else {
        // Otherwise, consume the remnants of a bad declaration from input, with nested, and return nothing.
        todo!()
    }

    // 4. Discard whitespace from input.
    input.discard_whitespace();

    // 5. Consume a list of component values from input, with nested,
    // and with <semicolon-token> as the stop token, and set decl’s value to the result.
    decl.value = consume_a_list_of_component_values(input, Some(&Token::Semicolon), nested);

    // If decl’s name is a custom property name string,
    if decl.name.starts_with("--") {
        // then set decl’s original text to the segment of the original source text string corresponding
        // to the tokens returned by the consume a list of component values call.
        todo!()
    }

    // If decl’s name is an ASCII case-insensitive match for "unicode-range",
    if decl.name.eq_ignore_ascii_case("unicode-range") {
        // consume the value of a unicode-range descriptor from the segment of
        // the original source text string corresponding to the tokens
        // returned by the consume a list of component values call,and replace decl’s value with the result.
        todo!()
    }

    // 6. If the last two non-<whitespace-token>s in decl’s value are a <delim-token> with the value "!"
    // followed by an <ident-token> with a value that is an ASCII case-insensitive match for "important",
    // remove them from decl’s value and set decl’s important flag.
    // FIXME: Implement

    // 7. While the last item in decl’s value is a <whitespace-token>, remove that token.
    while decl.value.last() == Some(&ComponentValue::PreservedToken(Token::Whitespace)) {
        decl.value.pop();
    }

    // 8. If decl is valid in the current context, return it; otherwise return nothing.
    // FIXME: Check if decl is valid in the current context.
    Some(decl)
}

/// 5.5.7. Consume a list of component value
///
/// https://drafts.csswg.org/css-syntax-3/#consume-component-value
pub(crate) fn consume_a_list_of_component_values(
    input: &TokenStream,
    stop_token: Option<&Token>,
    nested: bool,
) -> Vec<ComponentValue> {
    let mut values = vec![];

    input.process(|token| match token {
        token if token == &Token::EndOfFile || Some(token) == stop_token => {
            // Return values.
            ProcessResult::Return(values.clone())
        }
        Token::RightCurlyBracket => {
            // If nested is true, return nothing.
            if nested {
                return ProcessResult::Return(values.clone());
            }

            // Otherwise, this is a parse error.
            log_parser_error!();
            // Consume a token from input and append the result to values.
            values.push(ComponentValue::PreservedToken(
                input.consume_a_token().clone(),
            ));

            ProcessResult::Continue
        }
        _ => {
            // Consume a component value from input, and append the result to values.
            values.push(consume_a_component_value(input));
            ProcessResult::Continue
        }
    })
}

/// 5.5.8. Consume a component value
///
/// https://drafts.csswg.org/css-syntax-3/#consume-component-value
pub(crate) fn consume_a_component_value(input: &TokenStream) -> ComponentValue {
    input.process(|token| match token {
        Token::LeftCurlyBracket | Token::LeftSquareBracket | Token::LeftParenthesis => {
            // Consume a simple block from input and return the result.
            ProcessResult::Return(ComponentValue::SimpleBlock(consume_a_simple_block(input)))
        }
        Token::Function { .. } => {
            // Consume a function from input and return the result.
            ProcessResult::Return(consume_a_function(input))
        }
        _ => {
            // Consume a token from input and return the result.
            ProcessResult::Return(ComponentValue::PreservedToken(
                input.consume_a_token().clone(),
            ))
        }
    })
}

/// 5.5.9. Consume a simple block
///
/// https://drafts.csswg.org/css-syntax-3/#consume-a-simple-block
pub(crate) fn consume_a_simple_block(input: &TokenStream) -> SimpleBlock {
    // Let ending token be the mirror variant of the next token.
    // (E.g. if it was called with <[-token>, the ending token is <]-token>.)
    let ending_token = match input.next_token() {
        Token::LeftCurlyBracket => Token::RightCurlyBracket,
        Token::LeftSquareBracket => Token::RightSquareBracket,
        Token::LeftParenthesis => Token::RightParenthesis,
        _ => {
            // Assert: the next token of input is <{-token>, <[-token>, or <(-token>.
            panic!("A '{{', '[' or '(' was expected while consuming a simple block");
        }
    };

    // Let block be a new simple block with its associated token set
    // to the next token and with its value initially set to an empty list.
    let mut block = SimpleBlock {
        associated_token: input.next_token().clone(),
        values: vec![],
    };

    // Discard a token from input.
    input.discard_a_token();

    input.process(|token| match token {
        token if token == &Token::EndOfFile || token == &ending_token => {
            // Discard a token from input.
            input.discard_a_token();
            // Return block.
            ProcessResult::Return(block.clone())
        }
        _ => {
            // Consume a component value from input and append the result to block’s value.
            block.values.push(consume_a_component_value(input));

            ProcessResult::Continue
        }
    })
}

/// 5.5.10. Consume a function
///
/// https://drafts.csswg.org/css-syntax-3/#consume-a-function
pub(crate) fn consume_a_function(input: &TokenStream) -> ComponentValue {
    assert!(matches!(input.next_token(), Token::Function { .. }));

    // Consume a token from input,
    let function_token_name = match input.consume_a_token() {
        Token::Function { value } => value,
        _ => {
            // Assert: The next token is a <function-token>.
            panic!("A function token was expected while consuming a function");
        }
    };

    // and let function be a new function
    // with its name equal the returned token’s value, and a value set to an empty list.
    let mut function = Function {
        name: function_token_name.clone(),
        value: vec![],
    };

    input.process(|token| match token {
        Token::EndOfFile | Token::RightParenthesis => {
            // Discard a token from input.
            input.discard_a_token();
            // Return function.
            ProcessResult::Return(ComponentValue::Function(function.clone()))
        }
        _ => {
            // Consume a component value from input and append the result to function’s value.
            function.value.push(consume_a_component_value(input));

            ProcessResult::Continue
        }
    })
}
