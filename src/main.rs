use crate::chunk::{Chunk, OpCode};
use crate::rvm::RVM;

mod chunk;
mod common;
mod rvm;
mod value;

fn main() {
    let mut rvm = RVM::new();

    let mut chunk = Chunk::new();

    chunk.write_constant(1.2, 123);
    chunk.write_constant(3.4, 123);

    chunk.write(OpCode::Add as u8, 123);

    chunk.write_constant(5.6, 123);

    chunk.write(OpCode::Divide as u8, 123);
    chunk.write(OpCode::Negate as u8, 123);

    chunk.write(OpCode::Return as u8, 123);

    rvm.interpret(&chunk);

    chunk.free();
}
