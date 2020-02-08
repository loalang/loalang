use crate::*;
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
    DropLocal(u16),
    StoreGlobal(u64),
    LoadGlobal(u64),
    Return(u16),

    MarkClassString(u64),
    MarkClassCharacter(u64),
    MarkClassSymbol(u64),
    MarkClassU8(u64),
    MarkClassU16(u64),
    MarkClassU32(u64),
    MarkClassU64(u64),
    MarkClassU128(u64),
    MarkClassUBig(u64),
    MarkClassI8(u64),
    MarkClassI16(u64),
    MarkClassI32(u64),
    MarkClassI64(u64),
    MarkClassI128(u64),
    MarkClassIBig(u64),
    MarkClassF32(u64),
    MarkClassF64(u64),
    MarkClassFBig(u64),

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

const NOOP: u8 = 0xa0;
const HALT: u8 = 0xa1;
const PANIC: u8 = 0xa2;
const DECLARE_CLASS: u8 = 0xa3;
const DECLARE_METHOD: u8 = 0xa4;
const LOAD_OBJECT: u8 = 0xa5;
const CALL_METHOD: u8 = 0xa6;
const LOAD_LOCAL: u8 = 0xa7;
const DROP_LOCAL: u8 = 0xa8;
const STORE_GLOBAL: u8 = 0xa9;
const LOAD_GLOBAL: u8 = 0xaa;
const RETURN: u8 = 0xab;

const MARK_CLASS_STRING: u8 = 0xb0;
const MARK_CLASS_CHARACTER: u8 = 0xb1;
const MARK_CLASS_SYMBOL: u8 = 0xb2;
const MARK_CLASS_U8: u8 = 0xb3;
const MARK_CLASS_U16: u8 = 0xb4;
const MARK_CLASS_U32: u8 = 0xb5;
const MARK_CLASS_U64: u8 = 0xb6;
const MARK_CLASS_U128: u8 = 0xb7;
const MARK_CLASS_UBIG: u8 = 0xb8;
const MARK_CLASS_I8: u8 = 0xb9;
const MARK_CLASS_I16: u8 = 0xba;
const MARK_CLASS_I32: u8 = 0xbb;
const MARK_CLASS_I64: u8 = 0xbc;
const MARK_CLASS_I128: u8 = 0xbd;
const MARK_CLASS_IBIG: u8 = 0xbe;
const MARK_CLASS_F32: u8 = 0xbf;
const MARK_CLASS_F64: u8 = 0xc0;
const MARK_CLASS_FBIG: u8 = 0xc1;

