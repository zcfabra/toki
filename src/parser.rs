use crate::ast::{
    AstBinExpr, AstBlock, AstExpr, AstLiteral, AstNode, AstStmt,
    TypeAnnotation,
};
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
    InvalidExpressionStart(usize, usize),
    ExpectedTypeAnnotation(usize, usize),
    ExpectedNewline(usize, usize),
}

pub fn get_next_token<'src, I>(
    tokens: &mut Peekable<I>,
) -> Result<SpannedToken<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    match tokens.next() {
        Some(Ok(tok)) => Ok(tok),
        Some(Err(e)) => Err(ParseErr::LexErr(e)),
        None => Err(ParseErr::UnexpectedEnd),
    }
}

pub fn parse<'src, I>(tokens: I) -> Result<AstBlock<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    // Entry point of the parser
    let peekable_tokens = &mut tokens.peekable();
    return Ok(parse_block(peekable_tokens)?);
}

pub fn parse_block<'src, I>(
    tokens: &mut Peekable<I>,
) -> Result<AstBlock<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let mut stmts = Vec::new();

    loop {
        match tokens.peek() {
            None => break,
            Some(Ok((_, Token::Newline))) => {
                tokens.next();
                continue;
            }
            Some(Ok(_)) => stmts.push(parse_stmt(tokens)?),
            Some(Err(_)) => todo!(),
        }
        let (ix, tok) = get_next_token(tokens)?;
        match tok {
            Token::Semicolon => {
                if !matches!(tokens.peek(), Some(Ok((_, Token::Newline)))) {
                    let (ix, next_tok) = get_next_token(tokens)?;
                    return Err(ParseErr::ExpectedNewline(
                        ix,
                        next_tok.src_len(),
                    ));
                } else {
                    tokens.next();
                }
            }
            Token::Newline => {}
            x => {
                println!("Encountered: {}", x);
                todo!()
            }
        }
    }

    Ok(stmts.into())
}

pub fn parse_stmt<'src, I>(
    tokens: &mut Peekable<I>,
) -> Result<AstStmt<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    // If a statement starts w/ an identifier, it could be
    // 1. An assignment `a = 10;`
    // 2. A mutation statement `a += 10;`
    // 3. The beginning of an expr `a + 10`
    // 4. A call statement some_fn();
    // 5. A call expression some_fn()

    if matches!(tokens.peek(), Some(Ok((_, Token::Return)))) {
        tokens.next();
        let expr = parse_expr(tokens, Precedence::Lowest)?;
        return Ok(AstStmt::Return(expr));
    }

    let mut expr = parse_primary_expr(tokens)?;

    match tokens.peek() {
        Some(Ok((_, Token::Eq))) => {
            tokens.next();
            let to_assign = parse_expr(tokens, Precedence::Lowest)?;
            return Ok(AstStmt::Assignment {
                target: expr,
                assigned: to_assign,
            });
        }
        _ => {}
    }

    Ok(AstStmt::Expr {
        expr,
        has_semi: matches!(tokens.peek(), Some(Ok((_, Token::Semicolon)))),
    })
}

pub fn parse_primary_expr<'src, I>(
    tokens: &mut Peekable<I>,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let (ix, tok) = get_next_token(tokens)?;

    let expr = match tok {
        Token::LParen => return parse_expr(tokens, Precedence::Lowest),
        id @ Token::Ident(_) => {
            if matches!(tokens.peek(), Some(Ok((_, Token::Colon)))) {
                AstLiteral::TypedIdent {
                    name: id,
                    type_annotation: parse_annotation(tokens)?,
                }
            } else {
                AstLiteral::Ident(id)
            }
        }
        il @ Token::IntLiteral(_) => AstLiteral::Int(il),
        sl @ Token::StrLiteral(_) => AstLiteral::Str(sl),
        // fl @ Token::FloatLiteral(_) => AstLiteral::Str(sl).into(),
        _ => return Err(ParseErr::InvalidExpressionStart(ix, tok.src_len())),
    };

    Ok(expr.into())
}

pub fn parse_expr<'src, I>(
    tokens: &mut Peekable<I>,
    precedence: Precedence,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let lhs = parse_primary_expr(tokens)?;
    Ok(parse_expr_with(lhs, tokens, precedence)?)
}

pub fn parse_expr_with<'src, I>(
    parsed_expr: AstExpr<'src>,
    tokens: &mut Peekable<I>,
    precedence: Precedence,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let mut lhs = parsed_expr;

    loop {
        if let Some(Ok((ix, tok))) = tokens.peek() {
            if matches!(tok, Token::Semicolon) {
                break;
            }
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

            let t = tokens.next();

            let rhs = parse_expr(tokens, encountered_precedence)?;
            lhs = (lhs, op, rhs).into();
        } else {
            break;
        }
    }

    Ok(lhs)
}

fn parse_annotation<'src, I>(
    tokens: &mut Peekable<I>,
) -> Result<TypeAnnotation<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    // Consume the ':'
    tokens.next();

    println!("Annotation");
    let is_mut = if matches!(tokens.peek(), Some(Ok((_, Token::Mut)))) {
        println!("Mut");
        tokens.next();
        true
    } else {
        false
    };

    let (ix, tok) = match tokens.next() {
        Some(Ok(spanned)) => spanned,
        Some(Err(e)) => return Err(ParseErr::LexErr(e)),
        None => return Err(ParseErr::UnexpectedEnd),
    };

    Ok(match tok {
        Token::Ident(id) => {
            let type_ = TypeAnnotation::Dynamic(id);
            if is_mut {
                TypeAnnotation::Mut(Box::new(type_))
            } else {
                type_
            }
        }
        _ => return Err(ParseErr::ExpectedTypeAnnotation(ix, tok.src_len())),
    })
}
