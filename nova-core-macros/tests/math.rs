use nova_ta_core::Num;

/// Checked integer backend so the error path can actually fire.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
struct Chk(i64);

impl Num for Chk {
    type Error = &'static str;

    fn zero() -> Self {
        Chk(0)
    }
    fn one() -> Self {
        Chk(1)
    }
    fn from_usize(n: usize) -> Self {
        Chk(n as i64)
    }
    fn add(self, r: Self) -> Result<Self, Self::Error> {
        self.0.checked_add(r.0).map(Chk).ok_or("overflow")
    }
    fn sub(self, r: Self) -> Result<Self, Self::Error> {
        self.0.checked_sub(r.0).map(Chk).ok_or("overflow")
    }
    fn mul(self, r: Self) -> Result<Self, Self::Error> {
        self.0.checked_mul(r.0).map(Chk).ok_or("overflow")
    }
    fn div(self, r: Self) -> Result<Self, Self::Error> {
        self.0.checked_div(r.0).map(Chk).ok_or("div by zero")
    }
    fn sqrt(self) -> Result<Self, Self::Error> {
        Err("unsupported")
    }
    fn neg(self) -> Self {
        Chk(-self.0)
    }
}

#[derive(Debug, PartialEq)]
enum MyErr {
    Number(&'static str),
}

impl MyErr {
    fn number(e: &'static str) -> Self {
        MyErr::Number(e)
    }
}

nova_core_macros::math_support!(math, MyErr::number, Chk);

fn c(v: i64) -> Chk {
    Chk(v)
}

#[test]
fn precedence_and_parens() -> Result<(), MyErr> {
    let (a, b, cc, d, e) = (c(7), c(3), c(4), c(5), c(3));
    // 7 + 3*3 - (4+5)/3 = 13
    assert_eq!(math!(a + b * b - (cc + d) / e), c(13));
    Ok(())
}

#[test]
fn left_associativity() -> Result<(), MyErr> {
    let (a, b, cc) = (c(10), c(3), c(2));
    assert_eq!(math!(a - b - cc), c(5));
    assert_eq!(math!(a / cc / cc), c(2));
    Ok(())
}

#[test]
fn literals_and_usize_casts() -> Result<(), MyErr> {
    let a = c(10);
    let n = 4usize;
    assert_eq!(math!(a * 2 + 1), c(21));
    assert_eq!(math!(2), c(2));
    assert_eq!(math!(2 * 3), c(6));
    assert_eq!(math!((n * (n + 1) / 2) as usize), c(10));
    assert_eq!(math!(a + n as usize), c(14));
    Ok(())
}

#[test]
fn negation() -> Result<(), MyErr> {
    let a = c(5);
    assert_eq!(math!(-a + 2), c(-3));
    assert_eq!(math!(-(a * 2)), c(-10));
    Ok(())
}

#[test]
fn leaves_can_be_arbitrary_expressions() -> Result<(), MyErr> {
    struct S {
        x: Chk,
    }
    let s = S { x: c(6) };
    let v = [c(1), c(2)];
    fn two() -> Chk {
        Chk(2)
    }
    assert_eq!(math!(s.x * v[1] + two()), c(14));
    Ok(())
}

#[test]
fn errors_are_mapped_and_propagated() {
    fn run(a: Chk, b: Chk) -> Result<Chk, MyErr> {
        Ok(math!(a / b + 1))
    }
    assert_eq!(run(c(6), c(3)), Ok(c(3)));
    assert_eq!(run(c(6), c(0)), Err(MyErr::Number("div by zero")));

    fn overflow(a: Chk) -> Result<Chk, MyErr> {
        Ok(math!(a * a))
    }
    assert_eq!(overflow(c(i64::MAX)), Err(MyErr::Number("overflow")));
}

#[test]
fn works_in_closures_and_try_fold() {
    let sum: Result<Chk, MyErr> = [c(1), c(2), c(3)]
        .iter()
        .try_fold(Chk::zero(), |acc, &e| Ok(math!(acc + e)));
    assert_eq!(sum, Ok(c(6)));
}

#[test]
fn generic_type_parameter() {
    nova_core_macros::math_support!(gmath, MyErr::number, T);

    fn mean<T: Num<Error = &'static str>>(a: T, b: T) -> Result<T, MyErr> {
        Ok(gmath!((a + b) / 2))
    }
    assert_eq!(mean(c(4), c(8)), Ok(c(6)));
}

#[test]
fn works_with_f64() {
    nova_core_macros::math_support!(fmath, std::convert::identity, f64);
    // Infallible: map_err(identity) then `?` into Infallible-compatible fn.
    fn f(a: f64) -> Result<f64, std::convert::Infallible> {
        Ok(fmath!(a * 2 + 1))
    }
    assert_eq!(f(1.5), Ok(4.0));
}
