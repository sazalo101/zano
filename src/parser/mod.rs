use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod lexer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ZanoValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(std::collections::HashMap<String, ZanoValue>),
    Array(Vec<ZanoValue>),
    Function(String), // Function name/id for now
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    VarDeclaration {
        name: String,
        value: Option<Expression>,
        is_const: bool,
    },
    FunctionDeclaration {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
        is_async: bool,
    },
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Block(Vec<Statement>),
    Return(Option<Expression>),
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Try {
        try_block: Box<Statement>,
        catch_param: Option<String>,
        catch_block: Option<Box<Statement>>,
    },
    Throw(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(ZanoValue),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    Member {
        object: Box<Expression>,
        property: String,
    },
    Assignment {
        target: String,
        value: Box<Expression>,
    },
    Array(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Await(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    And, Or,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Number, String, Boolean, Null, Undefined,
    
    // Identifiers
    Identifier,
    
    // Keywords
    Let, Const, Var, Function, If, Else, While, Return, Async, Await,
    Try, Catch, Throw,
    
    // Operators
    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual, Bang, BangEqual,
    Greater, GreaterEqual, Less, LessEqual,
    AndAnd, OrOr,
    
    // Punctuation
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Colon,
    Comma, Semicolon, Dot,
    
    // Special
    Eof, Newline,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            if self.check(&TokenKind::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.statement()?);
        }
        
        Ok(statements)
    }
    
    fn statement(&mut self) -> Result<Statement> {
        if self.match_token(&TokenKind::Let) || self.match_token(&TokenKind::Const) || self.match_token(&TokenKind::Var) {
            self.var_declaration()
        } else if self.match_token(&TokenKind::Function) {
            self.function_declaration()
        } else if self.match_token(&TokenKind::If) {
            self.if_statement()
        } else if self.match_token(&TokenKind::While) {
            self.while_statement()
        } else if self.match_token(&TokenKind::Return) {
            self.return_statement()
        } else if self.match_token(&TokenKind::Try) {
            self.try_statement()
        } else if self.match_token(&TokenKind::Throw) {
            self.throw_statement()
        } else if self.match_token(&TokenKind::LeftBrace) {
            Ok(Statement::Block(self.block()?))
        } else {
            Ok(Statement::Expression(self.expression()?))
        }
    }
    
    fn var_declaration(&mut self) -> Result<Statement> {
        let is_const = self.previous().kind == TokenKind::Const;
        let name = self.consume(&TokenKind::Identifier, "Expected variable name")?.lexeme.clone();
        
        let value = if self.match_token(&TokenKind::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        
        self.consume_semicolon();
        
        Ok(Statement::VarDeclaration { name, value, is_const })
    }
    
    fn function_declaration(&mut self) -> Result<Statement> {
        let name = self.consume(&TokenKind::Identifier, "Expected function name")?.lexeme.clone();
        
        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                params.push(self.consume(&TokenKind::Identifier, "Expected parameter name")?.lexeme.clone());
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;
        self.consume(&TokenKind::LeftBrace, "Expected '{' before function body")?;
        
        let body = self.block()?;
        
        Ok(Statement::FunctionDeclaration {
            name,
            params,
            body,
            is_async: false, // TODO: Handle async functions
        })
    }
    
    fn if_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after if condition")?;
        
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&TokenKind::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        
        Ok(Statement::If { condition, then_branch, else_branch })
    }
    
    fn while_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after while condition")?;
        
        let body = Box::new(self.statement()?);
        
        Ok(Statement::While { condition, body })
    }
    
    fn return_statement(&mut self) -> Result<Statement> {
        let value = if self.check(&TokenKind::Semicolon) || self.check(&TokenKind::Newline) {
            None
        } else {
            Some(self.expression()?)
        };
        
        self.consume_semicolon();
        Ok(Statement::Return(value))
    }
    
    fn try_statement(&mut self) -> Result<Statement> {
        self.consume(&TokenKind::LeftBrace, "Expected '{' after 'try'")?;
        let try_block = Box::new(Statement::Block(self.block()?));
        
        let mut catch_param = None;
        let mut catch_block = None;
        
        if self.match_token(&TokenKind::Catch) {
            if self.match_token(&TokenKind::LeftParen) {
                if self.match_token(&TokenKind::Identifier) {
                    catch_param = Some(self.previous().lexeme.clone());
                }
                self.consume(&TokenKind::RightParen, "Expected ')' after catch parameter")?;
            }
            
            self.consume(&TokenKind::LeftBrace, "Expected '{' after catch")?;
            catch_block = Some(Box::new(Statement::Block(self.block()?)));
        }
        
        Ok(Statement::Try { try_block, catch_param, catch_block })
    }
    
    fn throw_statement(&mut self) -> Result<Statement> {
        let expr = self.expression()?;
        self.consume_semicolon();
        Ok(Statement::Throw(expr))
    }
    
    fn block(&mut self) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            if self.check(&TokenKind::Newline) {
                self.advance();
                continue;
            }
            statements.push(self.statement()?);
        }
        
        self.consume(&TokenKind::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }
    
    fn expression(&mut self) -> Result<Expression> {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Result<Expression> {
        let expr = self.or()?;
        
        if self.match_token(&TokenKind::Equal) {
            if let Expression::Identifier(name) = expr {
                let value = Box::new(self.assignment()?);
                return Ok(Expression::Assignment { target: name, value });
            }
        }
        
        Ok(expr)
    }
    
    fn or(&mut self) -> Result<Expression> {
        let mut expr = self.and()?;
        
        while self.match_token(&TokenKind::OrOr) {
            let operator = BinaryOp::Or;
            let right = Box::new(self.and()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn and(&mut self) -> Result<Expression> {
        let mut expr = self.equality()?;
        
        while self.match_token(&TokenKind::AndAnd) {
            let operator = BinaryOp::And;
            let right = Box::new(self.equality()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<Expression> {
        let mut expr = self.comparison()?;
        
        while self.match_tokens(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = match self.previous().kind {
                TokenKind::BangEqual => BinaryOp::NotEqual,
                TokenKind::EqualEqual => BinaryOp::Equal,
                _ => unreachable!(),
            };
            let right = Box::new(self.comparison()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn comparison(&mut self) -> Result<Expression> {
        let mut expr = self.term()?;
        
        while self.match_tokens(&[TokenKind::Greater, TokenKind::GreaterEqual, TokenKind::Less, TokenKind::LessEqual]) {
            let operator = match self.previous().kind {
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                TokenKind::Less => BinaryOp::Less,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = Box::new(self.term()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<Expression> {
        let mut expr = self.factor()?;
        
        while self.match_tokens(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = match self.previous().kind {
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::Plus => BinaryOp::Add,
                _ => unreachable!(),
            };
            let right = Box::new(self.factor()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<Expression> {
        let mut expr = self.unary()?;
        
        while self.match_tokens(&[TokenKind::Slash, TokenKind::Star, TokenKind::Percent]) {
            let operator = match self.previous().kind {
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Percent => BinaryOp::Mod,
                _ => unreachable!(),
            };
            let right = Box::new(self.unary()?);
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<Expression> {
        if self.match_token(&TokenKind::Await) {
            let expr = self.unary()?;
            return Ok(Expression::Await(Box::new(expr)));
        }
        
        self.call()
    }
    
    fn call(&mut self) -> Result<Expression> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenKind::Dot) {
                let name = self.consume(&TokenKind::Identifier, "Expected property name after '.'")?;
                expr = Expression::Member {
                    object: Box::new(expr),
                    property: name.lexeme.clone(),
                };
            } else if self.match_token(&TokenKind::LeftBracket) {
                let index = self.expression()?;
                self.consume(&TokenKind::RightBracket, "Expected ']' after array index")?;
                expr = Expression::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: Expression) -> Result<Expression> {
        let mut args = Vec::new();
        
        if !self.check(&TokenKind::RightParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenKind::RightParen, "Expected ')' after arguments")?;
        
        Ok(Expression::Call {
            callee: Box::new(callee),
            args,
        })
    }
    
    fn primary(&mut self) -> Result<Expression> {
        if self.match_token(&TokenKind::Boolean) {
            let value = self.previous().lexeme == "true";
            return Ok(Expression::Literal(ZanoValue::Boolean(value)));
        }
        
        if self.match_token(&TokenKind::Null) {
            return Ok(Expression::Literal(ZanoValue::Null));
        }
        
        if self.match_token(&TokenKind::Undefined) {
            return Ok(Expression::Literal(ZanoValue::Undefined));
        }
        
        if self.match_token(&TokenKind::Number) {
            let value = self.previous().lexeme.parse::<f64>()?;
            return Ok(Expression::Literal(ZanoValue::Number(value)));
        }
        
        if self.match_token(&TokenKind::String) {
            let value = self.previous().lexeme.clone();
            return Ok(Expression::Literal(ZanoValue::String(value)));
        }
        
        if self.match_token(&TokenKind::Identifier) {
            return Ok(Expression::Identifier(self.previous().lexeme.clone()));
        }
        
        if self.match_token(&TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }
        
        if self.match_token(&TokenKind::LeftBracket) {
            return self.array_literal();
        }
        
        if self.match_token(&TokenKind::LeftBrace) {
            return self.object_literal();
        }
        
        Err(anyhow::anyhow!("Unexpected token: {:?}", self.peek()))
    }
    
    fn array_literal(&mut self) -> Result<Expression> {
        let mut elements = Vec::new();
        
        if !self.check(&TokenKind::RightBracket) {
            loop {
                elements.push(self.expression()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }
        
        self.consume(&TokenKind::RightBracket, "Expected ']' after array elements")?;
        Ok(Expression::Array(elements))
    }
    
    fn object_literal(&mut self) -> Result<Expression> {
        let mut pairs = Vec::new();
        
        // Skip newlines at the beginning
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
        
        if !self.check(&TokenKind::RightBrace) {
            loop {
                // Skip newlines before property name
                while self.check(&TokenKind::Newline) {
                    self.advance();
                }
                
                let key = if self.check(&TokenKind::String) {
                    self.advance().lexeme.clone()
                } else if self.check(&TokenKind::Identifier) {
                    self.advance().lexeme.clone()
                } else {
                    return Err(anyhow::anyhow!("Expected property name"));
                };
                
                self.consume(&TokenKind::Colon, "Expected ':' after property name")?;
                let value = self.expression()?;
                
                pairs.push((key, value));
                
                // Skip newlines before comma or closing brace
                while self.check(&TokenKind::Newline) {
                    self.advance();
                }
                
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
                
                // Skip newlines after comma
                while self.check(&TokenKind::Newline) {
                    self.advance();
                }
            }
        }
        
        // Skip newlines before closing brace
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
        
        self.consume(&TokenKind::RightBrace, "Expected '}' after object properties")?;
        Ok(Expression::Object(pairs))
    }
    
    // Helper methods
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }
    
    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }
    
    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<&Token> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(anyhow::anyhow!("{}", message))
        }
    }
    
    fn consume_semicolon(&mut self) {
        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }
    }
}