use std::cmp::Ordering;
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

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let self_ = std::mem::replace(self, IntoIter::Zero);
        match self_ {
            IntoIter::Zero => None,
            IntoIter::One(i) => Some(i),
            IntoIter::More(mut v) => {
                let res = v.next_back();
                *self = IntoIter::More(v);
                res
            }
        }
    }
}

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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self {
            IterMut::Zero => 0,
            IterMut::One(_) => 1,
            IterMut::More(i) => i.len(),
        };
        (len, Some(len))
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let self_ = std::mem::replace(self, IterMut::Zero);
        match self_ {
            IterMut::Zero => None,
            IterMut::One(i) => Some(i),
            IterMut::More(mut v) => {
                let res = v.next_back();
                *self = IterMut::More(v);
                res
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
    /// This function only converts the vec into movec. It does not check for the length and always
    /// returns a `MostlyOneVec::More` variant. To get an optimized movec, use `MostlyOneVec::from_optimized`.
    fn from(value: Vec<T>) -> Self {
        Self::More(value)
    }
}

impl<T> Index<usize> for MostlyOneVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match self {
            MostlyOneVec::Zero => panic!(),
            MostlyOneVec::One(i) => {
                if index == 0 {
                    i
                } else {
                    panic!()
                }
            }
            MostlyOneVec::More(vec) => &vec[index],
        }
    }
}

impl<T> IndexMut<usize> for MostlyOneVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            MostlyOneVec::Zero => panic!(),
            MostlyOneVec::One(i) => {
                if index == 0 {
                    i
                } else {
                    panic!()
                }
            }
            MostlyOneVec::More(vec) => &mut vec[index],
        }
    }
}

impl<T: PartialEq> PartialEq for MostlyOneVec<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MostlyOneVec::Zero, MostlyOneVec::Zero) => true,
            (MostlyOneVec::Zero, MostlyOneVec::More(v))
            | (MostlyOneVec::More(v), MostlyOneVec::Zero) => v.is_empty(),
            (MostlyOneVec::Zero, _) => false,
            (_, MostlyOneVec::Zero) => false,
            (MostlyOneVec::One(i1), MostlyOneVec::One(i2)) => i1 == i2,
            (MostlyOneVec::One(i), MostlyOneVec::More(v))
            | (MostlyOneVec::More(v), MostlyOneVec::One(i)) => v.len() == 1 && i == &v[0],
            (MostlyOneVec::More(v1), MostlyOneVec::More(v2)) => v1 == v2,
        }
    }
}

