use std::fmt::{Display, Formatter};

pub type Value = f64;

// impl Display for Value {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self)
//     }
// }

pub struct ValueArray {
    pub values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn free(&mut self) {
        self.values = vec![]; // We want to actually drop the old memory
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }
}
