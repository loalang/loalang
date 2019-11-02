use crate::*;

#[derive(Debug, Clone)]
pub enum Instruction {
    ReferenceToClass(Id),
    LoadLocal(u16),
    DeclareClass(Id, String),
    SendMessage(Id),
    LoadArgument(u8),
    BeginMethod(String),
    EndMethod(Id),
    Return(u8),
}

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
