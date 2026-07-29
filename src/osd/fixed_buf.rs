#[cfg(feature = "serde")]
use sequential_storage::map::PostcardValue;
// Ensure serde features are present
#[cfg(feature = "serde")]
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixedBuf<const N: usize> {
    pub buf: [u8; N],
    pub length: usize,
}

impl<const N: usize> FixedBuf<N> {
    pub const fn new() -> Self {
        Self { buf: [0u8; N], length: 0 }
    }
}

impl<const N: usize> Default for FixedBuf<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl<const N: usize> FixedBuf<N> {
    // Returns a slice of the active raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.length]
    }

    // Returns a mutable slice of the active raw bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.length]
    }

    // Returns a valid string slice (&str).
    // Returns an Err if the data isn't valid UTF-8.
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }
}
// Allows referencing FixedBuf directly as a byte slice: &[u8]
impl<const N: usize> AsRef<[u8]> for FixedBuf<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

// Allows referencing FixedBuf directly as a mutable byte slice: &mut [u8]
impl<const N: usize> AsMut<[u8]> for FixedBuf<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

// Serialize Implementation for FixedBuf
#[cfg(feature = "serde")]
impl<const N: usize> Serialize for FixedBuf<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Only serialize the active part of the buffer (the valid slice)
        serializer.serialize_bytes(self.as_bytes())
    }
}

// Deserialize Implementation for FixedBuf
#[cfg(feature = "serde")]
impl<'de, const N: usize> Deserialize<'de> for FixedBuf<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FixedBufVisitor<const M: usize>;

        impl<'de, const M: usize> Visitor<'de> for FixedBufVisitor<M> {
            type Value = FixedBuf<M>;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a byte array or byte sequence")
            }

            // Handles contiguous byte slices optimally (e.g. from Postcard/Bincode)
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() > M {
                    return Err(E::custom("buffer overflow: input data exceeds capacity"));
                }
                let mut fixed_buf = FixedBuf::new();
                fixed_buf.buf[..v.len()].copy_from_slice(v);
                fixed_buf.length = v.len();
                Ok(fixed_buf)
            }

            // Fallback handler for generic data sequences (e.g. JSON arrays)
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut fixed_buf = FixedBuf::new();
                let mut idx = 0;
                while let Some(byte) = seq.next_element()? {
                    if idx >= M {
                        return Err(serde::de::Error::custom("buffer overflow"));
                    }
                    fixed_buf.buf[idx] = byte;
                    idx += 1;
                }
                fixed_buf.length = idx;
                Ok(fixed_buf)
            }
        }

        deserializer.deserialize_bytes(FixedBufVisitor::<N>)
    }
}

#[cfg(feature = "serde")]
impl<const N: usize> PostcardValue<'_> for FixedBuf<N> {}

#[cfg(test)]
mod tests {

    use super::*;

    fn _is_normal<T: Sized + Send + Sync + Unpin>() {}
    fn is_full<T: Sized + Send + Sync + Unpin + Copy + Clone + Default + PartialEq>() {}
    #[cfg(feature = "serde")]
    fn is_config<T: Serialize + for<'a> Deserialize<'a> + for<'a> PostcardValue<'a>>() {}

    #[test]
    fn normal_types() {
        is_full::<FixedBuf<16>>();
        #[cfg(feature = "serde")]
        is_config::<FixedBuf<16>>();
    }
}
