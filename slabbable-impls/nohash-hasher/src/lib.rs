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

use hashbrown::HashMap as HashBrownMap;
use nohash_hasher::BuildNoHashHasher;
use slabbable::Slabbable;

/// Error types
#[derive(Debug, PartialEq)]
pub enum NoHashSlabError {
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
pub struct NoHashSlab<Item> {
    inner: HashBrownMap<usize, Item, BuildNoHashHasher<usize>>,
    // wraps
    cur: usize,
    // wraps
    rev: usize,
}

impl<Item> NoHashSlab<Item> {
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

impl<Item> Slabbable<NoHashSlab<Item>, Item> for NoHashSlab<Item>
where
    Item: core::fmt::Debug + Clone,
{
    type Error = NoHashSlabError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        let inner: HashBrownMap<usize, Item, BuildNoHashHasher<usize>> =
            HashBrownMap::<usize, Item, BuildNoHashHasher<usize>>::with_capacity_and_hasher(
                cap,
                BuildNoHashHasher::default(),
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
            return Err(NoHashSlabError::AtCapacity(self.inner.capacity()));
        }
        let slot = self._take_next_cur();
        match self.inner.try_insert(slot, with) {
            Ok(_) => Ok(slot),
            _ => Err(NoHashSlabError::BugAlreadyOccupied),
        }
    }
    /// See trait
    #[inline]
    fn mark_for_reuse(&mut self, slot: usize) -> Result<Item, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(NoHashSlabError::InvalidIndex(slot));
        }
        match self.inner.remove(&slot) {
            Some(i) => Ok(i),
            None => Err(NoHashSlabError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(NoHashSlabError::InvalidIndex(slot));
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
