pub mod transducers;
pub mod applications;

pub trait Transducer<I, O> {
    fn accept(&self, value: I) -> Option<O>;

    #[inline]
    fn complete(&self) -> Option<O> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::transducers;
    use super::applications::vec::Ref;

    #[test]
    fn test_vec_ref() {
        let source = vec![1, 2, 3];
        let transducer = transducers::map(|x| x + 1);
        let result = source.trans_ref(transducer);
        assert_eq!(vec![2, 3, 4], result);
    }
}
