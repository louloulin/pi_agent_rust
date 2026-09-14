//! Pure search-index primitives shared by Pi index facades.
//!
//! This crate deliberately contains no filesystem, SQLite, network, or agent
//! integration. Hosts provide documents and retain their own public result
//! types while using these primitives for tokenization and relevance.

use std::collections::{BTreeMap, BTreeSet};

/// A normalized query token.
pub type Token = String;

/// Split a query into case-insensitive whitespace-delimited tokens.
#[must_use]
pub fn tokenize(query: &str) -> Vec<Token> {
    query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

/// Parse a query and return its unique terms in stable sorted order.
#[must_use]
pub fn parse_query(query: &str) -> Vec<Token> {
    tokenize(query)
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// A compact inverted index from token to document ids.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvertedIndex {
    postings: BTreeMap<Token, BTreeSet<usize>>,
}

impl InvertedIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_document<I, S>(&mut self, document_id: usize, terms: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for term in terms {
            self.postings
                .entry(term.as_ref().to_ascii_lowercase())
                .or_default()
                .insert(document_id);
        }
    }

    #[must_use]
    pub fn matching_documents(&self, query: &str) -> BTreeSet<usize> {
        let tokens = parse_query(query);
        let mut matches = BTreeSet::new();
        for token in tokens {
            if let Some(documents) = self.postings.get(&token) {
                matches.extend(documents);
            }
        }
        matches
    }

    #[must_use]
    pub fn postings(&self, token: &str) -> Option<&BTreeSet<usize>> {
        self.postings.get(&token.to_ascii_lowercase())
    }
}

/// Score one document using field-specific weights. Each matching query token
/// contributes at most once per field.
#[must_use]
pub fn relevance_score<'a, I>(query: &str, fields: I) -> i64
where
    I: IntoIterator<Item = (&'a str, i64)>,
{
    let tokens = tokenize(query);
    fields
        .into_iter()
        .map(|(value, weight)| {
            let value = value.to_ascii_lowercase();
            tokens
                .iter()
                .filter(|token| value.contains(token.as_str()))
                .count() as i64
                * weight
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenization_is_case_insensitive_and_ignores_whitespace() {
        assert_eq!(tokenize(" Alpha   BETA "), ["alpha", "beta"]);
    }

    #[test]
    fn query_parser_deduplicates_terms() {
        assert_eq!(parse_query("alpha ALPHA beta"), ["alpha", "beta"]);
    }

    #[test]
    fn inverted_index_returns_union_of_matching_postings() {
        let mut index = InvertedIndex::new();
        index.add_document(1, ["alpha", "beta"]);
        index.add_document(2, ["beta"]);
        assert_eq!(
            index.matching_documents("alpha beta"),
            BTreeSet::from([1, 2])
        );
    }

    #[test]
    fn relevance_counts_each_token_once_per_field() {
        assert_eq!(
            relevance_score("foo bar", [("foo foo", 300), ("bar", 60)]),
            360
        );
    }
}
