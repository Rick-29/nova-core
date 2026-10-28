//! The vocabulary both macros understand: every operator, function name and
//! constant maps to one `Num` or `Real` method.

/// The trait a method lives on. `Real` names are rejected by `math_impl!`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trait {
    Num,
    Real,
}

impl Trait {
    pub fn name(self) -> &'static str {
        match self {
            Trait::Num => "Num",
            Trait::Real => "Real",
        }
    }
}

/// One callable: how it is spelled in the expression and what it calls.
#[derive(Debug)]
pub struct Func {
    /// Name accepted in the expression (`pow`, `max`, `PI`...).
    pub alias: &'static str,
    /// Method called on the trait (`powf`, `max_of`, `pi`...).
    pub method: &'static str,
    pub trait_: Trait,
    pub arity: usize,
    /// Whether the method returns `Result` and needs `.map_err(..)?`.
    pub fallible: bool,
}

macro_rules! funcs {
    ($($alias:ident => $trait_:ident :: $method:ident / $arity:literal $(, $fallible:tt)?;)*) => {
        const FUNCS: &[Func] = &[$(Func {
            alias: stringify!($alias),
            method: stringify!($method),
            trait_: Trait::$trait_,
            arity: $arity,
            fallible: false $(|| { let _ = stringify!($fallible); true })?,
        }),*];
    };
}

funcs! {
    // Num: operators
    add => Num::add / 2, ?;
    sub => Num::sub / 2, ?;
    mul => Num::mul / 2, ?;
    div => Num::div / 2, ?;
    neg => Num::neg / 1;
    // Num: functions
    sqrt => Num::sqrt / 1, ?;
    abs => Num::abs / 1;
    max => Num::max_of / 2;
    min => Num::min_of / 2;
    from_usize => Num::from_usize / 1;
    // Real: functions
    from_f64 => Real::from_f64 / 1, ?;
    exp => Real::exp / 1, ?;
    ln => Real::ln / 1, ?;
    sin => Real::sin / 1, ?;
    cos => Real::cos / 1, ?;
    pow => Real::powf / 2, ?;
    powf => Real::powf / 2, ?;
    log => Real::log / 2, ?;
    eml => Real::eml / 2, ?;
    // Real: constants
    PI => Real::pi / 0, ?;
    E => Real::e / 0, ?;
    TAU => Real::tau / 0, ?;
    LN_2 => Real::ln_2 / 0, ?;
}

impl Func {
    /// Looks a name up by its alias.
    pub fn get(name: &str) -> Option<&'static Func> {
        FUNCS.iter().find(|f| f.alias == name)
    }
}
