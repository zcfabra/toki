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
                Self::TypedIdent { name, type_annotation } => return write!(f, "{}: {}", name, type_annotation),
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct AstBinExpr<'src> {
    pub op: Operator,
    pub l: Box<AstExpr<'src>>,
    pub r: Box<AstExpr<'src>>,
}

// For tests
impl<'src> From<(Token<'src>, Operator, Token<'src>)> for AstBinExpr<'src> {
    fn from(value: (Token<'src>, Operator, Token<'src>)) -> Self {
        let (l, op, r) = value;
        let l_lit = AstLiteral::Ident(l);
        let r_lit = AstLiteral::Ident(r);

        let l_expr = AstExpr::LitExpr(l_lit);
        let r_expr = AstExpr::LitExpr(r_lit);

        return AstBinExpr {
            op,
            l: Box::new(l_expr),
            r: Box::new(r_expr),
        };
    }
}

impl std::fmt::Display for AstBinExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.l, self.op, self.r)
    }
}

#[derive(Debug, PartialEq)]
pub struct AstConditional<'src> {
    pub condition: Box<AstExpr<'src>>,
    pub if_block: AstBlock<'src>,
    pub else_block: Option<Box<AstExpr<'src>>>,
}

impl<'src> From<AstConditional<'src>> for AstExpr<'src> {
    fn from(value: AstConditional<'src>) -> Self {
        AstExpr::ConditionalExpr(value)
    }
}
impl std::fmt::Display for AstConditional<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let else_ = if let Some(expr) = &self.else_block {
            let indent = if self.if_block.indent > 0 {
                self.if_block.indent - 1
            } else {
                0
            };
            let spaces = std::iter::repeat(" ").take(indent * 4).collect::<String>();
            format!("{}else:\n{}", spaces, expr)
        } else {
            "".to_string()
        };

        write!(f, "if {}:\n{}{}", self.condition, self.if_block, else_)
    }
}

#[derive(Debug, PartialEq)]
pub struct CallArg<'src> {
    pub name: Option<AstExpr<'src>>,
    pub expr: AstExpr<'src>,
}

impl std::fmt::Display for CallArg<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{}={}", name, self.expr),
            None => write!(f, "{}", self.expr),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct AstCallExpr<'src> {
    pub called_expr: Box<AstExpr<'src>>,
    pub args: Vec<CallArg<'src>>,
}

impl std::fmt::Display for AstCallExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self.args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(",");
        write!(f, "{}({})", self.called_expr, args)
    }
}

#[derive(Debug, PartialEq)]
pub enum AstExpr<'src> {
    BinExpr(AstBinExpr<'src>),
    LitExpr(AstLiteral<'src>),
    ConditionalExpr(AstConditional<'src>),
    BlockExpr(AstBlock<'src>),

    CallExpr(AstCallExpr<'src>),
}

impl<'src> From<AstCallExpr<'src>> for AstExpr<'src> {
    fn from(value: AstCallExpr<'src>) -> Self {
        return AstExpr::CallExpr(value);
    }
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
            Self::ConditionalExpr(c) => write!(f, "{}", c),
            Self::BinExpr(bin) => write!(f, "{}", bin),
            Self::LitExpr(lit) => write!(f, "{}", lit),
            Self::BlockExpr(block) => write!(f, "{}", block),
            Self::CallExpr(fn_) => write!(f, "{}", fn_),
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
    FnDef {
        name: AstLiteral<'src>,
        args: Vec<AstLiteral<'src>>,
        body: AstBlock<'src>,
        return_type: TypeAnnotation<'src>,
    },
    StructDef {
        name: AstLiteral<'src>,
        fields: Vec<AstLiteral<'src>>,
    },
}

impl std::fmt::Display for AstStmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StructDef { name, fields } => {
                let fields = fields
                    .iter()
                    .map(|a| format!("    {}", a.to_string()))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(f, "struct {}:\n{}", name, fields)
            }
            Self::FnDef {
                name,
                args,
                body,
                return_type,
            } => {
                // TODO: Make this efficient
                let args_str = args
                    .iter()
                    .map(|a| format!("{}", a))
                    .collect::<Vec<String>>()
                    .join(", ");

                return write!(f, "def {}({}) -> {}:\n{};", name, args_str, return_type, body);
            }
            Self::Assignment { target, assigned } => return write!(f, "{} = {};", target, assigned),
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
    pub indent: usize,
    pub stmts: Vec<AstStmt<'src>>,
    pub has_semi: bool,
}

impl<'src> std::fmt::Display for AstBlock<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        let spaces = std::iter::repeat(" ").take(self.indent * 4).collect::<String>();
        for stmt in &self.stmts {
            str.push_str(format!("{}{}\n", spaces, stmt).as_str());
        }
        write!(f, "{}", str)
    }
}
