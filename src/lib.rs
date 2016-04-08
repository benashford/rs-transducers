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

/// The result of a transduction step
pub enum TransductionResult<T> {
    /// End, there is nothing more to come
    End,
    /// None, there is nothing now, but may be more
    None,
    /// Some, a value
    Some(T)
}

pub trait Transducer<I, O> {
    /// A transduction step
    fn accept(&mut self, value: Option<I>) -> TransductionResult<O>;
}

pub struct ComposedTransducer<AT, BT, B> {
    a: AT,
    b: BT,
    phantom: PhantomData<B>
}

impl<A, AT, B, BT, C> Transducer<A, C> for ComposedTransducer<AT, BT, B>
    where AT: Transducer<A, B>,
          BT: Transducer<B, C> {
    #[inline]
    fn accept(&mut self, value: Option<A>) -> TransductionResult<C> {
        match self.a.accept(value) {
            TransductionResult::End => self.b.accept(None),
            TransductionResult::None => TransductionResult::None,
            TransductionResult::Some(interim) => self.b.accept(Some(interim))
        }
    }
}

pub fn compose<A, AT, B, BT, C>(b: BT,
                                a: AT) -> ComposedTransducer<AT, BT, B>
    where AT: Transducer<A, B>,
          BT: Transducer<B, C> {
    ComposedTransducer {
        a: a,
        b: b,
        phantom: PhantomData
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::transducers;
    use super::applications::vec::{Drain, Ref};
    use super::applications::channels::transducing_channel;

    #[test]
    fn test_vec_ref() {
        let source = vec![1, 2, 3];
        let transducer = transducers::map(|x| x + 1);
        let result = source.transduce_ref(transducer);
        assert_eq!(vec![2, 3, 4], result);
    }

    #[test]
    fn test_vec_drain() {
        let source = vec![1, 2, 3, 4, 5];
        let transducer = transducers::filter(|x| x % 2 == 0);
        let result = source.transduce_drain(transducer);
        assert_eq!(vec![2, 4], result);
    }

    #[test]
    fn test_compose() {
        let source = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let add_five = transducers::map(|x| x + 5);
        let filter_even = transducers::filter(|x| x % 2 == 0);
        let combined = super::compose(filter_even, add_five);
        let result = source.transduce_ref(combined);
        assert_eq!(vec![6, 8, 10, 12, 14], result);
    }

    #[test]
    fn test_partition() {
        let source = vec![1, 2, 3, 4, 5, 6];
        let transducer = transducers::partition(2);
        let result = source.transduce_drain(transducer);
        let expected_result:Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_take() {
        let source = vec![1, 2, 3, 4, 5, 6, 7];
        let transducer = transducers::take(5);
        let result = source.transduce_drain(transducer);
        assert_eq!(vec![1, 2, 3, 4, 5], result);

        let source2 = vec![1, 2, 3, 4, 5, 6, 7];
        let transducer2 = super::compose(transducers::take(2),
                                         transducers::filter(|x| x % 2 == 0));
        let result = source2.transduce_drain(transducer2);
        assert_eq!(vec![2, 4], result);
    }

    #[test]
    fn test_channels() {
        let transducer = super::compose(transducers::partition_all(6),
                                        transducers::filter(|x| x % 2 == 0));
        let (mut tx, rx) = transducing_channel(transducer);
        thread::spawn(move|| {
            for i in 0..10 {
                tx.send(i).unwrap();
            }
        });
        assert_eq!(vec![0, 2, 4, 6, 8], rx.recv().unwrap());
    }
}
