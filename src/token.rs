#[derive(Debug)]
pub enum Token<'src> {
    IntLiteral(i32),
    FloatLiteral(f32),
    StrLiteral(&'src str),
    Ident(&'src str),

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
                Self::BangEq=> "!=",

                Self::Not => "not",
                Self::And => "and",
                Self::Or => "or",
            }
        )
    }
}
