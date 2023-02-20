/* 
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Copyright 2021 Robert D. French
 */
//! Round-Robin Counter
//!
//! This module provides a way to loop arbitrarily many times over a finite collection.


/// Iterate arbitrarily many times over a slice.
///
/// # Example
/// ```
/// use portunusd::counter;
///
/// let items = vec![1, 2, 3];
///
/// let mut iter = counter::RoundRobin::new(&items);
/// assert_eq!(iter.next(), &1);
/// assert_eq!(iter.next(), &2);
/// assert_eq!(iter.next(), &3);
/// assert_eq!(iter.next(), &1);
/// ```
pub struct RoundRobin<'slice, T> {
    slice: &'slice [T],
    counter: usize
}

impl<'slice, T> RoundRobin<'slice, T> {
    /// Create a new RoundRobin iterator from a slice.
    ///
    /// The resulting iterator will be able to loop over the finitely sized slice arbitrarily many
    /// times without becoming exhausted.
    pub fn new(slice: &'slice [T]) -> Self {
        let counter = slice.len();
        Self{ slice, counter }
    }

    /// Retrieve the next item from the RoundRobin
    ///
    /// You will need some condition other than exhaustion to exit the loop.
    pub fn next<'rr>(&'rr mut self) -> &'slice T {
        self.counter += 1;
        if self.counter >= self.slice.len() {
            self.counter = 0;
        }
        &self.slice[self.counter]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn loops_around() {
        let data = vec![1,2,3];
        let mut rr = RoundRobin::new(&data);
        assert_eq!(rr.next(), &1);
        assert_eq!(rr.next(), &2);
        assert_eq!(rr.next(), &3);
        assert_eq!(rr.next(), &1);
    }
}
