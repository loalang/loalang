use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    ReferenceToClass(Id),
    LoadLocal(u16),
    DeclareClass(Id, String),
    SendMessage(Id),
    LoadArgument(u8),
    BeginMethod(String),
    EndMethod(Id),
    Return(u8),

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
