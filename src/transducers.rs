use super::Transducer;

pub struct MapTransducer<F> {
    f: F
}

impl<I, O, F> Transducer<I, O> for MapTransducer<F>
    where F: Fn(I) -> O {

    #[inline]
    fn accept(&self, value: I) -> Option<O> {
        Some((self.f)(value))
    }
}

pub fn map<F, I, O>(f: F) -> MapTransducer<F>
where F: Fn(I) -> O {
    MapTransducer {
        f: f
    }
}
