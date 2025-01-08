#![warn(
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]
#![allow(clippy::single_match, rustdoc::bare_urls)]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![doc = include_str!("../README.md")]

//! StableVec does not shift elements upon deletion and has stable index
//! and does not invalide indexes upon that happening.
//!
//! We are not concerned about continuous memory since we are not iterating
//! or sorting our collection.
//!
//! This impl of StableVec re-uses idx unlike impl that keeps track of rotating
//! index within.

use slabbable::Slabbable;

use stable_vec::{StableVecFacade, core::BitVecCore};

/// Error types
#[derive(Debug, PartialEq)]
pub enum StableVecSlabError {
    /// At Capacity, not able to take more
    AtCapacity(usize),
    /// Invalid index referred to
    InvalidIndex(usize),
}

/// Holder
#[derive(Debug)]
pub struct StableVecSlab<Item> {
    inner: StableVecFacade<Item, BitVecCore<Item>>,
}

impl<Item> Slabbable<StableVecSlab<Item>, Item> for StableVecSlab<Item>
where
    Item: core::fmt::Debug + Clone,
{
    type Error = StableVecSlabError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: StableVecFacade::<Item, BitVecCore<Item>>::with_capacity(cap),
        })
    }
    /// See trait
    #[inline]
    fn take_next_with(&mut self, with: Item) -> Result<usize, Self::Error> {
        // StableVec re-allocators upon grow - we want stable addresses
//        if self.inner.capacity() < self.num_elements() {
//            return Err(StableVecSlabError::AtCapacity(self.inner.capacity()));
//        }
        Ok(self.inner.push(with))
    }
    /// See trait
    #[inline]
    fn mark_for_reuse(&mut self, slot: usize) -> Result<Item, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(StableVecSlabError::InvalidIndex(slot));
        }
        match self.inner.remove(slot) {
            Some(i) => Ok(i),
            None => Err(StableVecSlabError::InvalidIndex(slot)),
        }

    }
    /// See trait
    #[inline]
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(StableVecSlabError::InvalidIndex(slot));
        }
        Ok(self.inner.get(slot))
        
    }
    /// See trait
    #[inline]
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    /// See trait
    #[inline]
    fn remaining(&self) -> Option<usize> {
        let rem = self.inner.capacity() - self.inner.num_elements();
        match rem {
            0 => None,
            1_usize.. => Some(rem),
        }
    }
    /// See trait
    fn reap(&mut self) -> Option<usize> {
        // We don't support it
        None
    }
}
