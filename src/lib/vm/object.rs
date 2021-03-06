use crate::syntax::characters_to_string;
use crate::vm::*;
use crate::*;
use std::f64::INFINITY;
use std::ptr::null;

pub static mut STRING_CLASS: *const Class = null();
pub static mut CHARACTER_CLASS: *const Class = null();
pub static mut SYMBOL_CLASS: *const Class = null();

pub static mut U8_CLASS: *const Class = null();
pub static mut U16_CLASS: *const Class = null();
pub static mut U32_CLASS: *const Class = null();
pub static mut U64_CLASS: *const Class = null();
pub static mut U128_CLASS: *const Class = null();
pub static mut UBIG_CLASS: *const Class = null();
pub static mut I8_CLASS: *const Class = null();
pub static mut I16_CLASS: *const Class = null();
pub static mut I32_CLASS: *const Class = null();
pub static mut I64_CLASS: *const Class = null();
pub static mut I128_CLASS: *const Class = null();
pub static mut IBIG_CLASS: *const Class = null();
pub static mut F32_CLASS: *const Class = null();
pub static mut F64_CLASS: *const Class = null();
pub static mut FBIG_CLASS: *const Class = null();

pub static mut TRUE: *const Arc<Object> = null();
pub static mut FALSE: *const Arc<Object> = null();

#[derive(Debug)]
pub struct Object {
    pub class: Option<Arc<Class>>,
    pub const_value: ConstValue,
}

impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        self.class
            .as_ref()
            .and_then(|lc| other.class.as_ref().map(|rc| Arc::ptr_eq(lc, rc)))
            .unwrap_or(false)
            && self.const_value == other.const_value
    }
}

impl Object {
    pub fn new(class: &Arc<Class>) -> Arc<Object> {
        Arc::new(Object {
            class: Some(class.clone()),
            const_value: ConstValue::Nothing,
        })
    }

    pub fn get_variable(&self, variable: &Variable) -> Option<Arc<Object>> {
        match self.const_value {
            ConstValue::InstanceVariables(ref v) => v.get(&variable.id).cloned(),
            _ => None,
        }
    }

    pub fn set_variable(&self, variable: &Variable, value: Arc<Object>) -> Arc<Object> {
        Arc::new(Object {
            class: self.class.clone(),
            const_value: match self.const_value {
                ConstValue::InstanceVariables(ref v) => {
                    let mut v = v.clone();
                    v.insert(variable.id, value);
                    ConstValue::InstanceVariables(v)
                }
                _ => {
                    ConstValue::InstanceVariables(vec![(variable.id, value)].into_iter().collect())
                }
            },
        })
    }

    pub fn box_bool(value: bool) -> Arc<Object> {
        if value {
            unsafe { (*TRUE).clone() }
        } else {
            unsafe { (*FALSE).clone() }
        }
    }

    pub fn box_const(const_value: ConstValue, class_ptr: &mut *const Class) -> Arc<Object> {
        #[cfg(debug_assertions)]
        {
            if *class_ptr == null() {
                panic!("Tried to box {:?}, but class was not loaded.", const_value);
            }
        }
        let class = unsafe { Arc::from_raw(*class_ptr) };
        *class_ptr = Arc::into_raw(class.clone());
        Arc::new(Object {
            class: Some(class.clone()),
            const_value,
        })
    }

    pub fn lazy(offset: u64, call_stack: CallStack, dependencies: Vec<Arc<Object>>) -> Arc<Object> {
        Arc::new(Object {
            class: None,
            const_value: ConstValue::Lazy(offset, call_stack, dependencies),
        })
    }

    pub fn box_string(value: String) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { STRING_CLASS })
    }

    pub fn box_character(value: u16) -> Arc<Object> {
        Object::box_const(ConstValue::Character(value), &mut unsafe {
            CHARACTER_CLASS
        })
    }

    pub fn box_symbol(value: String) -> Arc<Object> {
        Object::box_const(ConstValue::Symbol(value), &mut unsafe { SYMBOL_CLASS })
    }

    pub fn box_u8(value: u8) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { U8_CLASS })
    }

    pub fn box_u16(value: u16) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { U16_CLASS })
    }

    pub fn box_u32(value: u32) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { U32_CLASS })
    }

    pub fn box_u64(value: u64) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { U64_CLASS })
    }

    pub fn box_u128(value: u128) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { U128_CLASS })
    }

    pub fn box_ubig(value: BigUint) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { UBIG_CLASS })
    }

    pub fn box_i8(value: i8) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { I8_CLASS })
    }

    pub fn box_i16(value: i16) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { I16_CLASS })
    }

    pub fn box_i32(value: i32) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { I32_CLASS })
    }

    pub fn box_i64(value: i64) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { I64_CLASS })
    }

    pub fn box_i128(value: i128) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { I128_CLASS })
    }

    pub fn box_ibig(value: BigInt) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { IBIG_CLASS })
    }

    pub fn box_f32(value: f32) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { F32_CLASS })
    }

    pub fn box_f64(value: f64) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { F64_CLASS })
    }

    pub fn box_fbig(value: BigFraction) -> Arc<Object> {
        Object::box_const(value.into(), &mut unsafe { FBIG_CLASS })
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.const_value {
            ConstValue::Nothing => write!(
                f,
                "{}",
                self.class.as_ref().map(|c| c.name.as_ref()).unwrap_or("")
            ),
            ConstValue::InstanceVariables(ref v) => write!(
                f,
                "a {}({})",
                self.class.as_ref().map(|c| c.name.as_ref()).unwrap_or(""),
                {
                    let mut variables = std::collections::BTreeSet::new();

                    let class = self.class.as_ref().unwrap();
                    for (id, value) in v.iter() {
                        let mut var = class.variables.get(&id).unwrap().name.clone();
                        var.push('=');
                        var.push_str(format!("{}", value).as_ref());
                        variables.insert(var);
                    }

                    variables.into_iter().collect::<Vec<_>>().join(", ")
                }
            ),
            ConstValue::String(s) => write!(f, "{}", s),
            ConstValue::Lazy(_, _, _) => write!(f, "$lazy"),
            ConstValue::Character(c) => write!(f, "{}", characters_to_string([*c].iter().cloned())),
            ConstValue::Symbol(s) => write!(f, "#{}", s),
            ConstValue::U8(n) => write!(f, "{}", n),
            ConstValue::U16(n) => write!(f, "{}", n),
            ConstValue::U32(n) => write!(f, "{}", n),
            ConstValue::U64(n) => write!(f, "{}", n),
            ConstValue::U128(n) => write!(f, "{}", n),
            ConstValue::UBig(n) => write!(f, "{}", n),
            ConstValue::I8(n) => write!(f, "{}", n),
            ConstValue::I16(n) => write!(f, "{}", n),
            ConstValue::I32(n) => write!(f, "{}", n),
            ConstValue::I64(n) => write!(f, "{}", n),
            ConstValue::I128(n) => write!(f, "{}", n),
            ConstValue::IBig(n) => write!(f, "{}", n),
            ConstValue::F32(n) => write!(f, "{}", n),
            ConstValue::F64(n) => write!(f, "{}", n),
            ConstValue::FBig(n) => write!(f, "{:.1$}", n, INFINITY as usize),
        }
    }
}
