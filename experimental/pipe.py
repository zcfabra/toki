from abc import ABC, abstractmethod
from enum import Enum, auto
from typing import List, Optional


class T(Enum):
    PIPE = auto()
    IDENT = auto()
    ADD = auto()
    MUL = auto()
    LPAREN = auto()
    RPAREN = auto()

    ASSIGN = auto()
    EQ = auto()

    EOF = auto()


class TT:
    def __init__(self, type_: T, literal: Optional[str] = None):
        self.type_ = type_
        self.literal = literal

    def __repr__(self) -> str:
        return f"{self.type_} {self.literal or ''}"


class Tokenizer:
    def __init__(self, src: str):
        self.src = src
        self.l = 0
        self.r = 0

    def tokenize(self):
        tokens: List[TT] = []
        while self.r < len(self.src):
            match self.src[self.r]:
                case "|":
                    if self.next_char_is('>'):
                        tokens.append(TT(T.PIPE))
                case '+':
                    tokens.append(TT(T.ADD))
                case '*':
                    tokens.append(TT(T.MUL))
                case '(':
                    tokens.append(TT(T.LPAREN))
                case ')':
                    tokens.append(TT(T.RPAREN))
                case '=':
                    if self.next_char_is("="):
                        tokens.append(TT(T.EQ))
                    tokens.append(TT(T.ASSIGN))
                case str(c) if self.is_alpha(c):
                    while (
                        self.r < len(self.src)
                        and self.is_alpha(self.src[self.r])
                    ):
                        self.r += 1
                    literal = self.src[self.l:self.r]
                    self.l = self.r
                    tokens.append(TT(T.IDENT, literal))
                case _:
                    pass
            self.r += 1
            self.l = self.r

        tokens.append(TT(T.EOF))
        return tokens

    @staticmethod
    def is_alpha(ch: str) -> bool:
        return 'A' <= ch <= 'Z' or 'a' <= ch <= 'z'

    def next_char_is(self, tt: str) -> bool:
        return self.r + 1 < len(self.src) and self.src[self.r + 1] == tt


class NT:
    PIPE = auto()
    IDENT = auto()


class AST(ABC):
    def __init__(self) -> None:
        pass

    @abstractmethod
    def eval(self) -> None:
        pass


class PipeExpr(AST):
    def __init__(self) -> None:
        pass

    def eval(self) -> None:
        pass


class Stmt(AST):
    def __init__(self) -> None:
        pass

    def eval(self) -> None:
        pass


class Block(AST):
    """A group of statements of the same indent level"""

    def __init__(self, stmts: List[Stmt]) -> None:
        self.statements = stmts

    def eval(self) -> None:
        pass


class Parse:
    def __init__(self, tokens: List[TT]) -> None:
        self.tokens = tokens
        self.r = 0
        self.l = 0

    def parse(self) -> AST:
        # Top-level program parse
        stmts: List[Stmt] = []
        while self.r < len(self.tokens) and self.tokens[self.r].type_ != T.EOF:
            if stmt := self.parse_stmt():
                stmts.append(stmt)

        return Block(stmts)

    def parse_stmt(self) -> Optional[Stmt]:
        match self.tokens[self.r].type_:
            case T.IDENT:
                if self.next_token_is(T.PIPE):
                    pass
            case _:
                raise Exception("Hi")

    def next_token_is(self, t: T) -> bool:
        return (self.r + 1 < len(self.tokens)
                and self.tokens[self.r + 1].type_ == t)


SRC = \
    """
a = 10
100 * a 
|> print
|> double 
"""
tokens = Tokenizer(SRC).tokenize()
print(tokens)
Parse(tokens).parse()
