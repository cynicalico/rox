use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenKind};
use crate::value::Value;
use hashbrown::HashMap;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::mem::take;

pub fn compile(source: &str, chunk: &mut Chunk) -> bool {
    let mut scanner = Scanner::new(source);
    let mut parser = Parser::new();

    parser.advance(&mut scanner);
    parser.expression(&mut scanner, chunk);
    parser.consume(&mut scanner, TokenKind::Eof, "Expect end of expression");
    parser.end_compiler(chunk);

    !parser.had_error
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, FromPrimitive)]
#[repr(u8)]
pub enum Precedence {
    None,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

type ParserRuleFn<'a> = fn(&mut Parser<'a>, &mut Scanner<'a>, &mut Chunk);

pub struct ParseRule<'a> {
    prefix: Option<ParserRuleFn<'a>>,
    infix: Option<ParserRuleFn<'a>>,
    precedence: Precedence,
}

impl<'a> ParseRule<'a> {
    pub fn new(
        prefix: Option<ParserRuleFn<'a>>,
        infix: Option<ParserRuleFn<'a>>,
        precedence: Precedence,
    ) -> Self {
        Self {
            prefix,
            infix,
            precedence,
        }
    }
}

pub struct Parser<'a> {
    curr: Token<'a>,
    prev: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    rules: HashMap<TokenKind, ParseRule<'a>>,
}

