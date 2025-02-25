use crate::chunk::{Chunk, OpCode};

mod chunk;
mod common;
mod value;

fn main() {
    let mut chunk = Chunk::new();

    for i in 0..300 {
        chunk.write_constant(i as f64 + 300.0, i as usize);
    }

    chunk.write(OpCode::Return as u8, 123);

    chunk.disassemble("test_chunk");

    chunk.free();
}
