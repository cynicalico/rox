use std::fmt::{Display, Formatter};
#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => true,
            Value::Number(n) => *n != 0.0,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Number(0.0)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Boolean(a) => match other {
                Value::Boolean(b) => a == b,
                _ => false,
            },
            Value::Nil => match other {
                Value::Nil => true,
                _ => false,
            },
            Value::Number(a) => match other {
                Value::Number(b) => a == b,
                _ => false,
            },
        }
    }
}

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
