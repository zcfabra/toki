use crate::token::{SpannedToken, Token};

pub struct Lexer<'src> {
    src: &'src str,
    rest: &'src str,

    byte: usize,

    just_after_newline: bool,

    indent_level: usize,
}

type SourcePostion = usize;
pub type Result<T> = std::result::Result<T, LexErr>;

pub struct WithSrcErr<'src, 'err, E>
where
    E: std::error::Error,
{
    src: &'src str,
    err: &'err E,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexErr {
    UnknownToken(SourcePostion, Option<SourcePostion>),
    UnterminatedString(SourcePostion),
}

impl std::fmt::Display for LexErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownToken(ix, ed) => todo!(),
            Self::UnterminatedString(ix) => todo!(),
        }
    }
}

impl std::error::Error for LexErr {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(self)
    }
}

enum Started<'src> {
    IfEqualElse(Token<'src>, Token<'src>),
    Minus,
    String,
    Numeric,
    Ident,
    Spaces,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Lexer {
            src,
            rest: src,
            byte: 0,
            just_after_newline: false,
            indent_level: 0,
        }
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Result<SpannedToken<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut c_at = self.byte;
            let mut chars = self.rest.chars();
            let mut c = chars.next()?;

            let needs_dedent = self.indent_level > 0 && self.just_after_newline && c != ' ' && c != '\n';
            if needs_dedent {
                self.indent_level -= 1;
                return Some(Ok((c_at, Token::Dedent)));
            }

            while c == ' ' && !self.just_after_newline {
                self.byte += c.len_utf8();
                c_at = self.byte;
                self.rest = &self.rest[c.len_utf8()..];
                c = chars.next()?;
            }

            let c_rest = self.rest;

            self.byte += c.len_utf8();
            self.rest = chars.as_str();

            if c == '\n' {
                self.just_after_newline = true;
                return Some(Ok((c_at, Token::Newline)));
            } else {
                self.just_after_newline = false;
            }

            let started = match c {
                '(' => return Some(Ok((c_at, Token::LParen))),
                ')' => return Some(Ok((c_at, Token::RParen))),
                ':' => return Some(Ok((c_at, Token::Colon))),
                ';' => return Some(Ok((c_at, Token::Semicolon))),
                ',' => return Some(Ok((c_at, Token::Comma))),

                '-' => Started::Minus,
                '+' => Started::IfEqualElse(Token::Add, Token::AddEq),
                '*' => Started::IfEqualElse(Token::Mul, Token::MulEq),
                '/' => Started::IfEqualElse(Token::Div, Token::DivEq),
                '!' => Started::IfEqualElse(Token::Bang, Token::BangEq),
                '=' => Started::IfEqualElse(Token::Eq, Token::DoubleEq),

                ' ' => Started::Spaces,
                '"' => Started::String,
                '0'..='9' => Started::Numeric,
                a if a.is_alphabetic() => Started::Ident,

                _ => return Some(Err(LexErr::UnknownToken(c_at, None))),
            };

            return Some(Ok(match started {
                Started::Spaces => {
                    let space_end_ix = c_rest.find(|c| c != ' ').unwrap_or_else(|| c_rest.len());
                    let spaces = &c_rest[..space_end_ix];

                    // TODO: Add LexErr
                    assert!(spaces.len() % 4 == 0, "Required indent size is 4 spaces");

                    let indent = spaces.len() / 4;

                    if indent == self.indent_level {
                        continue;
                    }

                    let tok = if indent < self.indent_level {
                        let n_bytes = 4 - c.len_utf8();

                        self.byte += n_bytes;
                        self.rest = &self.rest[n_bytes..];
                        self.indent_level -= 1;
                        (c_at, Token::Dedent)
                    } else {
                        assert!(indent - self.indent_level == 1, "Unexpected Indent");

                        let n_bytes = 4 - c.len_utf8();

                        self.byte += n_bytes;
                        self.rest = &self.rest[n_bytes..];
                        self.indent_level += 1;
                        (c_at, Token::Indent)
                    };
                    tok
                }
                Started::Numeric => {
                    let numeric_end_ix = c_rest
                        .find(|c: char| !(c.is_numeric() || c == '_'))
                        .unwrap_or_else(|| c_rest.len());

                    let numeric_token = &c_rest[..numeric_end_ix];

                    let n_bytes = numeric_token.len() - c.len_utf8();
                    self.byte += n_bytes;
                    self.rest = &self.rest[n_bytes..];

                    let n: i32 = numeric_token.parse().expect("Should have checked");
                    (c_at, Token::IntLiteral(n))
                }
                Started::Ident => {
                    let ident_ed_ix = c_rest
                        .find(|c: char| !(c == '_' || c.is_alphanumeric()))
                        .unwrap_or_else(|| c_rest.len());

                    let ident = &c_rest[..ident_ed_ix];

                    let n_bytes = ident.len() - c.len_utf8();
                    self.byte += n_bytes;
                    self.rest = &self.rest[n_bytes..];

                    (c_at, get_keyword(ident).unwrap_or_else(|| Token::Ident(ident)))
                }
                Started::String => {
                    if let Some(str_end_ix) = c_rest[1..].find(|c| c == '"') {
                        let full_str = &c_rest[..=str_end_ix + 1];
                        let n_bytes = full_str.len() - c.len_utf8();

                        self.byte += n_bytes;
                        self.rest = &self.rest[n_bytes..];

                        (c_at, Token::StrLiteral(&full_str[1..&full_str.len() - 1]))
                    } else {
                        return Some(Err(LexErr::UnterminatedString(c_at)));
                    }
                }
                Started::IfEqualElse(no, yes) => {
                    let tok = if self.rest.starts_with('=') {
                        self.byte += '='.len_utf8();
                        self.rest = &self.rest[1..];
                        yes
                    } else {
                        no
                    };
                    (c_at, tok)
                }
                Started::Minus => {
                    let tok = if self.rest.starts_with('=') {
                        self.byte += '='.len_utf8();
                        self.rest = &self.rest[1..];
                        Token::SubEq
                    } else if self.rest.starts_with('>') {
                        self.byte += '='.len_utf8();
                        self.rest = &self.rest[1..];
                        Token::Arrow
                    } else {
                        Token::Sub
                    };
                    (c_at, tok)
                }
                _ => todo!(),
            }));
        }
    }
}

fn get_keyword<'src>(ident: &'src str) -> Option<Token<'src>> {
    Some(match ident {
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "mut" => Token::Mut,
        "return" => Token::Return,
        "if" => Token::If,
        "else" => Token::Else,
        "def" => Token::Def,
        "struct" => Token::Struct,
        _ => return None,
    })
}
