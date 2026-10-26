//! Procedural macros behind `nova-ta-core`'s fallible arithmetic helpers.
//!
//! Depend on `nova-ta-core` and use its re-exports instead of this crate
//! directly: the generated code refers to `::nova_ta_core::Num`.
//!
//! ```ignore
//! nova_ta_core::math_support!(math, NovaError::number, T);
//!
//! let m = math!((n_t * xy_sum - x_sum * y_sum) / (n_t * x_sq2 - x_sum2));
//! ```
//!
//! Operands are `T` values, integer literals and `expr as usize` become
//! `T::from_usize(..)`, and every `+ - * /` becomes a `Num` call whose error is
//! mapped and propagated with `?`. The macro evaluates to a plain `T`.

use zyn::syn::{
    self, BinOp, Expr, Lit, Token, Type, UnOp,
    parse::{Parse, ParseStream},
};
use zyn::{ToTokens, TokenStream, zyn};

enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl TryFrom<&BinOp> for Op {
    type Error = syn::Error;

    fn try_from(op: &BinOp) -> syn::Result<Self> {
        match op {
            BinOp::Add(_) => Ok(Op::Add),
            BinOp::Sub(_) => Ok(Op::Sub),
            BinOp::Mul(_) => Ok(Op::Mul),
            BinOp::Div(_) => Ok(Op::Div),
            other => Err(syn::Error::new_spanned(
                other,
                "unsupported operator, expected `+`, `-`, `*` or `/`",
            )),
        }
    }
}

impl ToTokens for Op {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::Div => "div",
        };
        zyn::format_ident!("{}", name).to_tokens(tokens);
    }
}

enum Node {
    /// Anything that is already a `T`: paths, fields, calls, indexing...
    Leaf(Expr),
    /// A `usize` expression converted with `Num::from_usize`.
    Usize(Expr),
    Neg(Box<Node>),
    Bin(Op, Box<Node>, Box<Node>),
}

impl Node {
    fn lower(expr: Expr) -> syn::Result<Node> {
        match expr {
            Expr::Paren(p) => Node::lower(*p.expr),
            Expr::Group(g) => Node::lower(*g.expr),
            Expr::Binary(b) => Ok(Node::Bin(
                Op::try_from(&b.op)?,
                Box::new(Node::lower(*b.left)?),
                Box::new(Node::lower(*b.right)?),
            )),
            Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => {
                Ok(Node::Neg(Box::new(Node::lower(*u.expr)?)))
            }
            Expr::Cast(c) if is_usize(&c.ty) => Ok(Node::Usize(*c.expr)),
            Expr::Lit(l) => match l.lit {
                Lit::Int(_) => Ok(Node::Usize(Expr::Lit(l))),
                _ => Err(syn::Error::new_spanned(
                    l,
                    "only integer literals are supported, use `x as usize` for usize values",
                )),
            },
            other => Ok(Node::Leaf(other)),
        }
    }
}

fn is_usize(ty: &Type) -> bool {
    matches!(ty, Type::Path(p) if p.qself.is_none() && p.path.is_ident("usize"))
}

/// Everything needed to render a [`Node`]: the number type and the error mapper.
struct Ctx {
    mapper: Expr,
    ty: Type,
}

struct Render<'a>(&'a Ctx, &'a Node);

impl ToTokens for Render<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Render(ctx, node) = *self;
        let (ty, mapper) = (&ctx.ty, &ctx.mapper);
        let out = match node {
            Node::Leaf(e) => zyn!({ { e } }),
            Node::Usize(e) => zyn!(<{{ ty }} as ::nova_ta_core::Num>::from_usize({{ e }})),
            Node::Neg(n) => {
                let n = Render(ctx, n);
                zyn!(<{{ ty }} as ::nova_ta_core::Num>::neg({{ n }}))
            }
            Node::Bin(op, l, r) => {
                let (l, r) = (Render(ctx, l), Render(ctx, r));
                zyn!(
                    <{{ ty }} as ::nova_ta_core::Num>::{{ op }}({{ l }}, {{ r }})
                        .map_err({{ mapper }})?
                )
            }
        };
        out.to_tokens(tokens);
    }
}

/// `math_impl!(mapper, Type, expr)`
struct ImplInput {
    ctx: Ctx,
    expr: Expr,
}

impl Parse for ImplInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mapper = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        input.parse::<Token![,]>()?;
        let expr = input.parse()?;
        Ok(ImplInput {
            ctx: Ctx { mapper, ty },
            expr,
        })
    }
}

/// Expands a math expression into fallible `Num` calls.
///
/// `math_impl!(NovaError::number, T, a + b * 2)`
#[proc_macro]
pub fn math_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ImplInput { ctx, expr } = zyn::syn::parse_macro_input!(input as ImplInput);
    match Node::lower(expr) {
        Ok(node) => Render(&ctx, &node).to_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// `math_support!(name, mapper, Type)`
struct SupportInput {
    name: syn::Ident,
    mapper: Expr,
    ty: Type,
}

impl Parse for SupportInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![,]>()?;
        let mapper = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        Ok(SupportInput { name, mapper, ty })
    }
}

/// Defines a local macro `name!(expr)` bound to a number type and an error mapper.
///
/// `math_support!(math, NovaError::number, T);` then `math!(a + b * 2)` is
/// `math_impl!(NovaError::number, T, a + b * 2)`.
///
/// `Type` is resolved where `name!` is used, so it can be a generic parameter.
/// The calling crate needs `nova-ta-core` as a dependency, which re-exports
/// both macros.
#[proc_macro]
pub fn math_support(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let SupportInput { name, mapper, ty } = zyn::syn::parse_macro_input!(input as SupportInput);
    zyn!(
        macro_rules! {{ name }} {
            ($($t:tt)*) => {
                ::nova_ta_core::math_impl!({{ mapper }}, {{ ty }}, $($t)*)
            };
        }
    )
    .to_token_stream()
    .into()
}
