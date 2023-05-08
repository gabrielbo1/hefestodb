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
    cell::UnsafeCell,
    sync::atomic::{compiler_fence, Ordering},
};

// -------------------------------------------------------------------
// Atomic types implemented using compile fences in Rust

pub struct AtomicPointer<T> {
    rep: UnsafeCell<*mut T>,
}

impl<T> AtomicPointer<T> {
    pub fn new(r: *mut T) -> Self {
        Self {
            rep: UnsafeCell::new(r),
        }
    }

    #[inline(always)]
    pub fn no_barrier_load(&self) -> *mut T { unsafe { *self.rep.get() } }

    #[inline(always)]
    pub fn no_barrier_store(&self, v: *mut T) {
        unsafe {
            *self.rep.get() = v;
        }
    }

    #[inline(always)]
    pub fn acquire_load(&self) -> *mut T {
        unsafe {
            let result = self.rep.get();
            compiler_fence(Ordering::Acquire);
            *result
        }
    }

    #[inline(always)]
    pub fn release_store(&mut self, v: *mut T) {
        compiler_fence(Ordering::Release);
        unsafe {
            *self.rep.get() = v;
        }
    }
}

unsafe impl<T> Sync for AtomicPointer<T> {}
unsafe impl<T> Send for AtomicPointer<T> {}

macro_rules! define_scalar_atomic {
    ($type_name:ident, $type:ty) => {
        pub struct $type_name {
            rep: UnsafeCell<$type>,
        }

        impl $type_name {
            pub fn new(r: $type) -> Self {
                Self {
                    rep: UnsafeCell::new(r),
                }
            }

            #[inline(always)]
            pub fn no_barrier_load(&self) -> $type { unsafe { *self.rep.get() } }

            #[inline(always)]
            pub fn no_barrier_store(&self, v: $type) {
                unsafe {
                    *self.rep.get() = v;
                }
            }

            #[inline(always)]
            pub fn acquire_load(&self) -> $type {
                unsafe {
                    let result = self.rep.get();
                    compiler_fence(Ordering::Acquire);
                    *result
                }
            }

            #[inline(always)]
            pub fn release_store(&self, v: $type) {
                compiler_fence(Ordering::Release);
                unsafe {
                    *self.rep.get() = v;
                }
            }
        }

        unsafe impl Send for $type_name {}
        unsafe impl Sync for $type_name {}
    };
}

define_scalar_atomic!(AtomicBool, bool);
define_scalar_atomic!(AtomicI8, i8);
define_scalar_atomic!(AtomicI16, i16);
define_scalar_atomic!(AtomicI32, i32);
define_scalar_atomic!(AtomicI64, i64);
define_scalar_atomic!(AtomicIsize, isize);
define_scalar_atomic!(AtomicU8, u8);
define_scalar_atomic!(AtomicU16, u16);
define_scalar_atomic!(AtomicU32, u32);
define_scalar_atomic!(AtomicU64, u64);
define_scalar_atomic!(AtomicUsize, usize);