impl<T: Eq> MostlyOneVec<T> {
    pub fn exact_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MostlyOneVec::Zero, MostlyOneVec::Zero) => true,
            (MostlyOneVec::One(i1), MostlyOneVec::One(i2)) => i1 == i2,
            (MostlyOneVec::More(v1), MostlyOneVec::More(v2)) => v1 == v2,
            _ => false,
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

    pub fn from_optimized(v: Vec<T>) -> Self {
        match v.len().cmp(&1) {
            Ordering::Less => Self::Zero,
            Ordering::Equal => Self::One(v.into_iter().next().unwrap()),
            Ordering::Greater => Self::More(v),
        }
    }

    pub(crate) fn iter(&self) -> Iter<'_, T> {
        match self {
            MostlyOneVec::Zero => Iter::Zero,
            MostlyOneVec::One(i) => Iter::One(i),
            MostlyOneVec::More(v) => Iter::More(v.iter()),
        }
    }
    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, T> {
        match self {
            MostlyOneVec::Zero => IterMut::Zero,
            MostlyOneVec::One(i) => IterMut::One(i),
            MostlyOneVec::More(v) => IterMut::More(v.iter_mut()),
        }
    }
    pub(crate) fn truncate(&mut self, len: usize) {
        match self {
            MostlyOneVec::One(_) if len == 0 => {
                *self = MostlyOneVec::Zero;
            }
            MostlyOneVec::More(v) => v.truncate(len),
            MostlyOneVec::Zero | MostlyOneVec::One(_) => (),
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
            _ => panic!(),
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
            }
            MostlyOneVec::Zero | MostlyOneVec::One(_) => panic!(
                "swap_remove index (is {index}) should be < len (is {})",
                self_.len()
            ),
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

    pub(crate) fn into_map_enumerate<U>(
        self,
        mut f: impl FnMut((usize, T)) -> U,
    ) -> MostlyOneVec<U> {
        match self {
            MostlyOneVec::Zero => MostlyOneVec::Zero,
            MostlyOneVec::One(i) => MostlyOneVec::One(f((0, i))),
            MostlyOneVec::More(v) => {
                MostlyOneVec::More(v.into_iter().enumerate().map(f).collect::<Vec<U>>())
            }
        }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        match self {
            MostlyOneVec::One(i) if index == 0 => Some(i),
            MostlyOneVec::Zero | MostlyOneVec::One(_) => None,
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

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            MostlyOneVec::Zero => true,
            MostlyOneVec::One(_) => false,
            MostlyOneVec::More(v) => v.is_empty(),
        }
    }
    pub(crate) fn sort_by<F: Fn(&T, &T) -> Ordering>(&mut self, f: F) {
        if let Self::More(v) = self {
            v.sort_by(f);
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
                while v.len() < new_len {
                    v.push(f());
                }
                *self = MostlyOneVec::More(v);
            }
            MostlyOneVec::One(i) => match new_len.cmp(&1) {
                Ordering::Less => {}
                Ordering::Equal => *self = MostlyOneVec::One(i),
                Ordering::Greater => {
                    let mut v = Vec::with_capacity(new_len);
                    v.push(i);
                    while v.len() < new_len {
                        v.push(f());
                    }
                    *self = MostlyOneVec::More(v);
                }
            },
            MostlyOneVec::More(mut v) => {
                v.resize_with(new_len, f);
                *self = MostlyOneVec::More(v);
            }
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
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        let mut iter = iter.into_iter();
        let Some(first) = iter.next() else {
            return MostlyOneVec::Zero;
        };
        let Some(second) = iter.next() else {
            return MostlyOneVec::One(first);
        };
        let mut v = vec![first, second];
        for i in iter {
            v.push(i);
        }
        MostlyOneVec::More(v)
    }
}

#[macro_export]
macro_rules! movec {
    [$(,)?] => {$crate::backend::mostly_one_vec::MostlyOneVec::Zero};
    [$a:expr $(,)?] => {$crate::backend::mostly_one_vec::MostlyOneVec::One($a)};
    [$a:expr, $($b:expr),+ $(,)?] => {$crate::backend::mostly_one_vec::MostlyOneVec::More(vec![$a, $($b),+])};
}

#[cfg(test)]
mod tests {
    use crate::backend::mostly_one_vec::MostlyOneVec;
    use rstest::rstest;

    #[test]
    fn simple_usage() {
        // with_capacity:
        let v: MostlyOneVec<usize> = MostlyOneVec::with_capacity(0);
        assert!(matches!(v, MostlyOneVec::Zero));
        let v: MostlyOneVec<usize> = MostlyOneVec::with_capacity(1);
        assert!(matches!(v, MostlyOneVec::Zero));
        let v: MostlyOneVec<usize> = MostlyOneVec::with_capacity(2);
        assert!(matches!(v, MostlyOneVec::More(v) if v.capacity() >= 2));
        let v: MostlyOneVec<usize> = MostlyOneVec::with_capacity(3);
        assert!(matches!(v, MostlyOneVec::More(v) if v.capacity() >= 3));

        // reserve:
        let mut v: MostlyOneVec<usize> = movec!();
        v.reserve(0);
        assert!(matches!(v, MostlyOneVec::Zero));
        v.reserve(1);
        assert!(matches!(v, MostlyOneVec::Zero));
        v.reserve(2);
        assert!(matches!(v, MostlyOneVec::More(v) if v.capacity() >= 2));

        let mut v = movec!(5);
        v.reserve(0);
        assert!(matches!(v, MostlyOneVec::One(5)));
        v.reserve(1);
        assert!(matches!(v, MostlyOneVec::More(v) if v.capacity() >= 2));

        let mut v = movec!(1, 2);
        v.reserve(0);
        assert!(matches!(&v, MostlyOneVec::More(v) if v.capacity() >= 2));
        v.reserve(3);
        assert!(matches!(v, MostlyOneVec::More(v) if v.capacity() >= 5));
    }

