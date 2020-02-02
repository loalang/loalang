use crate::bytecode::Instruction as BytecodeInstruction;
use crate::HashMap;
use std::fmt;

pub struct Cursor {
    pub end: u64,
    pub labels: HashMap<Label, u64>,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            end: 0,
            labels: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct Assembly {
    pub leading_sections: Vec<Section>,
    sections: Vec<Section>,
}

impl Assembly {
    pub fn new() -> Assembly {
        Assembly {
            sections: vec![],
            leading_sections: vec![],
        }
    }

    pub fn add_leading_section(&mut self, section: Section) {
        self.leading_sections.push(section);
    }

    pub fn add_section(&mut self, section: Section) {
        self.sections.push(section);
    }

    pub fn with_section(mut self, section: Section) -> Self {
        self.add_section(section);
        self
    }

    pub fn last_leading_mut(&mut self) -> &mut Section {
        if self.leading_sections.is_empty() {
            self.leading_sections.push(Section::unnamed());
        }
        self.leading_sections.last_mut().unwrap()
    }

    pub fn iter(
        &self,
    ) -> std::iter::Chain<std::slice::Iter<'_, Section>, std::slice::Iter<'_, Section>> {
        self.leading_sections.iter().chain(self.sections.iter())
    }

    pub fn into_iter(
        self,
    ) -> std::iter::Chain<std::vec::IntoIter<Section>, std::vec::IntoIter<Section>> {
        self.leading_sections
            .into_iter()
            .chain(self.sections.into_iter())
    }

    pub fn compile(self, cursor: &mut Cursor) -> Vec<BytecodeInstruction> {
        for section in self.iter() {
            if let Some(ref label) = section.label {
                cursor.labels.insert(label.clone(), cursor.end);
            }
            cursor.end += section.instructions.len() as u64;
        }

        let mut instructions = vec![];
        for section in self.into_iter() {
            for assembly_instruction in section.instructions {
                instructions.push(assembly_instruction.compile(&cursor.labels));
            }
        }
        instructions
    }
}

impl PartialEq for Assembly {
    fn eq(&self, other: &Assembly) -> bool {
        self.iter().collect::<Vec<_>>() == other.iter().collect::<Vec<_>>()
    }
}

impl Into<Vec<BytecodeInstruction>> for Assembly {
    fn into(self) -> Vec<BytecodeInstruction> {
        self.compile(&mut Cursor::new())
    }
}

impl fmt::Debug for Assembly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for section in self.iter() {
            let indent = if let Some(ref label) = section.label {
                write!(f, "@{}\n", label)?;
                true
            } else {
                false
            };

            for instruction in section.instructions.iter() {
                if indent {
                    write!(f, "  ")?;
                }
                write!(f, "{:?}\n", instruction)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Instruction {
    pub leading_comment: Option<String>,
    pub kind: InstructionKind,
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InstructionKind::*;
        match self.kind {
            Noop => write!(f, "Noop"),
            Halt => write!(f, "Halt"),
            Panic => write!(f, "Panic"),
            DeclareClass(ref name) => write!(f, "DeclareClass {:?}", name),
            DeclareMethod(ref selector, ref label) => {
                write!(f, "DeclareMethod {:?} @{}", selector, label)
            }
            LoadObject(ref label) => write!(f, "LoadObject @{}", label),
            CallMethod(ref label, ref uri, line, character) => {
                write!(f, "CallMethod @{} {:?} {} {}", label, uri, line, character)
            }
            LoadLocal(index) => write!(f, "LoadLocal {}", index),
            Return(arity) => write!(f, "Return {}", arity),
            LoadConstString(ref value) => write!(f, "LoadConstString {:?}", value),
        }
    }
}

pub type Label = String;

#[derive(PartialEq, Debug, Clone)]
pub enum InstructionKind {
    Noop,
    Halt,
    Panic,
    DeclareClass(String),
    DeclareMethod(String, Label),
    LoadObject(Label),
    CallMethod(Label, String, u64, u64),
    LoadLocal(u16),
    Return(u16),

    LoadConstString(String),
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
            InstructionKind::Panic => BytecodeInstruction::Panic,
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
            InstructionKind::LoadConstString(ref v) => {
                BytecodeInstruction::LoadConstString(v.clone())
            }
        }
    }
}

impl PartialEq for Instruction {
    fn eq(&self, rhs: &Instruction) -> bool {
        self.kind == rhs.kind
    }
}
