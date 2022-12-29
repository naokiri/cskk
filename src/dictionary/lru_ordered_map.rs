use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct LruEntry<K, V> {
    // FIXME: MaybeuninitのほうがOption分allocateしなくてよいうえにIterがわかりやすくなるか？ リファクタリング候補
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
/// LruOrderedMapのイテレータ
///
/// This `struct` is created by the [`iter_lru`], [`iter_ordered`]? method on [`LruOrderedMap`]
///
pub struct Iter<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *const LruEntry<K, V>,
    end: *const LruEntry<K, V>,

    phantom: PhantomData<&'a K>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
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

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
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

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {}

///
/// LruOrderedMapのイテレータ
///
/// This `struct` is created by the [`iter_mut_lru`], [`iter_mut_ordered`]? method on [`LruOrderedMap`]
///
pub struct IterMut<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *mut LruEntry<K, V>,
    end: *mut LruEntry<K, V>,

    phantom: PhantomData<&'a K>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
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

impl<'a, K, V> DoubleEndedIterator for IterMut<'a, K, V> {
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

impl<'a, K, V> ExactSizeIterator for IterMut<'a, K, V> {}

///
///辞書順に保持しつつLeast Recently Used順にも探索できる構造
///
/// LRUCacheのような最大数制限なし
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

impl<K, V> LruOrderedMap<K, V>
where
    K: Eq + Hash + Ord,
{
    pub fn new() -> Self {
        let mut initital_map = LruOrderedMap {
            value_map: HashMap::new(),
            keys: Vec::new(),
            lru_head: Box::into_raw(Box::new(LruEntry::new_marker())),
            lru_tail: Box::into_raw(Box::new(LruEntry::new_marker())),
        };
        unsafe {
            (*initital_map.lru_head).next = initital_map.lru_tail;
            (*initital_map.lru_tail).prev = initital_map.lru_head;
        }

        initital_map
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
    pub fn remove(&mut self, k: K) -> Option<V> {
        let node_ref = self.value_map.remove(&k);
        match node_ref {
            Some(mut node_ref) => {
                let node_ptr: *mut LruEntry<K, V> = &mut *node_ref;
                self.detach(node_ptr);
                if let Ok(idx) = self.keys.binary_search_by(|x| (**x).cmp(&k)) {
                    self.keys.remove(idx);
                }
                node_ref.val
            }
            None => None,
        }
    }

    // pub fn iter_sorted() -> Iter<>
    pub fn iter_lru(&self) -> Iter<'_, K, V> {
        Iter {
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
}
