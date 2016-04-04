pub mod vec {
    use ::{Transducer, TransductionResult};

    pub trait Ref {
        type Input;

        fn trans_ref<'a, T, O>(&'a self, transducer: T) -> Vec<O>
            where T: Transducer<&'a Self::Input, O>;
    }

    impl<X> Ref for Vec<X> {
        type Input = X;

        fn trans_ref<'a, T, O>(&'a self, mut transducer: T) -> Vec<O>
            where T: Transducer<&'a Self::Input, O> {

            let mut result = Vec::with_capacity(self.len());
            for val in self {
                match transducer.accept(Some(val)) {
                    TransductionResult::End => { return result; },
                    TransductionResult::None => (),
                    TransductionResult::Some(r) => { result.push(r); }
                }
            }
            loop {
                match transducer.accept(None) {
                    TransductionResult::End => { return result; },
                    TransductionResult::None => (),
                    TransductionResult::Some(r) => { result.push(r); }
                }
            }
        }
    }

    pub trait Drain {
        type Input;

        fn trans_drain<T, O>(mut self, transducer: T) -> Vec<O>
            where T: Transducer<Self::Input, O>;
    }

    impl<X> Drain for Vec<X> {
        type Input = X;

        fn trans_drain<T, O>(mut self, mut transducer: T) -> Vec<O>
            where T: Transducer<Self::Input, O> {

            let mut result = Vec::with_capacity(self.len());
            for val in self.drain(..) {
                match transducer.accept(Some(val)) {
                    TransductionResult::End => { return result; },
                    TransductionResult::None => (),
                    TransductionResult::Some(r) => { result.push(r); }
                }
            }
            loop {
                match transducer.accept(None) {
                    TransductionResult::End => { return result; },
                    TransductionResult::None => (),
                    TransductionResult::Some(r) => { result.push(r); }
                }
            }
        }
    }
}

pub mod iter {
    use std::marker::PhantomData;

    use ::{Transducer, TransductionResult};

    pub trait TransduceIter {
        type UnderlyingIterator;

        fn transduce<T, F, O>(self, transducer: T) -> TransduceIterator<Self::UnderlyingIterator, T, F, O>
            where T: Transducer<F, O>;
    }

    impl<I> TransduceIter for I
        where I: Iterator {

        type UnderlyingIterator = I;

        fn transduce<T, F, O>(self, transducer: T) -> TransduceIterator<Self::UnderlyingIterator, T, F, O>
            where T: Transducer<F, O> {

            TransduceIterator {
                underlying: self,
                finished: false,
                transducer: transducer,
                from: PhantomData,
                out: PhantomData
            }
        }
    }

    pub struct TransduceIterator<I, T, F, O> {
        underlying: I,
        finished: bool,
        transducer: T,
        from: PhantomData<F>,
        out: PhantomData<O>
    }

    impl<I, T, F, O> TransduceIterator<I, T, F, O>
        where T: Transducer<F, O> {
    }

    impl<I, T, F, O> Iterator for TransduceIterator<I, T, F, O>
        where I: Iterator<Item=F>,
              T: Transducer<F, O> {

        type Item = O;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let next_val = if self.finished {
                    None
                } else {
                    let interim = self.underlying.next();
                    if interim.is_none() {
                        self.finished = true
                    }
                    interim
                };
                match self.transducer.accept(next_val) {
                    TransductionResult::End => return None,
                    TransductionResult::None => (),
                    TransductionResult::Some(value) => return Some(value)
                }
            }
        }
    }
}
