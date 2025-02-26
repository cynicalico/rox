use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use num_traits::FromPrimitive;

const STACK_SIZE: usize = 256;

pub enum InterpretErr {
    Compile,
    Runtime,
}

pub struct RVM {
    ip: usize,
    stack: [Value; STACK_SIZE],
    stack_top: usize,
}

impl RVM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: [0.0; STACK_SIZE],
            stack_top: 0,
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        self.ip = 0;
        self.run(chunk)
    }

    pub fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    pub fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
    }

    pub fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        loop {
            #[cfg(feature = "debug-trace-execution")]
            {
                print!("          ");
                for i in 0..self.stack_top {
                    print!("[ ");
                    print!("{}", self.stack[i]);
                    print!(" ]");
                }
                println!();
                chunk.disassemble_instruction(self.ip);
            }

            match OpCode::from_u8(self.read_byte(chunk)) {
                Some(OpCode::Constant) => {
                    let constant = self.read_constant(chunk);
                    self.push(constant);
                }
                Some(OpCode::Add) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a + b);
                }
                Some(OpCode::Subtract) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a - b);
                }
                Some(OpCode::Multiply) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a * b);
                }
                Some(OpCode::Divide) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(a / b);
                }
                Some(OpCode::Negate) => {
                    let v = self.pop();
                    self.push(-v);
                }
                Some(OpCode::Return) => {
                    println!("{}", self.pop());
                    return Ok(());
                }
                _ => (),
            }
        }
    }

    fn read_byte(&mut self, chunk: &Chunk) -> u8 {
        let byte = chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        chunk.constants.values[self.read_byte(chunk) as usize]
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }
}
