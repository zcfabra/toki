use crate::ast::{
    AstBinExpr, AstBlock, AstCallExpr, AstConditional, AstExpr, AstLiteral, AstNode, AstStmt, AttrAccess, CallArg,
    FnDef, TypeAnnotation,
};
use crate::lexer::{LexErr, Result as LexResult};
use crate::token::{Operator, SpannedToken, Token};
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
    UnexpectedStmt(usize, usize),

    ExpectedTypeAnnotation(usize, usize),
    ExpectedNewline(usize, usize),
    ExpectedSemi(usize, usize),
    ExpectedColon(usize, usize),
    ExpectedFnName(usize, usize),

    // TODO: Make this an &str once or &Token once lifetime is removed
    ExpectedToken(usize, usize, String),
}

#[derive(Debug, Clone, Copy)]
struct ParseContext {
    can_parse_annotation: bool,
    is_in_paren_block: bool,
}

impl ParseContext {
    fn new() -> Self {
        ParseContext {
            can_parse_annotation: true,
            is_in_paren_block: false,
        }
    }
    fn entering_parens(mut self: Self) -> Self {
        self.is_in_paren_block = true;
        self
    }
    fn exiting_parens(mut self: Self) -> Self {
        self.is_in_paren_block = false;
        self
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

type TokenIter<'src> = LexResult<SpannedToken<'src>>;

type Result<T> = std::result::Result<T, ParseErr>;

pub fn get_next_token<'src, I>(tokens: &mut Peekable<I>) -> Result<SpannedToken<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    match tokens.next() {
        Some(Ok(tok)) => Ok(tok),
        Some(Err(e)) => Err(ParseErr::LexErr(e)),
        None => Err(ParseErr::UnexpectedEnd),
    }
}

pub fn parse<'src, I>(tokens: I) -> Result<AstBlock<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    // Entry point of the parser
    let peekable_tokens = &mut tokens.peekable();
    return Ok(parse_block(peekable_tokens, 0)?);
}

pub fn parse_block<'src, I>(tokens: &mut Peekable<I>, indent: usize) -> Result<AstBlock<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let mut stmts = Vec::new();
    let mut has_no_semi_expr = false;

    loop {
        match tokens.peek() {
            None => {
                break;
            }
            Some(Ok((_, Token::Newline))) => {
                tokens.next();
            }
            Some(Ok((_, Token::Indent))) => {
                let (ix, tok) = get_next_token(tokens)?;
                return Err(ParseErr::UnexpectedIndent(ix, tok.src_len(), 0));
            }
            Some(Ok((_, Token::Dedent))) => {
                tokens.next();
                break;
            }
            Some(Ok((ix, tok))) => {
                if has_no_semi_expr {
                    return Err(ParseErr::UnexpectedStmt(*ix, tok.src_len()));
                }

                let stmt = parse_stmt(tokens, indent)?;

                if matches!(stmt, AstStmt::Expr { has_semi: false, .. }) {
                    has_no_semi_expr = true;
                }

                stmts.push(stmt);
            }

            Some(Err(_)) => todo!(),
        };
    }

    let block = AstBlock {
        indent,
        stmts,
        has_semi: !has_no_semi_expr,
    };
    Ok(block)
}
pub fn parse_stmt<'src, I>(tokens: &mut Peekable<I>, indent: usize) -> Result<AstStmt<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
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

        let expr = parse_expr(tokens, Precedence::Lowest, indent, context)?;

        eat(tokens, Token::Semicolon)?;

        return Ok(AstStmt::Return(expr));
    }

    if matches!(tokens.peek(), Some(Ok((_, Token::Def)))) {
        return parse_fn_def(tokens, indent).map(|r| AstStmt::FnDef(r));
    }

    if matches!(tokens.peek(), Some(Ok((_, Token::Struct)))) {
        return Ok(parse_struct_def(tokens, indent)?);
    }

    let primary_expr = parse_primary_expr(tokens, indent, context)?;

    if let Some(Ok((_, Token::Eq))) = tokens.peek() {
        tokens.next();
        let to_assign = parse_expr(tokens, Precedence::Lowest, indent, context)?;

        eat(tokens, Token::Semicolon)?;

        return Ok(AstStmt::Assignment {
            target: primary_expr,
            assigned: to_assign,
        });
    }

    let expr = parse_expr_with(primary_expr, tokens, Precedence::Lowest, indent, context)?;

    let has_semi_next = matches!(tokens.peek(), Some(Ok((_, Token::Semicolon))));
    if has_semi_next {
        eat(tokens, Token::Semicolon)?;
    }

    let has_semi = expr_has_semi(&expr, has_semi_next);
    Ok(AstStmt::Expr { expr, has_semi })
}

