// MIT License
//
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Index,
    ptr, slice,
};

use crate::util::bit;

/// Just like Rust's slice, except there's no borrowing. Instead, the user needs to
/// guarantee that the instances of this struct should not live longer than the memory
/// that `data` points to.
#[derive(Clone, Debug)] // TODO: double check Hash
pub struct Slice {
    data: *const u8,
    size: usize,
}

impl Slice {
    /// Create an empty slice
    pub fn new_empty() -> Self { Self::new(ptr::null(), 0) }

    /// Create a slice that refers to `d`
    pub fn new(data: *const u8, size: usize) -> Self { Self { data, size } }

    /// Return the length (in bytes) of the referenced data
    #[inline]
    pub fn len(&self) -> usize { self.size }

    /// Return the raw pointer to the internal data
    #[inline]
    pub fn raw_data(&self) -> *const u8 { self.data }

    /// Return a slice to the internal data.
    /// This should be preferred over `raw_data()`.
    #[inline]
    pub fn data(&self) -> &[u8] { unsafe { slice::from_raw_parts(self.data, self.size) } }

    /// Return true iff the length of the referenced data is zero
    #[inline]
    pub fn empty(&self) -> bool { self.len() == 0 }

    /// Change this slice to refer to an empty array
    #[inline]
    pub fn clear(&mut self) {
        self.data = ptr::null();
        self.size = 0;
    }

    /// Advance and drop the first `n` bytes from this slice.
    pub fn skip(&mut self, n: usize) {
        assert!(n <= self.len());
        unsafe {
            self.data = self.data.offset(n as isize);
        }
        self.size -= n;
    }

    /// Return true iff `x` is a prefix of `self`
    pub fn starts_with(&self, x: &Slice) -> bool {
        unsafe { self.len() >= x.len() && bit::memcmp(self.data, x.data, x.len()) == 0 }
    }

    /// Returns a string from the slice data. Copying the contents.
    pub fn as_str(&self) -> &str {
        unsafe { ::std::str::from_utf8_unchecked(self.data()) }
    }

    /// Returns a string from the slice data. Copying the contents.
    pub fn to_string(&self) -> String { self.as_str().to_string() }

    /// Three-way comparison. Returns value:
    ///   `Ordering::Less`    iff `self` < `b`
    ///   `Ordering::Equal`   iff `self` = `b`
    ///   `Ordering::Greater` iff `self` > `b`
    #[inline]
    pub fn compare(&self, b: &Slice) -> Ordering {
        let min_len = if self.len() < b.len() {
            self.len()
        } else {
            b.len()
        };
        let r = unsafe { bit::memcmp(self.data, b.data, min_len) };
        match r {
            _ if r > 0 => Ordering::Greater,
            _ if r < 0 => Ordering::Less,
            0 if self.size > b.size => Ordering::Greater,
            0 if self.size < b.size => Ordering::Less,
            _ => Ordering::Equal,
        }
    }
}

impl Index<usize> for Slice {
    type Output = u8;

    /// Return the ith byte in the referenced data
    /// REQUIRES: index < self.len()
    fn index(&self, index: usize) -> &u8 { unsafe { &*self.data.offset(index as isize) } }
}

impl<'a> From<&'a [u8]> for Slice {
    #[inline]
    fn from(s: &'a [u8]) -> Self { Slice::new(s.as_ptr(), s.len()) }
}

impl<'a> From<&'a Vec<u8>> for Slice {
    #[inline]
    fn from(v: &'a Vec<u8>) -> Self { Slice::new(v[..].as_ptr(), v.len()) }
}

impl<'a> From<&'a str> for Slice {
    #[inline]
    fn from(s: &'a str) -> Self { Slice::new(s.as_ptr(), s.len()) }
}

impl From<String> for Slice {
    #[inline]
    fn from(s: String) -> Self { Slice::new(s.as_ptr(), s.len()) }
}

impl PartialEq for Slice {
    fn eq(&self, other: &Slice) -> bool { self.compare(other) == Ordering::Equal }
}

impl Eq for Slice {}

impl Hash for Slice {
    fn hash<H: Hasher>(&self, state: &mut H) { state.write(self.data()); }
}