// TODO: this is temporarily imported from substrate frame_support::storage because it is not included in V3.0.0 Remove once we upgrade to a version that includes it.

// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Traits, types and structs to support putting a bounded vector into storage, as a raw value, map
//! or a double map.

use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use core::{
    ops::{Deref, Index, IndexMut},
    slice::SliceIndex,
};
use frame_support::traits::Get;
use sp_std::prelude::*;
use sp_std::{convert::TryFrom, fmt, marker::PhantomData};

/// A bounded vector.
///
/// It has implementations for efficient append and length decoding, as with a normal `Vec<_>`, once
/// put into storage as a raw value, map or double-map.
///
/// As the name suggests, the length of the queue is always bounded. All internal operations ensure
/// this bound is respected.
#[derive(Encode)]
pub struct BoundedVec<T, S>(Vec<T>, PhantomData<S>);

/// A bounded slice.
///
/// Similar to a `BoundedVec`, but not owned and cannot be decoded.
#[derive(Encode)]
pub struct BoundedSlice<'a, T, S>(&'a [T], PhantomData<S>);

// `BoundedSlice`s encode to something which will always decode into a `BoundedVec` or a `Vec`.
impl<'a, T: Encode + Decode, S: Get<u32>> EncodeLike<BoundedVec<T, S>> for BoundedSlice<'a, T, S> {}
impl<'a, T: Encode + Decode, S: Get<u32>> EncodeLike<Vec<T>> for BoundedSlice<'a, T, S> {}

impl<'a, T, S: Get<u32>> TryFrom<&'a [T]> for BoundedSlice<'a, T, S> {
    type Error = ();
    fn try_from(t: &'a [T]) -> Result<Self, Self::Error> {
        if t.len() < S::get() as usize {
            Ok(BoundedSlice(t, PhantomData))
        } else {
            Err(())
        }
    }
}

impl<'a, T, S> From<BoundedSlice<'a, T, S>> for &'a [T] {
    fn from(t: BoundedSlice<'a, T, S>) -> Self {
        t.0
    }
}

impl<T: Decode, S: Get<u32>> Decode for BoundedVec<T, S> {
    fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
        let inner = Vec::<T>::decode(input)?;
        if inner.len() > S::get() as usize {
            return Err("BoundedVec exceeds its limit".into());
        }
        Ok(Self(inner, PhantomData))
    }

    fn skip<I: codec::Input>(input: &mut I) -> Result<(), codec::Error> {
        Vec::<T>::skip(input)
    }
}

// `BoundedVec`s encode to something which will always decode as a `Vec`.
impl<T: Encode + Decode, S: Get<u32>> EncodeLike<Vec<T>> for BoundedVec<T, S> {}

impl<T, S> BoundedVec<T, S> {
    /// Create `Self` from `t` without any checks.
    fn unchecked_from(t: Vec<T>) -> Self {
        Self(t, Default::default())
    }

    /// Consume self, and return the inner `Vec`. Henceforth, the `Vec<_>` can be altered in an
    /// arbitrary way. At some point, if the reverse conversion is required, `TryFrom<Vec<_>>` can
    /// be used.
    ///
    /// This is useful for cases if you need access to an internal API of the inner `Vec<_>` which
    /// is not provided by the wrapper `BoundedVec`.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Exactly the same semantics as [`Vec::remove`].
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> T {
        self.0.remove(index)
    }

    /// Exactly the same semantics as [`Vec::swap_remove`].
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn swap_remove(&mut self, index: usize) -> T {
        self.0.swap_remove(index)
    }

    /// Exactly the same semantics as [`Vec::retain`].
    pub fn retain<F: FnMut(&T) -> bool>(&mut self, f: F) {
        self.0.retain(f)
    }
}

impl<T, S: Get<u32>> From<BoundedVec<T, S>> for Vec<T> {
    fn from(x: BoundedVec<T, S>) -> Vec<T> {
        x.0
    }
}

impl<T, S: Get<u32>> BoundedVec<T, S> {
    /// Get the bound of the type in `usize`.
    pub fn bound() -> usize {
        S::get() as usize
    }

    /// Consumes self and mutates self via the given `mutate` function.
    ///
    /// If the outcome of mutation is within bounds, `Some(Self)` is returned. Else, `None` is
    /// returned.
    ///
    /// This is essentially a *consuming* shorthand [`Self::into_inner`] -> `...` ->
    /// [`Self::try_from`].
    pub fn try_mutate(mut self, mut mutate: impl FnMut(&mut Vec<T>)) -> Option<Self> {
        mutate(&mut self.0);
        (self.0.len() <= Self::bound()).then(move || self)
    }

