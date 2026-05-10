use std::ops::{Deref, DerefMut};

pub struct Array<T, const N: usize>(pub [T; N]);

pub struct ConstIterator<T, const N: usize> {
    values: [T; N],
    current: usize,
}

impl<T: Copy, const N: usize> const Iterator for ConstIterator<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= N {
            None
        } else {
            let next = self.values[self.current];
            self.current += 1;
            Some(next)
        }
    }
}

impl<T: Copy, const N: usize> const IntoIterator for Array<T, N> {
    type Item = T;

    type IntoIter = ConstIterator<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            values: self.0,
            current: 0,
        }
    }
}

impl<T, const N: usize> const Deref for Array<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> const DerefMut for Array<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
