use crate::vm::*;
use crate::*;
use fraction::BigFraction;
use num_bigint::{BigInt, BigUint};

#[derive(Debug, Clone)]
pub enum ConstValue {
    Nothing,
    InstanceVariables(HashMap<u64, Arc<Object>>),
    String(String),
    Lazy(u64, CallStack, Vec<Arc<Object>>),
    Character(u16),
    Symbol(String),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    UBig(BigUint),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    IBig(BigInt),
    F32(f32),
    F64(f64),
    FBig(BigFraction),
}

impl From<()> for ConstValue {
    fn from(_: ()) -> Self {
        ConstValue::Nothing
    }
}

impl Into<()> for ConstValue {
    fn into(self) -> () {}
}

impl From<String> for ConstValue {
    fn from(v: String) -> Self {
        ConstValue::String(v)
    }
}

impl Into<String> for ConstValue {
    fn into(self) -> String {
        if let ConstValue::String(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<u8> for ConstValue {
    fn from(v: u8) -> Self {
        ConstValue::U8(v)
    }
}

impl Into<u8> for ConstValue {
    fn into(self) -> u8 {
        if let ConstValue::U8(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<u16> for ConstValue {
    fn from(v: u16) -> Self {
        ConstValue::U16(v)
    }
}

impl Into<u16> for ConstValue {
    fn into(self) -> u16 {
        if let ConstValue::U16(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<u32> for ConstValue {
    fn from(v: u32) -> Self {
        ConstValue::U32(v)
    }
}

impl Into<u32> for ConstValue {
    fn into(self) -> u32 {
        if let ConstValue::U32(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<u64> for ConstValue {
    fn from(v: u64) -> Self {
        ConstValue::U64(v)
    }
}

impl Into<u64> for ConstValue {
    fn into(self) -> u64 {
        if let ConstValue::U64(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<u128> for ConstValue {
    fn from(v: u128) -> Self {
        ConstValue::U128(v)
    }
}

impl Into<u128> for ConstValue {
    fn into(self) -> u128 {
        if let ConstValue::U128(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<BigUint> for ConstValue {
    fn from(v: BigUint) -> Self {
        ConstValue::UBig(v)
    }
}

impl Into<BigUint> for ConstValue {
    fn into(self) -> BigUint {
        if let ConstValue::UBig(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<i8> for ConstValue {
    fn from(v: i8) -> Self {
        ConstValue::I8(v)
    }
}

impl Into<i8> for ConstValue {
    fn into(self) -> i8 {
        if let ConstValue::I8(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<i16> for ConstValue {
    fn from(v: i16) -> Self {
        ConstValue::I16(v)
    }
}

impl Into<i16> for ConstValue {
    fn into(self) -> i16 {
        if let ConstValue::I16(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<i32> for ConstValue {
    fn from(v: i32) -> Self {
        ConstValue::I32(v)
    }
}

impl Into<i32> for ConstValue {
    fn into(self) -> i32 {
        if let ConstValue::I32(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<i64> for ConstValue {
    fn from(v: i64) -> Self {
        ConstValue::I64(v)
    }
}

impl Into<i64> for ConstValue {
    fn into(self) -> i64 {
        if let ConstValue::I64(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<i128> for ConstValue {
    fn from(v: i128) -> Self {
        ConstValue::I128(v)
    }
}

impl Into<i128> for ConstValue {
    fn into(self) -> i128 {
        if let ConstValue::I128(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<BigInt> for ConstValue {
    fn from(v: BigInt) -> Self {
        ConstValue::IBig(v)
    }
}

impl Into<BigInt> for ConstValue {
    fn into(self) -> BigInt {
        if let ConstValue::IBig(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<f32> for ConstValue {
    fn from(v: f32) -> Self {
        ConstValue::F32(v)
    }
}

impl Into<f32> for ConstValue {
    fn into(self) -> f32 {
        if let ConstValue::F32(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<f64> for ConstValue {
    fn from(v: f64) -> Self {
        ConstValue::F64(v)
    }
}

impl Into<f64> for ConstValue {
    fn into(self) -> f64 {
        if let ConstValue::F64(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}

impl From<BigFraction> for ConstValue {
    fn from(v: BigFraction) -> Self {
        ConstValue::FBig(v)
    }
}

impl Into<BigFraction> for ConstValue {
    fn into(self) -> BigFraction {
        if let ConstValue::FBig(v) = self {
            v
        } else {
            panic!("failed to unbox")
        }
    }
}
