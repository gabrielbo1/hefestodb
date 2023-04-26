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

use std::{cell::RefCell, mem, ptr, rc::Rc};

const K_BLOCK_SIZE: usize = 4096;

pub type ArenaRef = Rc<RefCell<Arena>>;

pub struct Arena {
    ptr: *mut u8,
    bytes_remaining: usize,
    memory_usage: i64,
    blocks: Vec<Vec<u8>>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            bytes_remaining: 0,
            memory_usage: 0,
            blocks: Vec::new(),
        }
    }

    /// Allocate a byte slice with length `bytes`.
    /// Return a unique refrence to the slice allocated.
    pub fn alloc(&mut self, bytes: usize) -> *mut u8 {
        assert!(bytes > 0);
        let bytes_remaning = self.bytes_remaining;
        if bytes <= bytes_remaning {
            assert!(!self.ptr.is_null());
            let result = self.ptr;
            unsafe {
                self.ptr = self.ptr.offset(bytes as isize);
                self.bytes_remaining -= bytes;
                return result;
            }
        }
        return self.alloc_fallback(bytes);
    }

    /// Allocate a byte slice with lenght `bytes` that is aligned to pointer
    /// address.
    /// Return a unique reference to the slice allocated.
    pub fn alloc_aliged(&mut self, bytes: usize) -> *mut u8 {
        let ptr_size = mem::size_of::<usize>();
        assert!(ptr_size <= 128);
        let align = if ptr_size > 8 { ptr_size } else { 8 };
        assert_eq!(align & (align - 1), 0);

        let (bytes_remaining, slop) = {
            let current_mod = self.ptr as usize & (align - 1);
            let slop = if current_mod == 0 {
                0
            } else {
              align - current_mod
            };
            (self.bytes_remaining, slop)
        };
        let needed = bytes + slop;
        let result : *mut u8;
        if needed <= bytes_remaining {
            unsafe {
                let p = self.ptr.offset(slop as isize);
                self.ptr = self.ptr.offset(needed as isize);
                self.bytes_remaining -= needed;
                result = p
            }
        } else {
            result = self.alloc_fallback(bytes);
        }
        assert_eq!(result as usize & (align - 1), 0);
        result
    }

    /// Return the memory usage for the memory pool, in number of bytes allocated.
    pub fn memory_usage(&self) -> i64 { self.memory_usage }

    fn alloc_fallback(&mut self, bytes: usize) -> *mut u8 {
        if bytes > K_BLOCK_SIZE / 4 {
            return self.alloc_new(bytes);
        }

        self.ptr = self.alloc_new(K_BLOCK_SIZE);
        self.bytes_remaining = K_BLOCK_SIZE;

        let result = self.ptr;
        unsafe {
            self.ptr = self.ptr.offset(bytes as isize);
            self.bytes_remaining -= bytes;
            result
        }
    }

    fn alloc_new(&mut self, bytes: usize) -> *mut u8 {
        let mut v = Vec::with_capacity(bytes);
        unsafe  {
            v.set_len(bytes);
            ptr::write_bytes(v.as_mut_ptr(), 0, bytes);
        }
        let result  = v.as_mut_ptr();
        self.blocks.push(v);
        let memory_usage: i64 = self.memory_usage + bytes as i64;
        self.memory_usage = memory_usage;
        unsafe { mem::transmute(result) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let arena = Arena::new();
        check_current_block(&arena, true, 0);
        assert_eq!(arena.memory_usage(), 0);
    }

    #[test]
    fn alloc_new() {
        let mut arena = Arena::new();

        let _ = arena.alloc_new(128);
        check_current_block(&arena, true, 0);
        assert_eq!(arena.memory_usage(), 128);

        let _ = arena.alloc_new(256);
        check_current_block(&arena, true, 0);
        assert_eq!(arena.memory_usage(), 256 + 128);
    }

    #[test]
    fn alloc_fallback() {
        let mut arena = Arena::new();
        let _ = arena.alloc_fallback(1025);
        check_current_block(&arena, true, 0);
        assert_eq!(arena.memory_usage(), 1025);

        let _ = arena.alloc_fallback(512);
        check_current_block(&arena, false, K_BLOCK_SIZE - 512);
        assert_eq!(arena.memory_usage(), 1025 + K_BLOCK_SIZE as i64);
    }

    #[test]
    fn alloc_aligned() {
        let mut arena = Arena::new();
        let ptr_size = ::std::mem::size_of::<usize>();
        assert!(ptr_size > 1);

        let _ = arena.alloc_fallback(1);
        let _ = arena.alloc_aliged(512);
        check_current_block(&arena, false, K_BLOCK_SIZE - 512 - ptr_size);
    }

    #[test]
    fn alloc() {
        let mut arena = Arena::new();

        let _ = arena.alloc(128);

        check_current_block(&arena, false, 3968); // 4096 - 128
        assert_eq!(arena.memory_usage(), 4096);

        let _ = arena.alloc(1024); //should allocate from existing block

        check_current_block(&arena, false, 2944); //3968 - 1024
        assert_eq!(arena.memory_usage(), 4096);

        let _ = arena.alloc(8192); //should allocate new block

        check_current_block(&arena, false, 2944);
        assert_eq!(arena.memory_usage(), 12288); //8192 + 4096

        let _ = arena.alloc(2048); // should allocate from existing block
        check_current_block(&arena, false, 896); // 2944 - 2048
        assert_eq!(arena.memory_usage(), 12288);

        let _ = arena.alloc(1024); // should allocate new block

        check_current_block(&arena, false, 3072); // 4096 - 1024
        assert_eq!(arena.memory_usage(), 16384); //12288 + 4096
    }

    #[test]
    fn arena_ref() {
       let arena_ref: ArenaRef = Rc::new(RefCell::new(Arena::new()));
        assert_eq!(arena_ref.as_ptr().is_null(), false);
        assert_eq!(arena_ref.borrow_mut().memory_usage(), 0);

        let _ = arena_ref.borrow_mut().alloc(128);
        assert_eq!(arena_ref.borrow_mut().memory_usage(), 4096); //Block allocated
    }

    fn check_current_block(arena: &Arena, is_null: bool, bytes: usize) {
        assert_eq!(arena.ptr.is_null(), is_null);
        assert_eq!(arena.bytes_remaining, bytes);
    }
}
