use lexer::lexer::Lexer;
use lexer::token::{Token, TokenType};
use ast::{Program, Stmt, Expr, LiteralValue, Op, LogicalOp, UnaryOp, Type};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            peek_token,
        }
    }

    fn next_token(&mut self) {
        self.current_token = std::mem::replace(&mut self.peek_token, self.lexer.next_token());
    }



    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while self.current_token.token_type != TokenType::Eof {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            // Optional semicolons
            if self.current_token.token_type == TokenType::Semicolon {
                self.next_token();
            }
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        match &self.current_token.token_type {
            TokenType::Let => self.parse_let_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::LeftBrace => self.parse_block_statement(),
            TokenType::Import => self.parse_import_statement(),
            TokenType::Class => self.parse_class_statement(),
            TokenType::Fun => self.parse_fun_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_import_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'import'
        
        let mut path = Vec::new();
        match &self.current_token.token_type {
            TokenType::Identifier(name) => {
                path.push(name.clone());
                self.next_token();
            }
            _ => return Err(format!("Expected identifier after 'import' at {:?}", self.current_token.pos)),
        }
        
        while self.current_token.token_type == TokenType::Dot {
            self.next_token(); // consume '.'
            match &self.current_token.token_type {
                TokenType::Identifier(name) => {
                    path.push(name.clone());
                    self.next_token();
                }
                _ => return Err(format!("Expected identifier after '.' at {:?}", self.current_token.pos)),
            }
        }
        
        let mut alias = None;
        if self.current_token.token_type == TokenType::As {
            self.next_token(); // consume 'as'
            match &self.current_token.token_type {
                TokenType::Identifier(name) => {
                    alias = Some(name.clone());
                    self.next_token();
                }
                _ => return Err(format!("Expected identifier after 'as' at {:?}", self.current_token.pos)),
            }
        }
        
        if self.current_token.token_type == TokenType::Semicolon {
            self.next_token();
        }
        
        Ok(Stmt::Import { path, alias, pos })
    }

    fn parse_class_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'class'
        
        let name = match &self.current_token.token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => return Err(format!("Expected class name at {:?}", self.current_token.pos)),
        };
        self.next_token();
        
        let mut superclass = None;
        if self.current_token.token_type == TokenType::Extends {
            self.next_token(); // consume 'extends'
            match &self.current_token.token_type {
                TokenType::Identifier(n) => {
                    superclass = Some(n.clone());
                    self.next_token();
                }
                _ => return Err(format!("Expected superclass name at {:?}", self.current_token.pos)),
            }
        }
        
        let mut interfaces = Vec::new();
        if self.current_token.token_type == TokenType::Implements {
            self.next_token(); // consume 'implements'
            match &self.current_token.token_type {
                TokenType::Identifier(n) => {
                    interfaces.push(n.clone());
                    self.next_token();
                }
                _ => return Err(format!("Expected interface name at {:?}", self.current_token.pos)),
            }
            while self.current_token.token_type == TokenType::Comma {
                self.next_token(); // consume ','
                match &self.current_token.token_type {
                    TokenType::Identifier(n) => {
                        interfaces.push(n.clone());
                        self.next_token();
                    }
                    _ => return Err(format!("Expected interface name at {:?}", self.current_token.pos)),
                }
            }
        }
        
        if self.current_token.token_type != TokenType::LeftBrace {
            return Err(format!("Expected '{{' before class body at {:?}", self.current_token.pos));
        }
        self.next_token(); // consume '{'
        
        let mut methods = Vec::new();
        while self.current_token.token_type != TokenType::RightBrace && self.current_token.token_type != TokenType::Eof {
            methods.push(self.parse_statement()?);
        }
        
        if self.current_token.token_type != TokenType::RightBrace {
            return Err(format!("Expected '}}' at end of class body at {:?}", self.current_token.pos));
        }
        self.next_token(); // consume '}'
        
        Ok(Stmt::Class { name, superclass, interfaces, methods, pos })
    }

    fn parse_fun_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'func'
        
        let name = match &self.current_token.token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => return Err(format!("Expected function name at {:?}", self.current_token.pos)),
        };
        self.next_token();
        
        if self.current_token.token_type != TokenType::LeftParen {
            return Err(format!("Expected '(' after function name at {:?}", self.current_token.pos));
        }
        self.next_token(); // consume '('
        
        let mut params = Vec::new();
        if self.current_token.token_type != TokenType::RightParen {
            let param_name = match &self.current_token.token_type {
                TokenType::Identifier(n) => n.clone(),
                _ => return Err(format!("Expected parameter name at {:?}", self.current_token.pos)),
            };
            self.next_token();
            
            let param_type = if self.current_token.token_type == TokenType::Colon {
                self.next_token(); // consume ':'
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push((param_name, param_type));
            
            while self.current_token.token_type == TokenType::Comma {
                self.next_token(); // consume ','
                let param_name = match &self.current_token.token_type {
                    TokenType::Identifier(n) => n.clone(),
                    _ => return Err(format!("Expected parameter name at {:?}", self.current_token.pos)),
                };
                self.next_token();
                
                let param_type = if self.current_token.token_type == TokenType::Colon {
                    self.next_token(); // consume ':'
                    Some(self.parse_type()?)
                } else {
                    None
                };
                params.push((param_name, param_type));
            }
        }
        
        if self.current_token.token_type != TokenType::RightParen {
            return Err(format!("Expected ')' after parameters at {:?}", self.current_token.pos));
        }
        self.next_token(); // consume ')'
        
        let return_type = if self.current_token.token_type == TokenType::Colon {
            self.next_token(); // consume ':'
            Some(self.parse_type()?)
        } else {
            None
        };
        
        if self.current_token.token_type != TokenType::LeftBrace {
            return Err(format!("Expected '{{' before function body at {:?}", self.current_token.pos));
        }
        
        let body_stmt = self.parse_block_statement()?;
        let body = match body_stmt {
            Stmt::Block { statements, .. } => statements,
            _ => unreachable!(),
        };
        
        Ok(Stmt::Fun { name, params, body, return_type, pos })
    }

    fn parse_let_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'let'

        let name = match &self.current_token.token_type {
            TokenType::Identifier(n) => n.clone(),
            _ => return Err(format!("Expected identifier after 'let' at {:?}", self.current_token.pos)),
        };
        self.next_token();

        let type_ann = if self.current_token.token_type == TokenType::Colon {
            self.next_token(); // consume ':'
            let ty = self.parse_type()?;
            Some(ty)
        } else {
            None
        };

        let mut initializer = None;
        if self.current_token.token_type == TokenType::Equal {
            self.next_token(); // consume '='
            initializer = Some(self.parse_expression(Precedence::Lowest)?);
        }

        if self.current_token.token_type == TokenType::Semicolon {
            self.next_token();
        }

        Ok(Stmt::Let { name, type_ann, initializer, pos })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let ty = match &self.current_token.token_type {
            TokenType::Identifier(name) => match name.as_str() {
                "Int" | "int" => Type::Int,
                "Float" | "float" => Type::Float,
                "Boolean" | "bool" | "boolean" => Type::Boolean,
                "String" | "string" => Type::String,
                "Any" | "any" => Type::Any,
                "Void" | "void" => Type::Void,
                other => Type::Class(other.to_string()),
            },
            _ => return Err(format!("Expected type identifier at {:?}", self.current_token.pos)),
        };
        self.next_token();
        Ok(ty)
    }

    fn parse_if_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'if'

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.current_token.token_type != TokenType::LeftBrace {
            return Err(format!("Expected '{{' block after if condition, got {}", self.current_token.token_type));
        }
        let then_branch = Box::new(self.parse_block_statement()?);

        let mut else_branch = None;
        if self.current_token.token_type == TokenType::Else {
            self.next_token(); // consume 'else'
            if self.current_token.token_type == TokenType::LeftBrace {
                else_branch = Some(Box::new(self.parse_block_statement()?));
            } else if self.current_token.token_type == TokenType::If {
                else_branch = Some(Box::new(self.parse_if_statement()?));
            } else {
                return Err(format!("Expected '{{' block or 'if' after 'else', got {}", self.current_token.token_type));
            }
        }

        Ok(Stmt::If { condition, then_branch, else_branch, pos })
    }

    fn parse_while_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'while'

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.current_token.token_type != TokenType::LeftBrace {
            return Err("Expected '{' block after while condition".to_string());
        }
        let body = Box::new(self.parse_block_statement()?);

        Ok(Stmt::While { condition, body, pos })
    }

    fn parse_return_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume 'return'

        let mut value = None;
        if self.current_token.token_type != TokenType::Semicolon && self.current_token.token_type != TokenType::RightBrace && self.current_token.token_type != TokenType::Eof {
            value = Some(self.parse_expression(Precedence::Lowest)?);
        }

        if self.current_token.token_type == TokenType::Semicolon {
            self.next_token();
        }

        Ok(Stmt::Return { value, pos })
    }

    fn parse_block_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        self.next_token(); // consume '{'

        let mut statements = Vec::new();
        while self.current_token.token_type != TokenType::RightBrace && self.current_token.token_type != TokenType::Eof {
            statements.push(self.parse_statement()?);
            if self.current_token.token_type == TokenType::Semicolon {
                self.next_token();
            }
        }

        if self.current_token.token_type != TokenType::RightBrace {
            return Err(format!(
                "Expected '}}' at end of block, got {} instead at line {}, col {}",
                self.current_token.token_type, self.current_token.pos.line, self.current_token.pos.col
            ));
        }
        self.next_token(); // consume '}'

        Ok(Stmt::Block { statements, pos })
    }

    fn parse_expression_statement(&mut self) -> Result<Stmt, String> {
        let pos = self.current_token.pos;
        let expr = self.parse_expression(Precedence::Lowest)?;
        if self.current_token.token_type == TokenType::Semicolon {
            self.next_token();
        }
        Ok(Stmt::Expression { expr, pos })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr, String> {
        let mut left = self.parse_prefix()?;

        while self.current_token.token_type != TokenType::Semicolon 
            && self.current_token.token_type != TokenType::Eof
            && precedence < self.cur_precedence() 
        {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, String> {
        let pos = self.current_token.pos;
        match &self.current_token.token_type {
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                Ok(Expr::Identifier { name, pos })
            }
            TokenType::IntLiteral(val) => {
                let val = *val;
                self.next_token();
                Ok(Expr::Literal { value: LiteralValue::Int(val), pos })
            }
            TokenType::FloatLiteral(val) => {
                let val = *val;
                self.next_token();
                Ok(Expr::Literal { value: LiteralValue::Float(val), pos })
            }
            TokenType::StringLiteral(val) => {
                let val = val.clone();
                self.next_token();
                Ok(Expr::Literal { value: LiteralValue::String(val), pos })
            }
            TokenType::True => {
                self.next_token();
                Ok(Expr::Literal { value: LiteralValue::Boolean(true), pos })
            }
            TokenType::False => {
                self.next_token();
                Ok(Expr::Literal { value: LiteralValue::Boolean(false), pos })
            }
            TokenType::Null => {
                self.next_token();
                Ok(Expr::Literal { value: LiteralValue::Null, pos })
            }
            TokenType::Minus => {
                self.next_token();
                let right = self.parse_expression(Precedence::Prefix)?;
                Ok(Expr::Unary { op: UnaryOp::Neg, right: Box::new(right), pos })
            }
            TokenType::Bang => {
                self.next_token();
                let right = self.parse_expression(Precedence::Prefix)?;
                Ok(Expr::Unary { op: UnaryOp::Not, right: Box::new(right), pos })
            }
            TokenType::LeftParen => {
                self.next_token(); // consume '('
                let expr = self.parse_expression(Precedence::Lowest)?;
                if self.current_token.token_type != TokenType::RightParen {
                    return Err(format!("Expected ')' at line {}, col {}", self.current_token.pos.line, self.current_token.pos.col));
                }
                self.next_token(); // consume ')'
                Ok(expr)
            }
            TokenType::This => {
                self.next_token();
                Ok(Expr::This { pos })
            }
            other => Err(format!("Unexpected token in prefix: {} at line {}, col {}", other, pos.line, pos.col)),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr, String> {
        let pos = self.current_token.pos;
        let precedence = self.cur_precedence();
        
        match &self.current_token.token_type {
            TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash | TokenType::Percent |
            TokenType::EqualEqual | TokenType::BangEqual | TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual => {
                let op = match &self.current_token.token_type {
                    TokenType::Plus => Op::Add,
                    TokenType::Minus => Op::Sub,
                    TokenType::Star => Op::Mul,
                    TokenType::Slash => Op::Div,
                    TokenType::Percent => Op::Mod,
                    TokenType::EqualEqual => Op::Equal,
                    TokenType::BangEqual => Op::NotEqual,
                    TokenType::Less => Op::LessThan,
                    TokenType::LessEqual => Op::LessEqual,
                    TokenType::Greater => Op::GreaterThan,
                    TokenType::GreaterEqual => Op::GreaterEqual,
                    _ => unreachable!(),
                };
                self.next_token();
                let right = self.parse_expression(precedence)?;
                Ok(Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    pos,
                })
            }
            TokenType::And | TokenType::Or => {
                let op = match &self.current_token.token_type {
                    TokenType::And => LogicalOp::And,
                    TokenType::Or => LogicalOp::Or,
                    _ => unreachable!(),
                };
                self.next_token();
                let right = self.parse_expression(precedence)?;
                Ok(Expr::Logical {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    pos,
                })
            }
            TokenType::LeftParen => {
                // Function Call
                self.next_token(); // consume '('
                let mut arguments = Vec::new();
                if self.current_token.token_type != TokenType::RightParen {
                    arguments.push(self.parse_expression(Precedence::Lowest)?);
                    while self.current_token.token_type == TokenType::Comma {
                        self.next_token(); // consume ','
                        arguments.push(self.parse_expression(Precedence::Lowest)?);
                    }
                }
                if self.current_token.token_type != TokenType::RightParen {
                    return Err(format!("Expected ')' at end of call, got {}", self.current_token.token_type));
                }
                self.next_token(); // consume ')'
                Ok(Expr::Call {
                    callee: Box::new(left),
                    arguments,
                    pos,
                })
            }
            TokenType::Dot => {
                self.next_token(); // consume '.'
                let name = match &self.current_token.token_type {
                    TokenType::Identifier(n) => n.clone(),
                    _ => return Err(format!("Expected identifier after '.' at {:?}", self.current_token.pos)),
                };
                self.next_token();
                Ok(Expr::Get { object: Box::new(left), name, pos })
            }
            TokenType::Equal => {
                self.next_token(); // consume '='
                let right = self.parse_expression(Precedence::Lowest)?;
                match left {
                    Expr::Identifier { name, .. } => {
                        Ok(Expr::Assign { name, value: Box::new(right), pos })
                    }
                    Expr::Get { object, name, .. } => {
                        Ok(Expr::Set { object, name, value: Box::new(right), pos })
                    }
                    _ => Err(format!("Invalid assignment target at {:?}", pos)),
                }
            }
            other => Err(format!("Unexpected infix operator: {} at line {}, col {}", other, pos.line, pos.col)),
        }
    }

    fn cur_precedence(&self) -> Precedence {
        token_precedence(&self.current_token.token_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Assign,      // =
    Equals,      // == or !=
    LessGreater, // > or <
    Sum,         // + or -
    Product,     // * or /
    Prefix,      // -x or !x
    Call,        // myFunction(x)
    Index,       // [ or .
}

fn token_precedence(tok: &TokenType) -> Precedence {
    match tok {
        TokenType::Equal => Precedence::Assign,
        TokenType::EqualEqual | TokenType::BangEqual => Precedence::Equals,
        TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual => Precedence::LessGreater,
        TokenType::Plus | TokenType::Minus => Precedence::Sum,
        TokenType::Star | TokenType::Slash | TokenType::Percent => Precedence::Product,
        TokenType::And | TokenType::Or => Precedence::Equals, // logical ops same precedence as equals
        TokenType::LeftParen => Precedence::Call,
        TokenType::Dot | TokenType::LeftBracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}