    #[rstest]
    #[case(movec![], vec![], movec![], movec![])]
    #[case(movec![5], vec![5], movec![10], movec![5])]
    #[case(movec![5, 6], vec![5, 6], movec![10, 12], movec![5, 7])]
    #[case(movec![5, 6, 7], vec![5, 6, 7], movec![10, 12, 14], movec![5, 7, 9])]
    #[case(movec![1, 2, 3, 4, 5, 6, 7, 8, 9, 20], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 20], movec![2, 4, 6, 8, 10, 12, 14, 16, 18, 40], movec![1, 3, 5, 7, 9, 11, 13, 15, 17, 29])]
    fn simple_methods(
        #[case] movec: MostlyOneVec<usize>,
        #[case] real_vec: Vec<usize>,
        #[case] map_result: MostlyOneVec<usize>,
        #[case] enum_map_result: MostlyOneVec<usize>,
    ) {
        assert_eq!(format!("{:?}", real_vec), format!("{:?}", movec));
        assert_eq!(real_vec.len(), movec.len());
        assert_eq!(real_vec.is_empty(), movec.is_empty());
        for i in 0..20 {
            assert_eq!(real_vec.get(i), movec.get(i));
        }

        for (((i, j), k), l) in (&movec)
            .into_iter()
            .zip(&mut movec.clone())
            .zip(movec.clone().into_iter())
            .zip(&real_vec)
        {
            assert_eq!(i, j);
            assert_eq!(j, &k);
            assert_eq!(&k, l);
        }

        for (((i, j), k), l) in (&movec)
            .into_iter()
            .rev()
            .zip((&mut movec.clone()).into_iter().rev())
            .zip(movec.clone().into_iter().rev())
            .zip(real_vec.iter().rev())
        {
            assert_eq!(i, j);
            assert_eq!(j, &k);
            assert_eq!(&k, l);
        }

        assert_eq!(
            movec.clone().iter_mut().size_hint(),
            (movec.len(), Some(movec.len()))
        );
        assert_eq!(movec.iter().size_hint(), (movec.len(), Some(movec.len())));
        assert_eq!(
            movec.clone().into_iter().size_hint(),
            (movec.len(), Some(movec.len()))
        );

        assert_eq!(movec.map(|i| i * 2), map_result);
        assert_eq!(movec.clone().into_map(|i| i * 2), map_result);
        assert_eq!(movec.into_map_enumerate(|(i, j)| i + j), enum_map_result);
    }

    #[rstest]
    #[case(movec![], vec![])]
    #[case(movec![5], vec![5])]
    #[case(movec![5, 6], vec![5, 6])]
    #[case(movec![5, 6, 7], vec![5, 6, 7])]
    #[case(movec![1, 2, 3, 4, 5, 6, 7, 8, 9, 20], vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 20])]
    fn test_from(
        #[case] expected_from_optimized: MostlyOneVec<usize>,
        #[case] real_vec: Vec<usize>,
    ) {
        let movec_optimized = MostlyOneVec::from_optimized(real_vec.clone());
        let movec = MostlyOneVec::from(real_vec.clone());

        assert_eq!(format!("{:?}", real_vec), format!("{:?}", movec));
        assert_eq!(format!("{:?}", real_vec), format!("{:?}", movec_optimized));
        assert_eq!(real_vec.len(), movec.len());
        assert_eq!(real_vec.len(), movec_optimized.len());
        assert_eq!(real_vec.is_empty(), movec.is_empty());
        assert_eq!(real_vec.is_empty(), movec_optimized.is_empty());
        for i in 0..20 {
            assert_eq!(real_vec.get(i), movec.get(i));
            assert_eq!(real_vec.get(i), movec_optimized.get(i));
        }

        assert!(matches!(movec, MostlyOneVec::More(_)));
        assert!(matches!(movec_optimized, MostlyOneVec::More(_)) ^ (real_vec.len() < 2));

        assert!(movec_optimized.exact_eq(&expected_from_optimized));
        assert_eq!(movec, movec_optimized);
    }

