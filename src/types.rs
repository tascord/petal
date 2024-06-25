use std::{fmt::Debug, ops::Mul};

//

macro_rules! map {
    ($value:expr, $max:ty, $max_slf:expr, $($ty:ty => $slf:expr),*) => {
        if false { unreachable!() }
        $(else if ($value <= <$ty>::MAX as $max) { $slf($value as $ty) })*
        else { $max_slf($value) }
    };
}

//

pub trait VariablySized: Num + Sized {
    fn fit(value: Self::Max) -> Self;
}

pub trait Num {
    type Max: Mul;
    fn to_max_value(&self) -> Self::Max;
}

//

#[derive(Clone, Debug, Hash)]
pub enum Int {
    _8(i8),
    _16(i16),
    _32(i32),
    _64(i64),
    _128(i128),
}

impl Num for Int {
    type Max = i128;
    fn to_max_value(&self) -> Self::Max {
        match self {
            Self::_8(value) => *value as i128,
            Self::_16(value) => *value as i128,
            Self::_32(value) => *value as i128,
            Self::_64(value) => *value as i128,
            Self::_128(value) => *value,
        }
    }
}

impl VariablySized for Int {
    fn fit(value: Self::Max) -> Self {
        map! {
            value, i128, Self::_128,
            i8 => Self::_8,
            i16 => Self::_16,
            i32 => Self::_32,
            i64 => Self::_64
        }
    }
}

impl PartialEq for Int {
    fn eq(&self, other: &Self) -> bool {
        self.to_max_value() == other.to_max_value()
    }
}

//

#[derive(Clone, Debug)]
pub enum Float {
    _32(f32),
    _64(f64),
}

impl Num for Float {
    type Max = f64;
    fn to_max_value(&self) -> Self::Max {
        match self {
            Self::_32(value) => *value as f64,
            Self::_64(value) => *value,
        }
    }
}

impl VariablySized for Float {
    fn fit(value: Self::Max) -> Self {
        map! {
            value, f64, Self::_64,
            f32 => Self::_32
        }
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.to_max_value() == other.to_max_value()
    }
}
