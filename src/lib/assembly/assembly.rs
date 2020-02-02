use crate::bytecode::Instruction as BytecodeInstruction;
use crate::HashMap;
use crate::*;
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

            MarkClassString(ref label) => write!(f, "MarkClassString @{}", label),
            MarkClassCharacter(ref label) => write!(f, "MarkClassCharacter @{}", label),
            MarkClassSymbol(ref label) => write!(f, "MarkClassSymbol @{}", label),
            MarkClassU8(ref label) => write!(f, "MarkClassU8 @{}", label),
            MarkClassU16(ref label) => write!(f, "MarkClassU16 @{}", label),
            MarkClassU32(ref label) => write!(f, "MarkClassU32 @{}", label),
            MarkClassU64(ref label) => write!(f, "MarkClassU64 @{}", label),
            MarkClassU128(ref label) => write!(f, "MarkClassU128 @{}", label),
            MarkClassUBig(ref label) => write!(f, "MarkClassUBig @{}", label),
            MarkClassI8(ref label) => write!(f, "MarkClassI8 @{}", label),
            MarkClassI16(ref label) => write!(f, "MarkClassI16 @{}", label),
            MarkClassI32(ref label) => write!(f, "MarkClassI32 @{}", label),
            MarkClassI64(ref label) => write!(f, "MarkClassI64 @{}", label),
            MarkClassI128(ref label) => write!(f, "MarkClassI128 @{}", label),
            MarkClassIBig(ref label) => write!(f, "MarkClassIBig @{}", label),
            MarkClassF32(ref label) => write!(f, "MarkClassF32 @{}", label),
            MarkClassF64(ref label) => write!(f, "MarkClassF64 @{}", label),
            MarkClassFBig(ref label) => write!(f, "MarkClassFBig @{}", label),

            LoadConstString(ref value) => write!(f, "LoadConstString {:?}", value),
            LoadConstCharacter(ref value) => write!(f, "LoadConstCharacter {:?}", value),
            LoadConstSymbol(ref value) => write!(f, "LoadConstSymbol {:?}", value),
            LoadConstU8(ref value) => write!(f, "LoadConstU8 {:?}", value),
            LoadConstU16(ref value) => write!(f, "LoadConstU16 {:?}", value),
            LoadConstU32(ref value) => write!(f, "LoadConstU32 {:?}", value),
            LoadConstU64(ref value) => write!(f, "LoadConstU64 {:?}", value),
            LoadConstU128(ref value) => write!(f, "LoadConstU128 {:?}", value),
            LoadConstUBig(ref value) => write!(f, "LoadConstUBig {:?}", value),
            LoadConstI8(ref value) => write!(f, "LoadConstI8 {:?}", value),
            LoadConstI16(ref value) => write!(f, "LoadConstI16 {:?}", value),
            LoadConstI32(ref value) => write!(f, "LoadConstI32 {:?}", value),
            LoadConstI64(ref value) => write!(f, "LoadConstI64 {:?}", value),
            LoadConstI128(ref value) => write!(f, "LoadConstI128 {:?}", value),
            LoadConstIBig(ref value) => write!(f, "LoadConstIBig {:?}", value),
            LoadConstF32(ref value) => write!(f, "LoadConstF32 {:?}", value),
            LoadConstF64(ref value) => write!(f, "LoadConstF64 {:?}", value),
            LoadConstFBig(ref value) => write!(f, "LoadConstFBig {:?}", value),
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

    MarkClassString(Label),
    MarkClassCharacter(Label),
    MarkClassSymbol(Label),
    MarkClassU8(Label),
    MarkClassU16(Label),
    MarkClassU32(Label),
    MarkClassU64(Label),
    MarkClassU128(Label),
    MarkClassUBig(Label),
    MarkClassI8(Label),
    MarkClassI16(Label),
    MarkClassI32(Label),
    MarkClassI64(Label),
    MarkClassI128(Label),
    MarkClassIBig(Label),
    MarkClassF32(Label),
    MarkClassF64(Label),
    MarkClassFBig(Label),

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
        macro_rules! label {
            ($label:expr, $expected:expr) => {
                *offsets
                    .get($label)
                    .expect(format!("{} {} not found", $expected, $label).as_ref())
            };
        }
        match self.kind {
            InstructionKind::Noop => BytecodeInstruction::Noop,
            InstructionKind::Halt => BytecodeInstruction::Halt,
            InstructionKind::Panic => BytecodeInstruction::Panic,
            InstructionKind::DeclareClass(ref s) => BytecodeInstruction::DeclareClass(s.clone()),
            InstructionKind::DeclareMethod(ref s, ref l) => {
                BytecodeInstruction::DeclareMethod(s.clone(), label!(l, "method"))
            }
            InstructionKind::LoadObject(ref l) => {
                BytecodeInstruction::LoadObject(label!(l, "class"))
            }
            InstructionKind::CallMethod(ref l, ref uri, line, character) => {
                BytecodeInstruction::CallMethod(label!(l, "method"), uri.clone(), line, character)
            }
            InstructionKind::LoadLocal(i) => BytecodeInstruction::LoadLocal(i),
            InstructionKind::Return(a) => BytecodeInstruction::Return(a),

            InstructionKind::MarkClassString(ref l) => {
                BytecodeInstruction::MarkClassString(label!(l, "class"))
            }
            InstructionKind::MarkClassCharacter(ref l) => {
                BytecodeInstruction::MarkClassCharacter(label!(l, "class"))
            }
            InstructionKind::MarkClassSymbol(ref l) => {
                BytecodeInstruction::MarkClassSymbol(label!(l, "class"))
            }
            InstructionKind::MarkClassU8(ref l) => {
                BytecodeInstruction::MarkClassU8(label!(l, "class"))
            }
            InstructionKind::MarkClassU16(ref l) => {
                BytecodeInstruction::MarkClassU16(label!(l, "class"))
            }
            InstructionKind::MarkClassU32(ref l) => {
                BytecodeInstruction::MarkClassU32(label!(l, "class"))
            }
            InstructionKind::MarkClassU64(ref l) => {
                BytecodeInstruction::MarkClassU64(label!(l, "class"))
            }
            InstructionKind::MarkClassU128(ref l) => {
                BytecodeInstruction::MarkClassU128(label!(l, "class"))
            }
            InstructionKind::MarkClassUBig(ref l) => {
                BytecodeInstruction::MarkClassUBig(label!(l, "class"))
            }
            InstructionKind::MarkClassI8(ref l) => {
                BytecodeInstruction::MarkClassI8(label!(l, "class"))
            }
            InstructionKind::MarkClassI16(ref l) => {
                BytecodeInstruction::MarkClassI16(label!(l, "class"))
            }
            InstructionKind::MarkClassI32(ref l) => {
                BytecodeInstruction::MarkClassI32(label!(l, "class"))
            }
            InstructionKind::MarkClassI64(ref l) => {
                BytecodeInstruction::MarkClassI64(label!(l, "class"))
            }
            InstructionKind::MarkClassI128(ref l) => {
                BytecodeInstruction::MarkClassI128(label!(l, "class"))
            }
            InstructionKind::MarkClassIBig(ref l) => {
                BytecodeInstruction::MarkClassIBig(label!(l, "class"))
            }
            InstructionKind::MarkClassF32(ref l) => {
                BytecodeInstruction::MarkClassF32(label!(l, "class"))
            }
            InstructionKind::MarkClassF64(ref l) => {
                BytecodeInstruction::MarkClassF64(label!(l, "class"))
            }
            InstructionKind::MarkClassFBig(ref l) => {
                BytecodeInstruction::MarkClassFBig(label!(l, "class"))
            }

            InstructionKind::LoadConstString(ref v) => {
                BytecodeInstruction::LoadConstString(v.clone())
            }
            InstructionKind::LoadConstCharacter(ref v) => {
                BytecodeInstruction::LoadConstCharacter(v.clone())
            }
            InstructionKind::LoadConstSymbol(ref v) => {
                BytecodeInstruction::LoadConstSymbol(v.clone())
            }
            InstructionKind::LoadConstU8(ref v) => BytecodeInstruction::LoadConstU8(v.clone()),
            InstructionKind::LoadConstU16(ref v) => BytecodeInstruction::LoadConstU16(v.clone()),
            InstructionKind::LoadConstU32(ref v) => BytecodeInstruction::LoadConstU32(v.clone()),
            InstructionKind::LoadConstU64(ref v) => BytecodeInstruction::LoadConstU64(v.clone()),
            InstructionKind::LoadConstU128(ref v) => BytecodeInstruction::LoadConstU128(v.clone()),
            InstructionKind::LoadConstUBig(ref v) => BytecodeInstruction::LoadConstUBig(v.clone()),
            InstructionKind::LoadConstI8(ref v) => BytecodeInstruction::LoadConstI8(v.clone()),
            InstructionKind::LoadConstI16(ref v) => BytecodeInstruction::LoadConstI16(v.clone()),
            InstructionKind::LoadConstI32(ref v) => BytecodeInstruction::LoadConstI32(v.clone()),
            InstructionKind::LoadConstI64(ref v) => BytecodeInstruction::LoadConstI64(v.clone()),
            InstructionKind::LoadConstI128(ref v) => BytecodeInstruction::LoadConstI128(v.clone()),
            InstructionKind::LoadConstIBig(ref v) => BytecodeInstruction::LoadConstIBig(v.clone()),
            InstructionKind::LoadConstF32(ref v) => BytecodeInstruction::LoadConstF32(v.clone()),
            InstructionKind::LoadConstF64(ref v) => BytecodeInstruction::LoadConstF64(v.clone()),
            InstructionKind::LoadConstFBig(ref v) => BytecodeInstruction::LoadConstFBig(v.clone()),
        }
    }
}

impl PartialEq for Instruction {
    fn eq(&self, rhs: &Instruction) -> bool {
        self.kind == rhs.kind
    }
}
