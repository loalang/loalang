use crate::*;

#[derive(Debug)]
pub enum Instruction {}

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
