extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterType, ParameterValue, ReturnType, ReturnValue
};
use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
use hyperlight_common::flatbuffer_wrappers::util::get_flatbuffer_result;
use hyperlight_guest::error::HyperlightGuestError;
use hyperlight_guest::host_function_call::get_host_return_value;

pub trait ToFlatbufParameter: Sized {
    const TYPE: ParameterType;
    fn to_value(self) -> ParameterValue;
    fn from_value(v: ParameterValue) -> Result<Self, HyperlightGuestError>;
}

pub trait ToFlatbufReturn: Sized {
    const TYPE: ReturnType;
    fn to_value(self) -> Vec<u8>;
    fn from_value(v: ReturnValue) -> Result<Self, HyperlightGuestError>;
}

pub trait FromFlatbufReturn: Sized {
    const TYPE: ReturnType;
    fn from_call(v: Result<(), HyperlightGuestError>) -> Self;
}

pub trait IntoFlatbufReturn: Sized {
    const TYPE: ReturnType;
    fn to_value(self) -> Result<Vec<u8>, HyperlightGuestError>;
}

macro_rules! impl_to_flatbuf_parameter {
    ($($type:ty => $enum:ident;)+) => {
        fn param_type_name(v: &ParameterValue) -> &'static str {
            match v {
                $(ParameterValue::$enum(_) => core::any::type_name::<$type>()),*
            }
        }

        $(impl ToFlatbufParameter for $type {
            const TYPE: ParameterType = ParameterType::$enum;
            fn to_value(self) -> ParameterValue {
                ParameterValue::$enum(self)
            }
            fn from_value(v: ParameterValue) -> Result<Self, HyperlightGuestError> {
                if let ParameterValue::$enum(value) = v {
                    Ok(value)
                } else {
                    Err(HyperlightGuestError::new(
                        ErrorCode::GuestError,
                        format!(
                            "Expected parameter type {}, but got {}",
                            core::any::type_name::<$type>(),
                            param_type_name(&v)
                        ),
                    ))
                }
            }
        })*
    };
}

macro_rules! impl_to_flatbuf_return {
    ($($type:ty => $enum:ident$(($var:ident))?$(.$fcn:ident())?;)+) => {
        fn return_type_name(v: &ReturnValue) -> &'static str {
            match v {
                $(ReturnValue::$enum$(($var))? => core::any::type_name::<$type>()),+
            }
        }

        $(impl ToFlatbufReturn for $type {
            const TYPE: ReturnType = ReturnType::$enum;

            fn to_value(self) -> Vec<u8> {
                get_flatbuffer_result(self$(.$fcn())?)
            }

            fn from_value(v: ReturnValue) -> Result<Self, HyperlightGuestError> {
                if let ReturnValue::$enum$(($var))? = v {
                    return Ok(($($var)?));
                }
                Err(HyperlightGuestError::new(
                    ErrorCode::GuestFunctionParameterTypeMismatch,
                    format!(
                        "Expected return type {}, but got {}",
                        core::any::type_name::<$type>(),
                        return_type_name(&v)
                    ),
                ))
            }
        })*
    };
}

impl_to_flatbuf_parameter! {
    i32 => Int;
    u32 => UInt;
    i64 => Long;
    u64 => ULong;
    f32 => Float;
    f64 => Double;
    bool => Bool;
    String => String;
    Vec<u8> => VecBytes;
}

impl_to_flatbuf_return! {
    i32 => Int(_x);
    u32 => UInt(_x);
    i64 => Long(_x);
    u64 => ULong(_x);
    f32 => Float(_x);
    f64 => Double(_x);
    bool => Bool(_x);
    String => String(_x).as_str();
    Vec<u8> => VecBytes(_x).as_slice();
    () => Void;
}

impl<T: ToFlatbufReturn> FromFlatbufReturn for T {
    const TYPE: ReturnType = <T as ToFlatbufReturn>::TYPE;
    fn from_call(v: Result<(), HyperlightGuestError>) -> Self {
        if let Err(e) = v {
            panic!("{}", e.message);
        }
        let ret = get_host_return_value::<ReturnValue>().unwrap();
        T::from_value(ret).unwrap_or_else(|e| {
            panic!("{}", e.message);
        })
    }
}
    

impl<T: ToFlatbufReturn, E: From<HyperlightGuestError>> FromFlatbufReturn for Result<T, E> {
    const TYPE: ReturnType = <T as ToFlatbufReturn>::TYPE;
    fn from_call(v: Result<(), HyperlightGuestError>) -> Self {
        v?;
        let ret = get_host_return_value::<ReturnValue>().unwrap();
        Ok(T::from_value(ret)?)
    }
}

impl<T: ToFlatbufReturn> IntoFlatbufReturn for T {
    const TYPE: ReturnType = <T as ToFlatbufReturn>::TYPE;
    fn to_value(self) -> Result<Vec<u8>, HyperlightGuestError> {
        Ok(T::to_value(self))
    }
}

impl<T: ToFlatbufReturn, E: Into<HyperlightGuestError>> IntoFlatbufReturn for Result<T, E> {
    const TYPE: ReturnType = <T as ToFlatbufReturn>::TYPE;
    fn to_value(self) -> Result<Vec<u8>, HyperlightGuestError> {
        match self {
            Ok(v) => Ok(T::to_value(v)),
            Err(e) => Err(e.into()),
        }
    }
}
