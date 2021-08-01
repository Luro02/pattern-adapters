use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Debug;
use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use regex_syntax::ast;
use syn::Ident;

use crate::pattern_kind::{CharClosure, Literal, PatternKind};

#[derive(Debug, Clone)]
pub struct Pattern {
    kind: PatternKind,
    range: Option<ast::Span>,
}

impl Pattern {
    #[must_use]
    pub fn new(kind: PatternKind) -> Self {
        Self { kind, range: None }
    }

    #[must_use]
    pub fn then(first: Self, second: Self) -> Self {
        match (&first.kind, &second.kind) {
            // simplify: 'a' -> 'b' -> 'c' to "abc"
            // "string" -> "another"
            (PatternKind::Literal(first), PatternKind::Literal(second)) => {
                return Self::literal(format!("{}{}", first, second));
            }
            // simplify: (a | b) -> c to (ac | bc)
            (PatternKind::Or(a, b), PatternKind::Literal(literal)) => {
                //
                match (&a.kind, &b.kind) {
                    (PatternKind::Literal(lit_a), PatternKind::Literal(lit_b)) => {
                        return Self::or(
                            Self::literal(format!("{}{}", &lit_a, literal)),
                            Self::literal(format!("{}{}", &lit_b, literal)),
                        );
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        Self::new(PatternKind::Then(Box::new(first), Box::new(second)))
    }

    #[must_use]
    pub fn literal(literal: impl Into<Literal>) -> Self {
        Self::new(PatternKind::Literal(literal.into()))
    }

    #[must_use]
    pub fn or(a: Self, b: Self) -> Self {
        match (&a.kind, &b.kind) {
            (
                PatternKind::Literal(Literal::Char(c_a)),
                PatternKind::Literal(Literal::Char(c_b)),
            ) => {
                let c_a = *c_a;
                let c_b = *c_b;
                return Self::new(PatternKind::CharClosure(CharClosure::new(
                    Ident::new("c", proc_macro2::Span::call_site()),
                    Rc::new(move |ident| quote!(#c_a == #ident || #c_b == #ident)),
                )));
            }
            // (|c: char| { /* some conditions */ }).or('x') => |c: char| { /* some conditions */ || 'x' }
            (PatternKind::CharClosure(closure), PatternKind::Literal(Literal::Char(c)))
            | (PatternKind::Literal(Literal::Char(c)), PatternKind::CharClosure(closure)) => {
                let mut closure = closure.clone();
                let c = *c;

                closure.add(Rc::new(move |ident| quote!( #ident == #c )));

                return Self::new(PatternKind::CharClosure(closure));
            }
            _ => Self::new(PatternKind::Or(Box::new(a), Box::new(b))),
        }
    }

    #[must_use]
    pub fn with_range(mut self, range: ast::Span) -> Self {
        if self.range.is_none() {
            self.range = Some(range);
        }
        self
    }
}

impl ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // TODO: attach range?
        self.kind.to_tokens(tokens);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ToPatternError {
    kind: ToPatternErrorKind,
}

impl ToPatternError {
    pub fn unsupported_class() -> Self {
        Self {
            kind: ToPatternErrorKind::UnsupportedClass,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ToPatternErrorKind {
    UnsupportedClass,
}

impl TryFrom<char> for Pattern {
    type Error = !;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(Self::literal(value))
    }
}

impl TryFrom<ast::Literal> for Pattern {
    type Error = !;

    fn try_from(value: ast::Literal) -> Result<Self, Self::Error> {
        Self::try_from(value.c).map(|pattern| pattern.with_range(value.span))
    }
}

impl TryFrom<ast::Class> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::Class) -> Result<Self, Self::Error> {
        match value {
            // something like \pL or \p{Greek}
            ast::Class::Unicode(class) => class.try_into(),
            // something like \d or \W
            ast::Class::Perl(class) => class.try_into(),
            // [a-zA-Z\pL]
            ast::Class::Bracketed(class) => class.try_into(),
        }
    }
}

impl TryFrom<ast::ClassBracketed> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::ClassBracketed) -> Result<Self, Self::Error> {
        // TODO: negated + span!
        let ast::ClassBracketed {
            span,
            negated,
            kind,
        } = value;

        match &kind {
            ast::ClassSet::Item(item) => item.clone().try_into(),
            ast::ClassSet::BinaryOp(binary_op) => unimplemented!("binary op is not yet supported"),
        }
    }
}

impl TryFrom<ast::ClassSetItem> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::ClassSetItem) -> Result<Self, Self::Error> {
        match value {
            ast::ClassSetItem::Empty(span) => Ok(Pattern::literal("").with_range(span)),
            ast::ClassSetItem::Literal(literal) => Ok(literal.try_into().unwrap()),
            ast::ClassSetItem::Range(range) => Ok(range.try_into().unwrap()),
            _ => Err(ToPatternError::unsupported_class()),
        }
    }
}

impl TryFrom<ast::ClassSetRange> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::ClassSetRange) -> Result<Self, Self::Error> {
        if !value.is_valid() {
            unimplemented!("invalid range");
        }

        let ast::ClassSetRange { span, start, end } = value;

        // TODO: I think this is supposed to be something like this: start..end, where start and end are chars

        unimplemented!()
    }
}

impl TryFrom<ast::ClassPerl> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::ClassPerl) -> Result<Self, Self::Error> {
        let condition: fn(&Ident) -> TokenStream = {
            match &value.kind {
                // \d = [0-9]
                ast::ClassPerlKind::Digit => |ident| quote!( char::is_ascii_digit(&#ident) ),
                // \s = [ \t\n\x0B\f\r]
                ast::ClassPerlKind::Space => |ident| quote!( char::is_whitespace(#ident) ),
                // \w = [a-zA-Z_0-9]
                ast::ClassPerlKind::Word => {
                    |ident| quote!( char::is_ascii_alphanumeric(#ident) || #ident == '_' )
                }
            }
        };

        Ok(Pattern::new(PatternKind::CharClosure(CharClosure::new(
            Ident::new("c", proc_macro2::Span::call_site()),
            Rc::new(condition),
        ))))
    }
}

impl TryFrom<ast::ClassUnicode> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::ClassUnicode) -> Result<Self, Self::Error> {
        dbg!(&value.kind);
        // TODO: span
        // TODO: negated
        match value.kind {
            ast::ClassUnicodeKind::OneLetter(letter) => {
                todo!("one letter kind")
            }
            ast::ClassUnicodeKind::Named(string) => {
                todo!("what about this one?")
            }
            ast::ClassUnicodeKind::NamedValue { op, name, value } => {
                todo!("??")
            }
        }
    }
}

impl TryFrom<ast::Ast> for Pattern {
    type Error = ToPatternError;

    fn try_from(value: ast::Ast) -> Result<Self, Self::Error> {
        match &value {
            // TODO: this should match everything (not sure if "" matches everything?)
            ast::Ast::Empty(span) => Ok(Self::literal("").with_range(*span)),
            ast::Ast::Literal(literal) => Ok(Self::try_from(literal.clone()).unwrap()),
            ast::Ast::Class(class) => Self::try_from(class.clone()),
            ast::Ast::Alternation(alternation) => {
                let mut asts = alternation.asts.clone().into_iter();
                let first = Self::try_from(asts.next().expect("weird alternation?"))?;

                asts.into_iter()
                    .try_fold(first, |acc, ast| Ok(Pattern::or(acc, Self::try_from(ast)?)))
            }
            ast::Ast::Flags(_flags) => unimplemented!("flags are not yet supported"),
            ast::Ast::Concat(concat) => {
                let mut asts = concat.asts.clone().into_iter();
                let first = asts.next().expect("weird concat?").try_into()?;

                asts.try_fold(first, |acc, ast| Ok(Pattern::then(acc, ast.try_into()?)))
            }
            _ => unimplemented!("kind not yet supported!"),
        }
    }
}
