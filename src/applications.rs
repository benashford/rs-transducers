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

pub mod channels {
    use std::marker::PhantomData;
    use std::sync::mpsc::{Receiver, Sender, SendError, channel};

    use ::{Transducer, TransductionResult};

    pub struct TransducingSender<F, TR, T>
        where TR: Transducer<F, T> {

        sender: Sender<T>,
        from: PhantomData<F>,
        transducer: TR
    }

    impl<F, TR, T> TransducingSender<F, TR, T>
        where TR: Transducer<F, T> {

        pub fn send(&mut self, f: F) -> Result<(), SendError<T>> {
            match self.transducer.accept(Some(f)) {
                TransductionResult::End => Ok(()),
                TransductionResult::None => Ok(()),
                TransductionResult::Some(out) => {
                    self.sender.send(out)
                }
            }
        }

        pub fn close(&mut self) -> Result<(), SendError<T>> {
            loop {
                match self.transducer.accept(None) {
                    TransductionResult::End => return Ok(()),
                    TransductionResult::None => (),
                    TransductionResult::Some(out) => {
                        try!(self.sender.send(out));
                    }
                }
            }
        }
    }

    impl<F, TR, T> Drop for TransducingSender<F, TR, T>
        where TR: Transducer<F, T> {

        fn drop(&mut self) {
            self.close().expect("Channel to close successfully");
        }
    }

    pub fn transducing_channel<F, TR, T>(transducer: TR) -> (TransducingSender<F, TR, T>, Receiver<T>)
        where TR: Transducer<F, T> {

        let (tx, rx) = channel();
        let sender = TransducingSender {
            sender: tx,
            from: PhantomData,
            transducer: transducer
        };
        (sender, rx)
    }
}
