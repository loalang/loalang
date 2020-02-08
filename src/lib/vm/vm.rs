use crate::bytecode::Instruction;
use crate::vm::*;
use crate::*;
// use std::mem::replace;

pub struct VM {
    // declaring_method: Option<(u64, Method)>,
    stack: Vec<Arc<Object>>,
    // globals: HashMap<Id, Arc<Object>>,
    call_stack: CallStack,
    // behaviour_names: HashMap<u64, String>,
    program: Vec<Instruction>,
    pc: usize,

    classes: HashMap<usize, Arc<Class>>,
    globals: HashMap<usize, Arc<Object>>,
    declaring_class: usize,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::new(),
            // declaring_method: None,
            // globals: HashMap::new(),
            call_stack: CallStack::new(),
            // behaviour_names: HashMap::new(),
            program: Vec::new(),
            pc: 0,

            classes: HashMap::new(),
            globals: HashMap::new(),
            declaring_class: 0,
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

    pub fn stack(&self) -> &Vec<Arc<Object>> {
        &self.stack
    }

    #[inline]
    fn raw_class_ptr(&self, address: u64) -> VMResult<*const Class> {
        VMResult::Ok(
            expect!(
                self,
                self.classes.get(&(address as usize)),
                "no class found at {:X}",
                address
            )
            .as_ref() as *const _,
        )
    }

    #[inline]
    fn load_object(&mut self, object: Arc<Object>) {
        self.stack.push(object);
        self.pc += 1;
    }

