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

    fn complete(&mut self) -> Result<(), E> {
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

    fn complete(&mut self) -> Result<(), E> {
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

pub struct FilterTransducer<F> {
     f: F
}

pub struct FilterReducer<R, F> {
    rf: R,
    t: FilterTransducer<F>
}

impl<F, RI> Transducer<RI> for FilterTransducer<F> {
    type RO = FilterReducer<RI, F>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        FilterReducer {
            rf: reducing_fn,
            t: self
        }
    }
}

impl<R, F, I, OF, E> Reducing<I, OF, E> for FilterReducer<R, F>
    where F: Fn(&I) -> bool,
          R: Reducing<I, OF, E> {
    type Item = I;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<(), E> {
        if (self.t.f)(&value) {
            try!(self.rf.step(value));
        }
        Ok(())
    }

    fn complete(&mut self) -> Result<(), E> {
        self.rf.complete()
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
    all: bool,
    t: PhantomData<T>
}

pub struct PartitionReducer<RF, T> {
    t: PartitionTransducer<T>,
    rf: RF,
    holder: Vec<T>
}

impl<RI, T> Transducer<RI> for PartitionTransducer<T> {
    type RO = PartitionReducer<RI, T>;

    fn new(self, reducing_fn: RI) -> Self::RO {
        let size = self.size;
        PartitionReducer {
            t: self,
            rf: reducing_fn,
            holder: Vec::with_capacity(size)
        }
    }
}

impl<R, I, OF, E> Reducing<I, OF, E> for PartitionReducer<R, I>
    where R: Reducing<Vec<I>, OF, E> {

    type Item = Vec<I>;

    fn init(&mut self) {
        self.rf.init();
    }

    #[inline]
    fn step(&mut self, value: I) -> Result<(), E> {
        self.holder.push(value);
        if self.holder.len() == self.t.size {
            let mut other_holder = Vec::with_capacity(self.t.size);
            mem::swap(&mut other_holder, &mut self.holder);
            try!(self.rf.step(other_holder));
        }
        Ok(())
    }

    fn complete(&mut self) -> Result<(), E> {
        if self.t.all {
            let mut other_holder = Vec::new();
            mem::swap(&mut other_holder, &mut self.holder);
            try!(self.rf.step(other_holder));
        }
        self.rf.complete()
    }
}

pub fn partition<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        all: false,
        t: PhantomData
    }
}

pub fn partition_all<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        all: true,
        t: PhantomData
    }
}

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
