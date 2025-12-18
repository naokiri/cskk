use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LruEntry<K, V> {
    key: Option<Rc<K>>,
    val: Option<V>,
    prev: *mut LruEntry<K, V>,
    next: *mut LruEntry<K, V>,
}

impl<K, V> LruEntry<K, V> {
    pub fn new_marker() -> Self {
        Self {
            key: None,
            val: None,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }

    pub fn new(k: Rc<K>, v: V) -> Self {
        Self {
            key: Some(k),
            val: Some(v),
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}

///
/// LruOrderedMapのリストのイテレータ
///
/// This `struct` is created by the [`LruOrderedMap::iter_lru`] method
///
pub struct LinkedListIter<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *const LruEntry<K, V>,
    end: *const LruEntry<K, V>,

    phantom: PhantomData<&'a K>,
}

impl<'a, K, V> Iterator for LinkedListIter<'a, K, V> {
    type Item = (Option<&'a Rc<K>>, Option<&'a V>);

    fn next(&mut self) -> Option<(Option<&'a Rc<K>>, Option<&'a V>)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*self.ptr).key };
        let val = unsafe { &(*self.ptr).val };

        self.len -= 1;
        self.ptr = unsafe { (*self.ptr).next };

        Some((key.as_ref(), val.as_ref()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize {
        self.len
    }
}

impl<'a, K, V> DoubleEndedIterator for LinkedListIter<'a, K, V> {
    fn next_back(&mut self) -> Option<(Option<&'a Rc<K>>, Option<&'a V>)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*self.ptr).key };
        let val = unsafe { &(*self.ptr).val };

        self.len -= 1;
        self.end = unsafe { (*self.end).prev };

        Some((key.as_ref(), val.as_ref()))
    }
}

impl<'a, K, V> ExactSizeIterator for LinkedListIter<'a, K, V> {}

///
/// LruOrderedMapのリストのイテレータ
///
/// This `struct` is designed to be created by the [`LruOrderedMap::iter_mut_lru`] method
/// But not exposed as we didn't need in the cskk.
///
pub struct LinkedListIterMut<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *mut LruEntry<K, V>,
    end: *mut LruEntry<K, V>,

    phantom: PhantomData<&'a K>,
}

impl<'a, K, V> Iterator for LinkedListIterMut<'a, K, V> {
    type Item = (Option<&'a Rc<K>>, Option<&'a mut V>);

    fn next(&mut self) -> Option<(Option<&'a Rc<K>>, Option<&'a mut V>)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*self.ptr).key };
        let val = unsafe { &mut (*self.ptr).val };

        self.len -= 1;
        self.ptr = unsafe { (*self.ptr).next };

        Some((key.as_ref(), val.as_mut()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize {
        self.len
    }
}

impl<'a, K, V> DoubleEndedIterator for LinkedListIterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(Option<&'a Rc<K>>, Option<&'a mut V>)> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*self.ptr).key };
        let val = unsafe { &mut (*self.ptr).val };

        self.len -= 1;
        self.end = unsafe { (*self.end).prev };

        Some((key.as_ref(), val.as_mut()))
    }
}

impl<'a, K, V> ExactSizeIterator for LinkedListIterMut<'a, K, V> {}

///
/// LruOrderedMapのリストのイテレータ
///
/// This `struct` is created by the [`LruOrderedMap::iter_sorted`] method
///
pub struct SliceIter<'a, K: 'a, V: 'a>
where
    K: Eq + Hash + Ord,
{
    start: usize,
    end: usize,
    key_vec: &'a Vec<Rc<K>>,
    val_map: &'a HashMap<Rc<K>, Box<LruEntry<K, V>>>,
}

impl<'a, K, V> Iterator for SliceIter<'a, K, V>
where
    K: Eq + Hash + Ord,
{
    type Item = (Option<&'a Rc<K>>, Option<&'a V>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            let key = self.key_vec.get(self.start).unwrap();
            let entry = self
                .val_map
                .get(key)
                .expect("INVARIANT VIOLATION: Key in keys vector not found in value_map");
            self.start += 1;

            Some((entry.key.as_ref(), entry.val.as_ref()))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }

    fn count(self) -> usize {
        self.end - self.start
    }
}

impl<'a, K, V> DoubleEndedIterator for SliceIter<'a, K, V>
where
    K: Eq + Hash + Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start < self.end {
            self.end -= 1;
            let key = self.key_vec.get(self.end).unwrap();
            let entry = self
                .val_map
                .get(key)
                .expect("INVARIANT VIOLATION: Key in keys vector not found in value_map");

            Some((entry.key.as_ref(), entry.val.as_ref()))
        } else {
            None
        }
    }
}

