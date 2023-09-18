use std::collections::HashMap;
use std::hash::Hash;
use std::mem;
use std::mem::MaybeUninit;

pub struct OrderedHashSet<T> {
    i: usize,
    hash_map: HashMap<T, usize>,
}

impl<T> Default for OrderedHashSet<T> {
    fn default() -> Self {
        OrderedHashSet { i: 0, hash_map: HashMap::new() }
    }
}

impl<T> OrderedHashSet<T> where T: Eq + Hash {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, val: T) {
        if self.hash_map.contains_key(&val) {
            return;
        }
        self.hash_map.insert(val, self.i);
        self.i += 1;
    }

    pub fn into_vec(self) -> Vec<T> {
        let len = self.hash_map.len();
        let mut vec = Vec::with_capacity(len);
        unsafe {
            vec.set_len(len);
        }
        self.hash_map.into_iter().for_each(|(val, i)| {
            vec[i] = MaybeUninit::new(val);
        });
        unsafe {
            mem::transmute(vec)
        }
    }

    pub fn into_inner(self) -> HashMap<T, usize> {
        self.hash_map
    }
}

#[test]
fn test() {
    let mut ohs = OrderedHashSet::new();
    ohs.insert("a");// a
    ohs.insert("b");// b
    ohs.insert("c");// c
    ohs.insert("a");
    ohs.insert("c");
    ohs.insert("d");// d
    let vec = ohs.into_vec();
    assert_eq!(vec, vec!["a", "b", "c", "d"]);
}