impl<'a> Parser<'a> {
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            curr: Token::default(),
            prev: Token::default(),
            had_error: false,
            panic_mode: false,
            rules: HashMap::from([
                (TokenKind::LeftParen,    ParseRule::new(Some(Self::grouping), None,               Precedence::None)),
                (TokenKind::RightParen,   ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::LeftBrace,    ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::RightBrace,   ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Comma,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Dot,          ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Minus,        ParseRule::new(Some(Self::unary),    Some(Self::binary), Precedence::Term)),
                (TokenKind::Plus,         ParseRule::new(None,                 Some(Self::binary), Precedence::Term)),
                (TokenKind::Semicolon,    ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Slash,        ParseRule::new(None,                 Some(Self::binary), Precedence::Factor)),
                (TokenKind::Star,         ParseRule::new(None,                 Some(Self::binary), Precedence::Factor)),
                (TokenKind::Bang,         ParseRule::new(Some(Self::unary),    None,               Precedence::None)),
                (TokenKind::BangEqual,    ParseRule::new(None,                 Some(Self::binary), Precedence::Equality)),
                (TokenKind::Equal,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::EqualEqual,   ParseRule::new(None,                 Some(Self::binary), Precedence::Equality)),
                (TokenKind::Greater,      ParseRule::new(None,                 Some(Self::binary), Precedence::Comparison)),
                (TokenKind::GreaterEqual, ParseRule::new(None,                 Some(Self::binary), Precedence::Comparison)),
                (TokenKind::Less,         ParseRule::new(None,                 Some(Self::binary), Precedence::Comparison)),
                (TokenKind::LessEqual,    ParseRule::new(None,                 Some(Self::binary), Precedence::Comparison)),
                (TokenKind::Identifier,   ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::String,       ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Number,       ParseRule::new(Some(Self::number),   None,               Precedence::None)),
                (TokenKind::And,          ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Class,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Else,         ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::False,        ParseRule::new(Some(Self::literal),  None,               Precedence::None)),
                (TokenKind::For,          ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Fun,          ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::If,           ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Nil,          ParseRule::new(Some(Self::literal),  None,               Precedence::None)),
                (TokenKind::Or,           ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Print,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Return,       ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Super,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::This,         ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::True,         ParseRule::new(Some(Self::literal),  None,               Precedence::None)),
                (TokenKind::Var,          ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::While,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Error,        ParseRule::new(None,                 None,               Precedence::None)),
                (TokenKind::Eof,          ParseRule::new(None,                 None,               Precedence::None)),
            ])
        }
    }

    pub fn advance(&mut self, scanner: &mut Scanner<'a>) {
        self.prev = take(&mut self.curr);

        loop {
            self.curr = scanner.scan_token();
            if self.curr.kind != TokenKind::Error {
                break;
            }

            self.error_at_curr(self.curr.lexeme);
        }
    }

    pub fn consume(&mut self, scanner: &mut Scanner<'a>, kind: TokenKind, message: &'a str) {
        if self.curr.kind == kind {
            self.advance(scanner);
        } else {
            self.error_at_curr(message);
        }
    }

    fn emit_byte(&self, chunk: &mut Chunk, byte: u8) {
        chunk.write(byte, self.prev.line);
    }

    fn emit_bytes(&self, chunk: &mut Chunk, byte1: u8, byte2: u8) {
        self.emit_byte(chunk, byte1);
        self.emit_byte(chunk, byte2);
    }

    fn emit_return(&self, chunk: &mut Chunk) {
        self.emit_byte(chunk, OpCode::Return as u8);
    }

    fn emit_constant(&self, chunk: &mut Chunk, value: Value) {
        chunk.write_constant(value, self.prev.line);
    }

    fn end_compiler(&self, chunk: &mut Chunk) {
        self.emit_return(chunk);
        #[cfg(feature = "debug-print-code")]
        {
            if !self.had_error {
                chunk.disassemble("code");
            }
        }
    }

    fn binary(&mut self, scanner: &mut Scanner<'a>, chunk: &mut Chunk) {
        let op_kind = self.prev.kind;
        let rule = &self.rules[&op_kind];
        self.parse_precedence(
            scanner,
            chunk,
            Precedence::from_u8(rule.precedence as u8 + 1).unwrap(),
        );

        match op_kind {
            TokenKind::BangEqual => self.emit_bytes(chunk, OpCode::Equal as u8, OpCode::Not as u8),
            TokenKind::EqualEqual => self.emit_byte(chunk, OpCode::Equal as u8),
            TokenKind::Greater => self.emit_byte(chunk, OpCode::Greater as u8),
            TokenKind::GreaterEqual => {
                self.emit_bytes(chunk, OpCode::Less as u8, OpCode::Not as u8)
            }
            TokenKind::Less => self.emit_byte(chunk, OpCode::Less as u8),
            TokenKind::LessEqual => {
                self.emit_bytes(chunk, OpCode::Greater as u8, OpCode::Not as u8)
            }
            TokenKind::Plus => self.emit_byte(chunk, OpCode::Add as u8),
            TokenKind::Minus => self.emit_byte(chunk, OpCode::Subtract as u8),
            TokenKind::Star => self.emit_byte(chunk, OpCode::Multiply as u8),
            TokenKind::Slash => self.emit_byte(chunk, OpCode::Divide as u8),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, _scanner: &mut Scanner<'a>, chunk: &mut Chunk) {
        match self.prev.kind {
            TokenKind::False => self.emit_byte(chunk, OpCode::False as u8),
            TokenKind::Nil => self.emit_byte(chunk, OpCode::Nil as u8),
            TokenKind::True => self.emit_byte(chunk, OpCode::True as u8),
            _ => unreachable!(),
        }
    }

    fn grouping(&mut self, scanner: &mut Scanner<'a>, chunk: &mut Chunk) {
        self.expression(scanner, chunk);
        self.consume(
            scanner,
            TokenKind::RightParen,
            "Expect ')' after expression",
        );
    }

    fn number(&mut self, _scanner: &mut Scanner<'a>, chunk: &mut Chunk) {
        self.emit_constant(chunk, Value::Number(self.prev.lexeme.parse().unwrap()));
    }

    fn unary(&mut self, scanner: &mut Scanner<'a>, chunk: &mut Chunk) {
        let op_kind = self.prev.kind;

        self.parse_precedence(scanner, chunk, Precedence::Unary);

        match op_kind {
            TokenKind::Bang => self.emit_byte(chunk, OpCode::Not as u8),
            TokenKind::Minus => self.emit_byte(chunk, OpCode::Negate as u8),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(
        &mut self,
        scanner: &mut Scanner<'a>,
        chunk: &mut Chunk,
        precedence: Precedence,
    ) {
        self.advance(scanner);
        match self.rules[&self.prev.kind].prefix {
            None => {
                self.error("Expect expression");
                return;
            }
            Some(f) => f(self, scanner, chunk),
        };

        while precedence <= self.rules[&self.curr.kind].precedence {
            self.advance(scanner);
            self.rules[&self.prev.kind].infix.unwrap()(self, scanner, chunk);
        }
    }

    pub fn expression(&mut self, scanner: &mut Scanner<'a>, chunk: &mut Chunk) {
        self.parse_precedence(scanner, chunk, Precedence::Assignment);
    }

    fn error_at_curr(&mut self, message: &'a str) {
        self.error_at(self.curr.kind, self.curr.lexeme, self.curr.line, message);
    }

    fn error(&mut self, message: &'a str) {
        self.error_at(self.prev.kind, self.prev.lexeme, self.prev.line, message);
    }

    fn error_at(&mut self, kind: TokenKind, lexeme: &str, line: usize, message: &'a str) {
        if self.panic_mode {
            return;
        }
        eprint!("[line {}] Error", line);

        if kind == TokenKind::Eof {
            eprint!(" at end");
        } else if kind == TokenKind::Error {
            // do nothing
        } else {
            eprint!(" at {}", lexeme);
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }
}
