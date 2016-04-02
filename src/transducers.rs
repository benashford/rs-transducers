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

pub struct FilterTransducer<F> {
    f: F
}

impl<T, F> Transducer<T, T> for FilterTransducer<F>
    where F: Fn(&T) -> bool {

    #[inline]
    fn accept(&self, value: T) -> Option<T> {
        if (self.f)(&value) {
            Some(value)
        } else {
            None
        }
    }
}

pub fn filter<F, T>(f: F) -> FilterTransducer<F>
    where F: Fn(&T) -> bool {

    FilterTransducer {
        f: f
    }
}
