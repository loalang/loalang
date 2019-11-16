use crate::generation::{Instruction, Instructions};
use crate::vm::*;
use crate::*;
use std::mem::replace;

pub type CallStack = Vec<(String, Arc<Class>, Arc<Method>)>;

pub struct VM {
    classes: HashMap<Id, Arc<Class>>,
    declaring_method: Option<(u64, Method)>,
    stack: Vec<Arc<Object>>,
    globals: HashMap<Id, Arc<Object>>,
    call_stack: CallStack,
    behaviour_names: HashMap<u64, String>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            classes: HashMap::new(),
            stack: Vec::new(),
            declaring_method: None,
            globals: HashMap::new(),
            call_stack: Vec::new(),
            behaviour_names: HashMap::new(),
        }
    }

    pub fn panic<T>(&self, message: String) -> VMResult<T> {
        VMResult::Panic(message, self.call_stack.clone())
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
    fn raw_class_ptr(&self, id: Id) -> VMResult<*const Class> {
        VMResult::Ok(
            expect!(self, self.classes.get(&id), "unknown class {}", id).as_ref() as *const _,
        )
    }

    fn do_eval<M: Runtime>(&mut self, instructions: Vec<Instruction>) -> VMResult<()> {
        for instruction in instructions {
            if let Some((_, ref mut m)) = self.declaring_method {
                match instruction {
                    Instruction::LoadArgument(_)
                    | Instruction::CallNative(_)
                    | Instruction::Return(_)
                    | Instruction::LoadLocal(_)
                    | Instruction::ReferenceToClass(_)
                    | Instruction::SendMessage(_, _)
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
                        STRING_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassCharacter(id) => {
                        CHARACTER_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassSymbol(id) => {
                        SYMBOL_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }

                    Instruction::MarkClassU8(id) => {
                        U8_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassU16(id) => {
                        U16_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassU32(id) => {
                        U32_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassU64(id) => {
                        U64_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassU128(id) => {
                        U128_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassUBig(id) => {
                        UBIG_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassI8(id) => {
                        I8_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassI16(id) => {
                        I16_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassI32(id) => {
                        I32_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassI64(id) => {
                        I64_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassI128(id) => {
                        I128_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassIBig(id) => {
                        IBIG_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassF32(id) => {
                        F32_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassF64(id) => {
                        F64_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }
                    Instruction::MarkClassFBig(id) => {
                        FBIG_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    }

                    Instruction::LoadGlobal(id) => self
                        .stack
                        .push(expect!(self, self.globals.get(&id), "global not found").clone()),

                    Instruction::StoreGlobal(id) => {
                        self.globals.insert(
                            id,
                            expect!(self, self.stack.pop(), "nothing on stack to store"),
                        );
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
                        self.behaviour_names.insert(id, name.clone());
                        self.declaring_method = Some((
                            id,
                            Method {
                                name,
                                instructions: vec![],
                            },
                        ));
                    }
                    Instruction::EndMethod(class_id) => {
                        let class = expect!(
                            self,
                            self.classes.get_mut(&class_id),
                            "method declared on unknown class"
                        );
                        let class = expect!(
                            self,
                            Arc::get_mut(class),
                            "cannot declare method on class that has objects"
                        );
                        let (id, method) = expect!(
                            self,
                            replace(&mut self.declaring_method, None),
                            "cannot end method when not started"
                        );
                        class.methods.insert(id, Arc::new(method));
                    }

                    Instruction::LoadArgument(arity) => {
                        self.stack
                            .push(self.stack[self.stack.len() - (arity as usize)].clone());
                    }
                    Instruction::Return(arity) => {
                        let result = expect!(self, self.stack.pop(), "method didn't return");
                        for _ in 0..arity {
                            expect!(self, self.stack.pop(), "arguments were not loaded properly");
                        }
                        self.stack.push(result);
                    }
                    Instruction::LoadLocal(index) => {
                        let local = self.stack[self.stack.len() - (index as usize) - 1].clone();
                        self.stack.push(local);
                    }
                    Instruction::ReferenceToClass(id) => {
                        let class = expect!(self, self.classes.get(&id), "deref unknown class");
                        self.stack.push(Arc::new(Object {
                            class: class.clone(),
                            const_value: ConstValue::Nothing,
                        }));
                    }
                    Instruction::SendMessage(location, id) => {
                        let receiver = expect!(self, self.stack.last(), "empty stack");
                        let method = expect!(
                            self,
                            receiver.class.methods.get(&id),
                            "{} doesn't understand message {}",
                            receiver.class.name,
                            self.behaviour_names.get(&id).cloned().unwrap_or("".into()),
                        )
                        .clone();
                        self.call_stack
                            .push((location, receiver.class.clone(), method.clone()));
                        unwrap!(self, self.do_eval::<M>(method.instructions.clone()));
                        self.call_stack.pop();
                    }
                    Instruction::InheritMethod(superclass_id, subclass_id, behaviour_id) => {
                        let method = {
                            let super_class = expect!(
                                self,
                                self.classes.get(&superclass_id),
                                "inheriting from unknown class"
                            );

                            expect!(
                                self,
                                super_class.methods.get(&behaviour_id),
                                "inheriting unknown method"
                            )
                            .clone()
                        };
                        let sub_class = expect!(
                            self,
                            self.classes.get_mut(&subclass_id),
                            "unknown class cannot inherit method"
                        );
                        let sub_class = expect!(
                            self,
                            Arc::get_mut(sub_class),
                            "cannot inherit method onto class that has objects"
                        );

                        sub_class.methods.insert(behaviour_id, method);
                    }
                    Instruction::CallNative(method) => {
                        M::call(self, method);
                    }
                }
            }
        }
        VMResult::Ok(())
    }

    fn eval_catch<M: Runtime>(&mut self, instructions: Instructions) -> bool {
        match self.do_eval::<M>(instructions.into()) {
            VMResult::Ok(()) => false,
            VMResult::Panic(message, call_stack) => {
                M::print_panic(message, call_stack);
                true
            }
        }
    }

    pub fn eval<M: Runtime>(&mut self, instructions: Instructions) {
        self.eval_catch::<M>(instructions);
    }

    pub fn eval_pop<M: Runtime>(&mut self, instructions: Instructions) -> Option<Arc<Object>> {
        if self.eval_catch::<M>(instructions) {
            None
        } else {
            let result = self.stack.pop();
            if self.stack.len() > 0 {
                self.log_stack()
            }
            result
        }
    }

    pub fn pop(&mut self) -> VMResult<Arc<Object>> {
        VMResult::Ok(expect!(self, self.stack.pop(), "tried to pop empty stack"))
    }

    pub fn push(&mut self, object: Arc<Object>) {
        self.stack.push(object);
    }
}
