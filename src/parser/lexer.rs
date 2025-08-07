use super::{Token, TokenKind};
use anyhow::Result;

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
    
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            line: self.line,
        });
        
        Ok(self.tokens.clone())
    }
    
    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        
        match c {
            ' ' | '\r' | '\t' => {} // Ignore whitespace
            '\n' => {
                self.add_token(TokenKind::Newline);
                self.line += 1;
            }
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            '[' => self.add_token(TokenKind::LeftBracket),
            ']' => self.add_token(TokenKind::RightBracket),
            ':' => self.add_token(TokenKind::Colon),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '%' => self.add_token(TokenKind::Percent),
            '!' => {
                let kind = if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.add_token(kind);
            }
            '=' => {
                let kind = if self.match_char('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                self.add_token(kind);
            }
            '<' => {
                let kind = if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(kind);
            }
            '>' => {
                let kind = if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(kind);
            }
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenKind::AndAnd);
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenKind::OrOr);
                }
            }
            '/' => {
                if self.match_char('/') {
                    // Line comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_char('*') {
                    // Block comment
                    self.block_comment()?;
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }
            '"' => self.string()?,
            '\'' => self.string_single()?,
            _ => {
                if c.is_ascii_digit() {
                    self.number()?;
                } else if c.is_ascii_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return Err(anyhow::anyhow!("Unexpected character: {}", c));
                }
            }
        }
        
        Ok(())
    }
    
    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(anyhow::anyhow!("Unterminated string"));
        }
        
        // Closing "
        self.advance();
        
        // Trim quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenKind::String, value);
        
        Ok(())
    }
    
    fn string_single(&mut self) -> Result<()> {
        while self.peek() != '\'' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(anyhow::anyhow!("Unterminated string"));
        }
        
        // Closing '
        self.advance();
        
        // Trim quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenKind::String, value);
        
        Ok(())
    }
    
    fn number(&mut self) -> Result<()> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        
        // Look for decimal part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume '.'
            self.advance();
            
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        
        let value = self.source[self.start..self.current].to_string();
        self.add_token_with_literal(TokenKind::Number, value);
        
        Ok(())
    }
    
    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        
        let text = &self.source[self.start..self.current];
        let kind = match text {
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "var" => TokenKind::Var,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "throw" => TokenKind::Throw,
            "true" | "false" => TokenKind::Boolean,
            "null" => TokenKind::Null,
            "undefined" => TokenKind::Undefined,
            _ => TokenKind::Identifier,
        };
        
        self.add_token(kind);
    }
    
    fn block_comment(&mut self) -> Result<()> {
        let mut depth = 1;
        
        while depth > 0 && !self.is_at_end() {
            if self.peek() == '/' && self.peek_next() == '*' {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                }
                self.advance();
            }
        }
        
        if depth > 0 {
            return Err(anyhow::anyhow!("Unterminated block comment"));
        }
        
        Ok(())
    }
    
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }
        
        self.current += 1;
        true
    }
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }
    
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    
    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap_or('\0');
        self.current += 1;
        c
    }
    
    fn add_token(&mut self, kind: TokenKind) {
        let text = self.source[self.start..self.current].to_string();
        self.add_token_with_literal(kind, text);
    }
    
    fn add_token_with_literal(&mut self, kind: TokenKind, lexeme: String) {
        self.tokens.push(Token {
            kind,
            lexeme,
            line: self.line,
        });
    }
}