#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TokenKind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier,
    String,
    Number,

    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Error,
    Eof,
}

pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub line: usize,
}

pub struct Scanner<'a> {
    source: &'a String,
    start: usize,
    curr: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        Self {
            source,
            start: 0,
            curr: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.curr;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }

        let c = self.advance();

        if c.is_ascii_alphabetic() || c == '_' {
            return self.identifier();
        }

        if c.is_ascii_digit() {
            return self.number();
        }

        match c {
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '{' => self.make_token(TokenKind::LeftBrace),
            '}' => self.make_token(TokenKind::RightBrace),
            ';' => self.make_token(TokenKind::Semicolon),
            ',' => self.make_token(TokenKind::Comma),
            '.' => self.make_token(TokenKind::Dot),
            '-' => self.make_token(TokenKind::Minus),
            '+' => self.make_token(TokenKind::Plus),
            '/' => self.make_token(TokenKind::Slash),
            '*' => self.make_token(TokenKind::Star),
            '!' => {
                let kind = if self.advance_match('=') {
                    TokenKind::Bang
                } else {
                    TokenKind::BangEqual
                };
                self.make_token(kind)
            }
            '=' => {
                let kind = if self.advance_match('=') {
                    TokenKind::Equal
                } else {
                    TokenKind::EqualEqual
                };
                self.make_token(kind)
            }
            '<' => {
                let kind = if self.advance_match('=') {
                    TokenKind::Less
                } else {
                    TokenKind::LessEqual
                };
                self.make_token(kind)
            }
            '>' => {
                let kind = if self.advance_match('=') {
                    TokenKind::Greater
                } else {
                    TokenKind::GreaterEqual
                };
                self.make_token(kind)
            }
            '"' => self.string(),
            _ => self.error_token("Unexpected character"),
        }
    }

    fn is_at_end(&self) -> bool {
        self.curr == self.source.len()
    }

    fn peek(&self, n: usize) -> char {
        if self.curr + n >= self.source.len() {
            '\0'
        } else {
            self.source[self.curr + n..].chars().next().unwrap()
        }
    }

    fn peek_front(&self, n: usize) -> char {
        if self.start + n >= self.source.len() {
            '\0'
        } else {
            self.source[self.start + n..].chars().next().unwrap()
        }
    }

    fn advance(&mut self) -> char {
        let c = self.peek(0);
        self.curr += 1;
        c
    }

    fn advance_match(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek(0) != expected {
            false
        } else {
            self.curr += 1;
            true
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            let c = self.peek(0);
            if c == ' ' || c == '\r' || c == '\t' {
                self.advance();
            } else if c == '\n' {
                self.line += 1;
                self.advance();
            } else if c == '/' {
                if self.peek(1) == '/' {
                    while !self.is_at_end() && self.peek(0) != '\n' {
                        self.advance();
                    }
                } else {
                    return;
                }
            } else {
                return;
            }
        }
    }

    fn check_keyword(&self, start: usize, rest: &'static str, kind: TokenKind) -> TokenKind {
        if self.curr - self.start == start + rest.len()
            && &self.source[self.start + start..self.start + start + rest.len()] == rest
        {
            kind
        } else {
            TokenKind::Identifier
        }
    }

    fn identifier_kind(&self) -> TokenKind {
        match self.peek_front(0) {
            'a' => self.check_keyword(1, "nd", TokenKind::And),
            'c' => self.check_keyword(1, "lass", TokenKind::Class),
            'e' => self.check_keyword(1, "lse", TokenKind::Else),
            'f' => {
                if self.curr - self.start > 1 {
                    match self.peek_front(1) {
                        'a' => self.check_keyword(2, "lse", TokenKind::False),
                        'o' => self.check_keyword(2, "r", TokenKind::For),
                        'u' => self.check_keyword(2, "n", TokenKind::Fun),
                        _ => TokenKind::Identifier,
                    }
                } else {
                    TokenKind::Identifier
                }
            }
            'i' => self.check_keyword(1, "f", TokenKind::If),
            'n' => self.check_keyword(1, "il", TokenKind::Nil),
            'o' => self.check_keyword(1, "r", TokenKind::Or),
            'p' => self.check_keyword(1, "rint", TokenKind::Print),
            'r' => self.check_keyword(1, "eturn", TokenKind::Return),
            's' => self.check_keyword(1, "uper", TokenKind::Super),
            't' => {
                if self.curr - self.start > 1 {
                    match self.peek_front(1) {
                        'h' => self.check_keyword(2, "is", TokenKind::This),
                        'r' => self.check_keyword(2, "ue", TokenKind::True),
                        _ => TokenKind::Identifier,
                    }
                } else {
                    TokenKind::Identifier
                }
            }
            'v' => self.check_keyword(1, "ar", TokenKind::Var),
            'w' => self.check_keyword(1, "hile", TokenKind::While),
            _ => TokenKind::Identifier,
        }
    }

    fn identifier(&mut self) -> Token {
        while self.peek(0).is_ascii_alphanumeric() || self.peek(0) == '_' {
            self.advance();
        }

        self.make_token(self.identifier_kind())
    }

    fn number(&mut self) -> Token {
        while self.peek(0).is_ascii_digit() {
            self.advance();
        }

        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            self.advance(); // consume '.'

            while self.peek(0).is_ascii_digit() {
                self.advance();
            }
        }

        self.make_token(TokenKind::Number)
    }

    fn string(&mut self) -> Token {
        while !self.is_at_end() && self.peek(0) != '"' {
            if self.peek(0) == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if !self.is_at_end() {
            self.advance(); // closing quote
            self.make_token(TokenKind::String)
        } else {
            self.error_token("Unterminated string")
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            lexeme: &self.source[self.start..self.curr],
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token {
        Token {
            kind: TokenKind::Error,
            lexeme: message,
            line: self.line,
        }
    }
}
