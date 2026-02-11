pub mod language;
pub mod namespace;
pub mod scope_resolution;
pub mod utils;

pub use {
    language::{Language, MemoizedQuery, TSLanguage, TSLanguageConfig, ALL_LANGUAGES},
    namespace::*,
    scope_resolution::{NodeKind, ScopeGraph},
};

pub use core::definition::Definition;

use core::code_chunk::CodeChunkMiddle;
use scope_resolution::ResolutionMethod;
use std::collections::BTreeSet;
use tree_sitter::{Parser, Tree};

/// A tree-sitter representation of a file
pub struct TreeSitterFile<'a> {
    /// The original source that was used to generate this file.
    src: &'a [u8],

    /// The syntax tree of this file.
    tree: Tree,

    /// The supplied language for this file.
    language: &'static TSLanguageConfig,
}

#[derive(Debug)]
pub enum TreeSitterFileError {
    UnsupportedLanguage,
    ParseTimeout,
    LanguageMismatch,
    QueryError(tree_sitter::QueryError),
    FileTooLarge,
}

impl<'a> TreeSitterFile<'a> {
    /// Create a TreeSitterFile out of a sourcefile
    pub fn try_build(src: &'a [u8], lang_id: &str) -> Result<Self, TreeSitterFileError> {
        // no scope-res for files larger than 500kb
        if src.len() > 500 * 10usize.pow(3) {
            return Err(TreeSitterFileError::FileTooLarge);
        }

        let language = match TSLanguage::from_id(lang_id) {
            Language::Supported(language) => Ok(language),
            Language::Unsupported => Err(TreeSitterFileError::UnsupportedLanguage),
        }?;
        let mut parser = Parser::new();
        parser
            .set_language((language.grammar)())
            .map_err(|_| TreeSitterFileError::LanguageMismatch)?;

        // do not permit files that take >1s to parse
        parser.set_timeout_micros(10u64.pow(6));

        let tree = parser
            .parse(src, None)
            .ok_or(TreeSitterFileError::ParseTimeout)?;

        Ok(Self {
            src,
            tree,
            language,
        })
    }

    pub fn hoverable_ranges(
        self,
    ) -> Result<Vec<core::text_range::TextRange>, TreeSitterFileError> {
        let query = self
            .language
            .hoverable_query
            .query(self.language.grammar)
            .map_err(TreeSitterFileError::QueryError)?;
        let root_node = self.tree.root_node();
        let mut cursor = tree_sitter::QueryCursor::new();
        Ok(cursor
            .matches(query, root_node, self.src)
            .flat_map(|m| m.captures)
            .map(|c| c.node.range().into())
            .collect::<Vec<_>>())
    }

    /// Produce a lexical scope-graph for this TreeSitterFile.
    pub fn scope_graph(self) -> Result<ScopeGraph, TreeSitterFileError> {
        let query = self
            .language
            .scope_query
            .query(self.language.grammar)
            .map_err(TreeSitterFileError::QueryError)?;
        let root_node = self.tree.root_node();

        Ok(ResolutionMethod::Generic.build_scope(query, root_node, self.src, self.language))
    }

    pub fn filemap(self) -> Result<String, TreeSitterFileError> {
        let root_node = self.tree.root_node();
        (self.language.filemap)(&root_node, self.src, None).map_err(TreeSitterFileError::QueryError)
    }

    pub fn get_definition(
        self,
        rows: &[usize],
        paths: &str,
    ) -> Result<(String, Vec<Definition>), TreeSitterFileError> {
        let root_node = self.tree.root_node();
        (self.language.get_definition)(&root_node, self.src, rows, paths)
            .map_err(TreeSitterFileError::QueryError)
    }

    pub fn get_completed_body_by_chunks(
        self,
        chunks: &Vec<CodeChunkMiddle>,
        result: &mut String,
    ) -> Result<(), TreeSitterFileError> {
        let _definition_symbols = BTreeSet::<String>::new();
        (self.language.complete_chunks)(&self.tree.root_node(), self.src, chunks, result)
            .map_err(TreeSitterFileError::QueryError)
    }
}
