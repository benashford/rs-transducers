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