fn parse_fn_args<'src, I>(tokens: &mut Peekable<I>) -> Result<Vec<AstLiteral<'src>>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let mut args = Vec::new();

    loop {
        if !matches!(tokens.peek(), Some(Ok((_, Token::Ident(_))))) {
            break;
        }

        let (_, tok) = get_next_token(tokens)?;

        args.push(AstLiteral::TypedIdent {
            name: tok,
            type_annotation: parse_annotation(tokens)?,
        });

        if matches!(tokens.peek(), Some(Ok((_, Token::Comma)))) {
            tokens.next();
        }
    }
    eat(tokens, Token::RParen)?;
    Ok(args)
}

pub fn parse_struct_def<'src, I>(tokens: &mut Peekable<I>, indent: usize) -> Result<AstStmt<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    eat(tokens, Token::Struct)?;

    let (ix, struct_name) = get_next_token(tokens)?;
    if !matches!(struct_name, Token::Ident(_)) {
        return Err(ParseErr::ExpectedFnName(ix, struct_name.src_len()));
    }
    let name = AstLiteral::Ident(struct_name);

    eat(tokens, Token::Colon)?;
    eat(tokens, Token::Newline)?;
    eat(tokens, Token::Indent)?;

    let fields = parse_struct_fields(tokens)?;
    skip_newlines(tokens);

    let methods = parse_struct_methods(tokens)?;

    skip_newlines(tokens);
    eat(tokens, Token::Dedent)?;

    Ok(AstStmt::StructDef { name, fields, methods })
}

pub fn skip_newlines<'src, I>(tokens: &mut Peekable<I>)
where
    I: Iterator<Item = TokenIter<'src>>,
{
    while matches!(tokens.peek(), Some(Ok((_, Token::Newline)))) {
        // Skip any newlines after the struct
        tokens.next();
    }
}
pub fn parse_struct_fields<'src, I>(tokens: &mut Peekable<I>) -> Result<Vec<AstLiteral<'src>>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let mut fields = Vec::new();
    while matches!(tokens.peek(), Some(Ok((_, Token::Ident(_))))) {
        let (_, name) = get_next_token(tokens)?;
        let type_annotation = parse_annotation(tokens)?;
        fields.push(AstLiteral::TypedIdent { name, type_annotation });
        eat(tokens, Token::Newline)?;
    }
    Ok(fields)
}

pub fn parse_struct_methods<'src, I>(tokens: &mut Peekable<I>) -> Result<Vec<FnDef<'src>>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let mut methods = Vec::new();
    while matches!(tokens.peek(), Some(Ok((_, Token::Def)))) {
        methods.push(parse_fn_def(tokens, 0)?);
    }
    Ok(methods)
}

pub fn parse_fn_def<'src, I>(tokens: &mut Peekable<I>, indent: usize) -> Result<FnDef<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    eat(tokens, Token::Def)?;
    let (ix, fn_name) = get_next_token(tokens)?;

    if !matches!(fn_name, Token::Ident(_)) {
        return Err(ParseErr::ExpectedFnName(ix, fn_name.src_len()));
    }
    let name = AstLiteral::Ident(fn_name);

    eat(tokens, Token::LParen)?;

    let args = parse_fn_args(tokens)?;

    eat(tokens, Token::Arrow)?;
    let return_type = parse_type_decl(tokens)?;

    eat(tokens, Token::Colon)?;
    eat(tokens, Token::Newline)?;
    eat(tokens, Token::Indent)?;

    let body = parse_block(tokens, indent + 1)?;

    return Ok(FnDef {
        name,
        args,
        body,
        return_type,
    });
}

fn parse_type_decl<'src, I>(tokens: &mut Peekable<I>) -> Result<TypeAnnotation<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
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

