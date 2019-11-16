use crate::vm::NativeMethod;
use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // A class is declared simply by registering
    // an id and a name, which will create a
    // global class object in the VM.
    DeclareClass(Id, String),

    // These instructions mark builtin classes
    // in the VM, so that constants that are
    // loaded are boxed as the correct classes.
    MarkClassString(Id),
    MarkClassCharacter(Id),
    MarkClassSymbol(Id),

    MarkClassU8(Id),
    MarkClassU16(Id),
    MarkClassU32(Id),
    MarkClassU64(Id),
    MarkClassU128(Id),
    MarkClassUBig(Id),
    MarkClassI8(Id),
    MarkClassI16(Id),
    MarkClassI32(Id),
    MarkClassI64(Id),
    MarkClassI128(Id),
    MarkClassIBig(Id),
    MarkClassF32(Id),
    MarkClassF64(Id),
    MarkClassFBig(Id),

    // After all known classes have been declared,
    // methods can be declared on them, starting with
    // the beginning marker and ending with attaching
    // the method to a class. All instructions
    // between these two markers should be moved into
    // the method object, for later execution.
    BeginMethod(u64, String),
    EndMethod(Id),

    // After all concrete methods have been declared,
    // existing methods can be inherited into classes.
    InheritMethod(Id, Id, u64),

    // Below this point, the instructions defined are
    // will be moved into a method that is being defined,
    // meaning we're in between markers.

    // Mostly these instructions result in objects
    // being pushed onto the stack, but this instruction
    // instead sends the VM to executing a given method
    // of the class of the object at the TOS. It expects
    // the arguments to the method to be pushed in
    // reversed order on the stack, ending with the receiver.
    // Afterwards, this instruction will have replaced
    // the arguments and receiver with the result of
    // the method.
    SendMessage(String, u64),

    // The return instruction removes the stack frame,
    // leaving only the method result on the TOS.
    Return(u16),

    // This instruction pops the TOS and stores it as
    // a global in the VM.
    StoreGlobal(Id),

    // Below this point are all the instructions that push
    // things onto the stack, in preparation of message sends.
    ReferenceToClass(Id),
    LoadLocal(u16),
    LoadArgument(u8),

    LoadGlobal(Id),

    LoadConstString(String),
    LoadConstCharacter(u16),
    LoadConstSymbol(String),

    LoadConstU8(u8),
    LoadConstU16(u16),
    LoadConstU32(u32),
    LoadConstU64(u64),
    LoadConstU128(u128),
    LoadConstUBig(BigUint),
    LoadConstI8(i8),
    LoadConstI16(i16),
    LoadConstI32(i32),
    LoadConstI64(i64),
    LoadConstI128(i128),
    LoadConstIBig(BigInt),
    LoadConstF32(f32),
    LoadConstF64(f64),
    LoadConstFBig(BigFraction),

    CallNative(NativeMethod),
}

#[derive(Serialize, Deserialize)]
pub struct Instructions(Vec<Instruction>);

impl Instructions {
    pub fn new() -> Instructions {
        Instructions(vec![])
    }

    pub fn extend(&mut self, instructions: Instructions) {
        self.0.extend(instructions.0)
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.0.push(instruction)
    }

    pub fn iter(&self) -> std::slice::Iter<Instruction> {
        self.0.iter()
    }

    pub fn reverse(&mut self) {
        self.0.reverse();
    }

    pub fn to_bytes(&self) -> bincode::Result<Vec<u8>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> bincode::Result<Instructions> {
        bincode::deserialize(bytes)
    }
}

impl Into<Vec<Instruction>> for Instructions {
    fn into(self) -> Vec<Instruction> {
        self.0
    }
}

impl AsRef<Vec<Instruction>> for Instructions {
    fn as_ref(&self) -> &Vec<Instruction> {
        &self.0
    }
}

impl From<Instruction> for Instructions {
    fn from(i: Instruction) -> Self {
        Instructions(vec![i])
    }
}

impl From<Vec<Instruction>> for Instructions {
    fn from(i: Vec<Instruction>) -> Self {
        Instructions(i)
    }
}

impl fmt::Debug for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0.len() == 0 {
            write!(f, "; Noop")?;
        }
        for (i, inst) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "\n")?;
            }
            write!(f, "{:?}", inst)?;
        }
        Ok(())
    }
}
