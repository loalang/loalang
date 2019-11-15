use crate::generation::{Instruction, Instructions};
use crate::vm::*;
use crate::*;
use std::mem::replace;

pub struct VM {
    classes: HashMap<Id, Arc<Class>>,
    declaring_method: Option<(u64, Method)>,
    stack: Vec<Arc<Object>>,
    globals: HashMap<Id, Arc<Object>>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            classes: HashMap::new(),
            stack: Vec::new(),
            declaring_method: None,
            globals: HashMap::new(),
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

    #[inline]
    fn raw_class_ptr(&self, id: Id) -> *const Class {
        self.classes.get(&id).expect("unknown class").as_ref() as *const _
    }

    fn do_eval<M: NativeMethods>(&mut self, instructions: Vec<Instruction>) {
        for instruction in instructions {
            if let Some((_, ref mut m)) = self.declaring_method {
                match instruction {
                    Instruction::LoadArgument(_)
                    | Instruction::CallNative(_)
                    | Instruction::Return(_)
                    | Instruction::LoadLocal(_)
                    | Instruction::ReferenceToClass(_)
                    | Instruction::SendMessage(_)
                    | Instruction::LoadGlobal(_)
                    | Instruction::LoadConstString(_)
                    | Instruction::LoadConstCharacter(_)
                    | Instruction::LoadConstSymbol(_)
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

            unsafe {
                match instruction {
                    Instruction::MarkClassString(id) => {
                        STRING_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassCharacter(id) => {
                        CHARACTER_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassSymbol(id) => {
                        SYMBOL_CLASS = self.raw_class_ptr(id);
                    }

                    Instruction::MarkClassU8(id) => {
                        U8_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassU16(id) => {
                        U16_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassU32(id) => {
                        U32_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassU64(id) => {
                        U64_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassU128(id) => {
                        U128_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassUBig(id) => {
                        UBIG_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassI8(id) => {
                        I8_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassI16(id) => {
                        I16_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassI32(id) => {
                        I32_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassI64(id) => {
                        I64_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassI128(id) => {
                        I128_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassIBig(id) => {
                        IBIG_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassF32(id) => {
                        F32_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassF64(id) => {
                        F64_CLASS = self.raw_class_ptr(id);
                    }
                    Instruction::MarkClassFBig(id) => {
                        FBIG_CLASS = self.raw_class_ptr(id);
                    }

                    Instruction::LoadGlobal(id) => self
                        .stack
                        .push(self.globals.get(&id).expect("global not found").clone()),

                    Instruction::StoreGlobal(id) => {
                        self.globals
                            .insert(id, self.stack.pop().expect("nothing on stack to store"));
                    }

                    Instruction::LoadConstString(value) => {
                        self.stack.push(Object::box_string(value))
                    }
                    Instruction::LoadConstCharacter(value) => {
                        self.stack.push(Object::box_character(value))
                    }
                    Instruction::LoadConstSymbol(value) => {
                        self.stack.push(Object::box_symbol(value))
                    }
                    Instruction::LoadConstU8(value) => self.stack.push(Object::box_u8(value)),
                    Instruction::LoadConstU16(value) => self.stack.push(Object::box_u16(value)),
                    Instruction::LoadConstU32(value) => self.stack.push(Object::box_u32(value)),
                    Instruction::LoadConstU64(value) => self.stack.push(Object::box_u64(value)),
                    Instruction::LoadConstU128(value) => self.stack.push(Object::box_u128(value)),
                    Instruction::LoadConstUBig(value) => self.stack.push(Object::box_ubig(value)),
                    Instruction::LoadConstI8(value) => self.stack.push(Object::box_i8(value)),
                    Instruction::LoadConstI16(value) => self.stack.push(Object::box_i16(value)),
                    Instruction::LoadConstI32(value) => self.stack.push(Object::box_i32(value)),
                    Instruction::LoadConstI64(value) => self.stack.push(Object::box_i64(value)),
                    Instruction::LoadConstI128(value) => self.stack.push(Object::box_i128(value)),
                    Instruction::LoadConstIBig(value) => self.stack.push(Object::box_ibig(value)),
                    Instruction::LoadConstF32(value) => self.stack.push(Object::box_f32(value)),
                    Instruction::LoadConstF64(value) => self.stack.push(Object::box_f64(value)),
                    Instruction::LoadConstFBig(value) => self.stack.push(Object::box_fbig(value)),
                    Instruction::DeclareClass(id, name) => {
                        self.classes.insert(
                            id,
                            Arc::new(Class {
                                name: name.clone(),
                                methods: HashMap::new(),
                            }),
                        );
                    }
                    Instruction::BeginMethod(id, name) => {
                        self.declaring_method = Some((
                            id,
                            Method {
                                name: name.clone(),
                                instructions: vec![],
                            },
                        ));
                    }
                    Instruction::EndMethod(class_id) => {
                        let class = self
                            .classes
                            .get_mut(&class_id)
                            .expect("method declared on unknown class");
                        let class = Arc::get_mut(class)
                            .expect("cannot declare method on class that has objects");
                        let (id, method) = replace(&mut self.declaring_method, None)
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
                        self.do_eval::<M>(method.instructions.clone());
                    }
                    Instruction::InheritMethod(superclass_id, subclass_id, behaviour_id) => {
                        let method = {
                            let super_class = self
                                .classes
                                .get(&superclass_id)
                                .expect("inheriting from unknown class");

                            super_class
                                .methods
                                .get(&behaviour_id)
                                .expect("inheriting unknown method")
                                .clone()
                        };
                        let sub_class = self
                            .classes
                            .get_mut(&subclass_id)
                            .expect("unknown class cannot inherit method");
                        let sub_class = Arc::get_mut(sub_class)
                            .expect("cannot inherit method onto class that has objects");

                        sub_class.methods.insert(behaviour_id, method);
                    }
                    Instruction::CallNative(method) => {
                        M::call(self, method);
                    }
                }
            }
        }
    }

    pub fn eval<M: NativeMethods>(&mut self, instructions: Instructions) {
        self.do_eval::<M>(instructions.into());
    }

    pub fn eval_pop<M: NativeMethods>(
        &mut self,
        instructions: Instructions,
    ) -> Option<Arc<Object>> {
        self.do_eval::<M>(instructions.into());
        let result = self.stack.pop();
        if self.stack.len() > 0 {
            self.log_stack()
        }
        result
    }

    pub fn pop(&mut self) -> Arc<Object> {
        self.stack.pop().expect("tried to pop empty stack")
    }

    pub fn push(&mut self, object: Arc<Object>) {
        self.stack.push(object);
    }
}
