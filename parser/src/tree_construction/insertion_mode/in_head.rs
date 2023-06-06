use tokenizer::Token;

use crate::{
    is_parser_whitespace, log_parser_error, GenericParsingAlgorithm, InsertionMode, Parser,
};

impl<'a> Parser<'a> {
    pub(crate) fn handle_in_head(&'a self, token: &Token) {
        macro_rules! anything_else {
            () => {
                // Pop the current node (which will be the head element) off the stack of open elements.
                self.open_elements.pop();

                // Switch the insertion mode to "after head".
                self.switch_insertion_mode_to(InsertionMode::AfterHead);

                // Reprocess the token.
                self.process_token(token);
            };
        }

        match token {
            Token::Character { data } if is_parser_whitespace(*data) => {
                // Insert the character.
                self.insert_character(*data);
            }
            Token::Comment { data } => {
                // Insert a comment
                self.insert_comment(data);
            }
            Token::Doctype { .. } => {
                // Parse error. Ignore the token.
                log_parser_error!("Unexpected Doctype in head");
            }
            Token::StartTag { name, .. } if name == "html" => {
                // Process the token using the rules for the "in body" insertion mode.
                self.process_token_using_the_rules_for(InsertionMode::InBody, token);
            }
            Token::StartTag { name, .. }
                if name == "base" || name == "basefont" || name == "bgsound" || name == "link" =>
            {
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);

                // Immediately pop the current node off the stack of open elements.
                self.open_elements.pop();

                // Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();
            }
            Token::StartTag { name, .. } if name == "meta" => {
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);

                // Immediately pop the current node off the stack of open elements.
                self.open_elements.pop();

                // Acknowledge the token's self-closing flag, if it is set.
                token.acknowledge_self_closing_flag_if_set();

                // FIXME: If the active speculative HTML parser is null, then:
            }
            Token::StartTag { name, .. } if name == "title" => {
                self.follow_generic_parsing_algorithm(GenericParsingAlgorithm::RcData, token);
            }
            Token::StartTag { name, .. }
                if (name == "noscript" && self.scripting)
                    || (name == "noframes" || name == "style") =>
            {
                // Follow the generic raw text element parsing algorithm.
                self.follow_generic_parsing_algorithm(GenericParsingAlgorithm::RawText, token);
            }
            Token::StartTag { name, .. } if name == "noscript" && !self.scripting => {
                // Insert an HTML element for the token.
                self.insert_html_element_for_token(token);

                // Switch the insertion mode to "in head noscript".
                self.switch_insertion_mode_to(InsertionMode::InHeadNoscript);
            }
            Token::StartTag { name, .. } if name == "script" => todo!(),
            Token::EndTag { name, .. } if name == "head" => {
                // Pop the current node (which will be the head element) off the stack of open elements.
                self.open_elements.pop();

                // Switch the insertion mode to "after head".
                self.switch_insertion_mode_to(InsertionMode::AfterHead);
            }
            Token::EndTag { name, .. } if name == "body" || name == "html" || name == "br" => {
                // Act as described in the "anything else" entry below.
                anything_else!();
            }
            Token::StartTag { name, .. } if name == "template" => todo!(),
            Token::EndTag { name, .. } if name == "template" => todo!(),
            Token::StartTag { name, .. } if name == "head" => {
                // Parse error. Ignore the token.
                log_parser_error!("Unexpected head start tag in head");
            }
            Token::EndTag { name, .. } => {
                // Parse error. Ignore the token.
                log_parser_error!(format!("Unexpected end tag '{}' in head", name));
            }
            _ => {
                anything_else!();
            }
        }
    }
}
