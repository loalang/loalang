use crate::vm::{ConstValue, Object, VM};
use fraction::{BigFraction, BigUint};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub trait NativeMethods
where
    Self: Sized,
{
    #[inline]
    fn call(vm: &mut VM, method: NativeMethod) {
        match method {
            NativeMethod::Number_plus => Self::number_plus(vm),
        }
    }

    fn number_plus(vm: &mut VM) {
        let receiver = vm.pop();
        let operand = vm.pop();

        match (&receiver.const_value, &operand.const_value) {
            (ConstValue::Nothing, _)
            | (_, ConstValue::Nothing)
            | (ConstValue::String(_), _)
            | (_, ConstValue::String(_))
            | (ConstValue::Character(_), _)
            | (_, ConstValue::Character(_))
            | (ConstValue::Symbol(_), _)
            | (_, ConstValue::Symbol(_)) => panic!("not a number"),

            (ConstValue::U8(a), ConstValue::U8(b)) => vm.push(add_u8(*a, *b)),
            (ConstValue::U8(a), ConstValue::U16(b)) => vm.push(add_u16(*a as u16, *b)),
            (ConstValue::U8(a), ConstValue::U32(b)) => vm.push(add_u32(*a as u32, *b)),
            (ConstValue::U8(a), ConstValue::U64(b)) => vm.push(add_u64(*a as u64, *b)),
            (ConstValue::U8(a), ConstValue::U128(b)) => vm.push(add_u128(*a as u128, *b)),
            (ConstValue::U8(a), ConstValue::UBig(b)) => vm.push(add_ubig(&BigUint::from(*a), b)),

            (ConstValue::U16(a), ConstValue::U8(b)) => vm.push(add_u16(*a, *b as u16)),
            (ConstValue::U16(a), ConstValue::U16(b)) => vm.push(add_u16(*a, *b)),
            (ConstValue::U16(a), ConstValue::U32(b)) => vm.push(add_u32(*a as u32, *b)),
            (ConstValue::U16(a), ConstValue::U64(b)) => vm.push(add_u64(*a as u64, *b)),
            (ConstValue::U16(a), ConstValue::U128(b)) => vm.push(add_u128(*a as u128, *b)),
            (ConstValue::U16(a), ConstValue::UBig(b)) => vm.push(add_ubig(&BigUint::from(*a), b)),

            (ConstValue::U32(a), ConstValue::U8(b)) => vm.push(add_u32(*a, *b as u32)),
            (ConstValue::U32(a), ConstValue::U16(b)) => vm.push(add_u32(*a, *b as u32)),
            (ConstValue::U32(a), ConstValue::U32(b)) => vm.push(add_u32(*a, *b)),
            (ConstValue::U32(a), ConstValue::U64(b)) => vm.push(add_u64(*a as u64, *b)),
            (ConstValue::U32(a), ConstValue::U128(b)) => vm.push(add_u128(*a as u128, *b)),
            (ConstValue::U32(a), ConstValue::UBig(b)) => vm.push(add_ubig(&BigUint::from(*a), b)),

            (ConstValue::U64(a), ConstValue::U8(b)) => vm.push(add_u64(*a, *b as u64)),
            (ConstValue::U64(a), ConstValue::U16(b)) => vm.push(add_u64(*a, *b as u64)),
            (ConstValue::U64(a), ConstValue::U32(b)) => vm.push(add_u64(*a, *b as u64)),
            (ConstValue::U64(a), ConstValue::U64(b)) => vm.push(add_u64(*a, *b)),
            (ConstValue::U64(a), ConstValue::U128(b)) => vm.push(add_u128(*a as u128, *b)),
            (ConstValue::U64(a), ConstValue::UBig(b)) => vm.push(add_ubig(&BigUint::from(*a), b)),

            (ConstValue::U128(a), ConstValue::U8(b)) => vm.push(add_u128(*a, *b as u128)),
            (ConstValue::U128(a), ConstValue::U16(b)) => vm.push(add_u128(*a, *b as u128)),
            (ConstValue::U128(a), ConstValue::U32(b)) => vm.push(add_u128(*a, *b as u128)),
            (ConstValue::U128(a), ConstValue::U64(b)) => vm.push(add_u128(*a, *b as u128)),
            (ConstValue::U128(a), ConstValue::U128(b)) => vm.push(add_u128(*a, *b)),
            (ConstValue::U128(a), ConstValue::UBig(b)) => vm.push(add_ubig(&BigUint::from(*a), b)),

            (ConstValue::UBig(a), ConstValue::U8(b)) => vm.push(add_ubig(a, &BigUint::from(*b))),
            (ConstValue::UBig(a), ConstValue::U16(b)) => vm.push(add_ubig(a, &BigUint::from(*b))),
            (ConstValue::UBig(a), ConstValue::U32(b)) => vm.push(add_ubig(a, &BigUint::from(*b))),
            (ConstValue::UBig(a), ConstValue::U64(b)) => vm.push(add_ubig(a, &BigUint::from(*b))),
            (ConstValue::UBig(a), ConstValue::U128(b)) => vm.push(add_ubig(a, &BigUint::from(*b))),
            (ConstValue::UBig(a), ConstValue::UBig(b)) => vm.push(add_ubig(a, b)),

            (ConstValue::I8(a), ConstValue::I8(b)) => vm.push(add_i8(*a, *b)),
            (ConstValue::I8(a), ConstValue::I16(b)) => vm.push(add_i16(*a as i16, *b)),
            (ConstValue::I8(a), ConstValue::I32(b)) => vm.push(add_i32(*a as i32, *b)),
            (ConstValue::I8(a), ConstValue::I64(b)) => vm.push(add_i64(*a as i64, *b)),
            (ConstValue::I8(a), ConstValue::I128(b)) => vm.push(add_i128(*a as i128, *b)),
            (ConstValue::I8(a), ConstValue::IBig(b)) => vm.push(add_ibig(&BigInt::from(*a), b)),

            (ConstValue::I16(a), ConstValue::I8(b)) => vm.push(add_i16(*a, *b as i16)),
            (ConstValue::I16(a), ConstValue::I16(b)) => vm.push(add_i16(*a, *b)),
            (ConstValue::I16(a), ConstValue::I32(b)) => vm.push(add_i32(*a as i32, *b)),
            (ConstValue::I16(a), ConstValue::I64(b)) => vm.push(add_i64(*a as i64, *b)),
            (ConstValue::I16(a), ConstValue::I128(b)) => vm.push(add_i128(*a as i128, *b)),
            (ConstValue::I16(a), ConstValue::IBig(b)) => vm.push(add_ibig(&BigInt::from(*a), b)),

            (ConstValue::I32(a), ConstValue::I8(b)) => vm.push(add_i32(*a, *b as i32)),
            (ConstValue::I32(a), ConstValue::I16(b)) => vm.push(add_i32(*a, *b as i32)),
            (ConstValue::I32(a), ConstValue::I32(b)) => vm.push(add_i32(*a, *b)),
            (ConstValue::I32(a), ConstValue::I64(b)) => vm.push(add_i64(*a as i64, *b)),
            (ConstValue::I32(a), ConstValue::I128(b)) => vm.push(add_i128(*a as i128, *b)),
            (ConstValue::I32(a), ConstValue::IBig(b)) => vm.push(add_ibig(&BigInt::from(*a), b)),

            (ConstValue::I64(a), ConstValue::I8(b)) => vm.push(add_i64(*a, *b as i64)),
            (ConstValue::I64(a), ConstValue::I16(b)) => vm.push(add_i64(*a, *b as i64)),
            (ConstValue::I64(a), ConstValue::I32(b)) => vm.push(add_i64(*a, *b as i64)),
            (ConstValue::I64(a), ConstValue::I64(b)) => vm.push(add_i64(*a, *b)),
            (ConstValue::I64(a), ConstValue::I128(b)) => vm.push(add_i128(*a as i128, *b)),
            (ConstValue::I64(a), ConstValue::IBig(b)) => vm.push(add_ibig(&BigInt::from(*a), b)),

            (ConstValue::I128(a), ConstValue::I8(b)) => vm.push(add_i128(*a, *b as i128)),
            (ConstValue::I128(a), ConstValue::I16(b)) => vm.push(add_i128(*a, *b as i128)),
            (ConstValue::I128(a), ConstValue::I32(b)) => vm.push(add_i128(*a, *b as i128)),
            (ConstValue::I128(a), ConstValue::I64(b)) => vm.push(add_i128(*a, *b as i128)),
            (ConstValue::I128(a), ConstValue::I128(b)) => vm.push(add_i128(*a, *b)),
            (ConstValue::I128(a), ConstValue::IBig(b)) => vm.push(add_ibig(&BigInt::from(*a), b)),

            (ConstValue::IBig(a), ConstValue::I8(b)) => vm.push(add_ibig(a, &BigInt::from(*b))),
            (ConstValue::IBig(a), ConstValue::I16(b)) => vm.push(add_ibig(a, &BigInt::from(*b))),
            (ConstValue::IBig(a), ConstValue::I32(b)) => vm.push(add_ibig(a, &BigInt::from(*b))),
            (ConstValue::IBig(a), ConstValue::I64(b)) => vm.push(add_ibig(a, &BigInt::from(*b))),
            (ConstValue::IBig(a), ConstValue::I128(b)) => vm.push(add_ibig(a, &BigInt::from(*b))),
            (ConstValue::IBig(a), ConstValue::IBig(b)) => vm.push(add_ibig(a, b)),

            (ConstValue::U8(b), ConstValue::I8(a)) => vm.push(add_i16(*a as i16, *b as i16)),
            (ConstValue::U16(b), ConstValue::I8(a)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::U32(b), ConstValue::I8(a)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::U64(b), ConstValue::I8(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U128(b), ConstValue::I8(a)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::UBig(b), ConstValue::I8(a)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::U8(b), ConstValue::I16(a)) => vm.push(add_i16(*a as i16, *b as i16)),
            (ConstValue::U16(b), ConstValue::I16(a)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::U32(b), ConstValue::I16(a)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::U64(b), ConstValue::I16(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U128(b), ConstValue::I16(a)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::UBig(b), ConstValue::I16(a)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::U8(b), ConstValue::I32(a)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::U16(b), ConstValue::I32(a)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::U32(b), ConstValue::I32(a)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::U64(b), ConstValue::I32(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U128(b), ConstValue::I32(a)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::UBig(b), ConstValue::I32(a)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::U8(b), ConstValue::I64(a)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::U16(b), ConstValue::I64(a)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::U32(b), ConstValue::I64(a)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::U64(b), ConstValue::I64(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U128(b), ConstValue::I64(a)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::UBig(b), ConstValue::I64(a)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::U8(b), ConstValue::I128(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U16(b), ConstValue::I128(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U32(b), ConstValue::I128(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U64(b), ConstValue::I128(a)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::U128(b), ConstValue::I128(a)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::UBig(b), ConstValue::I128(a)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::U8(b), ConstValue::IBig(a)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::U16(b), ConstValue::IBig(a)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::U32(b), ConstValue::IBig(a)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::U64(b), ConstValue::IBig(a)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::U128(b), ConstValue::IBig(a)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::UBig(b), ConstValue::IBig(a)) => vm.push(add_ibig(a, &(b.clone().into()))),

            (ConstValue::I8(a), ConstValue::U8(b)) => vm.push(add_i16(*a as i16, *b as i16)),
            (ConstValue::I8(a), ConstValue::U16(b)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::I8(a), ConstValue::U32(b)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::I8(a), ConstValue::U64(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I8(a), ConstValue::U128(b)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::I8(a), ConstValue::UBig(b)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::I16(a), ConstValue::U8(b)) => vm.push(add_i16(*a as i16, *b as i16)),
            (ConstValue::I16(a), ConstValue::U16(b)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::I16(a), ConstValue::U32(b)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::I16(a), ConstValue::U64(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I16(a), ConstValue::U128(b)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::I16(a), ConstValue::UBig(b)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::I32(a), ConstValue::U8(b)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::I32(a), ConstValue::U16(b)) => vm.push(add_i32(*a as i32, *b as i32)),
            (ConstValue::I32(a), ConstValue::U32(b)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::I32(a), ConstValue::U64(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I32(a), ConstValue::U128(b)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::I32(a), ConstValue::UBig(b)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::I64(a), ConstValue::U8(b)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::I64(a), ConstValue::U16(b)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::I64(a), ConstValue::U32(b)) => vm.push(add_i64(*a as i64, *b as i64)),
            (ConstValue::I64(a), ConstValue::U64(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I64(a), ConstValue::U128(b)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::I64(a), ConstValue::UBig(b)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::I128(a), ConstValue::U8(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I128(a), ConstValue::U16(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I128(a), ConstValue::U32(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I128(a), ConstValue::U64(b)) => vm.push(add_i128(*a as i128, *b as i128)),
            (ConstValue::I128(a), ConstValue::U128(b)) => {
                vm.push(add_ibig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::I128(a), ConstValue::UBig(b)) => {
                vm.push(add_ibig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::IBig(a), ConstValue::U8(b)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::IBig(a), ConstValue::U16(b)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::IBig(a), ConstValue::U32(b)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::IBig(a), ConstValue::U64(b)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::IBig(a), ConstValue::U128(b)) => vm.push(add_ibig(a, &(*b).into())),
            (ConstValue::IBig(a), ConstValue::UBig(b)) => vm.push(add_ibig(a, &(b.clone().into()))),

            (ConstValue::F32(a), ConstValue::U8(b)) | (ConstValue::U8(b), ConstValue::F32(a)) => {
                vm.push(add_f32(*a, *b as f32))
            }
            (ConstValue::F32(a), ConstValue::I8(b)) | (ConstValue::I8(b), ConstValue::F32(a)) => {
                vm.push(add_f32(*a, *b as f32))
            }
            (ConstValue::F32(a), ConstValue::U16(b)) | (ConstValue::U16(b), ConstValue::F32(a)) => {
                vm.push(add_f32(*a, *b as f32))
            }
            (ConstValue::F32(a), ConstValue::I16(b)) | (ConstValue::I16(b), ConstValue::F32(a)) => {
                vm.push(add_f32(*a, *b as f32))
            }
            (ConstValue::F32(a), ConstValue::U32(b)) | (ConstValue::U32(b), ConstValue::F32(a)) => {
                vm.push(add_f64(*a as f64, *b as f64))
            }
            (ConstValue::F32(a), ConstValue::I32(b)) | (ConstValue::I32(b), ConstValue::F32(a)) => {
                vm.push(add_f64(*a as f64, *b as f64))
            }
            (ConstValue::F32(a), ConstValue::U64(b)) | (ConstValue::U64(b), ConstValue::F32(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F32(a), ConstValue::I64(b)) | (ConstValue::I64(b), ConstValue::F32(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F32(a), ConstValue::U128(b))
            | (ConstValue::U128(b), ConstValue::F32(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F32(a), ConstValue::I128(b))
            | (ConstValue::I128(b), ConstValue::F32(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F32(a), ConstValue::UBig(b))
            | (ConstValue::UBig(b), ConstValue::F32(a)) => {
                vm.push(add_fbig(&(*a).into(), &(b.clone().into())))
            }
            (ConstValue::F32(a), ConstValue::IBig(b))
            | (ConstValue::IBig(b), ConstValue::F32(a)) => {
                vm.push(add_fbig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::F64(a), ConstValue::U8(b)) | (ConstValue::U8(b), ConstValue::F64(a)) => {
                vm.push(add_f64(*a, *b as f64))
            }
            (ConstValue::F64(a), ConstValue::I8(b)) | (ConstValue::I8(b), ConstValue::F64(a)) => {
                vm.push(add_f64(*a, *b as f64))
            }
            (ConstValue::F64(a), ConstValue::U16(b)) | (ConstValue::U16(b), ConstValue::F64(a)) => {
                vm.push(add_f64(*a, *b as f64))
            }
            (ConstValue::F64(a), ConstValue::I16(b)) | (ConstValue::I16(b), ConstValue::F64(a)) => {
                vm.push(add_f64(*a, *b as f64))
            }
            (ConstValue::F64(a), ConstValue::U32(b)) | (ConstValue::U32(b), ConstValue::F64(a)) => {
                vm.push(add_f64(*a, *b as f64))
            }
            (ConstValue::F64(a), ConstValue::I32(b)) | (ConstValue::I32(b), ConstValue::F64(a)) => {
                vm.push(add_f64(*a, *b as f64))
            }
            (ConstValue::F64(a), ConstValue::U64(b)) | (ConstValue::U64(b), ConstValue::F64(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F64(a), ConstValue::I64(b)) | (ConstValue::I64(b), ConstValue::F64(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F64(a), ConstValue::U128(b))
            | (ConstValue::U128(b), ConstValue::F64(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F64(a), ConstValue::I128(b))
            | (ConstValue::I128(b), ConstValue::F64(a)) => {
                vm.push(add_fbig(&(*a).into(), &(*b).into()))
            }
            (ConstValue::F64(a), ConstValue::UBig(b))
            | (ConstValue::UBig(b), ConstValue::F64(a)) => {
                vm.push(add_fbig(&(*a).into(), &(b.clone().into())))
            }
            (ConstValue::F64(a), ConstValue::IBig(b))
            | (ConstValue::IBig(b), ConstValue::F64(a)) => {
                vm.push(add_fbig(&(*a).into(), &(b.clone().into())))
            }

            (ConstValue::FBig(a), ConstValue::U8(b)) | (ConstValue::U8(b), ConstValue::FBig(a)) => {
                vm.push(add_fbig(a, &(*b).into()))
            }
            (ConstValue::FBig(a), ConstValue::I8(b)) | (ConstValue::I8(b), ConstValue::FBig(a)) => {
                vm.push(add_fbig(a, &(*b).into()))
            }
            (ConstValue::FBig(a), ConstValue::U16(b))
            | (ConstValue::U16(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::I16(b))
            | (ConstValue::I16(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::U32(b))
            | (ConstValue::U32(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::I32(b))
            | (ConstValue::I32(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::U64(b))
            | (ConstValue::U64(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::I64(b))
            | (ConstValue::I64(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::U128(b))
            | (ConstValue::U128(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::I128(b))
            | (ConstValue::I128(b), ConstValue::FBig(a)) => vm.push(add_fbig(a, &(*b).into())),
            (ConstValue::FBig(a), ConstValue::UBig(b))
            | (ConstValue::UBig(b), ConstValue::FBig(a)) => {
                vm.push(add_fbig(a, &(b.clone().into())))
            }
            (ConstValue::FBig(a), ConstValue::IBig(b))
            | (ConstValue::IBig(b), ConstValue::FBig(a)) => {
                vm.push(add_fbig(a, &(b.clone().into())))
            }

            (ConstValue::F32(a), ConstValue::F32(b)) => vm.push(add_f32(*a, *b)),
            (ConstValue::F32(a), ConstValue::F64(b)) => vm.push(add_f64(*a as f64, *b)),
            (ConstValue::F64(a), ConstValue::F32(b)) => vm.push(add_f64(*a, *b as f64)),
            (ConstValue::F64(a), ConstValue::F64(b)) => vm.push(add_f64(*a, *b)),
            (ConstValue::F32(a), ConstValue::FBig(b)) => vm.push(add_fbig(&(*a).into(), b)),
            (ConstValue::F64(a), ConstValue::FBig(b)) => vm.push(add_fbig(&(*a).into(), b)),
            (ConstValue::FBig(b), ConstValue::F32(a)) => vm.push(add_fbig(&(*a).into(), b)),
            (ConstValue::FBig(b), ConstValue::F64(a)) => vm.push(add_fbig(&(*a).into(), b)),
            (ConstValue::FBig(a), ConstValue::FBig(b)) => vm.push(add_fbig(a, b)),
        }
    }
}

fn add_u8(lhs: u8, rhs: u8) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_u8(res),
        None => add_u16(lhs as u16, rhs as u16),
    }
}

fn add_u16(lhs: u16, rhs: u16) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_u16(res),
        None => add_u32(lhs as u32, rhs as u32),
    }
}

fn add_u32(lhs: u32, rhs: u32) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_u32(res),
        None => add_u64(lhs as u64, rhs as u64),
    }
}

fn add_u64(lhs: u64, rhs: u64) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_u64(res),
        None => add_u128(lhs as u128, rhs as u128),
    }
}

fn add_u128(lhs: u128, rhs: u128) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_u128(res),
        None => add_ubig(&BigUint::from(lhs), &BigUint::from(rhs)),
    }
}

fn add_ubig(lhs: &BigUint, rhs: &BigUint) -> Arc<Object> {
    Object::box_ubig(lhs + rhs)
}

fn add_i8(lhs: i8, rhs: i8) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_i8(res),
        None => add_i16(lhs as i16, rhs as i16),
    }
}

fn add_i16(lhs: i16, rhs: i16) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_i16(res),
        None => add_i32(lhs as i32, rhs as i32),
    }
}

fn add_i32(lhs: i32, rhs: i32) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_i32(res),
        None => add_i64(lhs as i64, rhs as i64),
    }
}

fn add_i64(lhs: i64, rhs: i64) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_i64(res),
        None => add_i128(lhs as i128, rhs as i128),
    }
}

fn add_i128(lhs: i128, rhs: i128) -> Arc<Object> {
    match lhs.checked_add(rhs) {
        Some(res) => Object::box_i128(res),
        None => add_ibig(&lhs.into(), &rhs.into()),
    }
}

fn add_ibig(lhs: &BigInt, rhs: &BigInt) -> Arc<Object> {
    Object::box_ibig(lhs + rhs)
}

fn add_f32(lhs: f32, rhs: f32) -> Arc<Object> {
    Object::box_f32(lhs + rhs)
}

fn add_f64(lhs: f64, rhs: f64) -> Arc<Object> {
    Object::box_f64(lhs + rhs)
}

fn add_fbig(lhs: &BigFraction, rhs: &BigFraction) -> Arc<Object> {
    Object::box_fbig(lhs + rhs)
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NativeMethod {
    Number_plus,
}

impl<'a> From<&'a str> for NativeMethod {
    fn from(name: &'a str) -> Self {
        use NativeMethod::*;
        match name {
            "Loa/Number#+" => Number_plus,
            n => panic!("unknown native method: {}", n),
        }
    }
}