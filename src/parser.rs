use crate::ast::{Expr, Stmt, BinOp};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while self.peek() != &Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            Token::Import => self.parse_import(),
            Token::Let => self.parse_let(),
            Token::Var => self.parse_var(),
            Token::Fn => self.parse_fn(),
            Token::If => self.parse_if(),
            Token::Loop => self.parse_loop(),
            Token::While => self.parse_while(),
            Token::Return => self.parse_return(),
            Token::Ident(_) => {
                if self.tokens.get(self.pos + 1) == Some(&Token::Assign) {
                    return self.parse_assign();
                }
                if self.tokens.get(self.pos + 1) == Some(&Token::LBracket) {
                    if let Some(close_pos) = self.find_close_bracket() {
                        if self.tokens.get(close_pos + 1) == Some(&Token::Assign) {
                            return self.parse_index_assign(close_pos);
                        }
                    }
                }
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_import(&mut self) -> Result<Stmt, String> {
        self.next();
        let path = match self.next() {
            Token::Text(s) => s,
            _ => return Err("متوقع مسار الملف بين علامتي تنصيص".to_string()),
        };
        Ok(Stmt::Import { path })
    }

    fn find_close_bracket(&self) -> Option<usize> {
        let mut i = self.pos + 1;
        let mut depth = 0;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::LBracket => depth += 1,
                Token::RBracket => {
                    depth -= 1;
                    if depth == 0 { return Some(i); }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }

    fn parse_index_assign(&mut self, close_pos: usize) -> Result<Stmt, String> {
        let array = match self.next() {
            Token::Ident(s) => s,
            _ => return Err("متوقع اسم مصفوفة".to_string()),
        };
        self.next();
        let index = self.parse_expr()?;
        while self.pos <= close_pos { self.next(); }
        if self.next() != Token::Assign {
            return Err("متوقع =".to_string());
        }
        let value = self.parse_expr()?;
        Ok(Stmt::IndexAssign { array, index, value })
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.next();
        let name = match self.next() {
            Token::Ident(s) => s,
            _ => return Err("متوقع اسم متغير".to_string()),
        };
        if self.next() != Token::Assign {
            return Err("متوقع =".to_string());
        }
        let value = self.parse_expr()?;
        Ok(Stmt::Let { name, value })
    }

    fn parse_var(&mut self) -> Result<Stmt, String> {
        self.next();
        let name = match self.next() {
            Token::Ident(s) => s,
            _ => return Err("متوقع اسم متغير".to_string()),
        };
        if self.next() != Token::Assign {
            return Err("متوقع =".to_string());
        }
        let value = self.parse_expr()?;
        Ok(Stmt::Var { name, value })
    }

    fn parse_assign(&mut self) -> Result<Stmt, String> {
        let name = match self.next() {
            Token::Ident(s) => s,
            _ => return Err("متوقع اسم متغير".to_string()),
        };
        self.next();
        let value = self.parse_expr()?;
        Ok(Stmt::Assign { name, value })
    }

    fn parse_fn(&mut self) -> Result<Stmt, String> {
        self.next();
        let name = match self.next() {
            Token::Ident(s) => s,
            _ => return Err("متوقع اسم دالة".to_string()),
        };
        if self.next() != Token::LParen {
            return Err("متوقع (".to_string());
        }
        let mut params = Vec::new();
        while self.peek() != &Token::RParen {
            match self.next() {
                Token::Ident(s) => params.push(s),
                _ => return Err("متوقع اسم معامل".to_string()),
            }
            if self.peek() == &Token::Comma { self.next(); }
        }
        self.next();
        if self.next() != Token::LBrace {
            return Err("متوقع {".to_string());
        }
        let body = self.parse_block()?;
        Ok(Stmt::Fn { name, params, body })
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.next();
        let condition = self.parse_expr()?;
        if self.next() != Token::LBrace {
            return Err("متوقع {".to_string());
        }
        let then_body = self.parse_block()?;
        let mut else_body = Vec::new();
        if self.peek() == &Token::Else {
            self.next();
            if self.next() != Token::LBrace {
                return Err("متوقع {".to_string());
            }
            else_body = self.parse_block()?;
        }
        Ok(Stmt::If { condition, then_body, else_body })
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.next();
        let var = match self.next() {
            Token::Ident(s) => s,
            _ => return Err("متوقع اسم متغير".to_string()),
        };
        if self.next() != Token::From {
            return Err("متوقع from".to_string());
        }
        let from = self.parse_expr()?;
        if self.next() != Token::To {
            return Err("متوقع to".to_string());
        }
        let to = self.parse_expr()?;
        if self.next() != Token::LBrace {
            return Err("متوقع {".to_string());
        }
        let body = self.parse_block()?;
        Ok(Stmt::Loop { var, from, to, body })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.next();
        let condition = self.parse_expr()?;
        if self.next() != Token::LBrace {
            return Err("متوقع {".to_string());
        }
        let body = self.parse_block()?;
        Ok(Stmt::While { condition, body })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        if self.next() != Token::RBrace {
            return Err("متوقع }".to_string());
        }
        Ok(stmts)
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.next();
        if self.peek() == &Token::RBrace || self.peek() == &Token::Eof {
            return Ok(Stmt::Return(None));
        }
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Some(expr)))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.peek() == &Token::Or {
            self.next();
            let right = self.parse_and()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while self.peek() == &Token::And {
            self.next();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                Token::Eq => BinOp::Eq,
                Token::NotEq => BinOp::NotEq,
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::LtEq => BinOp::LtEq,
                Token::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.next();
            let right = self.parse_additive()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.next();
            let right = self.parse_term()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.next();
            let right = self.parse_primary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        if self.peek() == &Token::Minus {
            self.next();
            match self.next() {
                Token::Number(n) => return Ok(Expr::Number(-n)),
                t => return Err(format!("متوقع رقم بعد -، وُجد {:?}", t)),
            }
        }
        
        match self.next() {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Text(s) => Ok(Expr::Text(s)),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Ident(name) => {
                if self.peek() == &Token::LParen {
                    self.next();
                    let mut args = Vec::new();
                    while self.peek() != &Token::RParen {
                        args.push(self.parse_expr()?);
                        if self.peek() == &Token::Comma { self.next(); }
                    }
                    self.next();
                    return Ok(Expr::Call { name, args });
                }
                if self.peek() == &Token::LBracket {
                    self.next();
                    let index = self.parse_expr()?;
                    if self.next() != Token::RBracket {
                        return Err("متوقع ]".to_string());
                    }
                    return Ok(Expr::Index {
                        array: Box::new(Expr::Ident(name)),
                        index: Box::new(index),
                    });
                }
                Ok(Expr::Ident(name))
            }
            Token::LParen => {
                let expr = self.parse_expr()?;
                if self.next() != Token::RParen {
                    return Err("متوقع )".to_string());
                }
                Ok(expr)
            }
            Token::LBracket => {
                let mut items = Vec::new();
                while self.peek() != &Token::RBracket {
                    items.push(self.parse_expr()?);
                    if self.peek() == &Token::Comma { self.next(); }
                }
                self.next();
                Ok(Expr::Array(items))
            }
            t => Err(format!("متوقع قيمة، وُجد {:?}", t)),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn next(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }
}