    fn do_eval<M: Runtime>(&mut self) -> VMResult<()> {
        loop {
            match self.program[self.pc] {
                Instruction::Noop => {
                    self.pc += 1;
                }

                Instruction::Halt => {
                    break;
                }

                Instruction::Panic => {
                    return VMResult::Panic(
                        format!(
                            "{:?}",
                            self.stack
                                .pop()
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or(String::new())
                        ),
                        self.call_stack.clone(),
                    )
                }

                Instruction::DeclareClass(ref name) => {
                    let class = Class::new(name.clone(), self.pc);
                    self.classes.insert(self.pc, class);
                    self.declaring_class = self.pc;
                    self.pc += 1;
                }

                Instruction::DeclareMethod(ref name, offset) => {
                    let method = Arc::new(Method {
                        name: name.clone(),
                        offset: offset as usize,
                    });
                    let class = expect!(
                        self,
                        self.classes.get_mut(&self.declaring_class),
                        "method outside class"
                    );
                    let class = expect!(self, Arc::get_mut(class), "class in use");
                    class.methods.insert(offset, method);
                    self.pc += 1;
                }

                Instruction::LoadObject(offset) => {
                    let offset = offset as usize;
                    let class = expect!(self, self.classes.get(&offset), "unknown class");
                    let object = Object::new(class);
                    self.push(object);
                    self.pc += 1;
                }

                Instruction::CallMethod(ref offset, ref uri, line, character) => {
                    let receiver = unwrap!(self, self.top());
                    let method = expect!(
                        self,
                        receiver.class.methods.get(offset),
                        "message #{:X} not understood by {}",
                        offset,
                        receiver
                    )
                    .clone();
                    let return_address = self.pc + 1;
                    self.pc = method.offset;
                    self.call_stack.push(
                        method,
                        return_address,
                        SourceCodeLocation(uri.clone(), line, character),
                    );
                }

                Instruction::LoadLocal(index) => {
                    let local = expect!(
                        self,
                        self.stack.get(index as usize),
                        "not enough locals on the stack"
                    )
                    .clone();
                    self.push(local);
                    self.pc += 1;
                }

                Instruction::DropLocal(index) => {
                    self.stack.remove(index as usize);
                    self.pc += 1;
                }

                Instruction::StoreGlobal(offset) => {
                    let offset = offset as usize;
                    self.globals.insert(
                        offset,
                        expect!(self, self.stack.pop(), "nothing on stack to store"),
                    );
                    self.pc += 1;
                }

                Instruction::LoadGlobal(offset) => {
                    let offset = offset as usize;
                    self.stack
                        .push(expect!(self, self.globals.get(&offset), "global not found").clone());
                    self.pc += 1;
                }

                Instruction::Return(arity) => {
                    let result = unwrap!(self, self.pop());

                    for _ in 0..arity {
                        unwrap!(self, self.pop());
                    }

                    self.push(result);
                    self.pc = expect!(self, self.call_stack.ret(), "empty call stack");
                }

                Instruction::MarkClassString(id) => unsafe {
                    STRING_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassCharacter(id) => unsafe {
                    CHARACTER_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassSymbol(id) => unsafe {
                    SYMBOL_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },

                Instruction::MarkClassU8(id) => unsafe {
                    U8_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassU16(id) => unsafe {
                    U16_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassU32(id) => unsafe {
                    U32_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassU64(id) => unsafe {
                    U64_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassU128(id) => unsafe {
                    U128_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassUBig(id) => unsafe {
                    UBIG_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassI8(id) => unsafe {
                    I8_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassI16(id) => unsafe {
                    I16_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassI32(id) => unsafe {
                    I32_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassI64(id) => unsafe {
                    I64_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassI128(id) => unsafe {
                    I128_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassIBig(id) => unsafe {
                    IBIG_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassF32(id) => unsafe {
                    F32_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassF64(id) => unsafe {
                    F64_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },
                Instruction::MarkClassFBig(id) => unsafe {
                    FBIG_CLASS = unwrap!(self, self.raw_class_ptr(id));
                    self.pc += 1;
                },

                Instruction::LoadConstString(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_string(value))
                }
                Instruction::LoadConstCharacter(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_character(value))
                }
                Instruction::LoadConstSymbol(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_symbol(value))
                }
                Instruction::LoadConstU8(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_u8(value))
                }
                Instruction::LoadConstU16(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_u16(value))
                }
                Instruction::LoadConstU32(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_u32(value))
                }
                Instruction::LoadConstU64(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_u64(value))
                }
                Instruction::LoadConstU128(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_u128(value))
                }
                Instruction::LoadConstUBig(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_ubig(value))
                }
                Instruction::LoadConstI8(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_i8(value))
                }
                Instruction::LoadConstI16(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_i16(value))
                }
                Instruction::LoadConstI32(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_i32(value))
                }
                Instruction::LoadConstI64(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_i64(value))
                }
                Instruction::LoadConstI128(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_i128(value))
                }
                Instruction::LoadConstIBig(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_ibig(value))
                }
                Instruction::LoadConstF32(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_f32(value))
                }
                Instruction::LoadConstF64(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_f64(value))
                }
                Instruction::LoadConstFBig(ref value) => {
                    let value = value.clone();
                    self.load_object(Object::box_fbig(value))
                }
            }
            /*

                unsafe {
                    match instruction {


                        Instruction::LoadConstString(value) => {
                            self.stack.push(Object::box_string(value))
                        }
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
                            log::info!("Stack after method (before return) (arity {}):", arity);
                            self.log_stack();
                            let result = expect!(self, self.stack.pop(), "method didn't return");
                            for _ in 0..arity {
                                expect!(self, self.stack.pop(), "arguments were not loaded properly");
                            }
                            self.stack.push(result);
                            log::info!("Stack after method (arity {}):", arity);
                            self.log_stack();
                        }
                        Instruction::LoadLocal(index) => {
                            let local = self.stack[self.stack.len() - (index as usize) - 1].clone();
                            self.stack.push(local);
                        }
                        Instruction::DropLocal(index) => {
                            self.stack.remove(self.stack.len() - (index as usize));
                        }
                        Instruction::ReferenceToClass(id) => {
                            let class = expect!(self, self.classes.get(&id), "deref unknown class");
                            self.stack.push(Arc::new(Object {
                                class: class.clone(),
                                const_value: ConstValue::Nothing,
                            }));
                        }
                        Instruction::SendMessage(location, id) => {
                            log::info!("Stack before message {}:", location);
                            self.log_stack();
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
            */
        }
        VMResult::Ok(())
    }

    fn eval_catch<M: Runtime>(&mut self, instructions: Vec<Instruction>) -> bool {
        self.pc = self.program.len();
        self.program.extend(instructions);
        match self.do_eval::<M>() {
            VMResult::Ok(()) => false,
            VMResult::Panic(message, call_stack) => {
                M::print_panic(message, call_stack);
                true
            }
        }
    }

    pub fn eval<M: Runtime>(&mut self, instructions: Vec<Instruction>) {
        self.eval_catch::<M>(instructions);
    }

    pub fn eval_pop<M: Runtime>(&mut self, instructions: Vec<Instruction>) -> Option<Arc<Object>> {
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

    pub fn top(&self) -> VMResult<&Arc<Object>> {
        VMResult::Ok(expect!(
            self,
            self.stack.last(),
            "tried to peek at top of empty stack"
        ))
    }

    pub fn push(&mut self, object: Arc<Object>) {
        self.stack.push(object);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::*;
    use crate::bytecode::{BytecodeEncoding, Instruction as BytecodeInstruction};

    fn assert_evaluates(input: &str) {
        let assembly = Parser::new().parse(input).unwrap();
        let mut vm = VM::new();
        let instructions: Vec<BytecodeInstruction> = assembly.into();
        vm.eval::<()>(instructions.rotate().unwrap());
    }

    fn assert_evaluates_to(input: &str, expected: &str) {
        let assembly = Parser::new().parse(input).unwrap();
        let mut vm = VM::new();
        let instructions: Vec<BytecodeInstruction> = assembly.into();
        let result = vm.eval_pop::<()>(instructions.rotate().unwrap()).unwrap();

        assert_eq!(result.to_string(), expected);
        assert_eq!(vm.stack().len(), 0, "stack should be empty");
    }

    #[test]
    fn noops_and_halt() {
        assert_evaluates(
            r#"
                Noop
                Noop
                Halt
            "#,
        );
    }

    #[test]
    fn declare_and_instantiate_class() {
        assert_evaluates_to(
            r#"
            @SomeClass
                DeclareClass "SomeClass"
                DeclareMethod "someMethod" @SomeClass#someMethod

            LoadObject @SomeClass
            CallMethod @SomeClass#someMethod "call site" 1 1
            Halt

            @SomeClass#someMethod
                LoadLocal 0
                Return 1
            "#,
            "a SomeClass",
        );
    }

    #[test]
    fn inherited_method() {
        assert_evaluates_to(
            r#"
            @B
                DeclareClass "B"
                DeclareMethod "a" @A#a

            @A
                DeclareClass "A"
                DeclareMethod "a" @A#a

            LoadObject @B
            CallMethod @A#a "call site" 1 1
            Halt

            @A#a
                LoadLocal 0
                Return 1
            "#,
            "a B",
        );
    }

    #[test]
    fn all_consts() {
        assert_evaluates_to(
            r#"
            @String
                DeclareClass "String"
                MarkClassString @String
            LoadConstString "Hello"
            Halt
            "#,
            "Hello",
        );
        assert_evaluates_to(
            r#"
            @Character
                DeclareClass "Character"
                MarkClassCharacter @Character
            LoadConstCharacter 'x'
            Halt
            "#,
            "x",
        );
        assert_evaluates_to(
            r#"
            @Symbol
                DeclareClass "Symbol"
                MarkClassSymbol @Symbol
            LoadConstSymbol #hello
            Halt
            "#,
            "#hello",
        );
        fn assert_number_evaluates(name: &str, literal: &str) {
            assert_evaluates_to(
                format!(
                    r#"
                    @Class
                        DeclareClass "Class"
                        MarkClass{} @Class
                    LoadConst{} {}
                    Halt
                    "#,
                    name, name, literal
                )
                .as_ref(),
                literal,
            );
        }
        assert_number_evaluates("U8", "255");
        assert_number_evaluates("U16", "1024");
        assert_number_evaluates("U32", "1024");
        assert_number_evaluates("U64", "1024");
        assert_number_evaluates("U128", "1024");
        assert_number_evaluates("UBig", "1024");
        assert_number_evaluates("I8", "25");
        assert_number_evaluates("I16", "1024");
        assert_number_evaluates("I32", "1024");
        assert_number_evaluates("I64", "1024");
        assert_number_evaluates("I128", "1024");
        assert_number_evaluates("IBig", "1024");
        assert_number_evaluates("I8", "-25");
        assert_number_evaluates("I16", "-1024");
        assert_number_evaluates("I32", "-1024");
        assert_number_evaluates("I64", "-1024");
        assert_number_evaluates("I128", "-1024");
        assert_number_evaluates("IBig", "-1024");
    }

    #[test]
    fn globals() {
        assert_evaluates_to(
            r#"
            @String
                DeclareClass "String"
                MarkClassString @String

            @global
                LoadConstString "global value"
                StoreGlobal @global

            LoadGlobal @global

            Halt
            "#,
            "global value",
        );
    }
}
