use crate::bytecode::Instruction as BytecodeInstruction;
use crate::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct Assembly {
    pub sections: Vec<Section>,
}

impl Assembly {
    pub fn new() -> Assembly {
        Assembly { sections: vec![] }
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    pub fn with_section(mut self, section: Section) -> Self {
        self.add_section(section);
        self
    }

    fn offsets(&self) -> HashMap<String, u64> {
        let mut offsets = HashMap::new();
        let mut offset: u64 = 0;

        for section in self.sections.iter() {
            if let Some(ref label) = section.label {
                offsets.insert(label.clone(), offset);
            }
            for instruction in section.instructions.iter() {
                offset += 1;
            }
        }

        offsets
    }
}

impl Into<Vec<BytecodeInstruction>> for Assembly {
    fn into(self) -> Vec<BytecodeInstruction> {
        let mut instructions = vec![];
        let offsets = self.offsets();
        for section in self.sections {
            for assembly_instruction in section.instructions {
                instructions.push(assembly_instruction.compile(&offsets));
            }
        }
        instructions
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub leading_comment: Option<String>,
    pub label: Option<String>,
    pub instructions: Vec<Instruction>,
}

impl Section {
    pub fn named<S: Into<String>>(label: S) -> Section {
        Section {
            leading_comment: None,
            label: Some(label.into()),
            instructions: vec![],
        }
    }

    pub fn unnamed() -> Section {
        Section {
            leading_comment: None,
            label: None,
            instructions: vec![],
        }
    }

    pub fn with_comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.leading_comment = Some(comment.into());
        self
    }

    pub fn add_instruction(&mut self, instruction: InstructionKind) {
        self.instructions
            .push(Instruction::uncommented(instruction));
    }

    pub fn with_instruction(mut self, instruction: InstructionKind) -> Self {
        self.add_instruction(instruction);
        self
    }

    pub fn with_commented_instruction<S: Into<String>>(
        mut self,
        comment: S,
        instruction: InstructionKind,
    ) -> Self {
        self.instructions
            .push(Instruction::commented(comment.into(), instruction));
        self
    }
}

impl PartialEq for Section {
    fn eq(&self, rhs: &Section) -> bool {
        self.label == rhs.label && self.instructions == rhs.instructions
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub leading_comment: Option<String>,
    pub kind: InstructionKind,
}

pub type Label = String;

#[derive(PartialEq, Debug, Clone)]
pub enum InstructionKind {
    Noop,
    Halt,
    DeclareClass(String),
    DeclareMethod(String, Label),
    LoadObject(Label),
    CallMethod(Label, String, u64, u64),
    LoadLocal(u16),
    Return(u16),
}

impl Instruction {
    pub fn commented(comment: String, kind: InstructionKind) -> Instruction {
        Instruction {
            leading_comment: Some(comment),
            kind,
        }
    }

    pub fn uncommented(kind: InstructionKind) -> Instruction {
        Instruction {
            leading_comment: None,
            kind,
        }
    }

    pub fn compile(&self, offsets: &HashMap<String, u64>) -> BytecodeInstruction {
        match self.kind {
            InstructionKind::Noop => BytecodeInstruction::Noop,
            InstructionKind::Halt => BytecodeInstruction::Halt,
            InstructionKind::DeclareClass(ref s) => BytecodeInstruction::DeclareClass(s.clone()),
            InstructionKind::DeclareMethod(ref s, ref l) => BytecodeInstruction::DeclareMethod(
                s.clone(),
                *offsets
                    .get(l)
                    .expect(format!("method {} not found", l).as_ref()),
            ),
            InstructionKind::LoadObject(ref l) => BytecodeInstruction::LoadObject(
                *offsets
                    .get(l)
                    .expect(format!("class {} not found", l).as_ref()),
            ),
            InstructionKind::CallMethod(ref l, ref uri, line, character) => {
                BytecodeInstruction::CallMethod(
                    *offsets
                        .get(l)
                        .expect(format!("method {} not found", l).as_ref()),
                    uri.clone(),
                    line,
                    character,
                )
            }
            InstructionKind::LoadLocal(i) => BytecodeInstruction::LoadLocal(i),
            InstructionKind::Return(a) => BytecodeInstruction::Return(a),
        }
    }
}

impl PartialEq for Instruction {
    fn eq(&self, rhs: &Instruction) -> bool {
        self.kind == rhs.kind
    }
}
