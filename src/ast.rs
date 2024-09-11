use crate::token::{Operator, Token};

#[derive(Debug, PartialEq, Eq)]
pub enum TypeAnnotation<'src> {
    Int,
    Str,
    Bool,
    Dynamic(&'src str),
    Mut(Box<TypeAnnotation<'src>>),
}

impl std::fmt::Display for TypeAnnotation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Int => "int",
                Self::Str => "str",
                Self::Bool => "bool",
                Self::Dynamic(d) => d,
                Self::Mut(t) => return write!(f, "mut {}", t),
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum AstLiteral<'src> {
    Int(Token<'src>),
    Str(Token<'src>),
    Ident(Token<'src>),
    TypedIdent {
        name: Token<'src>,
        type_annotation: TypeAnnotation<'src>,
    },
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
                Self::Ident(name) => name,
                Self::TypedIdent {
                    name,
                    type_annotation,
                } => return write!(f, "{}: {}", name, type_annotation),
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

impl<'src> From<AstExpr<'src>> for AstNode<'src> {
    fn from(value: AstExpr<'src>) -> Self {
        AstNode::Expr(value)
    }
}

#[derive(Debug, PartialEq)]
pub enum AstStmt<'src> {
    Expr {
        expr: AstExpr<'src>,
        has_semi: bool,
    },
    Return(AstExpr<'src>),
    Assignment {
        target: AstExpr<'src>,
        assigned: AstExpr<'src>,
    },
}

impl std::fmt::Display for AstStmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment { target, assigned } => {
                return write!(f, "{} = {};", target, assigned)
            }
            Self::Expr { expr, has_semi } => {
                let mut expr = format!("{}", expr);
                if *has_semi {
                    expr.push(';');
                }
                return write!(f, "{}", expr);
            }
            Self::Return(e) => return write!(f, "return {};", e),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AstBlock<'src> {
    stmts: Vec<AstStmt<'src>>,
}

impl<'src> std::fmt::Display for AstBlock<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for stmt in &self.stmts {
            str.push_str(format!("{}\n", stmt).as_str());
        }
        write!(f, "{}", str)
    }
}

impl<'src> From<Vec<AstStmt<'src>>> for AstBlock<'src> {
    fn from(value: Vec<AstStmt<'src>>) -> Self {
        AstBlock { stmts: value }
    }
}
