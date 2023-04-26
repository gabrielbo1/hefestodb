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

use crc::{Crc, CRC_32_ISCSI};

const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
const MASK_DELTA: u32 = 0xa282ead8;

pub fn value(data: &[u8]) -> u32 {
    CASTAGNOLI.checksum(data)
}

/// Return a masked representation of `crc`
pub fn mask(crc: u32) -> u32 {
    // Rotate right by 15 bits and add a constant
    ((crc >> 15) | (crc << 17)).wrapping_add(MASK_DELTA)
}

/// Return the crc whose masked representation is `masked_crc`.
pub fn unmask(masked_crc: u32) -> u32 {
    let rot = masked_crc.wrapping_sub(MASK_DELTA);
    (rot >> 17) | (rot << 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn startd_results() {
        assert_eq!(value(b"123456789"), 0xe3069283);
    }

    #[test]
    pub fn values() {
        assert_ne!(value("a".as_bytes()), value("foo".as_bytes()));
    }

    #[test]
    pub fn mask() {
        let crc = value("foo".as_bytes());
        assert_ne!(super::mask(crc), crc);
        assert_ne!(super::mask(super::mask(crc)), crc);
        assert_eq!(unmask(super::mask(crc)), crc);
        assert_eq!(unmask(unmask(super::mask(super::mask(crc)))), crc);
    }
}