impl<'a, K, V> ExactSizeIterator for SliceIter<'a, K, V> where K: Eq + Hash + Ord {}

///
/// 辞書順に保持しつつLeast Recently Used順にも探索できる構造
///
/// LRU Cacheのような最大数制限は実装していない。
///
/// K = Stringの見出し V = DictEntry を想定
///
#[derive(Debug)]
pub struct LruOrderedMap<K, V>
where
    K: Eq + Hash + Ord,
{
    // keys in vec to get the ordered entries
    keys: Vec<Rc<K>>,
    /// head and tail nodes are marker entry which doesn't contain real value to facilitate inserting entries in double linked list
    lru_head: *mut LruEntry<K, V>,
    lru_tail: *mut LruEntry<K, V>,
    value_map: HashMap<Rc<K>, Box<LruEntry<K, V>>>,
}

impl<K, V> Default for LruOrderedMap<K, V>
where
    K: Eq + Hash + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for LruOrderedMap<K, V>
where
    K: Eq + Hash + Ord,
{
    fn drop(&mut self) {
        // Clear the keys vector first to drop all Rc references
        self.keys.clear();

        // Clear the value_map to drop all entries
        self.value_map.clear();

        // Clean up the head marker node
        if !self.lru_head.is_null() {
            unsafe {
                // Take ownership of the raw pointer and drop it
                drop(Box::from_raw(self.lru_head));
            }
        }

        // Clean up the tail marker node
        if !self.lru_tail.is_null() {
            unsafe {
                // Take ownership of the raw pointer and drop it
                drop(Box::from_raw(self.lru_tail));
            }
        }

        // Set the pointers to null to prevent double-free if drop is somehow called again
        self.lru_head = ptr::null_mut();
        self.lru_tail = ptr::null_mut();
    }
}

