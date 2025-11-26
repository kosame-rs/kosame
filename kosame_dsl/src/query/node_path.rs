use convert_case::Casing;
use proc_macro2::Span;
use syn::{Ident, Path, PathSegment};

#[derive(Default, Clone)]
pub struct QueryNodePath {
    pub segments: Vec<Ident>,
}

impl QueryNodePath {
    #[must_use] 
    pub fn new() -> Self {
        Self { segments: vec![] }
    }

    pub fn append(&mut self, segment: Ident) {
        self.segments.push(segment);
    }

    #[must_use] 
    pub fn appended(mut self, segment: Ident) -> Self {
        self.append(segment);
        self
    }

    #[must_use] 
    pub fn to_struct_name(&self, prefix: &str) -> Ident {
        let mut struct_name = prefix.to_string();
        for segment in &self.segments {
            struct_name += &segment.to_string().to_case(convert_case::Case::Pascal);
        }
        Ident::new(&struct_name, Span::call_site())
    }

    #[must_use] 
    pub fn to_module_name(&self, prefix: &str) -> Ident {
        let mut module_name = prefix.to_string();
        for segment in &self.segments {
            module_name += "_";
            module_name += &segment.to_string();
        }
        Ident::new(&module_name, Span::call_site())
    }

    #[must_use] 
    pub fn resolve(&self, root_table: &Path) -> Path {
        let mut path = root_table.clone();
        for segment in &self.segments {
            path.segments
                .push(Ident::new("relations", Span::call_site()).into());
            path.segments.push(PathSegment::from(segment.clone()));
            path.segments
                .push(Ident::new("target_table", Span::call_site()).into());
        }
        path
    }
}
