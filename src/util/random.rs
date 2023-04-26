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
use std::cell::Cell;

pub struct Random {
    seed: Cell<u32>,
}

/// Prime number 2^31-1
const MERSENNE_PRIME: u32 = 2147483647;

/// A simple random number generator that uses multiplicative
/// congruential generator (MCG)
impl Random {
    pub fn new(s: u32) -> Self {
        let mut seed = s & 0x7fffffffu32;
        if seed == 0 || seed == MERSENNE_PRIME {
            seed = 1
        }
        Self {
            seed: Cell::new(seed)
        }
    }

    /// Return the next random number in this generator
    pub fn next(&self) -> u32 {
        // See https://en.wikipedia.org/wiki/Linear_congruential_generator
        let m: u32 = MERSENNE_PRIME;
        let a: u64 = 16807; // bits 14, 8, 7, 5, 2, 1, 0

        // We are computing
        //        seed = (seed * a) % m ,    where m = MERSENNE_PRIME
        //
        // seed must not be zero or m, or else all subsequent computed values
        // will be zero or m respectively. For all other values, seed will end
        // up cycling through every number in [1,M-1]
        let product: u64 = self.seed.get() as u64 * a;

        // Compute (product % m) using the fact that ((x << 31) % m) == x
        self.seed
            .set((product >> 31) as u32 + ((product as u32) & m));

        if self.seed.get() > m {
            self.seed.set(self.seed.get() - m);
        }
        self.seed.get()
    }

    /// Returns a uniformly distributed value in the range `[0..n)`
    #[inline(always)]
    pub fn uniform(&self, n: u32) -> u32 { self.next() % n }

    /// Randomly returns true ~ "1/n" of the time. False otherwise.
    #[inline(always)]
    pub fn one_in(&self, n: u32) -> bool { self.next() % n == 0 }

    /// Skewed: this first pick "base" uniformly from range `[0, max_log]`,
    /// and then return "base" random bits. The effect is to pick a random
    /// number in the range `[0, 2^max_log)` with exponential bias towards
    /// smaller numbers.
    #[inline(always)]
    pub fn skewed(&self, max_log: u32) -> u32 {
        let r = 1 << self.uniform(max_log + 1);
        self.uniform(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random() {
        let mut rnd  = Random::new(0);
        assert_eq!(rnd.seed.get(), 1);
        rnd = Random::new(MERSENNE_PRIME);
        assert_eq!(rnd.seed.get(), 1);

        rnd = Random::new(3);
        assert!(rnd.one_in(50421));
        assert_eq!(rnd.uniform(10), 7);
        assert_eq!(rnd.skewed(2), 1);
    }
}
