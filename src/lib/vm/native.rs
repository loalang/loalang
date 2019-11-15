use crate::vm::{ConstValue, Object, VM};
use serde::{Deserialize, Serialize};

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
            (ConstValue::U8(receiver), ConstValue::U8(operand)) => {
                vm.push(Object::box_u8(receiver + operand))
            }
            (ConstValue::U16(receiver), ConstValue::U16(operand)) => {
                vm.push(Object::box_u16(receiver + operand))
            }
            (ConstValue::U32(receiver), ConstValue::U32(operand)) => {
                vm.push(Object::box_u32(receiver + operand))
            }
            (ConstValue::U64(receiver), ConstValue::U64(operand)) => {
                vm.push(Object::box_u64(receiver + operand))
            }
            (ConstValue::U128(receiver), ConstValue::U128(operand)) => {
                vm.push(Object::box_u128(receiver + operand))
            }
            (ConstValue::UBig(receiver), ConstValue::UBig(operand)) => {
                vm.push(Object::box_ubig(receiver + operand))
            }
            (ConstValue::I8(receiver), ConstValue::I8(operand)) => {
                vm.push(Object::box_i8(receiver + operand))
            }
            (ConstValue::I16(receiver), ConstValue::I16(operand)) => {
                vm.push(Object::box_i16(receiver + operand))
            }
            (ConstValue::I32(receiver), ConstValue::I32(operand)) => {
                vm.push(Object::box_i32(receiver + operand))
            }
            (ConstValue::I64(receiver), ConstValue::I64(operand)) => {
                vm.push(Object::box_i64(receiver + operand))
            }
            (ConstValue::I128(receiver), ConstValue::I128(operand)) => {
                vm.push(Object::box_i128(receiver + operand))
            }
            (ConstValue::IBig(receiver), ConstValue::IBig(operand)) => {
                vm.push(Object::box_ibig(receiver + operand))
            }
            _ => panic!("expected u8 operands"),
        }
    }
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
