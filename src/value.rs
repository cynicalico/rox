use crate::object::{Obj, ObjKind, ObjString};
use std::fmt::{Display, Formatter};
use std::mem::discriminant;

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Nil,
    Number(f64),
    Obj(*mut Obj),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Nil => true,
            Value::Number(n) => *n != 0.0,
            Value::Obj(_) => false,
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Nil
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Number(n) => write!(f, "{}", n),
            Value::Obj(obj) => unsafe {
                match (**obj).kind {
                    ObjKind::String => {
                        let obj_s = *obj as *mut ObjString;
                        write!(f, "{}", (*obj_s).value)
                    }
                }
            },
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
            _ => unsafe {
                if discriminant(self) != discriminant(other) {
                    false
                } else {
                    let Value::Obj(a_obj) = self else {
                        unreachable!()
                    };
                    let Value::Obj(b_obj) = other else {
                        unreachable!()
                    };

                    match (**a_obj).kind {
                        ObjKind::String => match (**b_obj).kind {
                            ObjKind::String => {
                                let a = *a_obj as *mut ObjString;
                                let b = *b_obj as *mut ObjString;
                                (*a).value == (*b).value
                            }
                        },
                    }
                }
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
