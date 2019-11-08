use crate::generation::{Instruction, Instructions};
use crate::*;
use crate::{Arc, HashMap, Id};
use std::f64::INFINITY;
use std::fmt;
use std::mem::replace;

pub struct VM {
    classes: HashMap<Id, Arc<Class>>,
    last_class_id: Id,
    declaring_method: Option<Method>,
    stack: Vec<Arc<Object>>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            classes: HashMap::new(),
            stack: Vec::new(),
            declaring_method: None,
            last_class_id: Id::NULL,
        }
    }

    #[inline]
    fn log_stack(&self) {
        #[cfg(debug_assertions)]
        {
            log::info!("Stack ({})", self.stack.len());
            for (i, o) in self.stack.iter().rev().enumerate() {
                log::info!("{:0>2}  {}", i, o);
            }
        }
    }

    fn do_eval(&mut self, instructions: Vec<Instruction>) {
        for instruction in instructions {
            if let Some(ref mut m) = self.declaring_method {
                match instruction {
                    Instruction::LoadArgument(_)
                    | Instruction::Return(_)
                    | Instruction::LoadLocal(_)
                    | Instruction::ReferenceToClass(_)
                    | Instruction::SendMessage(_)
                    | Instruction::LoadConstU8(_)
                    | Instruction::LoadConstU16(_)
                    | Instruction::LoadConstU32(_)
                    | Instruction::LoadConstU64(_)
                    | Instruction::LoadConstU128(_)
                    | Instruction::LoadConstUBig(_)
                    | Instruction::LoadConstI8(_)
                    | Instruction::LoadConstI16(_)
                    | Instruction::LoadConstI32(_)
                    | Instruction::LoadConstI64(_)
                    | Instruction::LoadConstI128(_)
                    | Instruction::LoadConstIBig(_)
                    | Instruction::LoadConstF32(_)
                    | Instruction::LoadConstF64(_)
                    | Instruction::LoadConstFBig(_) => {
                        m.instructions.push(instruction);
                        continue;
                    }
                    _ => {}
                }
            }

            match instruction {
                Instruction::LoadConstU8(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::U8(value),
                    }));
                }
                Instruction::LoadConstU16(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::U16(value),
                    }));
                }
                Instruction::LoadConstU32(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::U32(value),
                    }));
                }
                Instruction::LoadConstU64(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::U64(value),
                    }));
                }
                Instruction::LoadConstU128(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::U128(value),
                    }));
                }
                Instruction::LoadConstUBig(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::UBig(value),
                    }));
                }
                Instruction::LoadConstI8(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::I8(value),
                    }));
                }
                Instruction::LoadConstI16(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::I16(value),
                    }));
                }
                Instruction::LoadConstI32(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::I32(value),
                    }));
                }
                Instruction::LoadConstI64(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::I64(value),
                    }));
                }
                Instruction::LoadConstI128(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::I128(value),
                    }));
                }
                Instruction::LoadConstIBig(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::IBig(value),
                    }));
                }
                Instruction::LoadConstF32(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::F32(value),
                    }));
                }
                Instruction::LoadConstF64(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::F64(value),
                    }));
                }
                Instruction::LoadConstFBig(value) => {
                    self.stack.push(Arc::new(Object {
                        class: self.classes.values().next().unwrap().clone(),
                        const_value: ConstValue::FBig(value),
                    }));
                }
                Instruction::DeclareClass(id, name) => {
                    self.classes.insert(
                        id,
                        Arc::new(Class {
                            name: name.clone(),
                            methods: HashMap::new(),
                        }),
                    );
                    self.last_class_id = id;
                }
                Instruction::BeginMethod(name) => {
                    self.declaring_method = Some(Method {
                        name: name.clone(),
                        instructions: vec![],
                    });
                }
                Instruction::EndMethod(id) => {
                    let class = self
                        .classes
                        .get_mut(&self.last_class_id)
                        .expect("method declared on unknown class");
                    let class = Arc::get_mut(class)
                        .expect("cannot declare method on class that has objects");
                    let method = replace(&mut self.declaring_method, None)
                        .expect("cannot end method when not started");
                    class.methods.insert(id, Arc::new(method));
                }

                Instruction::LoadArgument(arity) => {
                    self.stack
                        .push(self.stack[self.stack.len() - (arity as usize)].clone());
                }
                Instruction::Return(arity) => {
                    let result = self.stack.pop().expect("method didn't return");
                    for _ in 0..arity {
                        self.stack
                            .pop()
                            .expect("arguments were not loaded properly");
                    }
                    self.stack.push(result);
                }
                Instruction::LoadLocal(index) => {
                    let local = self.stack[self.stack.len() - (index as usize) - 1].clone();
                    self.stack.push(local);
                }
                Instruction::ReferenceToClass(id) => {
                    let class = self.classes.get(&id).expect("deref unknown class");
                    self.stack.push(Arc::new(Object {
                        class: class.clone(),
                        const_value: ConstValue::Nothing,
                    }));
                }
                Instruction::SendMessage(id) => {
                    let receiver = self.stack.last().expect("empty stack");
                    let method = receiver
                        .class
                        .methods
                        .get(&id)
                        .expect("object doesn't understand message")
                        .clone();
                    self.do_eval(method.instructions.clone());
                }
            }
        }
    }

    pub fn eval(&mut self, instructions: Instructions) -> Option<Arc<Object>> {
        self.do_eval(instructions.into());
        let result = self.stack.pop();
        if self.stack.len() > 0 {
            self.log_stack()
        }
        result
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<Id, Arc<Method>>,
}

#[derive(Debug)]
pub struct Object {
    pub class: Arc<Class>,
    pub const_value: ConstValue,
}

#[derive(Debug)]
pub enum ConstValue {
    Nothing,
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    UBig(BigUint),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    IBig(BigInt),
    F32(f32),
    F64(f64),
    FBig(BigFraction),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.const_value {
            ConstValue::Nothing => write!(f, "a {}", self.class.name),
            ConstValue::U8(n) => write!(f, "{}", n),
            ConstValue::U16(n) => write!(f, "{}", n),
            ConstValue::U32(n) => write!(f, "{}", n),
            ConstValue::U64(n) => write!(f, "{}", n),
            ConstValue::U128(n) => write!(f, "{}", n),
            ConstValue::UBig(n) => write!(f, "{}", n),
            ConstValue::I8(n) => write!(f, "{}", n),
            ConstValue::I16(n) => write!(f, "{}", n),
            ConstValue::I32(n) => write!(f, "{}", n),
            ConstValue::I64(n) => write!(f, "{}", n),
            ConstValue::I128(n) => write!(f, "{}", n),
            ConstValue::IBig(n) => write!(f, "{}", n),
            ConstValue::F32(n) => write!(f, "{}", n),
            ConstValue::F64(n) => write!(f, "{}", n),
            ConstValue::FBig(n) => write!(f, "{:.1$}", n, INFINITY as usize),
        }
    }
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub instructions: Vec<Instruction>,
}
