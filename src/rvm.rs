use crate::chunk::{Chunk, OpCode};
use crate::compiler::compile;
use crate::value::Value;
use num_traits::FromPrimitive;

pub enum InterpretErr {
    Compile,
    Runtime,
}

pub fn interpret(source: &String) -> Result<(), InterpretErr> {
    compile(&source);
    Ok(())
}

pub struct RVM {
    ip: usize,
    stack: Vec<Value>,
}

impl RVM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: vec![],
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        self.ip = 0;
        self.run(chunk)
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
        self.stack = vec![];
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        loop {
            #[cfg(feature = "debug-trace-execution")]
            {
                print!("          ");
                for v in &self.stack {
                    print!("[ ");
                    print!("{}", v);
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
}
