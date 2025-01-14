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

use slabbable::{ReservedSlot, Slabbable, SlabbableError};

#[derive(Debug)]
enum ReserveStatus<Item> {
    Reserved,
    Taken(Item),
}

use stable_vec::{core::BitVecCore, StableVecFacade};

/// Holder
#[derive(Debug)]
pub struct StableVecSlab<Item> {
    inner: StableVecFacade<ReserveStatus<Item>, BitVecCore<ReserveStatus<Item>>>,
}

impl<Item> Slabbable<StableVecSlab<Item>, Item> for StableVecSlab<Item>
where
    Item: core::fmt::Debug + Clone,
{
    type Error = SlabbableError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: StableVecFacade::<ReserveStatus<Item>, BitVecCore<ReserveStatus<Item>>>::with_capacity(cap),
        })
    }
    /// See trait
    #[inline]
    fn reserve_next(&mut self) -> Result<ReservedSlot, Self::Error> {
        // Slab re-allocators upon grow - we want stable addresses
        if self.inner.capacity() < self.inner.num_elements() + 1 {
            return Err(SlabbableError::AtCapacity(self.inner.capacity()));
        }
        let ins = self.inner.push(ReserveStatus::Reserved);
        Ok(ReservedSlot::issue(ins))
    }
    #[inline]
    fn take_reserved_with(
        &mut self,
        r_slot: ReservedSlot,
        with: Item,
    ) -> Result<usize, Self::Error> {
        let slot = r_slot.id();

        let v = match self.inner.get_mut(slot) {
            Some(v) => match v {
                ReserveStatus::Reserved => v,
                _ => return Err(SlabbableError::InvalidIndex(slot)),
            },
            _ => return Err(SlabbableError::InvalidIndex(slot)),
        };
        *v = ReserveStatus::Taken(with);
        Ok(slot)
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
        if slot > self.inner.capacity() {
            return Err(SlabbableError::InvalidIndex(slot));
        }
        match self.inner.remove(slot) {
            Some(ReserveStatus::Taken(i)) => Ok(i),
            _ => Err(SlabbableError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(SlabbableError::InvalidIndex(slot));
        }
        match self.inner.get(slot) {
            Some(ReserveStatus::Taken(itm_ref)) => Ok(Some(itm_ref)),
            _ => Err(SlabbableError::InvalidIndex(slot)),
        }
    }
    /// See trait
    #[inline]
    fn slot_get_mut(&mut self, slot: usize) -> Result<Option<&mut Item>, Self::Error> {
        if slot > self.inner.capacity() {
            return Err(SlabbableError::InvalidIndex(slot));
        }
        match self.inner.get_mut(slot) {
            Some(ReserveStatus::Taken(itm_ref)) => Ok(Some(itm_ref)),
            _ => Err(SlabbableError::InvalidIndex(slot)),
        }
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