pub fn expr_has_semi(expr: &AstExpr<'_>, has_semi_next: bool) -> bool {
    match expr {
        AstExpr::BlockExpr(AstBlock { has_semi, .. }) => *has_semi,
        AstExpr::ConditionalExpr(AstConditional {
            if_block, else_block, ..
        }) => {
            if let Some(eb) = else_block {
                expr_has_semi(&eb, has_semi_next)
            } else {
                if_block.has_semi
            }
        }
        _ => has_semi_next,
    }
}

fn parse_primary_expr<'src, I>(tokens: &mut Peekable<I>, indent: usize, context: ParseContext) -> Result<AstExpr<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let (ix, tok) = get_next_token(tokens)?;

    let expr = match tok {
        Token::LParen => {
            if matches!(tokens.peek(), Some(Ok((_, Token::Newline)))) {
                return parse_expr(tokens, Precedence::Lowest, indent, context);
            }
            return parse_expr(tokens, Precedence::Lowest, indent, context);
        }
        Token::If => return Ok(parse_conditional(tokens, indent)?.into()),
        id @ Token::Ident(_) => {
            if matches!(tokens.peek(), Some(Ok((_, Token::Colon)))) && context.can_parse_annotation {
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

    Ok(parse_postfix_expr(expr.into(), tokens)?)
}

fn parse_conditional<'src, I>(tokens: &mut Peekable<I>, indent: usize) -> Result<AstConditional<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let ctx = ParseContext::new().without_annotation_parsing();

    let cond_expr = parse_expr(tokens, Precedence::Lowest, indent, ctx)?;
    let condition = Box::new(cond_expr);

    let (ix, tok) = get_next_token(tokens)?;

    if !matches!(tok, Token::Colon) {
        return Err(ParseErr::ExpectedColon(ix, tok.src_len()));
    }

    if let Some(Ok((_, Token::Newline))) = tokens.peek() {
        tokens.next();
        eat(tokens, Token::Indent)?;
    }
    let if_block = parse_block(tokens, indent + 1)?;

    let else_block = if matches!(tokens.peek(), Some(Ok((_, Token::Else)))) {
        // Consume 'else'
        tokens.next();

        let (ix, tok) = get_next_token(tokens)?;
        let expr = match tok {
            Token::If => AstExpr::ConditionalExpr(parse_conditional(tokens, indent)?),
            Token::Colon => {
                eat(tokens, Token::Newline)?;
                eat(tokens, Token::Indent)?;
                AstExpr::BlockExpr(parse_block(tokens, indent + 1)?)
            }
            _ => return Err(ParseErr::ExpectedColon(ix, tok.src_len())),
        };

        Some(Box::new(expr))
    } else {
        None
    };

    Ok(AstConditional {
        condition,
        if_block,
        else_block,
    })
}

fn parse_expr<'src, I>(
    tokens: &mut Peekable<I>,
    precedence: Precedence,
    indent: usize,
    context: ParseContext,
) -> Result<AstExpr<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let lhs = parse_primary_expr(tokens, indent, context)?;
    Ok(parse_expr_with(lhs, tokens, precedence, indent, context)?)
}

fn parse_expr_with<'src, I>(
    parsed_expr: AstExpr<'src>,
    tokens: &mut Peekable<I>,
    precedence: Precedence,
    indent: usize,
    context: ParseContext,
) -> Result<AstExpr<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let mut lhs = parse_postfix_expr(parsed_expr, tokens)?;

    loop {
        if let Some(Ok((_, tok))) = tokens.peek() {
            if matches!(tok, Token::Semicolon) {
                break;
            }
            if matches!(tok, Token::RParen) {
                if !context.is_in_paren_block {
                    tokens.next();
                }
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

            let rhs = parse_expr(tokens, encountered_precedence, indent, context)?;
            lhs = parse_postfix_expr((lhs, op, rhs).into(), tokens)?;
        } else {
            break;
        }
    }

    Ok(parse_postfix_expr(lhs, tokens)?)
}

fn parse_postfix_expr<'src, I>(lhs: AstExpr<'src>, tokens: &mut Peekable<I>) -> Result<AstExpr<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let mut lhs = lhs;
    loop {
        match tokens.peek() {
            Some(Ok((_, Token::LParen))) => {
                lhs = parse_call_expr(lhs, tokens)?.into();
            }
            Some(Ok((_, Token::Dot))) => {
                lhs = parse_attr_access(lhs, tokens)?.into();
            }
            _ => break,
        }
    }

    Ok(lhs)
}