    #[rstest]
    #[case(movec![], 0, movec![])]
    #[case(movec![], 3, movec![])]
    #[case(movec![1], 0, movec![])]
    #[case(movec![1], 1, movec![1])]
    #[case(movec![1], 2, movec![1])]
    #[case(movec![1, 2], 2, movec![1, 2])]
    #[case(movec![1, 2], 1, movec![1])]
    #[case(movec![1, 2], 3, movec![1, 2])]
    #[case(movec![1, 2, 3, 4, 5], 0, movec![])]
    #[case(movec![1, 2, 3, 4, 5], 1, movec![1])]
    #[case(movec![1, 2, 3, 4, 5], 2, movec![1, 2])]
    #[case(movec![1, 2, 3, 4, 5], 3, movec![1, 2, 3])]
    fn truncate(
        #[case] mut movec: MostlyOneVec<usize>,
        #[case] index: usize,
        #[case] result: MostlyOneVec<usize>,
    ) {
        movec.truncate(index);
        assert_eq!(movec, result);
    }
    #[rstest]
    #[case(movec![], 0, movec![])]
    #[case(movec![], 1, movec![88])]
    #[case(movec![], 2, movec![88, 88])]
    #[case(movec![], 3, movec![88, 88, 88])]
    #[case(movec![1], 0, movec![])]
    #[case(movec![1], 1, movec![1])]
    #[case(movec![1], 2, movec![1, 88])]
    #[case(movec![1], 3, movec![1, 88, 88])]
    #[case(movec![1, 2], 2, movec![1, 2])]
    #[case(movec![1, 2], 1, movec![1])]
    #[case(movec![1, 2], 0, movec![])]
    #[case(movec![1, 2], 3, movec![1, 2, 88])]
    #[case(movec![1, 2, 3, 4, 5], 0, movec![])]
    #[case(movec![1, 2, 3, 4, 5], 1, movec![1])]
    #[case(movec![1, 2, 3, 4, 5], 2, movec![1, 2])]
    #[case(movec![1, 2, 3, 4, 5], 6, movec![1, 2, 3, 4, 5, 88])]
    fn resize_with(
        #[case] mut movec: MostlyOneVec<usize>,
        #[case] new_len: usize,
        #[case] result: MostlyOneVec<usize>,
    ) {
        movec.resize_with(new_len, || 88);
        assert_eq!(movec, result);
    }

    #[rstest]
    #[case(movec!(), 0, movec!(88))]
    #[should_panic]
    #[case(movec!(), 1, movec!())]
    #[case(movec!(1), 0, movec!(88, 1))]
    #[case(movec!(1), 1, movec!(1, 88))]
    #[should_panic]
    #[case(movec!(1), 2, movec!())]
    #[case(movec!(1, 2), 0, movec!(88, 1, 2))]
    #[case(movec!(1, 2), 1, movec!(1, 88, 2))]
    #[case(movec!(1, 2), 2, movec!(1, 2, 88))]
    fn insert(
        #[case] mut movec: MostlyOneVec<usize>,
        #[case] idx: usize,
        #[case] expected: MostlyOneVec<usize>,
    ) {
        movec.insert(idx, 88);
        assert_eq!(movec, expected);
    }

    #[rstest]
    #[case(movec!())]
    #[case(MostlyOneVec::from(vec![]))]
    fn push(#[case] mut movec: MostlyOneVec<usize>) {
        movec.push(1);
        movec.push(2);
        movec.push(3);
        movec.push(4);
        movec.push(5);

        assert_eq!(movec[0], 1);
        assert_eq!(movec[1], 2);
        assert_eq!(movec[2], 3);
        assert_eq!(movec[3], 4);
        assert_eq!(movec[4], 5);
        assert!(movec.get(5).is_none());
    }

    #[rstest]
    #[case(movec!(), None)]
    #[case(movec!(1), Some(1))]
    #[case(movec!(1, 2), Some(2))]
    #[case(movec!(1, 2, 3, 4, 5, 6), Some(6))]
    #[case(MostlyOneVec::from(vec![]), None)]
    #[case(MostlyOneVec::from(vec![1]), Some(1))]
    fn last(#[case] mut movec: MostlyOneVec<usize>, #[case] expected: Option<usize>) {
        assert_eq!(movec.last().copied(), expected);

        if expected.is_none() {
            assert!(movec.last_mut().is_none());
        } else {
            *movec.last_mut().unwrap() = 8888;
            assert_eq!(movec.last(), Some(&8888));
        }
    }

