/*
 * Copyright 2016 rs-transducers developers
 *
 * Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
 * <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
 * option. This file may not be copied, modified, or distributed
 * except according to those terms.
 */

pub mod vec {
    use ::{Transducer, Reducing};

    pub trait Ref {
        type Input;

        fn transduce_ref<'a, T, O, RO, E>(&'a self, transducer: T) -> Result<Vec<O>, E>
            where RO: Reducing<&'a Self::Input, Vec<O>, E>,
                  T: Transducer<VecReducer<O>, RO=RO>;
    }

    pub trait Into {
        type Input;

        fn transduce_into<T, O, RO, E>(self, transducer: T) -> Result<Vec<O>, E>
            where RO: Reducing<Self::Input, Vec<O>, E>,
                  T: Transducer<VecReducer<O>, RO=RO>;
    }

    pub struct VecReducer<O>(Vec<O>);

    impl<'a, O> Reducing<O, Vec<O>, ()> for VecReducer<O> {
        type Item = O;

        #[inline]
        fn step(&mut self, value: O) -> Result<(), ()> {
            self.0.push(value);
            Ok(())
        }

        fn complete(self) -> Vec<O> {
            self.0
        }
    }

    impl<X> Ref for Vec<X> {
        type Input = X;

        fn transduce_ref<'a, T, O, RO, E>(&'a self, mut transducer: T) -> Result<Vec<O>, E>
            where RO: Reducing<&'a Self::Input, Vec<O>, E>,
                  T: Transducer<VecReducer<O>, RO=RO> {
            let rr = VecReducer(Vec::with_capacity(self.len()));
            let mut reducing = transducer.new(rr);
            reducing.init();
            for val in self.iter() {
                try!(reducing.step(val));
            }
            Ok(reducing.complete())
        }
    }

    impl<X> Into for Vec<X> {
        type Input = X;

        fn transduce_into<T, O, RO, E>(self, transducer: T) -> Result<Vec<O>, E>
            where RO: Reducing<Self::Input, Vec<O>, E>,
                  T: Transducer<VecReducer<O>, RO=RO> {
            let rr = VecReducer(Vec::with_capacity(self.len()));
            let mut reducing = transducer.new(rr);
            reducing.init();
            for val in self.into_iter() {
                try!(reducing.step(val))
            }
            Ok(reducing.complete())
        }
    }
}

pub mod iter {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::marker::PhantomData;
    use std::rc::Rc;

    use ::{Transducer, Reducing};

    pub trait TransduceIter {
        type UnderlyingIterator;
        type Item;

        fn transduce<T, O, RO, E>(self, transducer: T) -> TransduceIterator<Self::UnderlyingIterator, O, RO>
            where RO: Reducing<Self::Item, (), E>,
                  T: Transducer<IterReducer<O>, RO=RO>;
    }

    impl<I, T> TransduceIter for I
        where I: Iterator<Item=T> {

        type UnderlyingIterator = I;
        type Item = T;

        fn transduce<TR, O, RO, E>(self, transducer: TR) -> TransduceIterator<Self::UnderlyingIterator, O, RO>
            where RO: Reducing<Self::Item, (), E>,
                  TR: Transducer<IterReducer<O>, RO=RO> {
            let buffer = Rc::new(RefCell::new(VecDeque::new()));

            TransduceIterator {
                underlying: self,
                buffer: buffer.clone(),
                rf: transducer.new(IterReducer(buffer.clone())),
                runoff: false
            }
        }
    }

    pub struct IterReducer<T>(Rc<RefCell<VecDeque<T>>>);

    impl<T> Reducing<T, (), ()> for IterReducer<T> {
        type Item = T;

        #[inline]
        fn step(&mut self, value: T) -> Result<(), ()> {
            self.0.borrow_mut().push_back(value);
            Ok(())
        }

        fn complete(self) -> () {
            ()
        }
    }

    pub struct TransduceIterator<I, O, RF> {
        underlying: I,
        buffer: Rc<RefCell<VecDeque<O>>>,
        rf: RF,
        runoff: bool
    }

    impl<I, IN, O, RF> Iterator for TransduceIterator<I, O, RF>
        where I: Iterator<Item=IN>,
              RF: Reducing<IN, (), ()> {

        type Item = O;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if !self.runoff && self.buffer.borrow().is_empty() {
                    match self.underlying.next() {
                        None => {
                            self.runoff = true;
                            // TODO - re-enable subsequent line, will not work correctly
                            // without it
                            //self.rf.complete();
                        },
                        Some(value) => {
                            self.rf.step(value).unwrap();
                        }
                    }
                }
                if self.runoff && self.buffer.borrow().is_empty() {
                    return None
                }
                match self.buffer.borrow_mut().pop_front() {
                    None => (),
                    Some(value) => return Some(value)
                }
            }
        }
    }
}

pub mod channels {
    use std::marker::PhantomData;
    use std::sync::mpsc::{Receiver, Sender, SendError, channel};

    use ::{Transducer, Reducing};

    pub struct TransducingSender<O, SR>
        where SR: Reducing<O, (), SendError<O>> {

        rf: SR,
        o_type: PhantomData<O>
    }

    pub struct SenderReducer<T>(Sender<T>);

    impl<O> Reducing<O, (), SendError<O>> for SenderReducer<O> {
        type Item = O;

        #[inline]
        fn step(&mut self, value: O) -> Result<(), SendError<O>> {
            self.0.send(value)
        }

        fn complete(self) -> () {
            ()
        }
    }

    impl<O, SR> TransducingSender<O, SR>
        where SR: Reducing<O, (), SendError<O>> {

        pub fn send(&mut self, f: O) -> Result<(), SendError<O>> {
            self.rf.step(f)
        }

        pub fn close(self) -> Result<(), SendError<O>> {
            Ok(self.rf.complete())
        }
    }

    pub fn transducing_channel<I, O, T, RO>(transducer: T) -> (TransducingSender<I, RO>,
                                                               Receiver<O>)
        where RO: Reducing<I, (), SendError<I>, Item=O>,
              T: Transducer<SenderReducer<O>, RO=RO> {
        let (tx, rx) = channel();
        let sender = TransducingSender {
            rf: transducer.new(SenderReducer(tx)),
            o_type: PhantomData
        };
        (sender, rx)
    }
}
