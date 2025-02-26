use crate::scanner::{Scanner, TokenKind};

pub fn compile(source: &String) {
    let mut scanner = Scanner::new(source);

    let mut line = usize::MAX;
    loop {
        let token = scanner.scan_token();
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }
        println!("{:12} '{}'", format!("{:?}", token.kind), token.lexeme);

        if token.kind == TokenKind::Eof {
            break;
        }
    }
}
