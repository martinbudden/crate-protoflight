/*
Implements `TryFrom<u8>` for an enum.
Assumes:

1. enum is #[repr(u8)]
2. enum implements `default`.
3. `from_u8` maps all invalid values of the enum to default.
4. default value is different from all other enum values (this is naturally true for an enum).

 */

#[allow(unused)]
macro_rules! impl_try_from_u8 {
    ($type:ty) => {
        impl TryFrom<u8> for $type {
            type Error = ();

            /// Validating conversion, invalid values return error.
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                let default = Self::default();

                if value == default as u8 {
                    // The input was actually the default value.
                    Ok(default)
                } else {
                    let ret = Self::from_u8(value);
                    if ret == default {
                        // The input was invalid and got converted to the default.
                        Err(())
                    } else {
                        Ok(ret)
                    }
                }
            }
        }
    };
}
