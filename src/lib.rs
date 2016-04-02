pub mod transducers;
pub mod applications;

pub trait Transducer<I, O> {
    fn accept(&self, value: I) -> Option<O>;

    #[inline]
    fn complete(&self) -> Option<Vec<O>> {
        None
    }
}

pub struct ComposedTransducer<'a, A: 'a, B: 'a, C: 'a> {
    a: &'a Transducer<A, B>,
    b: &'a Transducer<B, C>
}

impl<'a, A, B, C> Transducer<A, C> for ComposedTransducer<'a, A, B, C> {
    #[inline]
    fn accept(&self, value: A) -> Option<C> {
        match self.a.accept(value) {
            None => None,
            Some(interim) => self.b.accept(interim)
        }
    }

    #[inline]
    fn complete(&self) -> Option<Vec<C>> {
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

fn compose<'a, A, B, C>(a: &'a Transducer<A, B>,
                        b: &'a Transducer<B, C>) -> ComposedTransducer<'a, A, B, C> {
    ComposedTransducer {
        a: a,
        b: b
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
        let combined = super::compose(&add_five, &filter_even);
        let result = source.trans_ref(combined);
        assert_eq!(vec![6, 8, 10, 12, 14], result);
    }
}
