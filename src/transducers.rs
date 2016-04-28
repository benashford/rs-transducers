/*
 * Copyright 2016 rs-transducers developers
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::mem;

use super::{Transducer, Reducing};

pub struct MapTransducer<F> {
    f: F
}

pub struct MapReducer<R, F> {
    rf: R,
    t: MapTransducer<F>
}

impl<F, RI> Transducer<RI> for MapTransducer<F> {
    type RO = MapReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        MapReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, O, OF, E> Reducing<I, OF, E> for MapReducer<R, F>
    where F: Fn(I) -> O,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<(), E> {
        self.rf.step((self.t.f)(value))
    }

    fn complete(self) -> OF {
        self.rf.complete()
    }
}

pub fn map<F, I, O>(f: F) -> MapTransducer<F>
    where F: Fn(I) -> O {

    MapTransducer {
        f: f
    }
}

pub struct MapcatTransducer<F> {
    f: F
}

pub struct MapcatReducer<R, F> {
    rf: R,
    t: MapcatTransducer<F>
}

impl<F, RI> Transducer<RI> for MapcatTransducer<F> {
    type RO = MapcatReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        MapcatReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, O, IO, OF, E> Reducing<I, OF, E> for MapcatReducer<R, F>
    where IO: IntoIterator<Item=O>,
          F: Fn(I) -> IO,
          R: Reducing<O, OF, E> {

    type Item = O;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<(), E> {
        for o in (self.t.f)(value) {
            try!(self.rf.step(o));
        }
        Ok(())
    }

    fn complete(self) -> OF {
        self.rf.complete()
    }
}

pub fn mapcat<F, I, O, IO>(f: F) -> MapcatTransducer<F>
    where IO: IntoIterator<Item=O>,
          F: Fn(I) -> IO {

    MapcatTransducer {
        f: f
    }
}

// pub struct FilterTransducer<F> {
//     f: F
// }

// impl<T, F> Transducer<T, T> for FilterTransducer<F>
//     where F: Fn(&T) -> bool {

//     #[inline]
//     fn accept(&mut self, value: Option<T>) -> TransductionResult<T> {
//         match value {
//             None => TransductionResult::End,
//             Some(value) => {
//                 if (self.f)(&value) {
//                     TransductionResult::Some(value)
//                 } else {
//                     TransductionResult::None
//                 }
//             }
//         }
//     }
// }

// pub fn filter<F, T>(f: F) -> FilterTransducer<F>
//     where F: Fn(&T) -> bool {

//     FilterTransducer {
//         f: f
//     }
// }

// pub struct PartitionTransducer<T> {
//     size: usize,
//     holder: Vec<T>,
//     all: bool
// }

// impl<T> Transducer<T, Vec<T>> for PartitionTransducer<T> {
//     #[inline]
//     fn accept(&mut self, value: Option<T>) -> TransductionResult<Vec<T>> {
//         match value {
//             None => {
//                 if self.all {
//                     if self.holder.is_empty() {
//                         TransductionResult::End
//                     } else {
//                         let pending = mem::replace(&mut self.holder, Vec::with_capacity(0));
//                         TransductionResult::Some(pending)
//                     }
//                 } else {
//                     TransductionResult::End
//                 }
//             },
//             Some(value) => {
//                 self.holder.push(value);
//                 if self.holder.len() == self.size {
//                     let pending = mem::replace(&mut self.holder, Vec::with_capacity(self.size));
//                     TransductionResult::Some(pending)
//                 } else {
//                     TransductionResult::None
//                 }
//             }
//         }
//     }
// }

// pub fn partition<T>(num: usize) -> PartitionTransducer<T> {
//     PartitionTransducer {
//         size: num,
//         holder: Vec::with_capacity(num),
//         all: false
//     }
// }

// pub fn partition_all<T>(num: usize) -> PartitionTransducer<T> {
//     PartitionTransducer {
//         size: num,
//         holder: Vec::with_capacity(num),
//         all: true
//     }
// }

// pub struct TakeTransducer {
//     size: usize,
//     taken: usize
// }

// impl<T> Transducer<T, T> for TakeTransducer {
//     #[inline]
//     fn accept(&mut self, value: Option<T>) -> TransductionResult<T> {
//         if self.taken == self.size {
//             TransductionResult::End
//         } else {
//             match value {
//                 None => TransductionResult::End,
//                 Some(value) => {
//                     self.taken += 1;
//                     TransductionResult::Some(value)
//                 }
//             }
//         }
//     }
// }

// pub fn take(num: usize) -> TakeTransducer {
//     TakeTransducer {
//         size: num,
//         taken: 0
//     }
// }

// pub struct DropTransducer {
//     size: usize,
//     dropped: usize
// }

// impl<T> Transducer<T, T> for DropTransducer {
//     #[inline]
//     fn accept(&mut self, value: Option<T>) -> TransductionResult<T> {
//         match value {
//             None => TransductionResult::End,
//             Some(value) => {
//                 if self.dropped == self.size {
//                     TransductionResult::Some(value)
//                 } else {
//                     self.dropped += 1;
//                     TransductionResult::None
//                 }
//             }
//         }
//     }
// }

// pub fn drop(num: usize) -> DropTransducer {
//     DropTransducer {
//         size: num,
//         dropped: 0
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::super::applications::vec::Drain;

//     #[test]
//     fn test_mapcat() {
//         let source = vec![1, 2, 3];
//         let transducer = super::mapcat(|x| {
//             let mut v = Vec::new();
//             for i in 0..x {
//                 v.push(i)
//             }
//             v
//         });
//         let result = source.transduce_drain(transducer);
//         assert_eq!(vec![0, 0, 1, 0, 1, 2], result);
//     }
// }
