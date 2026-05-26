use crate::token::{Token, TokenType};
use ast::Position;
use logos::{Logos, SpannedIter};

/// The Lexer consumes a raw source string and lazily produces a stream of Tokens.
pub struct Lexer<'a> {
    source: &'a str,
    lexer: SpannedIter<'a, TokenType>,
}

impl<'a> Lexer<'a> {
    /// Initializes a new Lexer for the given source code.
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            lexer: TokenType::lexer(source).spanned(),
        }
    }

    /// Advances the lexer and returns the next parsed Token.
    pub fn next_token(&mut self) -> Token {
        match self.lexer.next() {
            Some((Ok(token_type), span)) => {
                let pos = self.get_position(span.start);
                let lexeme = self.source[span.clone()].to_string();
                Token::new(token_type, lexeme, pos, (span.start, span.end))
            }
            Some((Err(_), span)) => {
                let pos = self.get_position(span.start);
                let lexeme = self.source[span.clone()].to_string();
                // When Logos encounters invalid syntax, we wrap it in an Illegal token
                Token::new(TokenType::Illegal(lexeme.clone()), lexeme, pos, (span.start, span.end))
            }
            None => {
                let len = self.source.len();
                let pos = self.get_position(len);
                Token::new(TokenType::Eof, "".to_string(), pos, (len, len))
            }
        }
    }

    /// Translates a flat byte index into a 1-based (line, column) Position struct.
    fn get_position(&self, byte_index: usize) -> Position {
        let text_up_to_token = &self.source[..byte_index];
        let line = text_up_to_token.matches('\n').count() + 1;
        let last_newline = text_up_to_token.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let col = byte_index - last_newline + 1;
        Position::new(line, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_keywords_and_operators() {
        let source = "func main() { if true { return null; } }";
        let mut lexer = Lexer::new(source);

        assert_eq!(lexer.next_token().token_type, TokenType::Fun);
        assert_eq!(lexer.next_token().token_type, TokenType::Identifier("main".to_string()));
        assert_eq!(lexer.next_token().token_type, TokenType::LeftParen);
        assert_eq!(lexer.next_token().token_type, TokenType::RightParen);
        assert_eq!(lexer.next_token().token_type, TokenType::LeftBrace);
        assert_eq!(lexer.next_token().token_type, TokenType::If);
        assert_eq!(lexer.next_token().token_type, TokenType::True);
        assert_eq!(lexer.next_token().token_type, TokenType::LeftBrace);
        assert_eq!(lexer.next_token().token_type, TokenType::Return);
        assert_eq!(lexer.next_token().token_type, TokenType::Null);
        assert_eq!(lexer.next_token().token_type, TokenType::Semicolon);
        assert_eq!(lexer.next_token().token_type, TokenType::RightBrace);
        assert_eq!(lexer.next_token().token_type, TokenType::RightBrace);
        assert_eq!(lexer.next_token().token_type, TokenType::Eof);
    }

    #[test]
    fn test_lexer_literals_and_locations() {
        let source = "let val = 3.15;\nlet msg = \"hello\";";
        let mut lexer = Lexer::new(source);

        // let val = 3.15;
        assert_eq!(lexer.next_token().token_type, TokenType::Let);
        assert_eq!(lexer.next_token().token_type, TokenType::Identifier("val".to_string()));
        assert_eq!(lexer.next_token().token_type, TokenType::Equal);
        assert_eq!(lexer.next_token().token_type, TokenType::FloatLiteral(3.15));
        assert_eq!(lexer.next_token().token_type, TokenType::Semicolon);
        
        // Check line/column tracking on the string literal (line 2)
        let str_tok = lexer.next_token(); // let
        assert_eq!(str_tok.pos.line, 2);
        assert_eq!(str_tok.pos.col, 1);
    }
}