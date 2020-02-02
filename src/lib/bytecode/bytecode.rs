use std::io::{self, Read, Write};

pub enum Instruction {
    Noop,
    Halt,
    Panic,
    DeclareClass(String),
    DeclareMethod(String, u64),
    LoadObject(u64),
    CallMethod(u64, String, u64, u64),
    LoadLocal(u16),
    Return(u16),
    LoadConstString(String),
}

const NOOP: u8 = 0xa0;
const HALT: u8 = 0xa1;
const PANIC: u8 = 0xa2;
const DECLARE_CLASS: u8 = 0xa3;
const DECLARE_METHOD: u8 = 0xa4;
const LOAD_OBJECT: u8 = 0xa5;
const CALL_METHOD: u8 = 0xa6;
const LOAD_LOCAL: u8 = 0xa7;
const RETURN: u8 = 0xa8;

const LOAD_CONST_STRING: u8 = 0xa9;

impl BytecodeEncoding for Instruction {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        match self {
            Instruction::Noop => NOOP.serialize(w),
            Instruction::Halt => HALT.serialize(w),
            Instruction::Panic => PANIC.serialize(w),
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
            Instruction::LoadConstString(value) => {
                Ok(LOAD_CONST_STRING.serialize(&mut w)? + value.serialize(w)?)
            }
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
            [PANIC] => Ok(Instruction::Panic),
            [DECLARE_CLASS] => Ok(Instruction::DeclareClass(r.deserialize()?)),
            [DECLARE_METHOD] => Ok(Instruction::DeclareMethod(
                r.deserialize()?,
                r.deserialize()?,
            )),
            [LOAD_OBJECT] => Ok(Instruction::LoadObject(r.deserialize()?)),
            [CALL_METHOD] => Ok(Instruction::CallMethod(
                r.deserialize()?,
                r.deserialize()?,
                r.deserialize()?,
                r.deserialize()?,
            )),
            [LOAD_LOCAL] => Ok(Instruction::LoadLocal(r.deserialize()?)),
            [RETURN] => Ok(Instruction::Return(r.deserialize()?)),
            [LOAD_CONST_STRING] => Ok(Instruction::LoadConstString(r.deserialize()?)),
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
        let length: u16 = r.deserialize()?;
        let length = length as usize;
        let mut bytes = vec![0u8; length];
        r.read_exact(&mut bytes)?;
        Ok(unsafe { String::from_utf8_unchecked(bytes) })
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

    fn rotate(&self) -> io::Result<Self> {
        let mut buffer = vec![];
        self.serialize(&mut buffer)?;
        Ok(Self::deserialize(buffer.as_slice())?)
    }
}

pub trait BytecodeEncodingRead<T> {
    fn deserialize(&mut self) -> io::Result<T>;
}

impl<R, T> BytecodeEncodingRead<T> for R where R: Read, T: BytecodeEncoding {
    fn deserialize(&mut self) -> io::Result<T> {
        T::deserialize(self)
    }
}
