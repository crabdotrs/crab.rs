use tree_sitter_language::LanguageFn;
extern "C" {
    fn tree_sitter_crab() -> *const ();
}
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_crab) };
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");
#[cfg(with_highlights_query)]
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");
#[cfg(with_injections_query)]
pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");
#[cfg(with_locals_query)]
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");
#[cfg(with_tags_query)]
pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");
#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE.into())
            .expect("Error loading Crab parser");
    }
}
