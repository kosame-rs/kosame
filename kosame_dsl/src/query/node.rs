use crate::clause::peek_clause;
use crate::pretty::{BreakMode, Delim, PrettyPrint, Printer};
use crate::{
    clause::{Limit, Offset, OrderBy, Where},
    parse_option::ParseOption,
    quote_option::QuoteOption,
    row::Row,
    visit::Visit,
};

use super::star::Star;
use super::{CorrelationId, Field, Ident, PathExt, Query, QueryNodePath, ScopeId};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Path, PathSegment, Token, braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

pub struct Node {
    pub correlation_id: CorrelationId,
    pub scope_id: ScopeId,
    pub brace_token: syn::token::Brace,
    pub star: Option<Star>,
    pub fields: Punctuated<Field, Token![,]>,
    pub r#where: Option<Where>,
    pub order_by: Option<OrderBy>,
    pub limit: Option<Limit>,
    pub offset: Option<Offset>,
}

impl Node {
    pub fn to_row_tokens(
        &self,
        tokens: &mut TokenStream,
        query: &Query,
        node_path: &QueryNodePath,
    ) {
        let table_path = node_path.resolve(query.table.as_path());
        tokens.extend(self.to_autocomplete_module_tokens(
            node_path.to_module_name("autocomplete_row"),
            &table_path,
        ));

        let row = {
            let table_path = table_path.to_call_site(1);

            let star_field = self
                .star
                .as_ref()
                .and_then(|star| star.alias.is_some().then(|| star.to_row_field(&table_path)));

            Row::new(
                query.outer_attrs.clone(),
                node_path.to_struct_name("Row"),
                star_field
                    .into_iter()
                    .chain(
                        self.fields
                            .iter()
                            .map(|field| field.to_row_field(&table_path, node_path)),
                    )
                    .collect(),
            )
        };

        if let Some(star) = &self.star
            && star.alias.is_none()
        {
            let table_path = table_path.to_call_site(1);
            quote! {
                #table_path::star! {
                    (#table_path)
                    #row
                }
            }
            .to_tokens(tokens);
        } else {
            row.to_tokens(tokens);
        }

        // Recursively call to_tokens on child nodes.
        for field in &self.fields {
            if let Field::Relation { name, node, .. } = field {
                let mut node_path = node_path.clone();
                node_path.append(name.clone());
                node.to_row_tokens(tokens, query, &node_path);
            }
        }
    }

    fn to_autocomplete_module_tokens(
        &self,
        module_name: impl ToTokens,
        table_path: &Path,
    ) -> TokenStream {
        let table_path = table_path.to_call_site(2);
        let mut module_rows = vec![];

        for field in &self.fields {
            let name = match field {
                Field::Column { name, .. } => name,
                Field::Relation { name, .. } => name,
                Field::Expr { .. } => continue,
            };
            module_rows.push(quote! {
                use #table_path::columns_and_relations::#name;
            });
        }

        quote! {
            mod #module_name {
                #(#module_rows)*
            }
        }
    }

    pub fn to_query_node_tokens(
        &self,
        tokens: &mut TokenStream,
        query: &Query,
        node_path: &QueryNodePath,
    ) {
        self.scope_id.scope(|| {
            let table_path = node_path.resolve(query.table.as_path());
            let table_path_call_site = table_path.to_call_site(1);

            let mut fields = vec![];
            for field in &self.fields {
                match field {
                    Field::Column { name, alias, .. } => {
                        let alias = QuoteOption::from(alias);
                        fields.push(quote! {
                            ::kosame::repr::query::Field::Column {
                                column: &#table_path_call_site::columns::#name::COLUMN,
                                alias: #alias
                            }
                        });
                    }
                    Field::Relation {
                        name, node, alias, ..
                    } => {
                        let alias = QuoteOption::from(alias);

                        let node_path = node_path.clone().appended(name.clone());

                        let mut relation_path = table_path.clone();
                        relation_path
                            .segments
                            .push(Ident::new("relations", Span::call_site()).into());
                        relation_path.segments.push(PathSegment::from(name.clone()));

                        let mut tokens = TokenStream::new();
                        node.to_query_node_tokens(&mut tokens, query, &node_path);

                        let relation_path = relation_path.to_call_site(1);

                        fields.push(quote! {
                            ::kosame::repr::query::Field::Relation {
                                relation: &#relation_path::RELATION,
                                node: #tokens,
                                alias: #alias
                            }
                        });
                    }
                    Field::Expr { expr, alias, .. } => {
                        let alias = alias.ident.to_string();

                        fields.push(quote! {
                            ::kosame::repr::query::Field::Expr {
                                expr: #expr,
                                alias: #alias
                            }
                        });
                    }
                }
            }

            let star = self.star.is_some();

            let r#where = QuoteOption::from(&self.r#where);
            let order_by = QuoteOption::from(&self.order_by);
            let limit = QuoteOption::from(&self.limit);
            let offset = QuoteOption::from(&self.offset);

            quote! {
                ::kosame::repr::query::Node::new(
                    &#table_path_call_site::TABLE,
                    #star,
                    &[#(#fields),*],
                    #r#where,
                    #order_by,
                    #limit,
                    #offset,
                )
            }
            .to_tokens(tokens);
        });
    }
}

