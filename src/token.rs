use crate::parser::Precedence;

pub type SpannedToken<'src> = (usize, Token<'src>);

#[derive(Debug, PartialEq)]
pub enum Token<'src> {
    IntLiteral(i32),
    FloatLiteral(f32),
    StrLiteral(&'src str),
    Ident(&'src str),

    Newline,

    LParen,
    RParen,

    Add,
    Sub,
    Mul,
    Div,

    AddEq,
    SubEq,
    MulEq,
    DivEq,

    Bang,
    BangEq,

    Eq,
    DoubleEq,

    Not,
    And,
    Or,
}

impl Token<'_> {
    pub fn src_len(&self) -> usize {
        match self {
            Self::Eq
            | Self::Bang
            | Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::LParen
            | Self::RParen
            | Self::Newline => 1,
            Self::DoubleEq
            | Self::BangEq
            | Self::AddEq
            | Self::SubEq
            | Self::MulEq
            | Self::DivEq
            | Self::Or => 2,

            Self::And | Self::Not => 3,

            Self::IntLiteral(i) => format!("{}", i).len(),
            Self::FloatLiteral(f) => format!("{}", f).len(),
            Self::StrLiteral(s) => s.len() + 2, // Add length of quotations
            Self::Ident(id) => id.len(),
        }
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IntLiteral(i) => return write!(f, "{}", i),
                Self::FloatLiteral(fl) => return write!(f, "{}", fl),
                Self::StrLiteral(s) => return write!(f, "{}", s),
                Self::Ident(id) => return write!(f, "{}", id),

                Self::LParen => "(",
                Self::RParen => ")",

                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",

                Self::AddEq => "+=",
                Self::SubEq => "-=",
                Self::MulEq => "*=",
                Self::DivEq => "/=",

                Self::Eq => "=",
                Self::DoubleEq => "==",
                Self::Bang => "!",
                Self::BangEq => "!=",

                Self::Not => "not",
                Self::And => "and",
                Self::Or => "or",

                Self::Newline => "[NEWLINE]",
            }
        )
    }
}

impl Token<'_> {
    pub fn as_operator(&self) -> Option<Operator> {
        Some(match self {
            Self::Add => Operator::Add,
            Self::Sub => Operator::Sub,
            Self::Mul => Operator::Mul,
            Self::Div => Operator::Div,
            _ => return None,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Div,
    Mul,
}

impl Operator {
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Add | Self::Sub => Precedence::AddSub,
            Self::Mul | Self::Div => Precedence::MulDiv,
        }
    }
}
impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Add => "+",
                Self::Sub => "-",
                Self::Mul => "*",
                Self::Div => "/",
            }
        )
    }
}