fn parse_attr_access<'src, I>(lhs: AstExpr<'src>, tokens: &mut Peekable<I>) -> Result<AttrAccess<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    eat(tokens, Token::Dot)?;
    let (_, attr_tok) = get_next_token(tokens)?;
    assert!(matches!(attr_tok, Token::Ident(_)), "Expected Literal After Dot");

    let attribute = AstLiteral::Ident(attr_tok);
    Ok(AttrAccess {
        attribute,
        expr: Box::new(lhs),
    })
}

fn parse_call_expr<'src, I>(fn_expr: AstExpr<'src>, tokens: &mut Peekable<I>) -> Result<AstCallExpr<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    eat(tokens, Token::LParen)?;

    let mut call_args = Vec::new();
    let mut is_vertical = false;

    if matches!(tokens.peek(), Some(Ok((_, Token::Newline)))) {
        is_vertical = true;
        tokens.next();
    }

    if is_vertical {
        eat(tokens, Token::Indent)?;
    }

    while !matches!(tokens.peek(), Some(Ok((_, Token::RParen | Token::Dedent)))) {
        let ctx = ParseContext::new().entering_parens();
        let mut expr = parse_expr(tokens, Precedence::Lowest, 0, ctx)?;
        let mut name = None;

        let is_named_arg = matches!(expr, AstExpr::LitExpr(_)) && matches!(tokens.peek(), Some(Ok((_, Token::Eq))));
        if is_named_arg {
            eat(tokens, Token::Eq)?;
            name = Some(expr);
            expr = parse_expr(tokens, Precedence::Lowest, 0, ctx)?;
        }

        call_args.push(CallArg { expr, name });

        if !matches!(tokens.peek(), Some(Ok((_, Token::Comma)))) {
            break;
        }

        eat(tokens, Token::Comma)?;

        if is_vertical {
            eat(tokens, Token::Newline)?;
        }
    }

    if is_vertical {
        eat(tokens, Token::Dedent)?;
    }
    eat(tokens, Token::RParen)?;

    let expr = AstCallExpr {
        called_expr: Box::new(fn_expr),
        args: call_args,
    };

    Ok(expr)
}

fn parse_annotation<'src, I>(tokens: &mut Peekable<I>) -> Result<TypeAnnotation<'src>>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    // Consume the ':'
    eat(tokens, Token::Colon)?;

    Ok(parse_type_decl(tokens)?)
}

fn eat<'src, I>(tokens: &mut Peekable<I>, expected_type: Token) -> Result<()>
where
    I: Iterator<Item = TokenIter<'src>>,
{
    let (ix, tok) = get_next_token(tokens)?;
    if tok == expected_type {
        return Ok(());
    }
    Err(ParseErr::ExpectedToken(ix, tok.src_len(), expected_type.to_string()))
}

#[test]
fn test_parse_add() {
    let mut toks = vec![Token::Ident("a"), Token::Add, Token::Ident("b")]
        .into_iter()
        .enumerate()
        .map(|t| Ok(t))
        .peekable();

    let expected: AstBinExpr = (Token::Ident("a"), Operator::Add, Token::Ident("b")).into();
    assert_eq!(
        parse_expr(&mut toks, Precedence::Lowest, 0, ParseContext::new()),
        Ok(AstExpr::BinExpr(expected))
    );
}

#[test]
fn test_parse_add_multi() {
    let mut toks = vec![
        Token::Ident("a"),
        Token::Add,
        Token::Ident("b"),
        Token::Add,
        Token::Ident("c"),
    ]
    .into_iter()
    .enumerate()
    .map(|t| Ok(t))
    .peekable();

    let r = AstExpr::BinExpr((Token::Ident("b"), Operator::Add, Token::Ident("c")).into());
    let expected = AstBinExpr {
        l: Box::new(AstExpr::LitExpr(AstLiteral::Ident(Token::Ident("a")))),
        op: Operator::Add,
        r: Box::new(r),
    };
    assert_eq!(
        parse_expr(&mut toks, Precedence::Lowest, 0, ParseContext::new()),
        Ok(AstExpr::BinExpr(expected))
    );
}
