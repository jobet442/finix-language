use ast::Position;
use std::fmt;
use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore whitespace
#[logos(skip r"//.*")]      // Ignore single-line comments
pub enum TokenType {
    // Single-character tokens
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token("/")]
    Slash,
    #[token("*")]
    Star,
    #[token("%")]
    Percent,
    #[token(":")]
    Colon,

    // One or two character tokens
    #[token("!")]
    Bang,
    #[token("!=")]
    BangEqual,
    #[token("=")]
    Equal,
    #[token("==")]
    EqualEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token("&&")]
    And, // &&
    #[token("||")]
    Or,  // ||
    #[token("|>")]
    Pipeline, // |>
    #[token("?")]
    Question, // ?

    // Literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex(r#""(?:[^"\\]|\\.)*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    StringLiteral(String),
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap_or(0))]
    IntLiteral(i64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().unwrap_or(0.0))]
    FloatLiteral(f64),

    // Keywords
    #[token("let")]
    Let,
    #[token("func")]
    Fun,
    #[token("class")]
    Class,
    #[token("interface")]
    Interface,
    #[token("implements")]
    Implements,
    #[token("extends")]
    Extends,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("return")]
    Return,
    #[token("try")]
    Try,
    #[token("catch")]
    Catch,
    #[token("throw")]
    Throw,
    #[token("import")]
    Import,
    #[token("as")]
    As,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("this")]
    This,
    #[token("super")]
    Super,

    // Special
    Eof,
    Illegal(String),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::LeftParen => write!(f, "("),
            TokenType::RightParen => write!(f, ")"),
            TokenType::LeftBrace => write!(f, "{{"),
            TokenType::RightBrace => write!(f, "}}"),
            TokenType::LeftBracket => write!(f, "["),
            TokenType::RightBracket => write!(f, "]"),
            TokenType::Comma => write!(f, ","),
            TokenType::Dot => write!(f, "."),
            TokenType::Minus => write!(f, "-"),
            TokenType::Plus => write!(f, "+"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Slash => write!(f, "/"),
            TokenType::Star => write!(f, "*"),
            TokenType::Percent => write!(f, "%"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Bang => write!(f, "!"),
            TokenType::BangEqual => write!(f, "!="),
            TokenType::Equal => write!(f, "="),
            TokenType::EqualEqual => write!(f, "=="),
            TokenType::Greater => write!(f, ">"),
            TokenType::GreaterEqual => write!(f, ">="),
            TokenType::Less => write!(f, "<"),
            TokenType::LessEqual => write!(f, "<="),
            TokenType::And => write!(f, "&&"),
            TokenType::Or => write!(f, "||"),
            TokenType::Pipeline => write!(f, "|>"),
            TokenType::Question => write!(f, "?"),
            TokenType::Identifier(s) => write!(f, "Identifier({})", s),
            TokenType::StringLiteral(s) => write!(f, "String(\"{}\")", s),
            TokenType::IntLiteral(n) => write!(f, "Int({})", n),
            TokenType::FloatLiteral(n) => write!(f, "Float({})", n),
            TokenType::Let => write!(f, "let"),
            TokenType::Fun => write!(f, "fun"),
            TokenType::Class => write!(f, "class"),
            TokenType::Interface => write!(f, "interface"),
            TokenType::Implements => write!(f, "implements"),
            TokenType::Extends => write!(f, "extends"),
            TokenType::If => write!(f, "if"),
            TokenType::Else => write!(f, "else"),
            TokenType::While => write!(f, "while"),
            TokenType::For => write!(f, "for"),
            TokenType::In => write!(f, "in"),
            TokenType::Return => write!(f, "return"),
            TokenType::Try => write!(f, "try"),
            TokenType::Catch => write!(f, "catch"),
            TokenType::Throw => write!(f, "throw"),
            TokenType::Import => write!(f, "import"),
            TokenType::As => write!(f, "as"),
            TokenType::True => write!(f, "true"),
            TokenType::False => write!(f, "false"),
            TokenType::Null => write!(f, "null"),
            TokenType::This => write!(f, "this"),
            TokenType::Super => write!(f, "super"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Illegal(s) => write!(f, "Illegal({})", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub pos: Position,
    pub span: (usize, usize), // Byte offset tracking for miette diagnostics
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, pos: Position, span: (usize, usize)) -> Self {
        Self {
            token_type,
            lexeme,
            pos,
            span,
        }
    }
}