const LOAD_CONST_STRING: u8 = 0xc2;
const LOAD_CONST_CHARACTER: u8 = 0xc3;
const LOAD_CONST_SYMBOL: u8 = 0xc4;
const LOAD_CONST_U8: u8 = 0xc5;
const LOAD_CONST_U16: u8 = 0xc6;
const LOAD_CONST_U32: u8 = 0xc7;
const LOAD_CONST_U64: u8 = 0xc8;
const LOAD_CONST_U128: u8 = 0xc9;
const LOAD_CONST_UBIG: u8 = 0xca;
const LOAD_CONST_I8: u8 = 0xcb;
const LOAD_CONST_I16: u8 = 0xcc;
const LOAD_CONST_I32: u8 = 0xcd;
const LOAD_CONST_I64: u8 = 0xce;
const LOAD_CONST_I128: u8 = 0xcf;
const LOAD_CONST_IBIG: u8 = 0xd0;
const LOAD_CONST_F32: u8 = 0xd1;
const LOAD_CONST_F64: u8 = 0xd2;
const LOAD_CONST_FBIG: u8 = 0xd3;

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
            Instruction::DropLocal(index) => {
                Ok(DROP_LOCAL.serialize(&mut w)? + index.serialize(w)?)
            }
            Instruction::StoreGlobal(label) => {
                Ok(STORE_GLOBAL.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::LoadGlobal(label) => {
                Ok(LOAD_GLOBAL.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::Return(index) => Ok(RETURN.serialize(&mut w)? + index.serialize(w)?),

            Instruction::MarkClassString(label) => {
                Ok(MARK_CLASS_STRING.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassCharacter(label) => {
                Ok(MARK_CLASS_CHARACTER.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassSymbol(label) => {
                Ok(MARK_CLASS_SYMBOL.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassU8(label) => {
                Ok(MARK_CLASS_U8.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassU16(label) => {
                Ok(MARK_CLASS_U16.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassU32(label) => {
                Ok(MARK_CLASS_U32.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassU64(label) => {
                Ok(MARK_CLASS_U64.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassU128(label) => {
                Ok(MARK_CLASS_U128.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassUBig(label) => {
                Ok(MARK_CLASS_UBIG.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassI8(label) => {
                Ok(MARK_CLASS_I8.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassI16(label) => {
                Ok(MARK_CLASS_I16.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassI32(label) => {
                Ok(MARK_CLASS_I32.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassI64(label) => {
                Ok(MARK_CLASS_I64.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassI128(label) => {
                Ok(MARK_CLASS_I128.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassIBig(label) => {
                Ok(MARK_CLASS_IBIG.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassF32(label) => {
                Ok(MARK_CLASS_F32.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassF64(label) => {
                Ok(MARK_CLASS_F64.serialize(&mut w)? + label.serialize(w)?)
            }
            Instruction::MarkClassFBig(label) => {
                Ok(MARK_CLASS_FBIG.serialize(&mut w)? + label.serialize(w)?)
            }

            Instruction::LoadConstString(value) => {
                Ok(LOAD_CONST_STRING.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstCharacter(value) => {
                Ok(LOAD_CONST_CHARACTER.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstSymbol(value) => {
                Ok(LOAD_CONST_SYMBOL.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstU8(value) => {
                Ok(LOAD_CONST_U8.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstU16(value) => {
                Ok(LOAD_CONST_U16.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstU32(value) => {
                Ok(LOAD_CONST_U32.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstU64(value) => {
                Ok(LOAD_CONST_U64.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstU128(value) => {
                Ok(LOAD_CONST_U128.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstUBig(value) => {
                Ok(LOAD_CONST_UBIG.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstI8(value) => {
                Ok(LOAD_CONST_I8.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstI16(value) => {
                Ok(LOAD_CONST_I16.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstI32(value) => {
                Ok(LOAD_CONST_I32.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstI64(value) => {
                Ok(LOAD_CONST_I64.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstI128(value) => {
                Ok(LOAD_CONST_I128.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstIBig(value) => {
                Ok(LOAD_CONST_IBIG.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstF32(value) => {
                Ok(LOAD_CONST_F32.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstF64(value) => {
                Ok(LOAD_CONST_F64.serialize(&mut w)? + value.serialize(w)?)
            }
            Instruction::LoadConstFBig(value) => {
                Ok(LOAD_CONST_FBIG.serialize(&mut w)? + value.serialize(w)?)
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
            [DROP_LOCAL] => Ok(Instruction::DropLocal(r.deserialize()?)),
            [STORE_GLOBAL] => Ok(Instruction::StoreGlobal(r.deserialize()?)),
            [LOAD_GLOBAL] => Ok(Instruction::LoadGlobal(r.deserialize()?)),
            [RETURN] => Ok(Instruction::Return(r.deserialize()?)),

            [MARK_CLASS_STRING] => Ok(Instruction::MarkClassString(r.deserialize()?)),
            [MARK_CLASS_CHARACTER] => Ok(Instruction::MarkClassCharacter(r.deserialize()?)),
            [MARK_CLASS_SYMBOL] => Ok(Instruction::MarkClassSymbol(r.deserialize()?)),
            [MARK_CLASS_U8] => Ok(Instruction::MarkClassU8(r.deserialize()?)),
            [MARK_CLASS_U16] => Ok(Instruction::MarkClassU16(r.deserialize()?)),
            [MARK_CLASS_U32] => Ok(Instruction::MarkClassU32(r.deserialize()?)),
            [MARK_CLASS_U64] => Ok(Instruction::MarkClassU64(r.deserialize()?)),
            [MARK_CLASS_U128] => Ok(Instruction::MarkClassU128(r.deserialize()?)),
            [MARK_CLASS_UBIG] => Ok(Instruction::MarkClassUBig(r.deserialize()?)),
            [MARK_CLASS_I8] => Ok(Instruction::MarkClassI8(r.deserialize()?)),
            [MARK_CLASS_I16] => Ok(Instruction::MarkClassI16(r.deserialize()?)),
            [MARK_CLASS_I32] => Ok(Instruction::MarkClassI32(r.deserialize()?)),
            [MARK_CLASS_I64] => Ok(Instruction::MarkClassI64(r.deserialize()?)),
            [MARK_CLASS_I128] => Ok(Instruction::MarkClassI128(r.deserialize()?)),
            [MARK_CLASS_IBIG] => Ok(Instruction::MarkClassIBig(r.deserialize()?)),
            [MARK_CLASS_F32] => Ok(Instruction::MarkClassF32(r.deserialize()?)),
            [MARK_CLASS_F64] => Ok(Instruction::MarkClassF64(r.deserialize()?)),
            [MARK_CLASS_FBIG] => Ok(Instruction::MarkClassFBig(r.deserialize()?)),

            [LOAD_CONST_STRING] => Ok(Instruction::LoadConstString(r.deserialize()?)),
            [LOAD_CONST_CHARACTER] => Ok(Instruction::LoadConstCharacter(r.deserialize()?)),
            [LOAD_CONST_SYMBOL] => Ok(Instruction::LoadConstSymbol(r.deserialize()?)),
            [LOAD_CONST_U8] => Ok(Instruction::LoadConstU8(r.deserialize()?)),
            [LOAD_CONST_U16] => Ok(Instruction::LoadConstU16(r.deserialize()?)),
            [LOAD_CONST_U32] => Ok(Instruction::LoadConstU32(r.deserialize()?)),
            [LOAD_CONST_U64] => Ok(Instruction::LoadConstU64(r.deserialize()?)),
            [LOAD_CONST_U128] => Ok(Instruction::LoadConstU128(r.deserialize()?)),
            [LOAD_CONST_UBIG] => Ok(Instruction::LoadConstUBig(r.deserialize()?)),
            [LOAD_CONST_I8] => Ok(Instruction::LoadConstI8(r.deserialize()?)),
            [LOAD_CONST_I16] => Ok(Instruction::LoadConstI16(r.deserialize()?)),
            [LOAD_CONST_I32] => Ok(Instruction::LoadConstI32(r.deserialize()?)),
            [LOAD_CONST_I64] => Ok(Instruction::LoadConstI64(r.deserialize()?)),
            [LOAD_CONST_I128] => Ok(Instruction::LoadConstI128(r.deserialize()?)),
            [LOAD_CONST_IBIG] => Ok(Instruction::LoadConstIBig(r.deserialize()?)),
            [LOAD_CONST_F32] => Ok(Instruction::LoadConstF32(r.deserialize()?)),
            [LOAD_CONST_F64] => Ok(Instruction::LoadConstF64(r.deserialize()?)),
            [LOAD_CONST_FBIG] => Ok(Instruction::LoadConstFBig(r.deserialize()?)),

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

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 1];
        r.read_exact(&mut bytes)?;
        Ok(bytes[0])
    }
}

impl BytecodeEncoding for u16 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 2];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for u32 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for u64 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 8];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for u128 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 16];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for BigUint {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        let bytes = &self.to_bytes_be();
        let length = bytes.len() as u16;
        Ok(length.serialize(&mut w)? + w.write(bytes)?)
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let length: u16 = r.deserialize()?;
        let length = length as usize;
        let mut bytes = vec![0u8; length];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_bytes_be(bytes.as_slice()))
    }
}

impl BytecodeEncoding for i8 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 1];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for i16 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 2];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for i32 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for i64 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 8];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for i128 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 16];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for BigInt {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        let bytes = &self.to_signed_bytes_be();
        let length = bytes.len() as u16;
        Ok(length.serialize(&mut w)? + w.write(bytes)?)
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let length: u16 = r.deserialize()?;
        let length = length as usize;
        let mut bytes = vec![0u8; length];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_signed_bytes_be(bytes.as_slice()))
    }
}

impl BytecodeEncoding for f32 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 4];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

impl BytecodeEncoding for f64 {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        w.write(&self.to_be_bytes())
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let mut bytes = [0u8; 8];
        r.read_exact(&mut bytes)?;
        Ok(Self::from_be_bytes(bytes))
    }
}

const BIG_FRACTION_RATIONAL_TAG: u8 = 0xea;
const BIG_FRACTION_INFINITY_TAG: u8 = 0xeb;
const BIG_FRACTION_NAN_TAG: u8 = 0xec;

impl BytecodeEncoding for BigFraction {
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        match self {
            fraction::GenericFraction::Rational(ref sign, ref ratio) => {
                let mut out = 0;
                out += BIG_FRACTION_RATIONAL_TAG.serialize(&mut w)?;
                out += sign.serialize(&mut w)?;
                out += ratio.serialize(w)?;
                Ok(out)
            }

            fraction::GenericFraction::Infinity(ref sign) => {
                let mut out = 0;
                out += BIG_FRACTION_INFINITY_TAG.serialize(&mut w)?;
                out += sign.serialize(&mut w)?;
                Ok(out)
            }

            fraction::GenericFraction::NaN => BIG_FRACTION_NAN_TAG.serialize(w),
        }
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let tag: u8 = r.deserialize()?;
        match tag {
            BIG_FRACTION_RATIONAL_TAG => Ok(fraction::GenericFraction::Rational(
                r.deserialize()?,
                r.deserialize()?,
            )),
            BIG_FRACTION_INFINITY_TAG => Ok(fraction::GenericFraction::Infinity(r.deserialize()?)),
            BIG_FRACTION_NAN_TAG => Ok(fraction::GenericFraction::NaN),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid bigfloat tag",
            )),
        }
    }
}

const SIGN_PLUS_TAG: u8 = 0xf1;
const SIGN_MINUS_TAG: u8 = 0xf2;
impl BytecodeEncoding for fraction::Sign {
    fn serialize<W: Write>(&self, w: W) -> io::Result<usize> {
        match self {
            fraction::Sign::Plus => SIGN_MINUS_TAG.serialize(w),
            fraction::Sign::Minus => SIGN_PLUS_TAG.serialize(w),
        }
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        let tag: u8 = r.deserialize()?;
        match tag {
            SIGN_PLUS_TAG => Ok(fraction::Sign::Plus),
            SIGN_MINUS_TAG => Ok(fraction::Sign::Minus),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid sign tag",
            )),
        }
    }
}

impl<T> BytecodeEncoding for fraction::Ratio<T>
where
    T: BytecodeEncoding + fraction::Integer + Clone,
{
    fn serialize<W: Write>(&self, mut w: W) -> io::Result<usize> {
        Ok(self.numer().serialize(&mut w)? + self.denom().serialize(w)?)
    }

    fn deserialize<R: Read>(mut r: R) -> io::Result<Self> {
        Ok(fraction::Ratio::new(r.deserialize()?, r.deserialize()?))
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

impl<R, T> BytecodeEncodingRead<T> for R
where
    R: Read,
    T: BytecodeEncoding,
{
    fn deserialize(&mut self) -> io::Result<T> {
        T::deserialize(self)
    }
}
