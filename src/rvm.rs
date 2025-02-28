use crate::chunk::{Chunk, OpCode};
use crate::compiler::compile;
use crate::object::{Obj, ObjKind, ObjString};
use crate::value::Value;
use num_traits::FromPrimitive;
use std::ptr;

const STACK_MAX: usize = 256;

pub enum InterpretErr {
    Compile,
    Runtime,
}

pub fn interpret(source: &str) -> Result<(), InterpretErr> {
    let mut vm = RVM::new();
    vm.reset_stack();

    let mut chunk = Chunk::new();
    if !compile(&mut vm, source, &mut chunk) {
        Err(InterpretErr::Compile)
    } else {
        unsafe { vm.interpret(&chunk) }
    }
}

pub struct RVM {
    ip: *const u8,
    stack: [Value; STACK_MAX],
    stack_top: *mut Value,
    objects: *mut Obj,
}

impl RVM {
    pub fn new() -> Self {
        Self {
            ip: ptr::null(),
            stack: core::array::from_fn(|_| Value::default()),
            stack_top: ptr::null_mut(),
            objects: ptr::null_mut(),
        }
    }

    pub unsafe fn interpret(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        self.ip = chunk.code.as_ptr();
        self.run(chunk)
    }

    fn reset_stack(&mut self) {
        self.stack_top = self.stack.as_mut_ptr();
    }

    fn runtime_error(&mut self, chunk: &Chunk, message: &str) {
        eprintln!("{}", message);

        let offset = (self.ip as isize - chunk.code.as_ptr() as isize) / size_of::<u8>() as isize;
        let line = chunk.lines[offset as usize];
        eprintln!("[line {}] in script", line);

        self.reset_stack();
    }

    pub(crate) unsafe fn allocate_obj<T>(&mut self, obj: Box<T>) -> *mut Obj {
        let obj = Box::into_raw(obj) as *mut Obj;
        (*obj).next = self.objects;
        self.objects = obj;
        obj
    }

    unsafe fn read_byte(&mut self) -> u8 {
        let byte = *self.ip;
        self.ip = self.ip.add(1);
        byte
    }

    unsafe fn read_constant<'a>(&mut self, chunk: &'a Chunk) -> &'a Value {
        &chunk.constants.values[self.read_byte() as usize]
    }

    unsafe fn read_constant_long<'a>(&mut self, chunk: &'a Chunk) -> &'a Value {
        let v1 = (self.read_byte() as usize) << 16;
        let v2 = (self.read_byte() as usize) << 8;
        let v3 = self.read_byte() as usize;
        &chunk.constants.values[v1 | v2 | v3]
    }

    unsafe fn push(&mut self, value: Value) {
        *self.stack_top = value;
        self.stack_top = self.stack_top.add(1);
    }

    unsafe fn pop<'a>(&mut self) -> &'a Value {
        self.stack_top = self.stack_top.sub(1);
        &*self.stack_top
    }

    unsafe fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretErr> {
        macro_rules! binary_op {
            ($value_enum:expr, $op:tt) => {
                let b = self.pop();
                let a = self.pop();
                if let Value::Number(b) = b
                    && let Value::Number(a) = a
                {
                    self.push($value_enum(a $op b))
                } else {
                    self.runtime_error(chunk, "Operands must be numbers");
                    return Err(InterpretErr::Runtime);
                }
            };
        }

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
                    self.push(constant.clone());
                }
                Some(OpCode::ConstantLong) => {
                    let constant = self.read_constant_long(chunk);
                    self.push(constant.clone());
                }
                Some(OpCode::Nil) => {
                    self.push(Value::Nil);
                }
                Some(OpCode::True) => {
                    self.push(Value::Boolean(true));
                }
                Some(OpCode::False) => {
                    self.push(Value::Boolean(false));
                }
                Some(OpCode::Equal) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Boolean(a == b))
                }
                Some(OpCode::Greater) => {
                    binary_op!(Value::Boolean, >);
                }
                Some(OpCode::Less) => {
                    binary_op!(Value::Boolean, <);
                }
                Some(OpCode::Add) => {
                    let b = self.pop();
                    let a = self.pop();
                    if let Value::Number(b) = b
                        && let Value::Number(a) = a
                    {
                        self.push(Value::Number(a + b))
                    } else if let Value::Obj(b) = b
                        && (**b).kind == ObjKind::String
                        && let Value::Obj(a) = a
                        && (**a).kind == ObjKind::String
                    {
                        let b = &(*(*b as *mut ObjString)).value;
                        let a = &(*(*a as *mut ObjString)).value;
                        let c = Value::Obj(self.allocate_obj(ObjString::new(a.to_string() + b)));
                        self.push(c);
                    } else {
                        self.runtime_error(chunk, "Operands must be numbers or strings");
                        return Err(InterpretErr::Runtime);
                    }
                }
                Some(OpCode::Subtract) => {
                    binary_op!(Value::Number, -);
                }
                Some(OpCode::Multiply) => {
                    binary_op!(Value::Number, *);
                }
                Some(OpCode::Divide) => {
                    binary_op!(Value::Number, /);
                }
                Some(OpCode::Not) => {
                    let v = self.pop();
                    self.push(Value::Boolean(!v.is_falsey()));
                }
                Some(OpCode::Negate) => {
                    let v = self.pop();
                    if let Value::Number(v) = v {
                        self.push(Value::Number(-v))
                    } else {
                        self.runtime_error(chunk, "Operand must be a number");
                        return Err(InterpretErr::Runtime);
                    }
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
