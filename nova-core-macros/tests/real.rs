use nova_ta_core::{Num, Real, real_support};

/// Checked `f64` wrapper so the error path can fire.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
struct R(f64);

impl Num for R {
    type Error = &'static str;
    fn zero() -> Self {
        R(0.0)
    }
    fn one() -> Self {
        R(1.0)
    }
    fn from_usize(n: usize) -> Self {
        R(n as f64)
    }
    fn add(self, r: Self) -> Result<Self, Self::Error> {
        Ok(R(self.0 + r.0))
    }
    fn sub(self, r: Self) -> Result<Self, Self::Error> {
        Ok(R(self.0 - r.0))
    }
    fn mul(self, r: Self) -> Result<Self, Self::Error> {
        Ok(R(self.0 * r.0))
    }
    fn div(self, r: Self) -> Result<Self, Self::Error> {
        if r.0 == 0.0 {
            Err("div by zero")
        } else {
            Ok(R(self.0 / r.0))
        }
    }
    fn sqrt(self) -> Result<Self, Self::Error> {
        if self.0 < 0.0 {
            Err("sqrt of negative")
        } else {
            Ok(R(self.0.sqrt()))
        }
    }
    fn neg(self) -> Self {
        R(-self.0)
    }
}

impl Real for R {
    fn from_f64(x: f64) -> Result<Self, Self::Error> {
        if x.is_finite() {
            Ok(R(x))
        } else {
            Err("non-finite")
        }
    }
    fn exp(self) -> Result<Self, Self::Error> {
        Ok(R(self.0.exp()))
    }
    fn ln(self) -> Result<Self, Self::Error> {
        if self.0 <= 0.0 {
            Err("ln of non-positive")
        } else {
            Ok(R(self.0.ln()))
        }
    }
    fn sin(self) -> Result<Self, Self::Error> {
        Ok(R(self.0.sin()))
    }
    fn cos(self) -> Result<Self, Self::Error> {
        Ok(R(self.0.cos()))
    }
    fn powf(self, e: Self) -> Result<Self, Self::Error> {
        Ok(R(self.0.powf(e.0)))
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

real_support!(real, MyErr::number, R);

fn r(v: f64) -> R {
    R(v)
}

fn close(a: R, b: f64) -> bool {
    (a.0 - b).abs() < 1e-12
}

#[test]
fn num_subset_still_works() -> Result<(), MyErr> {
    let (a, b) = (r(7.0), r(3.0));
    assert_eq!(real!(a + b * 2 - (a + b) / 2), r(8.0));
    assert_eq!(real!(-a + 1), r(-6.0));
    assert_eq!(real!(sqrt(a + 2)), r(3.0));
    assert_eq!(real!(abs(-a)), r(7.0));
    assert_eq!(real!(max(a, b) + min(a, b)), r(10.0));
    assert_eq!(real!(a.max(b).sqrt() * 2 as usize), r(2.0 * 7f64.sqrt()));
    Ok(())
}

#[test]
fn float_literals_and_casts() -> Result<(), MyErr> {
    let x = r(2.0);
    let f = 1.5f64;
    assert_eq!(real!(x * 0.5), r(1.0));
    assert_eq!(real!(x + f as f64), r(3.5));
    assert_eq!(real!(x + (f * 2.0) as f64), r(5.0));
    assert_eq!(real!(1e-3), r(0.001));
    Ok(())
}

#[test]
fn trig_exp_ln() -> Result<(), MyErr> {
    let x = r(1.0);
    assert!(close(real!(cos(x) * cos(x) + sin(x) * sin(x)), 1.0));
    assert!(close(real!(ln(exp(x))), 1.0));
    assert!(close(real!(x.exp().ln()), 1.0));
    assert!(close(real!(exp(ln(x) + ln(x))), 1.0));
    assert!(close(real!(log(r(8.0), r(2.0))), 3.0));
    assert!(close(real!(r(8.0).log(2)), 3.0));
    assert!(close(real!(eml(x, E)), core::f64::consts::E - 1.0));
    Ok(())
}

#[test]
fn constants() -> Result<(), MyErr> {
    assert!(close(real!(PI), core::f64::consts::PI));
    assert!(close(real!(TAU / 2), core::f64::consts::PI));
    assert!(close(real!(exp(LN_2)), 2.0));
    assert!(close(real!(sin(PI)), 0.0));
    assert!(close(real!(cos(PI / 3)), 0.5));
    Ok(())
}

// rustfmt rewrites `x ** 2` as `x * *2` (same parse, uglier).
#[test]
#[rustfmt::skip]
fn pow_operator_and_functions() -> Result<(), MyErr> {
    let (x, y) = (r(2.0), r(3.0));
    assert_eq!(real!(x ** 3), r(8.0));
    assert_eq!(real!(pow(x, 3)), r(8.0));
    assert_eq!(real!(x.pow(y)), r(8.0));
    assert_eq!(real!(x.powf(0.5)), r(2f64.sqrt()));
    assert_eq!(real!(x ** 0.5), r(2f64.sqrt()));
    assert_eq!(real!(x ** -1), r(0.5));
    assert_eq!(real!(x ** (y - 1)), r(4.0));
    assert_eq!(real!(2 ** x), r(4.0));
    assert!(close(real!(x ** y.ln().exp()), 8.0));
    Ok(())
}

// rustfmt rewrites `x ** 2` as `x * *2` (same parse, uglier).
#[test]
#[rustfmt::skip]
fn pow_precedence_matches_python() -> Result<(), MyErr> {
    let (x, y, z) = (r(2.0), r(3.0), r(2.0));
    // tighter than `*` and `/`
    assert_eq!(real!(y * x ** 2), r(12.0));
    assert_eq!(real!(x ** 2 * y), r(12.0));
    assert_eq!(real!(y * z * x ** 2), r(24.0));
    assert_eq!(real!(r(8.0) / x ** 2), r(2.0));
    assert_eq!(real!((y * x) ** 2), r(36.0));
    // looser than `+`, `-`
    assert_eq!(real!(x ** 2 + 1), r(5.0));
    assert_eq!(real!(1 + x ** 2 * y), r(13.0));
    // unary minus applies to the result
    assert_eq!(real!(-x ** 2), r(-4.0));
    assert_eq!(real!((-x) ** 2), r(4.0));
    // right-associative
    assert_eq!(real!(x ** y ** z), r(512.0));
    assert_eq!(real!((x ** y) ** z), r(64.0));
    assert_eq!(real!(y * x ** y ** z), r(1536.0));
    Ok(())
}

// rustfmt rewrites `x ** 2` as `x * *2` (same parse, uglier).
#[test]
#[rustfmt::skip]
fn errors_are_mapped_and_propagated() {
    fn run(x: R) -> Result<R, MyErr> {
        Ok(real!(ln(x) + 1))
    }
    assert_eq!(run(r(1.0)), Ok(r(1.0)));
    assert_eq!(run(r(0.0)), Err(MyErr::Number("ln of non-positive")));

    fn lit() -> Result<R, MyErr> {
        Ok(real!(f64::INFINITY as f64))
    }
    assert_eq!(lit(), Err(MyErr::Number("non-finite")));

    fn root(x: R) -> Result<R, MyErr> {
        Ok(real!(sqrt(x) ** 2))
    }
    assert_eq!(root(r(-1.0)), Err(MyErr::Number("sqrt of negative")));
}

// rustfmt rewrites `x ** 2` as `x * *2` (same parse, uglier).
#[test]
#[rustfmt::skip]
fn unknown_functions_are_operands() -> Result<(), MyErr> {
    fn half(x: R) -> R {
        R(x.0 / 2.0)
    }
    struct S {
        v: [R; 2],
    }
    let s = S {
        v: [r(1.0), r(4.0)],
    };
    assert_eq!(real!(half(s.v[1]) + s.v[0] ** 2), r(3.0));
    Ok(())
}

// rustfmt rewrites `x ** 2` as `x * *2` (same parse, uglier).
#[test]
#[rustfmt::skip]
fn generic_and_f64() {
    real_support!(g, MyErr::number, T);
    fn pdf<T: Real<Error = &'static str>>(x: T) -> Result<T, MyErr> {
        Ok(g!(exp(-0.5 * x ** 2) / sqrt(2 * PI)))
    }
    assert!(close(pdf(r(0.0)).unwrap(), 0.3989422804014327));

    real_support!(f, core::convert::identity, f64);
    fn geo(a: f64, b: f64) -> Result<f64, core::convert::Infallible> {
        Ok(f!((a * b) ** 0.5))
    }
    assert_eq!(geo(4.0, 9.0), Ok(6.0));

    let v = (|| -> Result<f64, core::convert::Infallible> {
        Ok(nova_ta_core::real_impl!(
            core::convert::identity,
            f64,
            2.0 ** 3 + PI * 0
        ))
    })();
    assert_eq!(v, Ok(8.0));
}