pub fn visit_node<'a>(visit: &mut (impl Visit<'a> + ?Sized), node: &'a Node) {
    for field in &node.fields {
        match field {
            Field::Relation { node, .. } => visit.visit_node(node),
            Field::Expr { expr, .. } => visit.visit_expr(expr),
            Field::Column { .. } => {}
        }
    }

    if let Some(inner) = node.r#where.as_ref() {
        visit.visit_where(inner);
    }
    if let Some(inner) = node.order_by.as_ref() {
        visit.visit_order_by(inner);
    }
    if let Some(inner) = node.limit.as_ref() {
        visit.visit_limit(inner);
    }
    if let Some(inner) = node.offset.as_ref() {
        visit.visit_offset(inner);
    }
}

impl Parse for Node {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let brace_token = braced!(content in input);

        let star = if content.fork().parse::<Star>().is_ok() {
            let star = Some(content.parse()?);
            if !content.is_empty() {
                let _: Token![,] = content.parse()?;
            }
            star
        } else {
            None
        };

        let mut fields = Punctuated::<Field, _>::new();
        while !content.is_empty() {
            if peek_clause(&content) {
                break;
            }

            fields.push(content.parse()?);

            if !content.peek(Token![,]) {
                break;
            }
            fields.push_punct(content.parse()?);
        }

        let mut existing = vec![];
        for field in &fields {
            let name = field.name();

            if field.is_column() && star.is_some() {
                return Err(syn::Error::new(
                    field.span(),
                    "column references are not allowed after `*`",
                ));
            }

            let name_string = field.alias().map_or(name, |alias| &alias.ident).to_string();
            if existing.contains(&name_string) {
                return Err(syn::Error::new(
                    field.span(),
                    format!("duplicate field `{name_string}`"),
                ));
            }
            existing.push(name_string);
        }

        Ok(Self {
            correlation_id: CorrelationId::new(),
            scope_id: ScopeId::new(),
            brace_token,
            star,
            fields,
            r#where: content.call(Where::parse_option)?,
            order_by: content.call(OrderBy::parse_option)?,
            limit: content.call(Limit::parse_option)?,
            offset: content.call(Offset::parse_option)?,
        })
    }
}

impl PrettyPrint for Node {
    fn pretty_print(&self, printer: &mut Printer<'_>) {
        self.brace_token
            .pretty_print(printer, Some(BreakMode::Consistent), |printer| {
                self.star.pretty_print(printer);

                if self.star.is_some()
                    && (!self.fields.is_empty()
                        || self.r#where.is_some()
                        || self.order_by.is_some()
                        || self.limit.is_some()
                        || self.offset.is_some())
                {
                    ",".pretty_print(printer);
                    printer.scan_same_line_trivia();
                    printer.scan_break();
                    " ".pretty_print(printer);
                    printer.scan_trivia(true, true);
                }

                self.fields.pretty_print(printer);
                self.r#where.pretty_print(printer);
                self.order_by.pretty_print(printer);
                self.limit.pretty_print(printer);
                self.offset.pretty_print(printer);
            });
    }
}
