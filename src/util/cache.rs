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

use std::collections::HashMap;
use std::mem::swap;

// Implementation taken from here
// https://github.com/dermesser/leveldb-rs/blob/master/src/cache.rs

struct LRUNode<T> {
    next: Option<Box<LRUNode<T>>>,
    prev: Option<*mut LRUNode<T>>,
    data: Option<T>,
}

/// No clone, no copy! That asserts that an LRUHandle exists only once.
type LRUHandle<T> = *mut LRUNode<T>;

struct LRUList<T> {
    head: LRUNode<T>,
    count: usize,
}

/// This is likely unstable; more investigation is needed into correct behavior!
impl<T> LRUList<T> {
    fn new() -> LRUList<T> {
        LRUList {
            head: LRUNode {
                data: None,
                next: None,
                prev: None
            },
            count: 0,
        }
    }

    fn count(&self) -> usize {
        self.count
    }

    /// Inserts new element at front (least recently used element)
    fn insert(&mut self, elem: T) -> LRUHandle<T> {
        self.count += 1;

        //Not first element
        if self.head.next.is_some() {
            let mut new_node = Box::new(LRUNode {
                data: Some(elem),
                next: None,
                prev: Some(&mut self.head as *mut LRUNode<T>),
            });
            let new_pointer = new_node.as_mut() as *mut LRUNode<T>;

            // Set up the node after the new node
            self.head.next.as_mut().unwrap().prev = Some(new_pointer);
            // Replace head.next with Node and set the new node's to that
            new_node.next = self.head.next.take();
            self.head.next = Some(new_node);

            new_pointer
        } else {
            // First node; the only node right now is an empty head node
            let mut new_node = Box::new(LRUNode{
                data: Some(elem),
                next: None,
                prev: Some(&mut self.head as *mut LRUNode<T>),
            });

            let new_pointer = new_node.as_mut() as *mut LRUNode<T>;

            //Set tail
            self.head.prev = Some(new_pointer);
            //Set First
            self.head.next = Some(new_node);

            new_pointer
        }
    }

    fn remove_last(&mut self) -> Option<T> {
       if self.count() == 0 as usize {
           return None;
       }

       let mut lasto = unsafe { (*((*self.head.prev.unwrap()).prev.unwrap())).next.take() };

       assert!(lasto.is_some());
       if let Some(ref mut last) = lasto {
           assert!(last.prev.is_some());
           assert!(self.head.prev.is_some());
           self.head.prev = last.prev;
           self.count -= 1;
           (*last).data.take()
       } else {
           None
       }
    }

    fn remove(&mut self, node_handle: LRUHandle<T>) -> T {
        unsafe {
            let d = (*node_handle).data.take().unwrap();
            // Take ownership of node to be removed
            let mut current = (*(*node_handle).prev.unwrap()).next.take().unwrap();
            let prev = current.prev.unwrap();
            // Update previous node's sucessor
            if current.next.is_some() {
                // Update next node's predecessor.
                current.next.as_mut().unwrap().prev = current.prev.take();
            }
            (*prev).next = current.next.take();

            self.count -= 1;

            d
        }
    }
    
    /// Reinserts the reference node at front
    fn reinsert_front(&mut self, node_handle: LRUHandle<T>) {
        unsafe {
            let prevp = (*node_handle).prev.unwrap();

            // If not last node, update following node's prev
            if let Some(next) = (*node_handle).next.as_mut() {
                next.prev = Some(prevp);
            } else {
                // If last node, update head
                self.head.prev = Some(prevp);
            }

            // Swap this.next with prev.next. After that, this.next refers to this (!)
            swap(&mut (*prevp).next, &mut (*node_handle).next);
            // To reinsert at head, swap head's next with this.next
            swap(&mut (*node_handle).next, &mut self.head.next);
            // Update this prev reference to point to head

            //Update the second node's prev reference
            if let Some(ref mut newnext) = (*node_handle).next {
                (*node_handle).prev = newnext.prev;
                newnext.prev = Some(node_handle);
            } else {
                // Only one node, being the last one avoid head.prev pointing to head
                self.head.prev = Some(node_handle);
            }

            assert!(self.head.next.is_some());
            assert!(self.head.prev.is_some());
        }
    }

    fn _testing_head_ref(&self) -> Option<&T> {
        if let Some(ref first) = self.head.next {
            first.data.as_ref()
        } else {
            None
        }
    }
}

pub type CacheKey = [u8; 16];
pub type CacheID = u64;
type CacheEntry<T> = (T, LRUHandle<CacheKey>);

/// Implementation of 'ShardedLRUCache'.
/// See https://github.com/google/leveldb/blob/main/util/cache.cc
/// Based on a HashMap; the elements are linked in order to support the LRU ordering
pub struct Cache<T> {
    // note: CacheKeys (Vec<u8>) are duplicated between list and map. If this turns out to be a
    // performance bottleneck, another layer of indirection™ can solve this by mapping the key
    // to a numeric handle that keys both list and map.
    list: LRUList<CacheKey>,
    map: HashMap<CacheKey, CacheEntry<T>>,
    cap: usize,
    id: u64,
}

impl <T> Cache<T> {
    pub fn new(capacity: usize) ->  Cache<T> {
        assert!(capacity > 0);
        Cache {
            list: LRUList::new(),
            map: HashMap::with_capacity(1024),
            cap: capacity,
            id: 0,
        }
    }

