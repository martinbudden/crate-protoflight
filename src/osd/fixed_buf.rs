use core::{
    fmt::{self, Write},
    ops::{Index, IndexMut, Range, RangeBounds},
};
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
    /// Clears the buffer.
    pub fn clear(&mut self) {
        self.length = 0;
    }
    /// Returns a slice of the active raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.length]
    }

    /// Returns a mutable slice of the active raw bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.buf[..self.length]
    }

    /// Returns a valid string slice (&str).
    /// Returns an Err if the data isn't valid UTF-8.
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }

    /// Returns None if index is out of bounds.
    pub fn get(&self, index: usize) -> Option<&u8> {
        if index < self.length { Some(&self.buf[index]) } else { None }
    }

    // Mutable Getter: Use this to update a single byte safely
    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        if index < self.length { Some(&mut self.buf[index]) } else { None }
    }

    /// Fills a specific range of the active buffer with a single byte value.
    /// Returns `Err(())` if the range is out of bounds of the active length.
    pub fn try_fill_range<R>(&mut self, range: R, value: u8) -> Result<(), ()>
    where
        R: RangeBounds<usize>,
    {
        // 1. Resolve start and end bounds into concrete indices
        let start = match range.start_bound() {
            core::ops::Bound::Included(&s) => s,
            core::ops::Bound::Excluded(&s) => s + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(&e) => e + 1,
            core::ops::Bound::Excluded(&e) => e,
            core::ops::Bound::Unbounded => self.length,
        };

        // 2. Bound check against the logical active length
        if start > end || end > self.length {
            return Err(());
        }

        // 3. Perform the fill operations safely without panicking
        self.buf[start..end].fill(value);
        Ok(())
    }
}

/// Allows referencing `FixedBuf` directly as a byte slice: &[u8].
impl<const N: usize> AsRef<[u8]> for FixedBuf<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Allows referencing `FixedBuf` directly as a mutable byte slice: &mut [u8].
impl<const N: usize> AsMut<[u8]> for FixedBuf<N> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

// Immutable Index Trait: Implementation for reading syntax via `buf[i]`
impl<const N: usize> Index<usize> for FixedBuf<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        // Asserting bounds against length ensures you don't read uninitialized trailing bytes
        assert!(index < self.length, "FixedBuf index out of bounds");
        &self.buf[index]
    }
}

// Mutable Index Trait: Implementation for modifying syntax via `buf[i] = val`
impl<const N: usize> IndexMut<usize> for FixedBuf<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.length, "FixedBuf index out of bounds");
        &mut self.buf[index]
    }
}

// Immutable Range Slicing: Allows `&buf[1..4]`
impl<const N: usize> Index<Range<usize>> for FixedBuf<N> {
    type Output = [u8];

    fn index(&self, range: Range<usize>) -> &Self::Output {
        // Enforce safety: bounds check against active length, not the full capacity N
        assert!(range.end <= self.length, "FixedBuf slice index out of bounds");
        &self.buf[range]
    }
}

// Mutable Range Slicing: Allows `&mut buf[1..4]`
impl<const N: usize> IndexMut<Range<usize>> for FixedBuf<N> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        assert!(range.end <= self.length, "FixedBuf slice index out of bounds");
        &mut self.buf[range]
    }
}

// The core formatting trait
impl<const N: usize> Write for FixedBuf<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();

        // Safety: Prevent buffer overflow (equivalent to snprintf safety!)
        if self.length + bytes.len() > N {
            return Err(fmt::Error);
        }

        // Copy incoming formatted string slice into our inline stack array
        self.buf[self.length..self.length + bytes.len()].copy_from_slice(bytes);
        self.length += bytes.len();
        Ok(())
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
