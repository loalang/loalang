use std::io::{self, Read, Write};

pub enum Instruction {
    Noop,
    Halt,
    DeclareClass(String),
    DeclareMethod(String, u64),
    LoadObject(u64),
    CallMethod(u64, String, u64, u64),
    LoadLocal(u16),
    Return(u16),
}

const NOOP: u8 = 0xf0;
const HALT: u8 = 0xf1;
const DECLARE_CLASS: u8 = 0xf2;
const DECLARE_METHOD: u8 = 0xf3;
const LOAD_OBJECT: u8 = 0xf4;
const CALL_METHOD: u8 = 0xf5;
const LOAD_LOCAL: u8 = 0xf6;
const RETURN: u8 = 0xf7;

impl BytecodeEncoding for Instruction {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        match self {
            Instruction::Noop => NOOP.serialize(w),
            Instruction::Halt => HALT.serialize(w),
            Instruction::DeclareClass(ref name) => {
                Ok(DECLARE_CLASS.serialize(&mut w)? + name.serialize(w)?)
            }
            Instruction::DeclareMethod(ref name, id) => Ok(DECLARE_METHOD.serialize(&mut w)?
                + name.serialize(&mut w)?
                + id.serialize(w)?),
            Instruction::LoadObject(id) => Ok(LOAD_OBJECT.serialize(&mut w)? + id.serialize(w)?),
            Instruction::CallMethod(id, ref uri, line, character) => Ok(CALL_METHOD
                .serialize(&mut w)?
                + id.serialize(&mut w)?
                + uri.serialize(&mut w)?
                + line.serialize(&mut w)?
                + character.serialize(w)?),
            Instruction::LoadLocal(index) => {
                Ok(LOAD_LOCAL.serialize(&mut w)? + index.serialize(w)?)
            }
            Instruction::Return(index) => Ok(RETURN.serialize(&mut w)? + index.serialize(w)?),
        }
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Instruction> {
        let mut opcode = [0u8];
        if let 0 = r.read(&mut opcode)? {
            return Err(io::ErrorKind::Interrupted.into());
        }
        match opcode {
            [NOOP] => Ok(Instruction::Noop),
            [HALT] => Ok(Instruction::Halt),
            [DECLARE_CLASS] => Ok(Instruction::DeclareClass(BytecodeEncoding::deserialize(r)?)),
            [DECLARE_METHOD] => Ok(Instruction::DeclareMethod(
                BytecodeEncoding::deserialize(&mut r)?,
                BytecodeEncoding::deserialize(r)?,
            )),
            [LOAD_OBJECT] => Ok(Instruction::LoadObject(BytecodeEncoding::deserialize(r)?)),
            [CALL_METHOD] => Ok(Instruction::CallMethod(
                BytecodeEncoding::deserialize(&mut r)?,
                BytecodeEncoding::deserialize(&mut r)?,
                BytecodeEncoding::deserialize(&mut r)?,
                BytecodeEncoding::deserialize(r)?,
            )),
            [LOAD_LOCAL] => Ok(Instruction::LoadLocal(BytecodeEncoding::deserialize(r)?)),
            [RETURN] => Ok(Instruction::Return(BytecodeEncoding::deserialize(r)?)),
            _ => Err(io::ErrorKind::InvalidInput.into()),
        }
    }
}

impl BytecodeEncoding for String {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        let length = self.len() as u16;
        Ok(length.serialize(&mut w)? + w.write(self.as_bytes())?)
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<String> {
        let length: u16 = BytecodeEncoding::deserialize(&mut r)?;
        let length = length as usize;
        let mut result = String::with_capacity(length);
        if r.read_to_string(&mut result)? != length {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(result)
    }
}

impl BytecodeEncoding for u8 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&[*self])
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<u8> {
        let mut bytes = [0u8];
        r.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }
}

impl BytecodeEncoding for u16 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<u16> {
        let mut bytes = [0u8, 0u8];
        r.read_exact(&mut bytes)?;
        Ok(u16::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for u64 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<u64> {
        let mut bytes = [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8];
        r.read_exact(&mut bytes)?;
        Ok(u64::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for Vec<Instruction> {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        let mut written = 0;
        for instruction in self.iter() {
            written += instruction.serialize(&mut w)?;
        }
        Ok(written)
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Vec<Instruction>> {
        let mut instructions = vec![];
        loop {
            match Instruction::deserialize(&mut r) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted => break,
                Ok(instruction) => instructions.push(instruction),
                Err(e) => return Err(e),
            }
        }
        Ok(instructions)
    }
}

pub trait BytecodeEncoding
where
    Self: Sized,
{
    fn serialize<W: Write>(&self, w: W) -> io::Result<usize>;
    fn deserialize<R: Read>(r: R) -> io::Result<Self>;
}
