pub mod transducers;
pub mod applications;

use std::marker::PhantomData;

pub trait Transducer<I, O> {
    fn accept(&mut self, value: I) -> Option<O>;

    #[inline]
    fn complete(self) -> Option<Vec<O>>
        where Self: Sized {

        None
    }
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
    fn accept(&mut self, value: A) -> Option<C> {
        match self.a.accept(value) {
            None => None,
            Some(interim) => self.b.accept(interim)
        }
    }

    #[inline]
    fn complete(mut self) -> Option<Vec<C>> {
        match self.a.complete() {
            None => self.b.complete(),
            Some(mut interim) => {
                let mut finish = Vec::with_capacity(interim.len() + 1);
                for v in interim.drain(..) {
                    match self.b.accept(v) {
                        None => (),
                        Some(v2) => finish.push(v2)
                    }
                }
                match self.b.complete() {
                    None => (),
                    Some(mut vs) => { finish.append(&mut vs); }
                }
                Some(finish)
            }
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
    use super::transducers;
    use super::applications::vec::{Drain, Ref};

    #[test]
    fn test_vec_ref() {
        let source = vec![1, 2, 3];
        let transducer = transducers::map(|x| x + 1);
        let result = source.trans_ref(transducer);
        assert_eq!(vec![2, 3, 4], result);
    }

    #[test]
    fn test_vec_drain() {
        let source = vec![1, 2, 3, 4, 5];
        let transducer = transducers::filter(|x| x % 2 == 0);
        let result = source.trans_drain(transducer);
        assert_eq!(vec![2, 4], result);
    }

    #[test]
    fn test_compose() {
        let source = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let add_five = transducers::map(|x| x + 5);
        let filter_even = transducers::filter(|x| x % 2 == 0);
        let combined = super::compose(filter_even, add_five);
        let result = source.trans_ref(combined);
        assert_eq!(vec![6, 8, 10, 12, 14], result);
    }

    #[test]
    fn test_partition() {
        let source = vec![1, 2, 3, 4, 5, 6];
        let transducer = transducers::partition(2);
        let result = source.trans_drain(transducer);
        let expected_result:Vec<Vec<usize>> = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        assert_eq!(expected_result, result);
    }
}
