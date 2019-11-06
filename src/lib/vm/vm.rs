use crate::generation::{Instruction, Instructions};
use crate::{Arc, HashMap, Id};
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
            match instruction {
                Instruction::LoadConstU8(value) => {
                    println!("Box UInt8 {}", value);
                }
                Instruction::LoadConstU16(value) => {
                    println!("Box UInt16 {}", value);
                }
                Instruction::LoadConstU32(value) => {
                    println!("Box UInt32 {}", value);
                }
                Instruction::LoadConstU64(value) => {
                    println!("Box UInt64 {}", value);
                }
                Instruction::LoadConstU128(value) => {
                    println!("Box UInt128 {}", value);
                }
                Instruction::LoadConstUBig(value) => {
                    println!("Box BigNatural {}", value);
                }
                Instruction::LoadConstI8(value) => {
                    println!("Box Int8 {}", value);
                }
                Instruction::LoadConstI16(value) => {
                    println!("Box Int16 {}", value);
                }
                Instruction::LoadConstI32(value) => {
                    println!("Box Int32 {}", value);
                }
                Instruction::LoadConstI64(value) => {
                    println!("Box Int64 {}", value);
                }
                Instruction::LoadConstI128(value) => {
                    println!("Box Int128 {}", value);
                }
                Instruction::LoadConstIBig(value) => {
                    println!("Box BigInteger {}", value);
                }
                Instruction::LoadConstF32(value) => {
                    println!("Box Float32 {}", value);
                }
                Instruction::LoadConstF64(value) => {
                    println!("Box Float64 {}", value);
                }
                Instruction::LoadConstFBig(value) => {
                    println!("Box BigFloat {:.1$}", value, 999999999);
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

                Instruction::LoadArgument(arity) => match self.declaring_method {
                    Some(ref mut m) => {
                        m.instructions.push(instruction.clone());
                    }
                    None => self
                        .stack
                        .push(self.stack[self.stack.len() - (arity as usize)].clone()),
                },
                Instruction::Return(arity) => match self.declaring_method {
                    Some(ref mut m) => {
                        m.instructions.push(instruction.clone());
                    }
                    None => {
                        let result = self.stack.pop().expect("method didn't return");
                        for _ in 0..arity {
                            self.stack
                                .pop()
                                .expect("arguments were not loaded properly");
                        }
                        self.stack.push(result);
                    }
                },
                Instruction::LoadLocal(index) => match self.declaring_method {
                    Some(ref mut m) => {
                        m.instructions.push(instruction.clone());
                    }
                    None => {
                        let local = self.stack[self.stack.len() - (index as usize) - 1].clone();
                        self.stack.push(local);
                    }
                },
                Instruction::ReferenceToClass(id) => match self.declaring_method {
                    Some(ref mut m) => {
                        m.instructions.push(instruction.clone());
                    }
                    None => {
                        let class = self.classes.get(&id).expect("deref unknown class");
                        self.stack.push(Arc::new(Object {
                            class: class.clone(),
                        }));
                    }
                },
                Instruction::SendMessage(id) => match self.declaring_method {
                    Some(ref mut m) => {
                        m.instructions.push(instruction.clone());
                    }
                    None => {
                        let receiver = self.stack.last().expect("empty stack");
                        let method = receiver
                            .class
                            .methods
                            .get(&id)
                            .expect("object doesn't understand message")
                            .clone();
                        self.do_eval(method.instructions.clone());
                    }
                },
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
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a {}", self.class.name)
    }
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub instructions: Vec<Instruction>,
}
