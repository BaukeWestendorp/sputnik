use super::parser_algorithms::consume_a_stylesheets_content;
use super::token_streams::TokenStream;
use super::types::StyleSheet;
use super::Parser;

/// 5.4. Parser Entry Points
///
/// https://www.w3.org/TR/css-syntax-3/#parser-entry-points
impl<'a> Parser {
    /// 5.4.3. Parse a stylesheet
    ///
    /// "Parse a stylesheet" is intended to be the normal parser entry point, for parsing stylesheets.
    ///
    /// https://www.w3.org/TR/css-syntax-3/#parse-stylesheet
    pub fn parse_a_stylesheet(input: &TokenStream, location: Option<&str>) -> StyleSheet {
        // 1. If input is a byte stream for a stylesheet, decode bytes from input, and set input to the result.
        // 2. Normalize input, and set input to the result.
        // NOTE: In our case, these steps are obsolete, because we are using a TokenStream for input.

        // 3. Create a new stylesheet, with its location set to location (or null, if location was not passed).
        let mut stylesheet = StyleSheet::new(location);

        // 4. Consume a stylesheet’s contents from input, and set the stylesheet’s rules to the result.
        stylesheet.rules = consume_a_stylesheets_content(input);

        // 5. Return the stylesheet.
        stylesheet
    }
}