    /// Exactly the same semantics as [`Vec::insert`], but returns an `Err` (and is a noop) if the
    /// new length of the vector exceeds `S`.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    pub fn try_insert(&mut self, index: usize, element: T) -> Result<(), ()> {
        if self.len() < Self::bound() {
            self.0.insert(index, element);
            Ok(())
        } else {
            Err(())
        }
    }

    /// Exactly the same semantics as [`Vec::push`], but returns an `Err` (and is a noop) if the
    /// new length of the vector exceeds `S`.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds isize::MAX bytes.
    pub fn try_push(&mut self, element: T) -> Result<(), ()> {
        if self.len() < Self::bound() {
            self.0.push(element);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T, S> Default for BoundedVec<T, S> {
    fn default() -> Self {
        // the bound cannot be below 0, which is satisfied by an empty vector
        Self::unchecked_from(Vec::default())
    }
}

#[cfg(feature = "std")]
impl<T, S> fmt::Debug for BoundedVec<T, S>
where
    T: fmt::Debug,
    S: Get<u32>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BoundedVec")
            .field(&self.0)
            .field(&Self::bound())
            .finish()
    }
}

impl<T, S> Clone for BoundedVec<T, S>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        // bound is retained
        Self::unchecked_from(self.0.clone())
    }
}

impl<T, S: Get<u32>> TryFrom<Vec<T>> for BoundedVec<T, S> {
    type Error = ();
    fn try_from(t: Vec<T>) -> Result<Self, Self::Error> {
        if t.len() <= Self::bound() {
            // explicit check just above
            Ok(Self::unchecked_from(t))
        } else {
            Err(())
        }
    }
}

// It is okay to give a non-mutable reference of the inner vec to anyone.
impl<T, S> AsRef<Vec<T>> for BoundedVec<T, S> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T, S> AsRef<[T]> for BoundedVec<T, S> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, S> AsMut<[T]> for BoundedVec<T, S> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

// will allow for immutable all operations of `Vec<T>` on `BoundedVec<T>`.
impl<T, S> Deref for BoundedVec<T, S> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Allows for indexing similar to a normal `Vec`. Can panic if out of bound.
impl<T, S, I> Index<I> for BoundedVec<T, S>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T, S, I> IndexMut<I> for BoundedVec<T, S>
where
    I: SliceIndex<[T]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<T, S> sp_std::iter::IntoIterator for BoundedVec<T, S> {
    type Item = T;
    type IntoIter = sp_std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, S> codec::DecodeLength for BoundedVec<T, S> {
    fn len(self_encoded: &[u8]) -> Result<usize, codec::Error> {
        // `BoundedVec<T, _>` stored just a `Vec<T>`, thus the length is at the beginning in
        // `Compact` form, and same implementation as `Vec<T>` can be used.
        <Vec<T> as codec::DecodeLength>::len(self_encoded)
    }
}

// NOTE: we could also implement this as:
// impl<T: Value, S1: Get<u32>, S2: Get<u32>> PartialEq<BoundedVec<T, S2>> for BoundedVec<T, S1>
// to allow comparison of bounded vectors with different bounds.
impl<T, S> PartialEq for BoundedVec<T, S>
where
    T: PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T: PartialEq, S: Get<u32>> PartialEq<Vec<T>> for BoundedVec<T, S> {
    fn eq(&self, other: &Vec<T>) -> bool {
        &self.0 == other
    }
}

impl<T, S> Eq for BoundedVec<T, S> where T: Eq {}

// impl<T, S> StorageDecodeLength for BoundedVec<T, S> {}

// impl<T, S: Get<u32>> StorageTryAppend<T> for BoundedVec<T, S> {
//     fn bound() -> usize {
//         S::get() as usize
//     }
// }

impl<T, S> MaxEncodedLen for BoundedVec<T, S>
where
    T: MaxEncodedLen,
    S: Get<u32>,
    BoundedVec<T, S>: Encode,
{
    fn max_encoded_len() -> usize {
        // BoundedVec<T, S> encodes like Vec<T> which encodes like [T], which is a compact u32
        // plus each item in the slice:
        // https://substrate.dev/rustdocs/v3.0.0/src/parity_scale_codec/codec.rs.html#798-808
        codec::Compact(S::get())
            .encoded_size()
            .saturating_add(Self::bound().saturating_mul(T::max_encoded_len()))
    }
}