    /// Returns an ID that is unique for this cache and that can be used to partition the cache
    /// among several users.
    pub fn new_cache_id(&mut self) -> CacheID {
        self.id += 1;
        self.id
    }

    /// How many the cache currently contains
    pub fn count(&self) -> usize {
        self.list.count()
    }

    /// The capacity of this cache
    pub fn cap(&self) -> usize {
        self.cap
    }

    /// Insert a new element into the cache. The returned `CacheHandle` can be used for futher
    /// operations on that element.
    /// If the capacity has been reached, the last recently used element is removed from
    /// the cache
    pub fn insert(&mut self, key: &CacheKey, elem: T) {
        if self.list.count() >= self.cap {
            if let Some(removed_key) = self.list.remove_last() {
                assert!(self.map.remove(&removed_key).is_some());
            } else {
                panic!("could not removed_last(); bug!");
            }
        }

        let lru_handle = self.list.insert(*key);
        self.map.insert(*key, (elem, lru_handle));
    }

    /// Retrieve an element from the cache.
    /// If the element has been preempted from the cache in the meantime, this returns None
    pub fn get(&mut self, key: &CacheKey) -> Option<&T> {
        match self.map.get(key) {
            None => None,
            Some(&(ref elem, ref lru_handle)) => {
                self.list.reinsert_front(*lru_handle);
                Some(elem)
            }
        }
    }

    /// Remove  an element from the cache (for invalidation)
    pub fn remove(&mut self, key: &CacheKey) -> Option<T> {
        match self.map.remove(key) {
            None => None,
            Some((elem, lru_handle)) => {
                self.list.remove(lru_handle);
                Some(elem)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::LRUList;
    use super::*;

    fn make_key(a: u8, b: u8, c: u8) -> CacheKey {
        [a, b, c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    }

    #[test]
    fn test_blockcache_cache_add_rm() {
        let mut cache = Cache::new(128);

        let h_123 = make_key(1, 2, 3);
        let h_521 = make_key(1, 2, 4);
        let h_372 = make_key(3, 4, 5);
        let h_332 = make_key(6, 3, 1);
        let h_899 = make_key(8, 2, 1);

        cache.insert(&h_123, 123);
        cache.insert(&h_332, 332);
        cache.insert(&h_521, 521);
        cache.insert(&h_372, 372);
        cache.insert(&h_899, 899);

        assert_eq!(cache.count(), 5);

        assert_eq!(cache.get(&h_123), Some(&123));
        assert_eq!(cache.get(&h_372), Some(&372));

        assert_eq!(cache.remove(&h_521), Some(521));
        assert_eq!(cache.get(&h_521), None);
        assert_eq!(cache.remove(&h_521), None);

        assert_eq!(cache.count(), 4);
        assert_eq!(cache.cap(), 128);
        assert_eq!(cache.new_cache_id(), 1 as u64);
    }

    #[test]
    fn test_blockcache_lru_1() {
        let mut lru = LRUList::<usize>::new();

        lru.insert(56);
        lru.insert(22);
        lru.insert(244);
        lru.insert(12);

        assert_eq!(lru.count(), 4);

        assert_eq!(Some(56), lru.remove_last());
        assert_eq!(Some(22), lru.remove_last());
        assert_eq!(Some(244), lru.remove_last());

        assert_eq!(lru.count(), 1);

        assert_eq!(Some(12), lru.remove_last());

        assert_eq!(lru.count(), 0);

        assert_eq!(None, lru.remove_last());
    }

    #[test]
    fn test_blockcache_lru_reinsert() {
        let mut lru = LRUList::<usize>::new();

        let handle1 = lru.insert(56);
        let handle2 = lru.insert(22);
        let handle3 = lru.insert(244);

        assert_eq!(lru._testing_head_ref().map(|r| (*r)).unwrap(), 244);

        lru.reinsert_front(handle1);

        assert_eq!(lru._testing_head_ref().map(|r| (*r)).unwrap(), 56);

        lru.reinsert_front(handle3);

        assert_eq!(lru._testing_head_ref().map(|r| (*r)).unwrap(), 244);

        lru.reinsert_front(handle2);

        assert_eq!(lru._testing_head_ref().map(|r| (*r)).unwrap(), 22);

        assert_eq!(lru.remove_last(), Some(56));
        assert_eq!(lru.remove_last(), Some(244));
        assert_eq!(lru.remove_last(), Some(22));
    }

    #[test]
    fn test_blockcache_lru_reinsert_2() {
        let mut lru = LRUList::<usize>::new();

        let handles = vec![
            lru.insert(0),
            lru.insert(1),
            lru.insert(2),
            lru.insert(3),
            lru.insert(4),
            lru.insert(5),
            lru.insert(6),
            lru.insert(7),
            lru.insert(8),
        ];

        for i in 0..9 {
            lru.reinsert_front(handles[i]);
            assert_eq!(lru._testing_head_ref().map(|x| *x), Some(i));
        }
    }

    #[test]
    fn test_blockcache_lru_edge_cases() {
        let mut lru = LRUList::<usize>::new();

        let handle = lru.insert(3);

        lru.reinsert_front(handle);
        assert_eq!(lru._testing_head_ref().map(|x| *x), Some(3));
        assert_eq!(lru.remove_last(), Some(3));
        assert_eq!(lru.remove_last(), None);
        assert_eq!(lru.remove_last(), None);
    }
}

