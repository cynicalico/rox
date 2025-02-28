use crate::value::{Value, ValueArray};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Copy, Clone, Debug, FromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    ConstantLong,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Return,
}

pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: ValueArray::new(),
        }
    }

    pub fn free(&mut self) {
        self.code = vec![]; // We want to actually drop the old memory
        self.lines = vec![]; // We want to actually drop the old memory
        self.constants.free();
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_constant(&mut self, value: Value, line: usize) {
        let idx = self.add_constant(value);
        if idx <= u8::MAX as usize {
            self.write(OpCode::Constant as u8, line);
            self.write(idx as u8, line);
        } else {
            self.write(OpCode::ConstantLong as u8, line);
            self.write(((idx >> 16) & 0xff) as u8, line);
            self.write(((idx >> 8) & 0xff) as u8, line);
            self.write((idx & 0xff) as u8, line);
        }
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        match OpCode::from_u8(self.code[offset]) {
            Some(OpCode::Constant) => {
                let constant = self.code[offset + 1] as usize;
                println!(
                    "{:<16} {:4} '{}'",
                    "Constant", constant, self.constants.values[constant]
                );
                offset + 2
            }
            Some(OpCode::ConstantLong) => {
                let constant = ((self.code[offset + 1] as usize) << 16)
                    | ((self.code[offset + 2] as usize) << 8)
                    | (self.code[offset + 3] as usize);
                println!(
                    "{:<16} {:4} '{}'",
                    "ConstantLong", constant, self.constants.values[constant]
                );
                offset + 4
            }
            Some(OpCode::Nil) => {
                println!("Nil");
                offset + 1
            }
            Some(OpCode::True) => {
                println!("True");
                offset + 1
            }
            Some(OpCode::False) => {
                println!("False");
                offset + 1
            }
            Some(OpCode::Equal) => {
                println!("Equal");
                offset + 1
            }
            Some(OpCode::Greater) => {
                println!("Greater");
                offset + 1
            }
            Some(OpCode::Less) => {
                println!("Less");
                offset + 1
            }
            Some(OpCode::Add) => {
                println!("Add");
                offset + 1
            }
            Some(OpCode::Subtract) => {
                println!("Subtract");
                offset + 1
            }
            Some(OpCode::Multiply) => {
                println!("Multiply");
                offset + 1
            }
            Some(OpCode::Divide) => {
                println!("Divide");
                offset + 1
            }
            Some(OpCode::Not) => {
                println!("Not");
                offset + 1
            }
            Some(OpCode::Negate) => {
                println!("Negate");
                offset + 1
            }
            Some(OpCode::Return) => {
                println!("Return");
                offset + 1
            }
            None => {
                println!("Unknown opcode {}", self.code[offset]);
                offset + 1
            }
        }
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.values.len() - 1
    }
}
