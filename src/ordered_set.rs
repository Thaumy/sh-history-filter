use std::collections::BTreeMap;
use std::mem;
use std::mem::MaybeUninit;

pub struct OrderedBTreeSet<T> {
    i: usize,
    map: BTreeMap<T, usize>,
}

impl<T> Default for OrderedBTreeSet<T> {
    fn default() -> Self {
        Self {
            i: 0,
            map: BTreeMap::new(),
        }
    }
}

impl<T> OrderedBTreeSet<T>
where
    T: Ord,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, val: T) {
        if self.map.contains_key(&val) {
            return;
        }
        self.map.insert(val, self.i);
        self.i += 1;
    }

    pub fn into_vec(self) -> Vec<T> {
        let len = self.map.len();
        let mut vec = Vec::with_capacity(len);
        unsafe {
            vec.set_len(len);
        }
        self.map.into_iter().for_each(|(val, i)| {
            vec[i] = MaybeUninit::new(val);
        });
        unsafe { mem::transmute(vec) }
    }

    pub fn into_inner(self) -> BTreeMap<T, usize> {
        self.map
    }
}

#[test]
fn test() {
    let mut ohs = OrderedBTreeSet::new();
    ohs.insert("a"); // a
    ohs.insert("b"); // b
    ohs.insert("c"); // c
    ohs.insert("a");
    ohs.insert("c");
    ohs.insert("d"); // d
    let vec = ohs.into_vec();
    assert_eq!(vec, vec!["a", "b", "c", "d"]);
}
