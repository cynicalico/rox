use std::ptr;

#[derive(Clone, Debug, PartialEq)]
pub enum ObjKind {
    String,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Obj {
    pub kind: ObjKind,
    pub next: *mut Obj,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ObjString {
    pub obj: Obj,
    pub value: String,
}

impl ObjString {
    pub fn new(value: String) -> Box<Self> {
        Box::new(Self {
            obj: Obj {
                kind: ObjKind::String,
                next: ptr::null_mut(),
            },
            value,
        })
    }
}
