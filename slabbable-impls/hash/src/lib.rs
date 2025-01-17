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

#[cfg(not(slabbable_hasmap = "_somethingelse"))]
use hashbrown::HashMap as SelectedHashMap;

#[cfg(not(slabbable_hasher = "_somethingelse"))]
use nohash_hasher::BuildNoHashHasher as SelectedHasher;

use slabbable::{ReservedSlot, Slabbable, SlabbableError};

#[derive(Debug)]
enum ReserveStatus<Item> {
    Reserved,
    Taken(Item),
}

/// Holder
#[derive(Debug)]
pub struct HashSlab<Item> {
    inner: SelectedHashMap<usize, ReserveStatus<Item>, SelectedHasher<usize>>,
    // HashBrown seems to report wrong capacity() for guaranteed no-realloc
    // so we have to track our own to ensure it doesn't move things around.
    max_capacity: usize,
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
    type Error = SlabbableError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        let inner: SelectedHashMap<usize, ReserveStatus<Item>, SelectedHasher<usize>> =
            SelectedHashMap::<usize, ReserveStatus<Item>, SelectedHasher<usize>>::with_capacity_and_hasher(
                cap,
                SelectedHasher::default(),
            );
        Ok(Self {
            inner,
            max_capacity: cap,
            cur: 0,
            rev: 0,
        })
    }
    #[inline]
    fn reserve_next(&mut self) -> Result<ReservedSlot, Self::Error> {
        // Slab re-allocators upon grow - we want stable addresses
        if self.max_capacity < self.inner.len() + 1 {
            return Err(SlabbableError::AtCapacity(self.max_capacity));
        }
        let slot = self._take_next_cur();
        // TOOD: std hashmap try_insert is experimental
        match self.inner.try_insert(slot, ReserveStatus::Reserved) {
            Ok(_) => Ok(ReservedSlot::issue(slot)),
            _ => Err(SlabbableError::Bug(
                "Next entry by _take_next_cur() already occupied.",
            )),
        }
    }
    #[inline]
    fn take_reserved_with(&mut self, slot: ReservedSlot, with: Item) -> Result<usize, Self::Error> {
        let id = slot.id();

        match self.inner.insert(id, ReserveStatus::Taken(with)) {
            Some(v) => match v {
                ReserveStatus::Reserved => Ok(id),
                _ => Err(SlabbableError::Bug("Key was already occupied.")),
            },
            None => Err(SlabbableError::Bug("Key was not reserved correctly.")),
        }
    }
    /// See trait
    #[inline]
    fn take_next_with(&mut self, with: Item) -> Result<usize, Self::Error> {
        let reserved_slot = self.reserve_next()?;
        self.take_reserved_with(reserved_slot, with)
    }
    /// See trait
    #[inline]
    fn mark_for_reuse(&mut self, slot: usize) -> Result<Item, Self::Error> {
        match self.inner.remove(&slot) {
            Some(ReserveStatus::Taken(i)) => Ok(i),
            _ => Err(SlabbableError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_mut(&mut self, slot: usize) -> Result<Option<&mut Item>, Self::Error> {
        match self.inner.get_mut(&slot) {
            Some(ReserveStatus::Taken(itm_ref)) => Ok(Some(itm_ref)),
            _ => Err(SlabbableError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        match self.inner.get(&slot) {
            Some(ReserveStatus::Taken(itm_ref)) => Ok(Some(itm_ref)),
            _ => Err(SlabbableError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn capacity(&self) -> usize {
        self.max_capacity
    }
    /// See trait
    #[inline]
    fn remaining(&self) -> Option<usize> {
        let rem = self.max_capacity - self.inner.len();
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
