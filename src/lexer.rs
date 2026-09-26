#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let, Var, Fn, Return, If, Else, While, Loop, From, To,
    And, Or, Not,
    True, False,
    Int, Float, Str, Bool,
    Ident(String),
    Number(i64),
    Text(String),
    Plus, Minus, Star, Slash,
    Assign,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    LParen, RParen,
    LBrace, RBrace,
    LBracket, RBracket,
    Comma, Colon, Semicolon,
    Eof,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            
            if c.is_whitespace() {
                if c == '\n' { self.line += 1; }
                self.pos += 1;
                continue;
            }
            
            if c == '/' && self.peek() == Some('/') {
                while self.pos < self.chars.len() 
                    && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            
            let token = match c {
                '+' => { self.pos += 1; Token::Plus }
                '-' => { self.pos += 1; Token::Minus }
                '*' => { self.pos += 1; Token::Star }
                '/' => { self.pos += 1; Token::Slash }
                '=' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        Token::Eq
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        Token::NotEq
                    } else {
                        Token::Not
                    }
                }
                '<' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        Token::LtEq
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        Token::GtEq
                    } else {
                        Token::Gt
                    }
                }
                '(' => { self.pos += 1; Token::LParen }
                ')' => { self.pos += 1; Token::RParen }
                '{' => { self.pos += 1; Token::LBrace }
                '}' => { self.pos += 1; Token::RBrace }
                '[' => { self.pos += 1; Token::LBracket }
                ']' => { self.pos += 1; Token::RBracket }
                ',' => { self.pos += 1; Token::Comma }
                ':' => { self.pos += 1; Token::Colon }
                ';' => { self.pos += 1; Token::Semicolon }
                '"' => self.read_string()?,
                c if c.is_ascii_digit() => self.read_number(),
                c if c.is_alphabetic() || c == '_' => self.read_ident(),
                _ => return Err(self.err(&format!("حرف غير معروف: '{}'", c))),
            };
            
            tokens.push(token);
        }
        
        tokens.push(Token::Eof);
        Ok(tokens)
    }
    
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }
    
    fn err(&self, msg: &str) -> String {
        format!("خطأ سطر {}: {}", self.line, msg)
    }
    
    fn read_string(&mut self) -> Result<Token, String> {
        self.pos += 1;
        let mut s = String::new();
        while let Some(&c) = self.chars.get(self.pos) {
            if c == '"' { break; }
            if c == '\\' {
                self.pos += 1;
                if let Some(&esc) = self.chars.get(self.pos) {
                    let ch = match esc {
                        'n' => '\n',
                        't' => '\t',
                        '\\' => '\\',
                        '"' => '"',
                        _ => esc,
                    };
                    s.push(ch);
                    self.pos += 1;
                }
            } else {
                s.push(c);
                self.pos += 1;
            }
        }
        if self.chars.get(self.pos) != Some(&'"') {
            return Err(self.err("نص غير مغلق"));
        }
        self.pos += 1;
        Ok(Token::Text(s))
    }
    
    fn read_number(&mut self) -> Token {
        let mut s = String::new();
        while let Some(&c) = self.chars.get(self.pos) {
            if c.is_ascii_digit() {
                s.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        Token::Number(s.parse().unwrap_or(0))
    }
    
    fn read_ident(&mut self) -> Token {
        let mut s = String::new();
        while let Some(&c) = self.chars.get(self.pos) {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        
        match s.as_str() {
            "let" => Token::Let,
            "var" => Token::Var,
            "fn" => Token::Fn,
            "return" => Token::Return,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "loop" => Token::Loop,
            "from" => Token::From,
            "to" => Token::To,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            "true" => Token::True,
            "false" => Token::False,
            "Int" => Token::Int,
            "Float" => Token::Float,
            "Str" => Token::Str,
            "Bool" => Token::Bool,
            _ => Token::Ident(s),
        }
    }
}
