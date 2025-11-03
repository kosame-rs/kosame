mod delete;
mod insert;
mod select;
mod update;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Attribute, Path,
    parse::{Parse, ParseStream},
};

pub use delete::*;
pub use insert::*;
pub use select::*;
pub use update::*;

use crate::{
    clause::{Fields, FromItem, With},
    keyword,
    part::TableAlias,
    quote_option::QuoteOption,
    scope_module::ScopeModule,
    visitor::Visitor,
};

pub struct Command {
    pub attrs: Vec<Attribute>,
    pub with: Option<With>,
    pub command_type: CommandType,
}

impl Command {
    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        visitor.visit_command(self);
        {
            if let Some(inner) = &self.with {
                inner.accept(visitor)
            }
            self.command_type.accept(visitor);
        }
        visitor.end_command();
    }

    pub fn fields(&self) -> Option<&Fields> {
        self.command_type.fields()
    }
}

impl Parse for Command {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            with: input.call(With::parse_optional)?,
            command_type: input.parse()?,
        })
    }
}

impl ToTokens for Command {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let with = QuoteOption::from(&self.with);
        let command_type = &self.command_type;

        let scope_module = ScopeModule::new(self);

        quote! {
            {
                const command: ::kosame::repr::command::Command<'static> = ::kosame::repr::command::Command::new(#with, #command_type);

                #scope_module

                command
            }
        }.to_tokens(tokens);
    }
}

pub enum CommandType {
    Delete(Delete),
    Insert(Insert),
    Select(Box<Select>),
    Update(Update),
}

impl CommandType {
    pub fn peek(input: ParseStream) -> bool {
        Delete::peek(input) || Insert::peek(input) || Select::peek(input) || Update::peek(input)
    }

    pub fn fields(&self) -> Option<&Fields> {
        match self {
            Self::Delete(..) => None,
            Self::Insert(..) => None,
            Self::Select(inner) => Some(&inner.select.fields),
            Self::Update(..) => None,
        }
    }

    pub fn accept<'a>(&'a self, visitor: &mut impl Visitor<'a>) {
        match self {
            Self::Delete(inner) => inner.accept(visitor),
            Self::Insert(inner) => inner.accept(visitor),
            Self::Select(inner) => inner.accept(visitor),
            Self::Update(inner) => inner.accept(visitor),
        }
    }

    pub fn table(&self) -> Option<(&Path, Option<&TableAlias>)> {
        match self {
            Self::Delete(delete) => Some((&delete.table, None)),
            Self::Insert(insert) => Some((&insert.table, None)),
            Self::Select(..) => None,
            Self::Update(update) => Some((&update.table, None)),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_item(&self) -> Option<&FromItem> {
        match self {
            Self::Delete(delete) => delete.using.as_ref().map(|using| &using.item),
            Self::Insert(..) => None,
            Self::Select(select) => select.from.as_ref().map(|from| &from.item),
            Self::Update(update) => update.from.as_ref().map(|from| &from.item),
        }
    }
}

impl Parse for CommandType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if Delete::peek(input) {
            Ok(Self::Delete(input.parse()?))
        } else if Insert::peek(input) {
            Ok(Self::Insert(input.parse()?))
        } else if Select::peek(input) {
            Ok(Self::Select(input.parse()?))
        } else if Update::peek(input) {
            Ok(Self::Update(input.parse()?))
        } else {
            keyword::group_command::error(input);
        }
    }
}

impl ToTokens for CommandType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Delete(inner) => quote! {
                ::kosame::repr::command::CommandType::Delete(#inner)
            },
            Self::Insert(inner) => quote! {
                ::kosame::repr::command::CommandType::Insert(#inner)
            },
            Self::Select(inner) => quote! {
                ::kosame::repr::command::CommandType::Select(#inner)
            },
            Self::Update(inner) => quote! {
                ::kosame::repr::command::CommandType::Update(#inner)
            },
        }
        .to_tokens(tokens)
    }
}
