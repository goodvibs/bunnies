//! Const-friendly array wrapper with iterator support.

use std::ops::{Deref, DerefMut};

/// A thin wrapper around `[T; N]` enabling const trait implementations.
///
/// Used throughout uglychild for lookup tables and fixed-size collections
/// that need to work in const contexts where standard library arrays
/// don't yet implement required traits.
pub struct Array<T, const N: usize>(pub [T; N]);

/// Const iterator over an [`Array`] by value.
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
    type IntoIter = ConstIterator<Self::Item, N>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            values: self.0,
            current: 0,
        }
    }
}

/// Const iterator over an [`Array`] by reference.
pub struct ConstRefIterator<'a, T, const N: usize> {
    values: &'a [T; N],
    current: usize,
}

impl<'a, T, const N: usize> const Iterator for ConstRefIterator<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= N {
            None
        } else {
            let next = &self.values[self.current];
            self.current += 1;
            Some(next)
        }
    }
}

impl<'a, T, const N: usize> const IntoIterator for &'a Array<T, N> {
    type Item = &'a T;
    type IntoIter = ConstRefIterator<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        ConstRefIterator {
            values: &self.0,
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
