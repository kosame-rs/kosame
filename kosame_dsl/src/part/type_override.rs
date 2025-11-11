use syn::{
    Path, Token,
    parse::{Parse, ParseBuffer, ParseStream},
};

pub struct TypeOverride {
    pub _colon: Token![:],
    pub type_path: Path,
}

impl TypeOverride {
    pub fn parse_optional(input: ParseStream) -> syn::Result<Option<Self>> {
        Self::peek(input).then(|| input.parse()).transpose()
    }

    pub fn peek(input: &ParseBuffer<'_>) -> bool {
        input.peek(Token![:])
    }
}

impl Parse for TypeOverride {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            _colon: input.parse()?,
            type_path: input.parse()?,
        })
    }
}
