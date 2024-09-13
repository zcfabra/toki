use crate::ast::{
    AstBinExpr, AstBlock, AstConditional, AstExpr, AstLiteral, AstNode,
    AstStmt, TypeAnnotation,
};
use crate::lexer::LexErr;
use crate::token::{SpannedToken, Token};
use core::iter::Peekable;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Lowest,
    AddSub,
    MulDiv,
    Equality,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErr {
    LexErr(LexErr),
    InvalidExpressionStart(usize, usize),

    UnexpectedEnd,
    UnexpectedIndent(usize, usize, usize),

    ExpectedTypeAnnotation(usize, usize),
    ExpectedNewline(usize, usize),
    ExpectedSemi(usize, usize),
    ExpectedColon(usize, usize),
}

#[derive(Debug, Clone, Copy)]
struct ParseContext {
    can_parse_annotation: bool,
}

impl ParseContext {
    fn new() -> Self {
        ParseContext {
            can_parse_annotation: true,
        }
    }
    fn with_annotation_parsing(mut self: Self) -> Self {
        self.can_parse_annotation = true;
        self
    }
    fn without_annotation_parsing(mut self: Self) -> Self {
        self.can_parse_annotation = false;
        self
    }
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

    // Need to handle no spaces to deal w/ the very first statement in the program
    let indent =
        if let Some((Ok((_, Token::Spaces(n_spaces))))) = tokens.peek() {
            let indent = n_spaces / 4;
            tokens.next();
            indent
        } else {
            0
        };

    loop {
        let stmt = match tokens.peek() {
            None => break,
            Some(Ok((_, Token::Newline))) => {
                tokens.next();
            }
            Some(Ok((_, Token::Spaces(n_spaces)))) => {
                let found_indent = n_spaces / 4;
                if found_indent < indent {
                    tokens.next();
                    if !matches!(tokens.peek(), Some(Ok((_, Token::Newline))))
                    {
                        break;
                    }
                } else if found_indent > indent {
                    let (ix, tok) = get_next_token(tokens)?;
                    return Err(ParseErr::UnexpectedIndent(
                        ix,
                        tok.src_len(),
                        indent,
                    ));
                } else {
                    tokens.next();
                }
            }
            Some(Ok(_)) => stmts.push(parse_stmt(tokens)?),
            Some(Err(_)) => todo!(),
        };
    }

    Ok((stmts, indent).into())
}
pub fn parse_stmt_end<'src, I>(
    parsed_stmt: &AstStmt<'src>,
    tokens: &mut Peekable<I>,
) -> Result<(), ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let expects_semi =
        !matches!(parsed_stmt, AstStmt::Expr { has_semi: true, .. });

    let (ix, tok) = get_next_token(tokens)?;
    if expects_semi && !matches!(tok, Token::Semicolon) {
        return Err(ParseErr::ExpectedSemi(ix, tok.src_len()));
    }

    match tok {
        Token::Semicolon => {
            if !matches!(tokens.peek(), Some(Ok((_, Token::Newline)))) {
                let (ix, next_tok) = get_next_token(tokens)?;
                return Err(ParseErr::ExpectedNewline(ix, next_tok.src_len()));
            } else {
                tokens.next();
            }
        }
        Token::Newline => {}
        x => {
            println!("Encountered Token {:?}", x);
            todo!()
        }
    }

    Ok(())
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

    let context = ParseContext::new();
    if matches!(tokens.peek(), Some(Ok((_, Token::Return)))) {
        tokens.next();

        let expr = parse_expr(tokens, Precedence::Lowest, context)?;

        eat_semi(tokens)?;

        return Ok(AstStmt::Return(expr));
    }

    let primary_expr = parse_primary_expr(tokens, context)?;

    if let Some(Ok((_, Token::Eq))) = tokens.peek() {
        tokens.next();
        let to_assign = parse_expr(tokens, Precedence::Lowest, context)?;

        eat_semi(tokens)?;

        return Ok(AstStmt::Assignment {
            target: primary_expr,
            assigned: to_assign,
        });
    }

    let expr =
        parse_expr_with(primary_expr, tokens, Precedence::Lowest, context)?;

    let has_semi = matches!(tokens.peek(), Some(Ok((_, Token::Semicolon))));
    if has_semi {
        eat_semi(tokens)?;
    }

    Ok(AstStmt::Expr { expr, has_semi })
}

pub fn eat_semi<'src, I>(tokens: &mut Peekable<I>) -> Result<(), ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    if matches!(tokens.peek(), Some(Ok((_, Token::Semicolon)))) {
        tokens.next();
        Ok(())
    } else {
        let (ix, tok) = get_next_token(tokens)?;
        Err(ParseErr::ExpectedSemi(ix, tok.src_len()))
    }
}

pub fn parse_primary_expr<'src, I>(
    tokens: &mut Peekable<I>,
    context: ParseContext,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let (ix, tok) = get_next_token(tokens)?;

    let expr = match tok {
        Token::LParen => {
            return parse_expr(tokens, Precedence::Lowest, context)
        }
        Token::If => return Ok(parse_conditional(tokens)?.into()),
        id @ Token::Ident(_) => {
            if matches!(tokens.peek(), Some(Ok((_, Token::Colon))))
                && context.can_parse_annotation
            {
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
        ref x => {
            println!("Encountered Invalid Expression Start: {}", x);
            return Err(ParseErr::InvalidExpressionStart(ix, tok.src_len()));
        }
    };

    Ok(expr.into())
}

pub fn parse_conditional<'src, I>(
    tokens: &mut Peekable<I>,
) -> Result<AstConditional<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let condition = Box::new(parse_expr(
        tokens,
        Precedence::Lowest,
        ParseContext::new().without_annotation_parsing(),
    )?);

    let (ix, tok) = get_next_token(tokens)?;

    if !matches!(tok, Token::Colon) {
        return Err(ParseErr::ExpectedColon(ix, tok.src_len()));
    }

    if let Some(Ok((_, Token::Newline))) = tokens.peek() {
        tokens.next();
    }
    let if_block = parse_block(tokens)?;

    let else_block = if matches!(tokens.peek(), Some(Ok((_, Token::Else)))) {
        tokens.next();
        Some(parse_block(tokens)?)
    } else {
        None
    };

    Ok(AstConditional {
        condition,
        if_block,
        else_block,
    })
}

pub fn parse_expr<'src, I>(
    tokens: &mut Peekable<I>,
    precedence: Precedence,
    context: ParseContext,
) -> Result<AstExpr<'src>, ParseErr>
where
    I: Iterator<Item = Result<SpannedToken<'src>, LexErr>>,
{
    let lhs = parse_primary_expr(tokens, context)?;
    Ok(parse_expr_with(lhs, tokens, precedence, context)?)
}

pub fn parse_expr_with<'src, I>(
    parsed_expr: AstExpr<'src>,
    tokens: &mut Peekable<I>,
    precedence: Precedence,
    context: ParseContext,
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

            let rhs = parse_expr(tokens, encountered_precedence, context)?;
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

    let is_mut = if matches!(tokens.peek(), Some(Ok((_, Token::Mut)))) {
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
