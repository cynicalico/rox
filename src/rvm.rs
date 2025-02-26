use crate::chunk::{Chunk, OpCode};
use crate::compiler::compile;
use crate::value::Value;
use num_traits::FromPrimitive;
use std::{mem, ptr};

const STACK_MAX: usize = 256;

pub enum InterpretErr {
    Compile,
    Runtime,
}

pub fn interpret(source: &str) -> Result<(), InterpretErr> {
    let mut chunk = Chunk::new();
    if !compile(source, &mut chunk) {
        Err(InterpretErr::Compile)
    } else {
        let mut vm = RVM::new();
        vm.reset_stack();

        vm.interpret(&chunk)
    }
}

pub struct RVM {
    ip: *const u8,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
}

impl RVM {
    pub fn new() -> Self {
        Self {
            ip: ptr::null(),
            stack: [0.0; STACK_MAX],
            stack_top: ptr::null_mut(),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        self.ip = chunk.code.as_ptr();
        unsafe { self.run(chunk) }
    }

    unsafe fn read_byte(&mut self) -> u8 {
        let byte = *self.ip;
        self.ip = self.ip.add(1);
        byte
    }

    unsafe fn read_constant(&mut self, chunk: &Chunk) -> Value {
        chunk.constants.values[self.read_byte() as usize]
    }

    unsafe fn read_constant_long(&mut self, chunk: &Chunk) -> Value {
        let v1 = (self.read_byte() as usize) << 16;
        let v2 = (self.read_byte() as usize) << 8;
        let v3 = self.read_byte() as usize;
        chunk.constants.values[v1 | v2 | v3]
    }

    fn reset_stack(&mut self) {
        self.stack_top = self.stack.as_mut_ptr();
    }

    unsafe fn push(&mut self, value: Value) {
        *self.stack_top = value;
        self.stack_top = self.stack_top.add(1);
    }

    unsafe fn pop(&mut self) -> Value {
        self.stack_top = self.stack_top.sub(1);
        *self.stack_top
    }

    unsafe fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        loop {
            #[cfg(feature = "debug-trace-execution")]
            {
                print!("          ");
                let mut slot = self.stack.as_ptr();
                while slot < self.stack_top {
                    print!("[ ");
                    print!("{}", *slot);
                    print!(" ]");
                    slot = slot.add(1);
                }
                println!();
                let offset =
                    (self.ip as isize - chunk.code.as_ptr() as isize) / size_of::<u8>() as isize;
                chunk.disassemble_instruction(offset as usize);
            }

            match OpCode::from_u8(self.read_byte()) {
                Some(OpCode::Constant) => {
                    let constant = self.read_constant(chunk);
                    self.push(constant);
                }
                Some(OpCode::ConstantLong) => {
                    let constant = self.read_constant_long(chunk);
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
