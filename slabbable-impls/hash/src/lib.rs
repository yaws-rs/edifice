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

#[cfg(not(slabbable_hasmap = "std"))]
use hashbrown::HashMap as SelectedHashMap;

#[cfg(slabbable_hasmap = "std")]
use std::collections::HashMap as SelectedHashMap;

#[cfg(not(slabbable_hasher = "_somethingelse"))]
use nohash_hasher::BuildNoHashHasher as SelectedHasher;

use slabbable::Slabbable;

/// Error types
#[derive(Debug, PartialEq)]
pub enum HashSlabError {
    /// At Capacity, not able to take more
    AtCapacity(usize),
    /// Invalid index referred to
    InvalidIndex(usize),
    /// Entry already exists bug internal slot mechanism took it.
    /// This is a bug and should not happen.
    BugAlreadyOccupied,
}

/// Holder
#[derive(Debug)]
pub struct HashSlab<Item> {
    inner: SelectedHashMap<usize, Item, SelectedHasher<usize>>,
    // wraps
    cur: usize,
    // wraps
    rev: usize,
}

impl<Item> HashSlab<Item> {
    fn _take_next_cur(&mut self) -> usize {
        let spot = self.cur;
        if self.cur == usize::MAX {
            self.cur = 0;
            self.rev = match self.rev {
                usize::MAX => 0,
                _ => self.rev + 1,
            };
        } else {
            self.cur += 1;
        }
        spot
    }
}

impl<Item> Slabbable<HashSlab<Item>, Item> for HashSlab<Item>
where
    Item: core::fmt::Debug + Clone,
{
    type Error = HashSlabError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        let inner: SelectedHashMap<usize, Item, SelectedHasher<usize>> =
            SelectedHashMap::<usize, Item, SelectedHasher<usize>>::with_capacity_and_hasher(
                cap,
                SelectedHasher::default(),
            );
        Ok(Self {
            inner,
            cur: 0,
            rev: 0,
        })
    }
    /// See trait
    #[inline]
    fn take_next_with(&mut self, with: Item) -> Result<usize, Self::Error> {
        // Slab re-allocators upon grow - we want stable addresses
        if self.inner.capacity() < self.inner.len() + 1 {
            return Err(HashSlabError::AtCapacity(self.inner.capacity()));
        }
        let slot = self._take_next_cur();
        // TOOD: std hashmap try_insert is experimental
        match self.inner.try_insert(slot, with) {
            Ok(_) => Ok(slot),
            _ => Err(HashSlabError::BugAlreadyOccupied),
        }
    }
    /// See trait
    #[inline]
    fn mark_for_reuse(&mut self, slot: usize) -> Result<Item, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(HashSlabError::InvalidIndex(slot));
        }
        match self.inner.remove(&slot) {
            Some(i) => Ok(i),
            None => Err(HashSlabError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(HashSlabError::InvalidIndex(slot));
        }
        Ok(self.inner.get(&slot))
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
