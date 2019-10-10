use std::iter::FromIterator;

trait CollectBy1<T1>: Iterator<Item = (T1,)> {
    fn collect_1(self) -> (Vec<T1>,);
}

impl<I, T1> CollectBy1<T1> for I
where
    I: Iterator<Item = (T1,)>,
{
    fn collect_1(mut self) -> (Vec<T1>,) {
        let mut v1 = Vec::new();

        v1.reserve(self.size_hint().0);

        while let Some((v,)) = self.next() {
            v1.push(v);
        }

        (v1,)
    }
}

#[cfg(test)]
mod test {
    use crate::iter::*;

    #[test]
    fn collect1() {
        let vec: Vec<(i32,)> = vec![(0,), (1,), (2,)];
        let (vec2,): (Vec<(i32)>,) = vec.clone().into_iter().collect_1();
        vec.iter().zip(vec2.iter()).for_each(|(&(a,), &b)| {
            assert_eq!(a, b);
        })
    }
}
