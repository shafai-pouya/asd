use std::fmt::{Debug, Formatter};
use std::ops::{Index, IndexMut};

pub enum MostlyOneVec<T> {
    Zero,
    One(T),
    More(Vec<T>),
}

impl<T: Debug> Debug for MostlyOneVec<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MostlyOneVec::Zero => ([] as [T; 0]).fmt(f),
            MostlyOneVec::One(i) => [i].fmt(f),
            MostlyOneVec::More(v) => v.fmt(f),
        }
    }
}

pub enum Iter<'a, T> {
    Zero,
    One(&'a T),
    More(std::slice::Iter<'a, T>),
}
pub enum IterMut<'a, T> {
    Zero,
    One(&'a mut T),
    More(std::slice::IterMut<'a, T>),
}

pub enum IntoIter<T> {
    Zero,
    One(T),
    More(std::vec::IntoIter<T>),
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let self_ = std::mem::replace(self, IntoIter::Zero);
        match self_ {
            IntoIter::Zero => None,
            IntoIter::One(i) => Some(i),
            IntoIter::More(mut v) => {
                let to_return = v.next();
                *self = IntoIter::More(v);
                to_return
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            IntoIter::Zero => 0,
            IntoIter::One(_) => 1,
            IntoIter::More(i) => i.len(),
        };
        (len, Some(len))
    }
}

impl <T> ExactSizeIterator for IntoIter<T> {}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let self_ = std::mem::replace(self, Iter::Zero);
        match self_ {
            Iter::Zero => None,
            Iter::One(i) => Some(i),
            Iter::More(mut ii) => {
                let to_return = ii.next();
                *self = Iter::More(ii);
                to_return
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            Iter::Zero => 0,
            Iter::One(_) => 1,
            Iter::More(i) => i.len(),
        };
        (len, Some(len))
    }
}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let self_ = std::mem::replace(self, Iter::Zero);
        match self_ {
            Iter::Zero => None,
            Iter::One(i) => Some(i),
            Iter::More(mut ii) => {
                let to_return = ii.next_back();
                *self = Iter::More(ii);
                to_return
            }
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let self_ = std::mem::replace(self, IterMut::Zero);
        match self_ {
            IterMut::Zero => None,
            IterMut::One(i) => Some(i),
            IterMut::More(mut ii) => {
                let to_return = ii.next();
                *self = IterMut::More(ii);
                to_return
            }
        }
    }
}


impl<'a, T> IntoIterator for &'a mut MostlyOneVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> IntoIterator for &'a MostlyOneVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for MostlyOneVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            MostlyOneVec::Zero => IntoIter::Zero,
            MostlyOneVec::One(i) => IntoIter::One(i),
            MostlyOneVec::More(v) => IntoIter::More(v.into_iter()),
        }
    }
}

impl<T> From<Vec<T>> for MostlyOneVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self::More(value)
    }
}

impl<T> Index<usize> for MostlyOneVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            MostlyOneVec::Zero => None.unwrap(),
            MostlyOneVec::One(i) => {
                if index == 0 {
                    i
                } else {
                    None.unwrap()
                }
            }
            MostlyOneVec::More(vec) => &vec[index],
        }
    }
}

impl<T> IndexMut<usize> for MostlyOneVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            MostlyOneVec::Zero => None.unwrap(),
            MostlyOneVec::One(i) => {
                if index == 0 {
                    i
                } else {
                    None.unwrap()
                }
            }
            MostlyOneVec::More(vec) => &mut vec[index],
        }
    }
}

