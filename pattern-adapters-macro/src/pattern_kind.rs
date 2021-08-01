use std::borrow::Cow;
use std::fmt::{self, Debug, Write};
use std::rc::Rc;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Ident;

use crate::pattern::Pattern;

pub trait Closure: Fn(&Ident) -> TokenStream {}

impl<F> Closure for F where F: Fn(&Ident) -> TokenStream {}

#[derive(Clone)]
pub struct CharClosure {
    pub ident: Ident,
    first_condition: Rc<dyn Closure>,
    conditions: Vec<Rc<dyn Closure>>,
}

impl fmt::Debug for CharClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CharClosure")
            .field("ident", &self.ident)
            .finish()
    }
}

impl CharClosure {
    #[must_use]
    pub fn new(ident: Ident, first_condition: Rc<dyn Closure>) -> Self {
        Self {
            ident,
            first_condition,
            conditions: Vec::new(),
        }
    }

    pub fn add(&mut self, condition: Rc<dyn Closure>) -> &mut Self {
        self.conditions.push(condition);
        self
    }
}

impl ToTokens for CharClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let conditions = self
            .conditions
            .iter()
            .map(|condition| (*condition)(&self.ident));

        let ident = &self.ident;
        let first = (self.first_condition)(&self.ident);

        tokens.append_all(quote!((|#ident: char| { (#first) #(|| (#conditions))* })));
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Char(char),
    String(Cow<'static, str>),
}

impl From<char> for Literal {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<&'static str> for Literal {
    fn from(value: &'static str) -> Self {
        Self::String(value.into())
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Char(c) => f.write_char(*c),
            Self::String(string) => f.write_str(string),
        }
    }
}

impl ToTokens for Literal {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match &self {
            Self::Char(c) => {
                tokens.append_all(quote!(#c));
            }
            Self::String(string) => {
                let string = &**string;
                tokens.append_all(&[string]);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    Literal(Literal),
    CharClosure(CharClosure),
    Then(Box<Pattern>, Box<Pattern>),
    Or(Box<Pattern>, Box<Pattern>),
}

impl ToTokens for PatternKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Literal(literal) => literal.to_tokens(tokens),
            Self::CharClosure(closure) => closure.to_tokens(tokens),
            Self::Then(first, second) => {
                tokens.append_all(
                    quote!(::pattern_adapters::adapters::ThenPattern::new(#first, #second)),
                );
            }
            Self::Or(first, second) => {
                tokens.append_all(
                    quote!(::pattern_adapters::adapters::OrPattern::new(#first, #second)),
                );
            }
        }
    }
}
