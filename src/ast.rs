use crate::token::{Operator, Token};

#[derive(Debug, PartialEq)]
pub enum AstLiteral<'src> {
    Int(Token<'src>),
    Str(Token<'src>),
    Ident(Token<'src>),
}

impl<'src> From<AstLiteral<'src>> for AstExpr<'src> {
    fn from(value: AstLiteral<'src>) -> Self {
        AstExpr::LitExpr(value)
    }
}

impl std::fmt::Display for AstLiteral<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int(i) => i,
                Self::Str(s) => s,
                Self::Ident(id) => id,
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct AstBinExpr<'src> {
    op: Operator,
    l: Box<AstExpr<'src>>,
    r: Box<AstExpr<'src>>,
}

impl std::fmt::Display for AstBinExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.l, self.op, self.r)
    }
}

#[derive(Debug, PartialEq)]
pub enum AstExpr<'src> {
    BinExpr(AstBinExpr<'src>),
    LitExpr(AstLiteral<'src>),
}

impl<'src> From<(AstExpr<'src>, Operator, AstExpr<'src>)> for AstExpr<'src> {
    fn from(value: (AstExpr<'src>, Operator, AstExpr<'src>)) -> Self {
        let (l, op, r) = value;
        let expr = AstBinExpr {
            op,
            l: Box::new(l),
            r: Box::new(r),
        };
        AstExpr::BinExpr(expr)
    }
}

impl std::fmt::Display for AstExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinExpr(bin) => write!(f, "{}", bin),
            Self::LitExpr(lit) => write!(f, "{}", lit),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstNode<'src> {
    Expr(AstExpr<'src>),
}

impl<'src> From<AstExpr<'src>> for AstNode<'src> {
    fn from(value: AstExpr<'src>) -> Self {
        AstNode::Expr(value)
    }
}

impl std::fmt::Display for AstNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Expr(expr) => expr,
            }
        )
    }
}