#[allow(dead_code)]
impl<T> MostlyOneVec<T> {
    pub(crate) fn len(&self) -> usize {
        match self {
            MostlyOneVec::Zero => 0,
            MostlyOneVec::One(_) => 1,
            MostlyOneVec::More(vec) => vec.len(),
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, T> {
        match self {
            MostlyOneVec::Zero => Iter::Zero,
            MostlyOneVec::One(i) => Iter::One(i),
            MostlyOneVec::More(v) => {
                Iter::More(v.iter())
            }
        }
    }
    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, T> {
        match self {
            MostlyOneVec::Zero => IterMut::Zero,
            MostlyOneVec::One(i) => IterMut::One(i),
            MostlyOneVec::More(v) => {
                IterMut::More(v.iter_mut())
            }
        }
    }
    pub(crate) fn truncate(&mut self, len: usize) {
        match self {
            MostlyOneVec::One(_) if len == 0 => {
                *self = MostlyOneVec::Zero;
            }
            MostlyOneVec::More(v) =>
                v.truncate(len),
            MostlyOneVec::Zero |
            MostlyOneVec::One(_) => (),
        }
    }
    pub(crate) fn push(&mut self, item: T) {
        let self_ = std::mem::replace(self, MostlyOneVec::Zero);
        match self_ {
            MostlyOneVec::Zero => *self = MostlyOneVec::One(item),
            MostlyOneVec::One(i) => *self = MostlyOneVec::More(vec![i, item]),
            MostlyOneVec::More(mut v) => {
                v.push(item);
                *self = MostlyOneVec::More(v);
            }
        }
    }
    pub(crate) fn insert(&mut self, index: usize, item: T) {
        let self_ = std::mem::replace(self, MostlyOneVec::Zero);
        match self_ {
            MostlyOneVec::Zero if index == 0 => {
                *self = MostlyOneVec::One(item);
            }
            MostlyOneVec::One(i) if index == 0 => {
                *self = MostlyOneVec::More(vec![item, i]);
            }
            MostlyOneVec::One(i) if index == 1 => {
                *self = MostlyOneVec::More(vec![i, item]);
            }
            MostlyOneVec::More(mut v) => {
                v.insert(index, item);
                *self = MostlyOneVec::More(v);
            }
            _ => None.unwrap(),
        }
    }
    pub(crate) fn with_capacity(cap: usize) -> Self {
        if cap < 2 {
            MostlyOneVec::Zero
        } else {
            MostlyOneVec::More(Vec::with_capacity(cap))
        }
    }
    pub(crate) fn swap_remove(&mut self, index: usize) -> T {
        let self_ = std::mem::replace(self, MostlyOneVec::Zero);
        match self_ {
            MostlyOneVec::One(i) if index == 0 => {
                *self = MostlyOneVec::Zero;
                i
            },
            MostlyOneVec::Zero |
            MostlyOneVec::One(_) => None.unwrap(),
            MostlyOneVec::More(mut v) => {
                let to_return = v.swap_remove(index);
                *self = MostlyOneVec::More(v);
                to_return
            }
        }
    }

    pub(crate) fn map<U>(&self, f: impl Fn(&T) -> U) -> MostlyOneVec<U> {
        match self {
            MostlyOneVec::Zero => MostlyOneVec::Zero,
            MostlyOneVec::One(i) => MostlyOneVec::One(f(i)),
            MostlyOneVec::More(v) => MostlyOneVec::More(v.iter().map(f).collect::<Vec<U>>()),
        }
    }

    pub(crate) fn into_map<U>(self, f: impl Fn(T) -> U) -> MostlyOneVec<U> {
        match self {
            MostlyOneVec::Zero => MostlyOneVec::Zero,
            MostlyOneVec::One(i) => MostlyOneVec::One(f(i)),
            MostlyOneVec::More(v) => MostlyOneVec::More(v.into_iter().map(f).collect::<Vec<U>>()),
        }
    }

    pub(crate) fn into_map_enumerate<U>(self, mut f: impl FnMut((usize, T)) -> U) -> MostlyOneVec<U> {
        match self {
            MostlyOneVec::Zero => MostlyOneVec::Zero,
            MostlyOneVec::One(i) => MostlyOneVec::One(f((0, i))),
            MostlyOneVec::More(v) => MostlyOneVec::More(v.into_iter().enumerate().map(f).collect::<Vec<U>>()),
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        match self {
            MostlyOneVec::One(i) if index == 0 => Some(i),
            MostlyOneVec::Zero |
            MostlyOneVec::One(_) => None,
            MostlyOneVec::More(v) => v.get(index),
        }
    }

    pub(crate) fn last(&self) -> Option<&T> {
        match self {
            MostlyOneVec::Zero => None,
            MostlyOneVec::One(i) => Some(i),
            MostlyOneVec::More(v) => v.last(),
        }
    }

    pub(crate) fn last_mut(&mut self) -> Option<&mut T> {
        match self {
            MostlyOneVec::Zero => None,
            MostlyOneVec::One(i) => Some(i),
            MostlyOneVec::More(v) => v.last_mut(),
        }
    }

    pub(crate) fn reserve(&mut self, size: usize) {
        let self_ = std::mem::replace(self, MostlyOneVec::Zero);
        match self_ {
            MostlyOneVec::Zero => {
                if size > 1 {
                    *self = MostlyOneVec::More(Vec::with_capacity(size));
                }
            }
            MostlyOneVec::One(i) => {
                if size > 0 {
                    let mut vec = Vec::with_capacity(size + 1);
                    vec.push(i);
                    *self = MostlyOneVec::More(vec);
                } else {
                    *self = MostlyOneVec::One(i);
                }
            }
            MostlyOneVec::More(mut v) => {
                v.reserve(size);
                *self = MostlyOneVec::More(v);
            }
        }
    }

    pub(crate) fn first_or_insert(&mut self, value: T) -> &T {
        if self.is_empty() {
            *self = MostlyOneVec::One(value);
        }
        &self[0]
    }

    pub(crate) fn first_or_insert_mut(&mut self, value: T) -> &mut T {
        if self.is_empty() {
            *self = MostlyOneVec::One(value);
        }
        &mut self[0]
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            MostlyOneVec::Zero => true,
            MostlyOneVec::One(_) => false,
            MostlyOneVec::More(v) => v.is_empty(),
        }
    }
}
impl<T: Ord> MostlyOneVec<T> {
    pub(crate) fn sort(&mut self) {
        if let Self::More(v) = self {
            v.sort();
        }
    }
}

#[allow(dead_code)]
impl<T> MostlyOneVec<T> {
    pub(crate) fn resize_with(&mut self, new_len: usize, f: impl Fn() -> T) {
        let self_ = std::mem::replace(self, MostlyOneVec::Zero);
        match self_ {
            MostlyOneVec::Zero => {
                let mut v = Vec::with_capacity(new_len);
                for _ in 0..new_len {
                    v.push(f());
                }
                *self = MostlyOneVec::More(v);
            }
            MostlyOneVec::One(i) => {
                let mut v = Vec::with_capacity(new_len);
                v.push(i);
                for _ in 1..new_len {
                    v.push(f());
                }
                *self = MostlyOneVec::More(v);
            }
            MostlyOneVec::More(mut v) => {
                v.resize_with(new_len, f);
                *self = MostlyOneVec::More(v);
            }
        }
    }
    pub(crate) fn zip_same_size_and_map<U, V, F: Fn((T, &U)) -> V>(self, other: &MostlyOneVec<U>, f: F) -> MostlyOneVec<V> {
        match (self, other) {
            (MostlyOneVec::Zero, MostlyOneVec::Zero) => MostlyOneVec::Zero,
            (MostlyOneVec::One(i), MostlyOneVec::One(j)) => MostlyOneVec::One(f((i, j))),
            (MostlyOneVec::More(v1), MostlyOneVec::More(v2)) => MostlyOneVec::More(v1.into_iter().zip(v2.into_iter()).map(f).collect::<Vec<_>>()),
            (MostlyOneVec::Zero, MostlyOneVec::One(_)) |
            (MostlyOneVec::Zero, MostlyOneVec::More(_)) |
            (MostlyOneVec::One(_), MostlyOneVec::Zero) |
            (MostlyOneVec::One(_), MostlyOneVec::More(_)) |
            (MostlyOneVec::More(_), MostlyOneVec::Zero) |
            (MostlyOneVec::More(_), MostlyOneVec::One(_)) => unreachable!(),
        }
    }
}

impl<T: Clone> Clone for MostlyOneVec<T> {
    fn clone(&self) -> Self {
        match self {
            MostlyOneVec::Zero => MostlyOneVec::Zero,
            MostlyOneVec::One(i) => MostlyOneVec::One(i.clone()),
            MostlyOneVec::More(v) => MostlyOneVec::More(v.clone()),
        }
    }
}

impl<T> FromIterator<T> for MostlyOneVec<T> {
    fn from_iter<U: IntoIterator<Item=T>>(iter: U) -> Self {
        let mut iter = iter.into_iter();
        let Some(first) = iter.next() else {
            return MostlyOneVec::Zero;
        };
        let Some(second) = iter.next() else {
            return MostlyOneVec::One(first);
        };
        let mut v = vec![first, second];
        while let Some(i) = iter.next() {
            v.push(i);
        }
        MostlyOneVec::More(v)
    }
}

#[macro_export]
macro_rules! movec {
    [] => {crate::backend::mostly_one_vec::MostlyOneVec::Zero};
    [$a:expr] => {crate::backend::mostly_one_vec::MostlyOneVec::One($a)};
    [$a:expr, $($b:expr),+] => {crate::backend::mostly_one_vec::MostlyOneVec::More(vec![$a, $($b),+])};
}
