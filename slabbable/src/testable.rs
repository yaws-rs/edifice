//! Dummy Slabbable impl for testing.
//! This isn't a performance implementation but simple enough to quickly test / showcase the trait.

use super::{ReservedSlot, Slabbable, SlabbableError};

#[derive(Debug)]
pub struct TestableSlab<Item> {
    // Vec makes some guarantees for our purposes:
    //   > push and insert will never (re)allocate if the reported capacity is sufficient.
    // None = Vacant, Some(Item) = Occupied
    occupied: usize,
    inner: Vec<Option<Item>>,
    idx: Vec<Option<usize>>,
    reserved: Vec<usize>,
    // (wrapping) Current index
    cur: usize,
    // (wrapping) Revolution
    rev: usize,
}

impl<Item> TestableSlab<Item> {
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
    fn _find(&self, slot: usize) -> Option<usize> {
        for x in 0..self.inner.capacity() {
            if self.idx[x] == Some(slot) {
                return Some(x);
            }
        }
        return None;
    }
}

impl<Item> Slabbable<TestableSlab<Item>, Item> for TestableSlab<Item>
where
    Item: core::fmt::Debug + Clone,
{
    type Error = SlabbableError;
    /// See trait
    fn with_fixed_capacity(cap: usize) -> Result<Self, Self::Error> {
        Ok(Self {
            occupied: 0,
            inner: vec![None; cap],
            idx: vec![None; cap],
            reserved: vec![],
            cur: 0,
            rev: 0,
        })
    }
    /// See trait
    fn reserve_next(&mut self) -> Result<ReservedSlot, Self::Error> {
        if self.occupied == self.inner.capacity() {
            return Err(SlabbableError::AtCapacity(self.inner.capacity()));
        }
        self.occupied += 1;
        let slot = self._take_next_cur();
        self.reserved.push(slot);
        Ok(ReservedSlot::issue(slot))
    }
    fn take_reserved_with(&mut self, slot: ReservedSlot, with: Item) -> Result<usize, Self::Error> {
        let mut found: Option<usize> = None;
        for ri in 0..self.reserved.len() {
            if self.reserved[ri] == slot.id() {
                found = Some(slot.id());
                break;
            }
        }

        if found.is_none() {
            return Err(SlabbableError::InvalidIndex(slot.id()));
        }
        if let Some(found) = found {
            for x in 0..self.inner.capacity() {
                if self.inner[x].is_none() {
                    self.occupied += 1;
                    self.inner[x] = Some(with);
                    self.idx[x] = Some(found);
                    return Ok(x);
                }
            }
        }
        Err(SlabbableError::Bug("Capacity check"))
    }
    /// See trait - Let's just do simple non-optimised linear search for testing
    fn take_next_with(&mut self, with: Item) -> Result<usize, Self::Error> {
        if self.occupied == self.inner.capacity() {
            return Err(SlabbableError::AtCapacity(self.inner.capacity()));
        }
        for x in 0..self.inner.capacity() {
            if self.inner[x].is_none() {
                self.occupied += 1;
                self.inner[x] = Some(with);
                self.idx[x] = Some(self._take_next_cur());
                return Ok(x);
            }
        }
        Err(SlabbableError::Bug("Capacity check"))
    }
    /// See trait    
    fn mark_for_reuse(&mut self, slot: usize) -> Result<Item, Self::Error> {
        if let Some(x) = self._find(slot) {
            let ret = match &self.inner[x] {
                Some(item) => item.clone(),
                None => return Err(SlabbableError::Bug("Find-check is buggy")),
            };
            self.occupied -= 1;
            self.inner[x] = None;
            self.idx[x] = None;
            return Ok(ret);
        }
        Err(SlabbableError::InvalidIndex(slot))
    }
    /// See trait
    fn slot_get_mut(&mut self, slot: usize) -> Result<Option<&mut Item>, Self::Error> {
        if let Some(x) = self._find(slot) {
            if let Some(ref mut itm) = &mut self.inner[x] {
                return Ok(Some(itm));
            } else {
                return Err(SlabbableError::Bug("Get ref."));
            }
        }
        Ok(None)
    }
    /// See trait
    fn slot_get_ref(&self, slot: usize) -> Result<Option<&Item>, Self::Error> {
        if let Some(x) = self._find(slot) {
            if let Some(itm) = &self.inner[x] {
                return Ok(Some(&itm));
            } else {
                return Err(SlabbableError::Bug("Get ref."));
            }
        }
        Ok(None)
    }
    /// See trait    
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    /// See trait    
    fn remaining(&self) -> Option<usize> {
        let rem = self.inner.capacity() - self.occupied;
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
