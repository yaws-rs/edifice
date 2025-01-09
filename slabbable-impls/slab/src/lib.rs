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

use slab::Slab;
use slabbable::Slabbable;

/// Error types
#[derive(Debug, PartialEq)]
pub enum SlabSlabError {
    /// At Capacity, not able to take more
    AtCapacity(usize),
    /// Invalid index referred to
    InvalidIndex(usize),
}

/// Holder
#[derive(Debug)]
pub struct SlabSlab<Item> {
    inner: Slab<Item>,
}

impl<Item> Slabbable<SlabSlab<Item>, Item> for SlabSlab<Item>
where
    Item: core::fmt::Debug + Clone,
{
    type Error = SlabSlabError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: Slab::with_capacity(cap),
        })
    }
    /// See trait
    #[inline]
    fn take_next_with(&mut self, with: Item) -> Result<usize, Self::Error> {
        // Slab re-allocators upon grow - we want stable addresses
        if self.inner.capacity() < self.inner.len() + 1 {
            return Err(SlabSlabError::AtCapacity(self.inner.capacity()));
        }
        Ok(self.inner.insert(with))
    }
    /// See trait
    #[inline]
    fn mark_for_reuse(&mut self, slot: usize) -> Result<Item, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(SlabSlabError::InvalidIndex(slot));
        }
        match self.inner.try_remove(slot) {
            Some(i) => Ok(i),
            None => Err(SlabSlabError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(SlabSlabError::InvalidIndex(slot));
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
        let rem = self.inner.capacity() - self.inner.len();
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
