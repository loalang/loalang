use crate::bytecode::Instruction;
use crate::vm::*;
use crate::*;

pub struct VM {
    stack: Stack<Arc<Object>>,
    call_stack: CallStack,
    program: Vec<Instruction>,
    pc: usize,

    classes: HashMap<u64, Arc<Class>>,
    methods: HashMap<u64, Arc<Method>>,
    globals: HashMap<u64, Arc<Object>>,
    declaring_class: u64,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Stack::new(),
            call_stack: CallStack::new(),
            program: Vec::new(),
            pc: 0,

            classes: HashMap::new(),
            methods: HashMap::new(),
            globals: HashMap::new(),
            declaring_class: 0,
        }
    }

    pub fn panic<T>(&mut self, message: String) -> VMResult<T> {
        VMResult::Panic(message, self.call_stack.detach())
    }

    #[inline]
    fn log_stack(&self) {
        #[cfg(debug_assertions)]
        log::info!("{:?}", self.stack);
    }

    pub fn stack(&self) -> &Stack<Arc<Object>> {
        &self.stack
    }

    pub fn print_stack(&self) {
        println!("{:?}", self.stack);
    }

    #[inline]
    fn raw_class_ptr(&mut self, address: u64) -> VMResult<*const Class> {
        VMResult::Ok(
            expect!(
                self,
                self.classes.get(&address),
                "no class found at {:X}",
                address
            )
            .as_ref() as *const _,
        )
    }

    #[inline]
    fn load_object(&mut self, object: Arc<Object>) {
        self.push(object);
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
                        self.call_stack.detach(),
                    )
                }

                Instruction::DumpStack => {
                    self.print_stack();
                    self.pc += 1;
                }

                Instruction::DeclareClass(ref name) => {
                    let class = Class::new(name.clone(), self.pc);
                    let class_id = self.pc as u64;
                    self.classes.insert(class_id, class);
                    self.declaring_class = class_id;
                    self.pc += 1;
                }

                Instruction::DeclareMethod(ref name, offset) => {
                    let method = Arc::new(Method {
                        name: name.clone(),
                        offset: offset as usize,
                    });
                    self.methods.insert(offset, method);
                    self.pc += 1;
                }

                Instruction::UseMethod(offset) => {
                    let method =
                        expect!(self, self.methods.get(&offset), "cannot use unknown method");
                    let class = expect!(
                        self,
                        self.classes.get_mut(&self.declaring_class),
                        "method outside class"
                    );
                    let class = expect!(self, Arc::get_mut(class), "class in use");
                    class.methods.insert(offset, method.clone());
                    self.pc += 1;
                }

                Instruction::OverrideMethod(source_offset, target_offset) => {
                    let method = expect!(
                        self,
                        self.methods.get(&target_offset),
                        "cannot use unknown method"
                    );
                    let class = expect!(
                        self,
                        self.classes.get_mut(&self.declaring_class),
                        "method outside class"
                    );
                    let class = expect!(self, Arc::get_mut(class), "class in use");
                    class.methods.insert(source_offset, method.clone());
                    self.pc += 1;
                }

                Instruction::LoadObject(offset) => {
                    let class = expect!(self, self.classes.get(&offset), "unknown class");
                    let object = Object::new(class);
                    self.push(object);
                    self.pc += 1;
                }

                // TODO: Optimize this so Instruction doesn't have to be cloned
                ref i @ Instruction::CallMethod(_, _, _, _) => {
                    if let Instruction::CallMethod(ref offset, ref uri, line, character) = i.clone() {
                        let top = expect!(self, self.stack.top(), "empty stack").clone();
                        let receiver = match self.eval_lazy::<M>(top) {
                            None => continue,
                            Some(r) => r,
                        };
                        
                        let class = expect!(
                            self,
                            &receiver.class,
                            "cannot call method on object without class"
                        );
                        let method = expect!(
                            self,
                            class.methods.get(offset),
                            "message #{:X} not understood by {}",
                            offset,
                            receiver
                        )
                        .clone();
                        let return_address = self.pc + 1;
                        self.pc = method.offset;
                        self.call_stack.push(
                            receiver,
                            method,
                            return_address,
                            SourceCodeLocation(uri.clone(), line, character),
                        );
                    }
                }

                Instruction::CallNative(ref method) => {
                    let method = method.clone();
                    unwrap!(self, M::call(self, method));
                    self.pc += 1;
                }

                Instruction::LoadLocal(index) => {
                    let local = expect!(
                        self,
                        self.stack.at(index as usize),
                        "not enough locals on the stack"
                    )
                    .clone();
                    self.push(local);
                    self.pc += 1;
                }

                Instruction::DropLocal(index) => {
                    self.stack.drop(index as usize);
                    self.pc += 1;
                }

                Instruction::StoreGlobal(offset) => {
                    self.globals.insert(
                        offset,
                        expect!(self, self.stack.pop(), "nothing on stack to store"),
                    );
                    self.pc += 1;
                }

                Instruction::LoadGlobal(offset) => {
                    self.push(expect!(self, self.globals.get(&offset), "global not found").clone());
                    self.pc += 1;
                }

                Instruction::LoadLazy(arity, offset) => {
                    let mut dependencies = vec![];
                    for _ in 0..arity {
                        dependencies.push(unwrap!(self, self.pop()));
                    }
                    self.push(Object::lazy(offset, self.call_stack.clone(), dependencies));
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

                Instruction::ReturnLazy(arity) => {
                    let result = unwrap!(self, self.pop());

                    for _ in 0..arity {
                        unwrap!(self, self.pop());
                    }

                    self.push(result);
                    break;
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
        }
        VMResult::Ok(())
    }

    #[inline]
    fn eval_lazy<M: Runtime>(&mut self, object: Arc<Object>) -> Option<Arc<Object>> {
        match object.const_value {
            ConstValue::Lazy(offset, ref call_stack, ref dependencies) => {
                for dep in dependencies.iter().cloned() {
                    self.push(dep);
                }
                let return_offset = self.pc;
                let call_stack = std::mem::replace(&mut self.call_stack, call_stack.clone());

                self.pc = offset as usize;
                self.do_eval::<M>().report::<M>()?;
                let result = self.pop().report::<M>()?;

                self.pc = return_offset;
                self.call_stack = call_stack;

                self.eval_lazy::<M>(result)
            }
            _ => Some(object),
        }
    }

    fn eval_catch<M: Runtime>(&mut self, instructions: Vec<Instruction>) -> bool {
        self.pc = self.program.len();
        self.program.extend(instructions);
        self.do_eval::<M>().report::<M>().is_none()
    }

    pub fn eval<M: Runtime>(&mut self, instructions: Vec<Instruction>) {
        self.eval_catch::<M>(instructions);
    }

    pub fn eval_pop<M: Runtime>(&mut self, instructions: Vec<Instruction>) -> Option<Arc<Object>> {
        if self.eval_catch::<M>(instructions) {
            None
        } else {
            let result = self.stack.pop();
            if self.stack.size() > 0 {
                self.log_stack()
            }
            self.eval_lazy::<M>(result?)
        }
    }

    pub fn pop(&mut self) -> VMResult<Arc<Object>> {
        VMResult::Ok(expect!(self, self.stack.pop(), "tried to pop empty stack"))
    }

    pub fn pop_eval<M: Runtime>(&mut self) -> VMResult<Arc<Object>> {
        let o = unwrap!(self, self.pop());
        let o = expect!(self, self.eval_lazy::<M>(o), "failed to eval lazy");
        VMResult::Ok(o)
    }

    #[inline]
    pub fn top(&mut self) -> VMResult<&Arc<Object>> {
        VMResult::Ok(expect!(
            self,
            self.stack.top(),
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
        assert_eq!(vm.stack().size(), 0, "stack should be empty");
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
            @SomeClass$methods
                DeclareMethod "someMethod" @SomeClass#someMethod

            @SomeClass
                DeclareClass "SomeClass"
                UseMethod @SomeClass#someMethod

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
            @B$methods
                DeclareMethod "a" @A#a

            @A$methods
                DeclareMethod "a" @A#a

            @B
                DeclareClass "B"
                UseMethod @A#a

            @A
                DeclareClass "A"
                UseMethod @A#a

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

    #[test]
    fn native_method() {
        assert_evaluates_to(
            r#"
            @UInt8
                DeclareClass "UInt8"
                MarkClassU8 @UInt8

            LoadConstU8 12
            LoadConstU8 13
            CallNative Number_plus

            Halt
            "#,
            "25",
        );
    }

    #[test]
    fn binary_call() {
        assert_evaluates_to(
            r#"
            @A$methods
                DeclareMethod "+" @A#+

            @A
                DeclareClass "A"
                UseMethod @A#+

            @B
                DeclareClass "B"

            ; Right-hand operand
            LoadObject @B

            ; Left-hand operand (receiver)
            LoadObject @A

            CallMethod @A#+ "call site" 42 42
            Halt

            @A#+
                LoadLocal 1
                Return 2
            "#,
            "a B",
        );
    }

    #[test]
    fn lazy_object_with_no_dependencies() {
        assert_evaluates_to(
            r#"
            @SomeClass
                DeclareClass "SomeClass"

            LoadLazy 0 @lazy
            Halt

            @lazy
                LoadObject @SomeClass
                ReturnLazy 0
            "#,
            "a SomeClass",
        );
    }

    #[test]
    fn lazy_object_with_dependencies() {
        assert_evaluates_to(
            r#"
            @UInt8
                DeclareClass "UInt8"
                MarkClassU8 @UInt8

            LoadConstU8 1
            LoadConstU8 2
            LoadLazy 2 @lazy
            Halt

            @lazy
                LoadLocal 1
                LoadLocal 1
                CallNative Number_plus
                ReturnLazy 2
            "#,
            "3",
        );
    }
}