impl<K, V> LruOrderedMap<K, V>
where
    K: Eq + Hash + Ord,
{
    pub fn new() -> Self {
        let initial_map = LruOrderedMap {
            value_map: HashMap::new(),
            keys: Vec::new(),
            lru_head: Box::into_raw(Box::new(LruEntry::new_marker())),
            lru_tail: Box::into_raw(Box::new(LruEntry::new_marker())),
        };
        unsafe {
            (*initial_map.lru_head).next = initial_map.lru_tail;
            (*initial_map.lru_tail).prev = initial_map.lru_head;
        }

        initial_map
    }

    ///
    /// kが存在しない場合は(k,v)を最も最近のエントリとして入れる。
    /// kが存在した場合は既存の(k,old_v)を(k,v)に置き換えて最も最近のエントリにする。
    pub fn push(&mut self, k: K, v: V) {
        let node_ref = self.value_map.get_mut(&k);
        match node_ref {
            Some(node_ref) => {
                let node_ptr: *mut LruEntry<K, V> = &mut **node_ref;
                unsafe {
                    (*node_ptr).val = Some(v);
                }
                self.detach(node_ptr);
                self.attach(node_ptr);
            }

            None => {
                let keyref: Rc<K> = Rc::from(k);
                let mut node = Box::new(LruEntry::new(Rc::clone(&keyref), v));
                let node_ptr: *mut LruEntry<K, V> = &mut *node;
                self.attach(node_ptr);
                let idx = self.keys.partition_point(|x| (*x).lt(&keyref));
                self.keys.insert(idx, Rc::clone(&keyref));
                self.value_map.insert(keyref, node);
            }
        }
    }

    /// When k exists, make that entry most recently used and returns mutable ref
    /// When k does not exist, returns None
    pub fn get_mut<KCmp>(&mut self, k: &KCmp) -> Option<&mut V>
    where
        KCmp: Eq + Hash + ?Sized,
        Rc<K>: Borrow<KCmp>,
    {
        let node_ref = self.value_map.get_mut(k);
        match node_ref {
            Some(node_ref) => {
                let node_ptr: *mut LruEntry<K, V> = &mut **node_ref;
                self.detach(node_ptr);
                self.attach(node_ptr);
                // detach,attachでmutable borrowしているのでselfのnode_refは使えない。node_ptrから返すためunsafeの必要あり。
                unsafe { (*node_ptr).val.as_mut() }
            }
            None => None,
        }
    }

    /// When k exists, make that entry most recently used and returns value ref
    /// When k does not exist, returns None
    #[allow(dead_code)]
    pub fn get(&mut self, k: &K) -> Option<&V> {
        let node_ref = self.value_map.get_mut(k);
        match node_ref {
            Some(node_ref) => {
                let node_ptr: *mut LruEntry<K, V> = &mut **node_ref;
                self.detach(node_ptr);
                self.attach(node_ptr);
                // detach,attachでmutable borrowしているのでselfのnode_refは使えない。node_ptrから返すためunsafeの必要あり。
                unsafe { (*node_ptr).val.as_ref() }
            }
            None => None,
        }
    }

    /// When k exists, returns value ref but don't change the lru
    /// When k does not exist, returns None
    pub fn peek(&self, k: &K) -> Option<&V> {
        let node_ref = self.value_map.get(k);
        match node_ref {
            Some(node_ref) => node_ref.val.as_ref(),
            None => None,
        }
    }

    /// When k exists, remove from the this map and returns the value.
    /// When k does not exist, returns None
    #[allow(dead_code)]
    pub fn remove(&mut self, k: K) -> Option<V> {
        let node_ref = self.value_map.remove(&k);
        match node_ref {
            Some(mut node_ref) => {
                let node_ptr: *mut LruEntry<K, V> = &mut *node_ref;
                self.detach(node_ptr);
                let idx = self
                    .keys
                    .binary_search_by(|x| (**x).cmp(&k))
                    .expect("INVARIANT VIOLATION: Key exists in value_map but not in keys vector");
                self.keys.remove(idx);
                node_ref.val
            }
            None => None,
        }
    }

    ///
    /// keyのソート昇順のIteratorを返す。
    ///
    pub fn iter_sorted(&self) -> SliceIter<'_, K, V> {
        SliceIter {
            start: 0,
            end: self.keys.len(),
            key_vec: &self.keys,
            val_map: &self.value_map,
        }
    }

    pub fn iter_lru(&self) -> LinkedListIter<'_, K, V> {
        LinkedListIter {
            len: self.keys.len(),
            ptr: unsafe { (*self.lru_head).next },
            end: unsafe { (*self.lru_tail).prev },
            phantom: PhantomData,
        }
    }

    fn detach(&mut self, ptr: *mut LruEntry<K, V>) {
        unsafe {
            (*(*ptr).prev).next = (*ptr).next;
            (*(*ptr).next).prev = (*ptr).prev;
        }
    }

    fn attach(&mut self, ptr: *mut LruEntry<K, V>) {
        unsafe {
            (*ptr).next = (*self.lru_head).next;
            (*ptr).prev = self.lru_head;
            (*self.lru_head).next = ptr;
            (*(*ptr).next).prev = ptr;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn init_lru_list_links() {
        let initial: LruOrderedMap<String, String> = LruOrderedMap::new();
        unsafe {
            assert_eq!(initial.lru_head, (*(*initial.lru_head).next).prev);
            assert_eq!(initial.lru_tail, (*(*initial.lru_tail).prev).next);
        }
    }

    #[test]
    pub fn remove() {
        let mut target = LruOrderedMap::new();
        target.push("a", "a");
        target.push("b", "b");
        target.push("c", "c");
        let result = target.remove("a");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!("a", result);
        assert_eq!(2, target.iter_lru().len);
    }

    #[test]
    pub fn push_get_lru_order() {
        let mut target = LruOrderedMap::new();
        target.push("a", "a");
        target.push("b", "b");
        target.push("c", "c");
        target.get(&"b");

        for (idx, (k, v)) in target.iter_lru().enumerate() {
            match idx {
                0 => {
                    assert_eq!("b", **k.unwrap());
                    assert_eq!("b", *v.unwrap());
                }
                1 => {
                    assert_eq!("c", **k.unwrap());
                    assert_eq!("c", *v.unwrap());
                }
                2 => {
                    assert_eq!("a", **k.unwrap());
                    assert_eq!("a", *v.unwrap());
                }
                _ => {
                    panic!();
                }
            }
        }
    }

    #[test]
    pub fn get_ord_order() {
        let mut target = LruOrderedMap::new();
        target.push("b", "b");
        target.push("a", "a");
        target.push("c", "c");

        for (idx, (k, v)) in target.iter_sorted().enumerate() {
            match idx {
                0 => {
                    assert_eq!("a", **k.unwrap());
                    assert_eq!("a", *v.unwrap());
                }
                1 => {
                    assert_eq!("b", **k.unwrap());
                    assert_eq!("b", *v.unwrap());
                }
                2 => {
                    assert_eq!("c", **k.unwrap());
                    assert_eq!("c", *v.unwrap());
                }
                _ => {
                    panic!();
                }
            }
        }
    }

    /// This test verifies that remove() detects corrupted state
    ///
    /// Verifies the fix for the bug where remove() silently failed when keys vector
    /// was out of sync with value_map. Now it properly panics with a clear error.
    ///
    /// This test manually corrupts the state to trigger the bug condition, then calls
    /// remove(). The fixed code should panic with "INVARIANT VIOLATION".
    #[test]
    #[should_panic(expected = "INVARIANT VIOLATION")]
    fn test_remove_detects_corrupted_state() {
        let mut target = LruOrderedMap::new();
        target.push("a", 1);
        target.push("b", 2);
        target.push("c", 3);

        // MANUALLY CORRUPT STATE: Remove "b" from keys vector but leave it in value_map
        // This simulates a bug where push() or another operation failed to keep them in sync
        if let Ok(idx) = target.keys.binary_search_by(|x| (**x).cmp("b")) {
            target.keys.remove(idx);
        }

        // At this point:
        // - value_map contains: {a: 1, b: 2, c: 3}
        // - keys contains: [a, c]  (missing b!)
        // - Data structure is CORRUPTED

        // Now try to remove "b"
        // Current behavior: remove() successfully removes from value_map,
        //                   binary_search fails to find in keys,
        //                   if let Ok silently continues
        //                   Result: value_map doesn't have b, keys still doesn't have b
        //                   TEST PASSES (bad!)
        //
        // Fixed behavior: remove() successfully removes from value_map,
        //                binary_search fails to find in keys,
        //                .expect() PANICS with "INVARIANT VIOLATION"
        //                TEST FAILS with expected panic (good!)

        target.remove("b");

        // If we get here without panic, the bug exists (silent corruption)
    }

    /// Test that iter_sorted().next() detects corrupted state
    ///
    /// Verifies the fix where the iterator now panics with a clear error
    /// when encountering corrupted state (key in keys vector but not in value_map).
    ///
    /// Previously: Silently returned None early (data loss)
    /// Now: Panics with "INVARIANT VIOLATION"
    #[test]
    #[should_panic(expected = "INVARIANT VIOLATION")]
    fn test_iter_sorted_next_detects_corrupted_state() {
        let mut target = LruOrderedMap::new();
        target.push("a", 1);
        target.push("b", 2);
        target.push("c", 3);

        // MANUALLY CORRUPT: Remove "b" from value_map but leave it in keys
        // This simulates a bug where remove() or another operation failed to keep them in sync
        target.value_map.remove(&"b");

        // State is now corrupted:
        // - keys = ["a", "b", "c"]
        // - value_map = {"a": 1, "c": 3}  (missing "b")

        let mut iter = target.iter_sorted();

        // First item should work
        let first = iter.next();
        assert!(first.is_some());

        // Second item is "b" which doesn't exist in value_map
        // Current behavior: iterator returns None, silently stops
        // Fixed behavior: should panic with "INVARIANT VIOLATION"
        let second = iter.next();

        // If we get here, the bug exists (silent failure)
        // The iterator returned None early instead of panicking
        assert!(
            second.is_some(),
            "Iterator should panic, not return None early"
        );
    }

    /// Test that iter_sorted().next_back() detects corrupted state
    ///
    /// Verifies the fix where the iterator now panics with a clear error
    /// when encountering corrupted state during backward iteration.
    ///
    /// Previously: Silently returned None early (data loss)
    /// Now: Panics with "INVARIANT VIOLATION"
    #[test]
    #[should_panic(expected = "INVARIANT VIOLATION")]
    fn test_iter_sorted_next_back_detects_corrupted_state() {
        let mut target = LruOrderedMap::new();
        target.push("a", 1);
        target.push("b", 2);
        target.push("c", 3);

        // MANUALLY CORRUPT: Remove "b" from value_map but leave it in keys
        target.value_map.remove(&"b");

        // State is now corrupted:
        // - keys = ["a", "b", "c"]
        // - value_map = {"a": 1, "c": 3}  (missing "b")

        let mut iter = target.iter_sorted();

        // First item from back should work
        let first = iter.next_back();
        assert!(first.is_some());

        // Second item from back is "b" which doesn't exist in value_map
        // Current behavior: iterator returns None, silently stops
        // Fixed behavior: should panic with "INVARIANT VIOLATION"
        let second = iter.next_back();

        // If we get here, the bug exists (silent failure)
        assert!(
            second.is_some(),
            "Iterator should panic, not return None early"
        );
    }
}
