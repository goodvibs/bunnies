use crate::utilities::Array;

/// Common shape for enums that expose a canonical list of variants.
pub const trait IterableEnum<const N: usize>: Copy + TryFrom<u8> + Into<u8> {
    const ALL: Array<Self, N>;
}

#[macro_export]
macro_rules! impl_u8_conversions {
    ($enum:ty, $count:expr) => {
        impl const TryFrom<u8> for $enum {
            type Error = &'static str;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                if value < $count {
                    Ok(unsafe { std::mem::transmute::<u8, Self>(value) })
                } else {
                    Err("Value out of bounds")
                }
            }
        }

        #[allow(clippy::from_over_into)]
        impl const Into<u8> for $enum {
            fn into(self) -> u8 {
                self as u8
            }
        }
    };
}
