/*
 * Copyright 2016 rs-transducers developers
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */
pub mod transducers;
pub mod applications;

use std::marker::PhantomData;

/// Defines a reducing function from I to O with step errors of E
pub trait Reducing<I, O, E> {
    /// The type of each value after the reducing function
    type Item;

    /// Transducers must call the underlying `init`
    /// TODO: may not be required at all. Not currently used by any implementation
    fn init(&mut self) {}

    /// Each step, may fail
    /// TODO: the return type to contain an indicator of early termination
    fn step(&mut self, value: I) -> Result<(), E>;

    /// Transducers must call the underlying `complete`
    fn complete(&mut self) -> Result<(), E>;
}

/// Defines a transducer that transforms a reducing function RI into
/// a reducing function RO
pub trait Transducer<RI> {
    type RO;
    fn new(self, reducing_fn: RI) -> Self::RO;
}

/// Composed transducers
pub struct ComposedTransducer<AT, BT> {
    a: AT,
    b: BT
}

impl <RI, RT, RO, AT, BT> Transducer<RI> for ComposedTransducer<AT, BT>
    where AT: Transducer<RI, RO=RT>,
          BT: Transducer<RT, RO=RO> {
    type RO = RO;

    fn new(self, reducing_fn: RI) -> Self::RO {
        self.b.new(self.a.new(reducing_fn))
    }
}

pub fn compose<AT, BT>(a: AT, b: BT) -> ComposedTransducer<AT, BT> {
    ComposedTransducer {
        a: a,
        b: b
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::transducers;
    use super::applications::vec::{Into, Ref};
    use super::applications::iter::TransduceIter;
    use super::applications::channels::transducing_channel;

    #[test]
    fn test_vec_ref() {
        let source = vec![1, 2, 3];
        let transducer = transducers::map(|x| x + 1);
        let result = source.transduce_ref(transducer).unwrap();
        assert_eq!(vec![2, 3, 4], result);
    }

    /// A trivial function to test function references
    fn duplicator(i: isize) -> Vec<isize> {
        vec![i, i]
    }

    #[test]
    fn test_compose() {
        let source = vec![1, 2, 3];
        let ta = transducers::mapcat(duplicator);
        let tb = transducers::map(|x| x * 2);
        let transducer = super::compose(tb, ta);
        let result = source.transduce_into(transducer).unwrap();
        assert_eq!(vec![2, 2, 4, 4, 6, 6], result);
    }

    #[test]
    fn test_iterator() {
        let source = vec![1, 2, 3];
        let transducer = transducers::mapcat(duplicator);
        let result:Vec<isize> = source.into_iter().transduce(transducer).collect();
        assert_eq!(vec![1, 1, 2, 2, 3, 3], result);
    }

    #[test]
    fn test_filter() {
        let source = vec![1, 2, 3, 4, 5];
        let transducer = transducers::filter(|x| x % 2 == 0);
        let result = source.transduce_into(transducer).unwrap();
        assert_eq!(vec![2, 4], result);
    }

    #[test]
    fn test_partition() {
        {
            let source = vec![1, 2, 3, 4, 5, 6, 7];
            let transducer = transducers::partition(2);
            let result = source.transduce_into(transducer).unwrap();
            let expected_result:Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
            assert_eq!(expected_result, result);
        }
        {
            let source = vec![1, 2, 3, 4, 5, 6, 7];
            let transducer = transducers::partition_all(2);
            let result = source.transduce_into(transducer).unwrap();
            let expected_result:Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5, 6], vec![7]];
            assert_eq!(expected_result, result);
        }
    }

    #[test]
    fn test_take() {
        let source = vec![1, 2, 3, 4, 5, 6, 7];
        let transducer = transducers::take(5);
        let result = source.transduce_into(transducer).unwrap();
        assert_eq!(vec![1, 2, 3, 4, 5], result);

        let source2 = vec![1, 2, 3, 4, 5, 6, 7];
        let transducer2 = super::compose(transducers::take(2),
                                         transducers::filter(|x| x % 2 == 0));
        let result = source2.transduce_into(transducer2).unwrap();
        assert_eq!(vec![2, 4], result);
    }

    #[test]
    fn test_channels() {
        let transducer = transducers::map(|x| x + 1);
        let (mut tx, rx) = transducing_channel(transducer);
        thread::spawn(move|| {
            for i in 0..3 {
                tx.send(i).unwrap();
            }
            tx.close().unwrap();
        });
        assert_eq!(1, rx.recv().unwrap());
        assert_eq!(2, rx.recv().unwrap());
        assert_eq!(3, rx.recv().unwrap());
    }
}
