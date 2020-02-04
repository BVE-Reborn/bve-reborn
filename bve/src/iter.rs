//! Various iterator helpers

trait CollectBy1<T1>: Iterator<Item = (T1,)> {
    fn collect1(self) -> (Vec<T1>,);
}

impl<I, T1> CollectBy1<T1> for I
where
    I: Iterator<Item = (T1,)>,
{
    fn collect1(self) -> (Vec<T1>,) {
        let mut a1 = Vec::new();

        a1.reserve(self.size_hint().0);

        for (v1,) in self {
            a1.push(v1);
        }

        (a1,)
    }
}

trait CollectBy2<T1, T2>: Iterator<Item = (T1, T2)> {
    fn collect2(self) -> (Vec<T1>, Vec<T2>);
}

impl<I, T1, T2> CollectBy2<T1, T2> for I
where
    I: Iterator<Item = (T1, T2)>,
{
    fn collect2(self) -> (Vec<T1>, Vec<T2>) {
        let mut a1 = Vec::new();
        let mut a2 = Vec::new();

        let size = self.size_hint().0;
        a1.reserve(size);
        a2.reserve(size);

        for (v1, v2) in self {
            a1.push(v1);
            a2.push(v2);
        }

        (a1, a2)
    }
}

trait CollectBy3<T1, T2, T3>: Iterator<Item = (T1, T2, T3)> {
    fn collect3(self) -> (Vec<T1>, Vec<T2>, Vec<T3>);
}

impl<I, T1, T2, T3> CollectBy3<T1, T2, T3> for I
where
    I: Iterator<Item = (T1, T2, T3)>,
{
    fn collect3(self) -> (Vec<T1>, Vec<T2>, Vec<T3>) {
        let mut a1 = Vec::new();
        let mut a2 = Vec::new();
        let mut a3 = Vec::new();

        let size = self.size_hint().0;
        a1.reserve(size);
        a2.reserve(size);
        a3.reserve(size);

        for (v1, v2, v3) in self {
            a1.push(v1);
            a2.push(v2);
            a3.push(v3);
        }

        (a1, a2, a3)
    }
}

trait CollectBy4<T1, T2, T3, T4>: Iterator<Item = (T1, T2, T3, T4)> {
    fn collect4(self) -> (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>);
}

impl<I, T1, T2, T3, T4> CollectBy4<T1, T2, T3, T4> for I
where
    I: Iterator<Item = (T1, T2, T3, T4)>,
{
    fn collect4(self) -> (Vec<T1>, Vec<T2>, Vec<T3>, Vec<T4>) {
        let mut a1 = Vec::new();
        let mut a2 = Vec::new();
        let mut a3 = Vec::new();
        let mut a4 = Vec::new();

        let size = self.size_hint().0;
        a1.reserve(size);
        a2.reserve(size);
        a3.reserve(size);
        a4.reserve(size);

        for (v1, v2, v3, v4) in self {
            a1.push(v1);
            a2.push(v2);
            a3.push(v3);
            a4.push(v4);
        }

        (a1, a2, a3, a4)
    }
}

#[cfg(test)]
mod test {
    use crate::iter::*;
    use itertools::multizip;

    #[bve_derive::bve_test]
    #[test]
    fn collect1() {
        let in_vec: Vec<(i32,)> = vec![(0,), (1,), (2,)];
        let (vec1,): (Vec<i32>,) = in_vec.clone().into_iter().collect1();
        in_vec.iter().zip(vec1.iter()).for_each(|(&(a,), &b)| {
            assert_eq!(a, b);
        })
    }

    #[bve_derive::bve_test]
    #[test]
    fn collect2() {
        let in_vec: Vec<(i32, i32)> = vec![(0, 1), (1, 2), (2, 3)];
        let (vec1, vec2): (Vec<i32>, Vec<i32>) = in_vec.clone().into_iter().collect2();
        multizip((in_vec, vec1, vec2)).for_each(|((lhs1, lhs2), rhs1, rhs2)| {
            assert_eq!(lhs1, rhs1);
            assert_eq!(lhs2, rhs2);
        })
    }

    #[bve_derive::bve_test]
    #[test]
    fn collect3() {
        let in_vec: Vec<(i32, i32, i32)> = vec![(0, 1, 2), (1, 2, 3), (2, 3, 4)];
        let (vec1, vec2, vec3): (Vec<i32>, Vec<i32>, Vec<i32>) = in_vec.clone().into_iter().collect3();
        multizip((in_vec, vec1, vec2, vec3)).for_each(|((lhs1, lhs2, lhs3), rhs1, rhs2, rhs3)| {
            assert_eq!(lhs1, rhs1);
            assert_eq!(lhs2, rhs2);
            assert_eq!(lhs3, rhs3);
        })
    }

    #[bve_derive::bve_test]
    #[test]
    fn collect4() {
        let in_vec: Vec<(i32, i32, i32, i32)> = vec![(0, 1, 2, 3), (1, 2, 3, 4), (2, 3, 4, 5)];
        let (vec1, vec2, vec3, vec4): (Vec<i32>, Vec<i32>, Vec<i32>, Vec<i32>) = in_vec.clone().into_iter().collect4();
        multizip((in_vec, vec1, vec2, vec3, vec4)).for_each(|((lhs1, lhs2, lhs3, lhs4), rhs1, rhs2, rhs3, rhs4)| {
            assert_eq!(lhs1, rhs1);
            assert_eq!(lhs2, rhs2);
            assert_eq!(lhs3, rhs3);
            assert_eq!(lhs4, rhs4);
        })
    }
}
