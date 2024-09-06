use crate::ast::{AstBinExpr, AstExpr, AstLiteral, AstNode};
use crate::lexer::LexErr;
use crate::token::{SpannedToken, Token};
use core::iter::Peekable;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,
    AddSub,
    MulDiv,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErr {
    UnexpectedEnd,
    LexErr(LexErr),
    InvalidExpressionStart(usize),
}
pub fn parse<'src, I>(tokens: I) -> Result<AstNode<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let peekable_tokens = &mut tokens.peekable();

    Ok(parse_expr(peekable_tokens, Precedence::Lowest)?.into())
}

pub fn parse_expr<'src, I>(
    tokens: &mut Peekable<I>,
    precedence: Precedence,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let mut lhs = parse_primary_expr(tokens, precedence)?;

    loop {
        if let Some(Ok((ix, tok))) = tokens.peek() {
            if matches!(tok, Token::RParen) {
                tokens.next();
                break;
            }

            let op = match tok.as_operator() {
                None => break,
                Some(op) => op,
            };

            let encountered_precedence = op.precedence();
            if encountered_precedence < precedence {
                break;
            }

            tokens.next();

            let rhs = parse_expr(tokens, encountered_precedence)?;
            lhs = (lhs, op, rhs).into();
        } else {
            break;
        }
    }

    Ok(lhs)
}

pub fn parse_primary_expr<'src, I>(
    tokens: &mut Peekable<I>,
    precedence: Precedence,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    if let Some(tok_res) = tokens.next() {
        let (ix, tok) = match tok_res {
            Ok(spanned_tok) => spanned_tok,
            Err(e) => return Err(ParseErr::LexErr(e)),
        };

        Ok(match tok {
            id @ Token::Ident(_) => AstLiteral::Ident(id).into(),
            il @ Token::IntLiteral(_) => AstLiteral::Int(il).into(),
            sl @ Token::StrLiteral(_) => AstLiteral::Str(sl).into(),
            Token::LParen => parse_expr(tokens, precedence)?,
            _ => return Err(ParseErr::InvalidExpressionStart(ix)),
        })
    } else {
        Err(ParseErr::UnexpectedEnd)
    }
}
