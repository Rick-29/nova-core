//! Lowering of a `syn::Expr` into a tree of trait calls, and rendering it
//! back as `<T as Trait>::method(..).map_err(mapper)?` code.

use zyn::syn::{self, BinOp, Expr, Lit, Type, UnOp, spanned::Spanned};
use zyn::{Span, ToTokens, TokenStream, format_ident, zyn};

use crate::expand::Ctx;
use crate::func::{Func, Trait};

pub enum Node {
    /// Anything that is already a `T`, or the raw argument of a conversion.
    Leaf(Expr),
    Call(&'static Func, Vec<Node>),
}

/// Which vocabulary a macro accepts: `Num` only, or `Num` plus `Real`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Num,
    Real,
}

/// The plain identifier an expression is, or `""`.
fn ident(e: &Expr) -> String {
    match e {
        Expr::Path(p) => p
            .path
            .get_ident()
            .map(ToString::to_string)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// The `**` operator: Rust parses `a ** b` as `a * (*b)`.
#[allow(clippy::result_large_err)]
fn as_pow(op: &BinOp, right: Expr) -> Result<Expr, Expr> {
    match right {
        Expr::Unary(u) if matches!((op, &u.op), (BinOp::Mul(_), UnOp::Deref(_))) => Ok(*u.expr),
        other => Err(other),
    }
}

fn binop(op: &BinOp) -> syn::Result<&'static str> {
    Ok(match op {
        BinOp::Add(_) => "add",
        BinOp::Sub(_) => "sub",
        BinOp::Mul(_) => "mul",
        BinOp::Div(_) => "div",
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "unsupported operator, expected `+`, `-`, `*`, `/` or `**`",
            ));
        }
    })
}

fn is(ty: &Type, name: &str) -> bool {
    matches!(ty, Type::Path(p) if p.qself.is_none() && p.path.is_ident(name))
}

impl Level {
    pub fn lower(self, expr: Expr) -> syn::Result<Node> {
        Ok(match expr {
            Expr::Paren(p) => self.lower(*p.expr)?,
            Expr::Group(g) => self.lower(*g.expr)?,
            Expr::Binary(b) => match as_pow(&b.op, *b.right) {
                Ok(exp) => {
                    let span = exp.span();
                    let exp = self.lower(exp)?;
                    self.pow(*b.left, exp, span)?
                }
                Err(r) => self.call(binop(&b.op)?, b.op.span(), vec![*b.left, r])?,
            },
            Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => {
                self.call("neg", u.op.span(), vec![*u.expr])?
            }
            Expr::Cast(c) if is(&c.ty, "usize") => self.conv("from_usize", *c.expr)?,
            Expr::Cast(c) if is(&c.ty, "f64") => self.conv("from_f64", *c.expr)?,
            Expr::Lit(l) => match l.lit {
                Lit::Int(_) => self.conv("from_usize", Expr::Lit(l))?,
                Lit::Float(_) if self == Level::Real => self.conv("from_f64", Expr::Lit(l))?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        l,
                        "only integer literals are supported, use `x as usize` for usize values",
                    ));
                }
            },
            Expr::Call(c) if Func::get(&ident(&c.func)).is_some() => {
                self.call(&ident(&c.func), c.func.span(), c.args.into_iter().collect())?
            }
            Expr::MethodCall(m) if Func::get(&m.method.to_string()).is_some() => {
                let args = core::iter::once(*m.receiver).chain(m.args).collect();
                self.call(&m.method.to_string(), m.method.span(), args)?
            }
            Expr::Path(p)
                if Func::get(&ident(&Expr::Path(p.clone()))).is_some_and(|f| f.arity == 0) =>
            {
                self.call(&ident(&Expr::Path(p.clone())), p.span(), vec![])?
            }
            other => Node::Leaf(other),
        })
    }

    /// `base ** exp`. Because Rust reads `**` as `* (*..)`, `x * y ** 2` arrives
    /// as `(x * y) ** 2`, `-x ** 2` as `(-x) ** 2` and `x ** y ** z` as
    /// `(x ** y) ** z`. This rebinds them so `**` is tighter than `*`, `/` and
    /// unary `-`, and right-associative, as in Python. Parenthesised bases are
    /// left as written.
    fn pow(self, base: Expr, exp: Node, span: Span) -> syn::Result<Node> {
        match base {
            Expr::Binary(b) if matches!(b.op, BinOp::Mul(_) | BinOp::Div(_)) => {
                match as_pow(&b.op, *b.right) {
                    // `l ** inner ** exp`
                    Ok(inner) => {
                        let exp = self.pow(inner, exp, span)?;
                        self.pow(*b.left, exp, span)
                    }
                    // `l op r ** exp`
                    Err(r) => {
                        let l = self.lower(*b.left)?;
                        let r = self.pow(r, exp, span)?;
                        self.nodes(binop(&b.op)?, span, vec![l, r])
                    }
                }
            }
            Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => {
                let inner = self.pow(*u.expr, exp, span)?;
                self.nodes("neg", span, vec![inner])
            }
            base => {
                let base = self.lower(base)?;
                self.nodes("pow", span, vec![base, exp])
            }
        }
    }

    /// A conversion: the argument stays plain Rust, not a `T` expression.
    fn conv(self, name: &str, raw: Expr) -> syn::Result<Node> {
        let span = raw.span();
        self.nodes(name, span, vec![Node::Leaf(raw)])
    }

    /// A call whose arguments are `T` expressions to lower.
    fn call(self, name: &str, span: Span, args: Vec<Expr>) -> syn::Result<Node> {
        let args = args
            .into_iter()
            .map(|a| self.lower(a))
            .collect::<syn::Result<_>>()?;
        self.nodes(name, span, args)
    }

    /// A call with already-lowered arguments; checks the level and arity.
    fn nodes(self, name: &str, span: Span, args: Vec<Node>) -> syn::Result<Node> {
        let f = Func::get(name).expect("name comes from the table");
        if f.trait_ == Trait::Real && self == Level::Num {
            let msg = format!("`{name}` needs `Real`; use `real_support!` / `real_impl!`");
            return Err(syn::Error::new(span, msg));
        }
        if f.arity != args.len() {
            let msg = format!("`{name}` takes {} argument(s), got {}", f.arity, args.len());
            return Err(syn::Error::new(span, msg));
        }
        Ok(Node::Call(f, args))
    }
}

pub struct Render<'a>(pub &'a Ctx, pub &'a Node);

impl ToTokens for Render<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Render(ctx, node) = *self;
        let (ty, mapper) = (&ctx.ty, &ctx.mapper);
        match node {
            Node::Leaf(e) => e.to_tokens(tokens),
            Node::Call(f, args) => {
                let tr = format_ident!("{}", f.trait_.name());
                let method = format_ident!("{}", f.method);
                let args = args.iter().map(|a| Render(ctx, a));
                zyn!(
                    <{{ ty }} as ::nova_ta_core::{{ tr }}>::{{ method }}(@for (a in args) { {{ a }}, })
                    @if (f.fallible) { .map_err({{ mapper }})? }
                )
                .to_tokens(tokens);
            }
        }
    }
}
