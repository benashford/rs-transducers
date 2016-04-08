/*
 * Copyright 2016 rs-transducers developers
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */
use std::mem;

use super::{Transducer, TransductionResult};

pub struct MapTransducer<F> {
    f: F
}

impl<I, O, F> Transducer<I, O> for MapTransducer<F>
    where F: Fn(I) -> O {

    #[inline]
    fn accept(&mut self, value: Option<I>) -> TransductionResult<O> {
        match value {
            None => TransductionResult::End,
            Some(value) => TransductionResult::Some((self.f)(value))
        }
    }
}

pub fn map<F, I, O>(f: F) -> MapTransducer<F>
    where F: Fn(I) -> O {

    MapTransducer {
        f: f
    }
}

pub struct FilterTransducer<F> {
    f: F
}

impl<T, F> Transducer<T, T> for FilterTransducer<F>
    where F: Fn(&T) -> bool {

    #[inline]
    fn accept(&mut self, value: Option<T>) -> TransductionResult<T> {
        match value {
            None => TransductionResult::End,
            Some(value) => {
                if (self.f)(&value) {
                    TransductionResult::Some(value)
                } else {
                    TransductionResult::None
                }
            }
        }
    }
}

pub fn filter<F, T>(f: F) -> FilterTransducer<F>
    where F: Fn(&T) -> bool {

    FilterTransducer {
        f: f
    }
}

pub struct PartitionTransducer<T> {
    size: usize,
    holder: Vec<T>,
    all: bool
}

impl<T> Transducer<T, Vec<T>> for PartitionTransducer<T> {
    #[inline]
    fn accept(&mut self, value: Option<T>) -> TransductionResult<Vec<T>> {
        match value {
            None => {
                if self.all {
                    if self.holder.is_empty() {
                        TransductionResult::End
                    } else {
                        let pending = mem::replace(&mut self.holder, Vec::with_capacity(0));
                        TransductionResult::Some(pending)
                    }
                } else {
                    TransductionResult::End
                }
            },
            Some(value) => {
                self.holder.push(value);
                if self.holder.len() == self.size {
                    let pending = mem::replace(&mut self.holder, Vec::with_capacity(self.size));
                    TransductionResult::Some(pending)
                } else {
                    TransductionResult::None
                }
            }
        }
    }
}

pub fn partition<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        holder: Vec::with_capacity(num),
        all: false
    }
}

pub fn partition_all<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        holder: Vec::with_capacity(num),
        all: true
    }
}

pub struct TakeTransducer {
    size: usize,
    taken: usize
}

impl<T> Transducer<T, T> for TakeTransducer {
    #[inline]
    fn accept(&mut self, value: Option<T>) -> TransductionResult<T> {
        if self.taken == self.size {
            TransductionResult::End
        } else {
            match value {
                None => TransductionResult::End,
                Some(value) => {
                    self.taken += 1;
                    TransductionResult::Some(value)
                }
            }
        }
    }
}

pub fn take(num: usize) -> TakeTransducer {
    TakeTransducer {
        size: num,
        taken: 0
    }
}
