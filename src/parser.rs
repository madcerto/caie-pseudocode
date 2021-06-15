use super::token::{Token, TokenType};
use super::expr::Expr;
use super::stmt::Stmt;
use super::env::Type;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0
        }
    }

    pub fn parse(&mut self) -> Stmt {
        match self.program() {
            Ok(prog) => prog,
            Err(err) => match err.token.ttype {
                TokenType::End => panic!("error at end: {}", err.msg),
                _ => panic!("error at token {}: {}", err.token, err.msg)
            }
        }
    }

    fn is_at_end(&self) -> bool {
        match self.peak().ttype {
            TokenType::End => true,
            _ => false
        }
    }
    fn peak(&self) -> Token {
        self.tokens[self.current].clone()
    }
    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.current += 1 }
        self.previous()
    }
    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn program(&mut self) -> Result<Stmt, ParseError> {
        self.block(vec![TokenType::End])
    }
    fn block(&mut self, terminators: Vec<TokenType>) -> Result<Stmt, ParseError> {
        while self.peak().ttype == TokenType::NL { self.advance(); } // get rid of newlines at the start

        let mut statements = Vec::new();
        'outer: loop {
            for terminator in terminators.clone() {
                if self.peak().ttype == terminator { break 'outer }
            }
            statements.push(self.statement()?)
        }
        self.advance();
        Ok(Stmt::Block(statements))
    }
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let tkn = self.peak();
        let stmt = match tkn.ttype {
            TokenType::DECLARE => self.declare(),
            TokenType::CONSTANT => self.constant(),
            TokenType::Identifier => self.assign(tkn),
            TokenType::OUTPUT => self.output(),
            TokenType::INPUT => self.input(),
            TokenType::IF => self.ifthen(),
            _ => Ok(Stmt::ExprStmt(self.expr()?))
        };
        match self.peak().ttype {
            TokenType::NL | TokenType::End => { self.advance(); stmt },
            _ => Err(ParseError::new(self.peak(), "Expected newline".into()))
        }
    }
    fn declare(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let name  = self.peak();
        if name.ttype == TokenType::Identifier
        { self.advance(); }
        else { return Err(ParseError::new(self.peak(), "Expected identifier".into())) }
        if self.peak().ttype == TokenType::Colon { self.advance(); Ok(Stmt::Declare(name, self.dtype()?)) }
        else { Err(ParseError::new(self.peak(), "Expected ':' token".into())) }
    }
    fn constant(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let name  = self.peak();
        if name.ttype == TokenType::Identifier
        { self.advance(); }
        else { return Err(ParseError::new(self.peak(), "Expected identifier".into())) }
        if self.peak().ttype == TokenType::Equal { self.advance(); Ok(Stmt::Constant(name, self.expr()?)) }
        else { Err(ParseError::new(self.peak(), "Expected '=' token".into())) }
    }
    fn assign(&mut self, name: Token) -> Result<Stmt, ParseError> {
        self.advance();
        if self.peak().ttype == TokenType::Arrow
        { self.advance(); Ok(Stmt::Assign(name, self.expr()?)) }
        else { Err(ParseError::new(self.peak(), "Expected '<-' token".into())) }
    }
    fn input(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        Ok(Stmt::Input(self.expr()?))
    }
    fn output(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let mut exprs = Vec::new();
        exprs.push(self.expr()?);
        while self.peak().ttype == TokenType::Comma {
            self.advance();
            exprs.push(self.expr()?);
        }
        Ok(Stmt::Output(exprs))
    }
    fn ifthen(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let condition = self.expr()?;
        if self.peak().ttype == TokenType::NL { self.advance(); }
        else { return Err(ParseError::new(self.peak(), "Expected newline".into())) }
        if self.peak().ttype == TokenType::THEN { self.advance(); }
        else { return Err(ParseError::new(self.peak(), "'THEN' required after 'IF'".into())) }
        if self.peak().ttype == TokenType::NL { self.advance(); }
        else { return Err(ParseError::new(self.peak(), "Expected newline".into())) }
        let then_block = self.block(vec![TokenType::ELSE, TokenType::ENDIF])?;
        if let TokenType::ELSE = self.previous().ttype {
            if self.peak().ttype == TokenType::NL { self.advance(); }
            else { return Err(ParseError::new(self.peak(), "Expected newline".into())) }
            let else_block = self.block(vec![TokenType::ENDIF])?;
            Ok(Stmt::IfThen(condition, Box::new(then_block), Some(Box::new(else_block))))
        }
        else { Ok(Stmt::IfThen(condition, Box::new(then_block), None)) }
    }

    pub fn expr(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic()?;
        let mut tkn = self.peak();
        loop { match tkn.ttype {
                TokenType::Equal | TokenType::NotEqual => {
                    self.advance();
                    expr = Expr::Binary(Box::new(expr), tkn.clone(), Box::new(self.logic()?));
                },
                _ => {break;}
            }
            tkn = self.peak();
        }
        Ok(expr)
    }
    fn logic(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        let mut tkn = self.peak();
        loop { match tkn.ttype {
                TokenType::AND | TokenType::OR => {
                    self.advance();
                    expr = Expr::Binary(Box::new(expr), tkn.clone(), Box::new(self.comparison()?));
                },
                _ => {break;}
            }
            tkn = self.peak();
        }
        Ok(expr)
    }
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        let mut tkn = self.peak();
        loop { match tkn.ttype {
                TokenType::Greater | TokenType::Less | TokenType::GreaterEqual | TokenType::LessEqual => {
                    self.advance();
                    expr = Expr::Binary(Box::new(expr), tkn.clone(), Box::new(self.term()?));
                },
                _ => {break;}
            }
            tkn = self.peak();
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        let mut tkn = self.peak();
        loop { match tkn.ttype {
                TokenType::Plus | TokenType::Minus => {
                    self.advance();
                    expr = Expr::Binary(Box::new(expr), tkn.clone(), Box::new(self.factor()?));
                },
                _ => {break;}
            }
            tkn = self.peak();
        }
        Ok(expr)
    }
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        let mut tkn = self.peak();
        loop { match tkn.ttype {
                TokenType::Slash | TokenType::Star | TokenType::MOD | TokenType::DIV => {
                    self.advance();
                    expr = Expr::Binary(Box::new(expr), tkn.clone(), Box::new(self.unary()?));
                }
                _ => {break;}
            }
            tkn = self.peak();
        }
        Ok(expr)
    }
    fn unary(&mut self) -> Result<Expr, ParseError> {
        let tkn = self.peak();
        if let TokenType::Minus | TokenType::NOT = tkn.ttype
        {self.advance(); Ok(Expr::Unary(tkn, Box::new(self.primary()?)))}
        else { self.primary() }
    }
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let tkn = self.peak();
        // println!("{}", tkn);
        match tkn.ttype {
            TokenType::Literal(lit) => {self.advance(); Ok(Expr::Literal(lit))},
            // TokenType::Identifier => {
            //     self.advance();
            //     let mut expr = Expr::Literal(tkn);
            //     let mut tkn = self.peak();
            //     loop { match tkn.ttype {
            //             TokenType::Period => {
            //                 self.advance();
            //                 expr = Expr::Binary(Box::new(expr), tkn.clone(), Box::new(self.primary()));
            //             }
            //             TokenType::LeftBracket => {
            //                 let expr
            //             }
            //             _ => {break}
            //         }
            //         tkn = self.peak();
            //     }
            //     expr
            // }
            TokenType::Identifier => { self.advance(); Ok(Expr::IdentExpr(tkn)) },
            TokenType::LeftParen => {
                self.advance();
                let expr = Expr::Grouping(Box::new(self.expr()?));
                if self.peak().ttype != TokenType::RightParen { return Err(ParseError::new(self.peak(), "Unterminated Grouping".to_string())) }
                self.advance();
                Ok(expr)
            },
            TokenType::End => Err(ParseError::new(tkn, "Expected expression".to_string())),
            _ => Err(ParseError::new(tkn, "Invalid expression-starting token".to_string()))
        }
    }

    fn dtype(&mut self) -> Result<Type, ParseError> {
        let dtype = match self.peak().ttype {
            TokenType::BOOLEAN => Ok(Type::Bool),
            TokenType::INTEGER => Ok(Type::Int),
            TokenType::REAL => Ok(Type::Float),
            TokenType::CHAR => Ok(Type::Char),
            TokenType::STRING => Ok(Type::String),
            TokenType::DATE => Ok(Type::Date),
            // TokenType::Identifier => Ok(Type::UDT), TODO
            // TokenType::ARRAY => Ok(Type::Array(Type, size)),
            _ => Err(ParseError::new(self.peak(), "Expected type".into()))
        };
        self.advance();
        dtype
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub msg: String,
    pub token: Token
}

impl ParseError {
    pub fn new(token: Token, msg: String) -> Self {
        Self {
            token,
            msg
        }
    }
}