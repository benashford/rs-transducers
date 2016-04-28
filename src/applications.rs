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
                  T: Transducer<RefReducer<O>, RO=RO>;
    }


    struct RefReducer<O>(Vec<O>);

    impl<'a, O> Reducing<O, Vec<O>, ()> for RefReducer<O> {
        type Item = O;

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
                  T: Transducer<RefReducer<O>, RO=RO> {
            let rr = RefReducer(Vec::with_capacity(self.len()));
            let mut reducing = transducer.new(rr);
            reducing.init();
            for val in self.iter() {
                try!(reducing.step(val));
            }
            Ok(reducing.complete())
        }
    }

    // pub trait Drain {
    //     type Input;

    //     fn transduce_drain<T, O>(mut self, transducer: T) -> Vec<O>
    //         where T: Transducer<Self::Input, O>;
    // }

    // impl<X> Drain for Vec<X> {
    //     type Input = X;

    //     fn transduce_drain<T, O>(mut self, mut transducer: T) -> Vec<O>
    //         where T: Transducer<Self::Input, O> {

    //         let mut result = Vec::with_capacity(self.len());
    //         for val in self.drain(..) {
    //             match transducer.accept(Some(val)) {
    //                 TransductionResult::End => { return result; },
    //                 TransductionResult::None => (),
    //                 TransductionResult::Some(r) => { result.push(r); }
    //             }
    //         }
    //         loop {
    //             match transducer.accept(None) {
    //                 TransductionResult::End => { return result; },
    //                 TransductionResult::None => (),
    //                 TransductionResult::Some(r) => { result.push(r); }
    //             }
    //         }
    //     }
    // }
}

// pub mod iter {
//     use std::marker::PhantomData;

//     use ::{Transducer, TransductionResult};

//     pub trait TransduceIter {
//         type UnderlyingIterator;

//         fn transduce<T, F, O>(self, transducer: T) -> TransduceIterator<Self::UnderlyingIterator, T, F, O>
//             where T: Transducer<F, O>;
//     }

//     impl<I> TransduceIter for I
//         where I: Iterator {

//         type UnderlyingIterator = I;

//         fn transduce<T, F, O>(self, transducer: T) -> TransduceIterator<Self::UnderlyingIterator, T, F, O>
//             where T: Transducer<F, O> {

//             TransduceIterator {
//                 underlying: self,
//                 finished: false,
//                 transducer: transducer,
//                 from: PhantomData,
//                 out: PhantomData
//             }
//         }
//     }

//     pub struct TransduceIterator<I, T, F, O> {
//         underlying: I,
//         finished: bool,
//         transducer: T,
//         from: PhantomData<F>,
//         out: PhantomData<O>
//     }

//     impl<I, T, F, O> TransduceIterator<I, T, F, O>
//         where T: Transducer<F, O> {
//     }

//     impl<I, T, F, O> Iterator for TransduceIterator<I, T, F, O>
//         where I: Iterator<Item=F>,
//               T: Transducer<F, O> {

//         type Item = O;

//         #[inline]
//         fn next(&mut self) -> Option<Self::Item> {
//             loop {
//                 let next_val = if self.finished {
//                     None
//                 } else {
//                     let interim = self.underlying.next();
//                     if interim.is_none() {
//                         self.finished = true
//                     }
//                     interim
//                 };
//                 match self.transducer.accept(next_val) {
//                     TransductionResult::End => return None,
//                     TransductionResult::None => (),
//                     TransductionResult::Some(value) => return Some(value)
//                 }
//             }
//         }
//     }
// }

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
