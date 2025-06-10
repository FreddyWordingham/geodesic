use nalgebra::RealField;
use std::any::type_name;

use crate::error::NumericError;

pub trait FallibleNumeric<T> {
    type Error;

    fn try_min_value() -> Result<T, Self::Error>;
    fn try_max_value() -> Result<T, Self::Error>;
    fn try_from_u8(n: u8) -> Result<T, Self::Error>;
    fn try_from_f32(n: f32) -> Result<T, Self::Error>;
    fn try_from_f64(n: f64) -> Result<T, Self::Error>;
    fn try_from_usize(n: usize) -> Result<T, Self::Error>;
}

impl<T: RealField + Copy> FallibleNumeric<T> for T {
    type Error = NumericError;

    fn try_min_value() -> Result<T, Self::Error> {
        T::min_value().ok_or_else(|| NumericError::BoundsUnavailable {
            type_name: type_name::<T>().to_string(),
        })
    }

    fn try_max_value() -> Result<T, Self::Error> {
        T::max_value().ok_or_else(|| NumericError::BoundsUnavailable {
            type_name: type_name::<T>().to_string(),
        })
    }

    fn try_from_u8(n: u8) -> Result<T, Self::Error> {
        T::from_u8(n).ok_or_else(|| NumericError::TypeConversion {
            from_type: "u8".to_string(),
            to_type: type_name::<T>().to_string(),
        })
    }

    fn try_from_f32(n: f32) -> Result<T, Self::Error> {
        T::from_f32(n).ok_or_else(|| NumericError::TypeConversion {
            from_type: "f32".to_string(),
            to_type: type_name::<T>().to_string(),
        })
    }

    fn try_from_f64(n: f64) -> Result<T, Self::Error> {
        T::from_f64(n).ok_or_else(|| NumericError::TypeConversion {
            from_type: "f64".to_string(),
            to_type: type_name::<T>().to_string(),
        })
    }

    fn try_from_usize(n: usize) -> Result<T, Self::Error> {
        T::from_usize(n).ok_or_else(|| NumericError::TypeConversion {
            from_type: "usize".to_string(),
            to_type: type_name::<T>().to_string(),
        })
    }
}
