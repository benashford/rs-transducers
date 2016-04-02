use std::mem;

use super::Transducer;

pub struct MapTransducer<F> {
    f: F
}

impl<I, O, F> Transducer<I, O> for MapTransducer<F>
    where F: Fn(I) -> O {

    #[inline]
    fn accept(&mut self, value: I) -> Option<O> {
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
    fn accept(&mut self, value: T) -> Option<T> {
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

pub struct PartitionTransducer<T> {
    size: usize,
    holder: Vec<T>,
    all: bool
}

impl<T> Transducer<T, Vec<T>> for PartitionTransducer<T> {
    #[inline]
    fn accept(&mut self, value: T) -> Option<Vec<T>> {
        self.holder.push(value);
        if self.holder.len() == self.size {
            let mut other_vec = Vec::with_capacity(self.size);
            mem::swap(&mut self.holder, &mut other_vec);

            Some(other_vec)
        } else {
            None
        }
    }

    #[inline]
    fn complete(self) -> Option<Vec<Vec<T>>> {
        if self.all {
            if self.holder.is_empty() {
                None
            } else {
                let mut result = Vec::with_capacity(1);
                result.push(self.holder);
                Some(result)
            }
        } else {
            None
        }
    }
}

pub fn partition<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        holder: Vec::with_capacity(num),
        all: false
    }
}

pub fn partition_all<T>(num: usize) -> PartitionTransducer<T> {
    PartitionTransducer {
        size: num,
        holder: Vec::with_capacity(num),
        all: true
    }
}