    #[rstest]
    #[should_panic]
    #[case(movec!(), 0, movec!(32342342342435))]
    #[should_panic]
    #[case(movec!(), 1, movec!(32342342342435))]
    #[case(movec!(1), 0, movec!())]
    #[should_panic]
    #[case(movec!(1), 1, movec!(32342342342435))]
    #[case(movec!(1, 2), 0, movec!(2))]
    #[case(movec!(1, 2), 1, movec!(1))]
    #[should_panic]
    #[case(movec!(1, 2), 2, movec!(32342342342435))]
    #[case(movec!(1, 2, 3), 0, movec!(3, 2))]
    #[case(movec!(1, 2, 3), 1, movec!(1, 3))]
    #[case(movec!(1, 2, 3), 2, movec!(1, 2))]
    fn swap_remove(
        #[case] mut movec: MostlyOneVec<usize>,
        #[case] index: usize,
        #[case] expected: MostlyOneVec<usize>,
    ) {
        movec.swap_remove(index);
        assert_eq!(movec, expected);
    }

    #[test]
    fn test_not_eq() {
        let some_movecs = [
            movec!(),
            MostlyOneVec::from(vec![]),
            movec!(1),
            movec!(2),
            MostlyOneVec::from(vec![3]),
            MostlyOneVec::from(vec![4]),
            movec!(1, 2, 3),
            movec!(4, 5, 6, 7),
        ];

        for i in 0..some_movecs.len() {
            for j in 0..some_movecs.len() {
                if i == j {
                    continue;
                }
                if i + j == 1 {
                    continue;
                } // This means both of them are empty (but different variants)
                assert_ne!(some_movecs[i], some_movecs[j]);
            }
        }
    }

    #[rstest]
    #[case(MostlyOneVec::from(vec![]), movec!())]
    #[case(MostlyOneVec::from(vec![1]), movec!())]
    #[case(MostlyOneVec::from(vec![1, 2, 3]), movec!())]
    #[case(movec!(1, 2, 3), movec!(1, 2, 4))]
    #[case(movec!(1), movec!(2))]
    #[case(movec!(0), movec!(2))]
    fn test_not_exact_eq(#[case] a: MostlyOneVec<usize>, #[case] b: MostlyOneVec<usize>) {
        assert!(!a.exact_eq(&b));
    }

    #[rstest]
    #[should_panic]
    #[case(movec!(), 0, 1234)]
    #[should_panic]
    #[case(movec!(), 1, 1234)]
    #[case(movec!(1), 0, 1)]
    #[should_panic]
    #[case(movec!(1), 1, 1234)]
    #[case(movec!(1, 2), 0, 1)]
    #[case(movec!(1, 2), 1, 2)]
    #[should_panic]
    #[case(movec!(1, 2), 2, 1234)]
    fn test_index(
        #[case] movec: MostlyOneVec<usize>,
        #[case] index: usize,
        #[case] expected: usize,
    ) {
        assert_eq!(movec[index], expected);
    }
    #[rstest]
    #[should_panic]
    #[case(movec!(), 0)]
    #[should_panic]
    #[case(movec!(), 1)]
    #[case(movec!(1), 0)]
    #[should_panic]
    #[case(movec!(1), 1)]
    #[case(movec!(1, 2), 0)]
    #[case(movec!(1, 2), 1)]
    #[should_panic]
    #[case(movec!(1, 2), 2)]
    fn test_index_mut(#[case] mut movec: MostlyOneVec<usize>, #[case] index: usize) {
        movec[index] = 10;
        assert_eq!(movec[index], 10);
    }

    #[rstest]
    #[case(movec!(), movec!())]
    #[case(movec!(1), movec!(1))]
    #[case(movec!(5, 8, 3, 7, 4), movec!(3, 4, 5, 7, 8))]
    fn sort(#[case] mut movec: MostlyOneVec<usize>, #[case] expected: MostlyOneVec<usize>) {
        {
            let mut movec = movec.clone();
            movec.sort();
            assert_eq!(movec, expected);
        }
        movec.sort_by(|a, b| a.cmp(b));
        assert_eq!(movec, expected);
    }

    #[test]
    fn from_iterator() {
        for len in 0..4 {
            let vec = (0..len).collect();
            let movec = (0..len).collect();

            let expected = MostlyOneVec::from_optimized(vec);
            assert!(expected.exact_eq(&movec));
        }
    }
}
