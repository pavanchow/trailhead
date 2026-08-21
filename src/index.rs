//! The inverted index: term -> postings list, plus TF-IDF scoring.

use crate::tokenizer::tokenize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// One occurrence of a term in a document: which doc and how many times.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Posting {
    pub doc_id: usize,
    pub term_freq: u32,
}

/// Metadata kept per indexed document.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DocInfo {
    pub path: String,
    pub token_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Index {
    pub postings: HashMap<String, Vec<Posting>>,
    pub docs: Vec<DocInfo>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            postings: HashMap::new(),
            docs: Vec::new(),
        }
    }

    /// Adds one document's text under the given path label.
    pub fn add_document(&mut self, path: &str, text: &str) {
        let tokens = tokenize(text, true);
        let doc_id = self.docs.len();
        self.docs.push(DocInfo {
            path: path.to_string(),
            token_count: tokens.len() as u32,
        });

        let mut counts: HashMap<String, u32> = HashMap::new();
        for tok in tokens {
            *counts.entry(tok).or_insert(0) += 1;
        }
        for (term, term_freq) in counts {
            self.postings
                .entry(term)
                .or_insert_with(Vec::new)
                .push(Posting { doc_id, term_freq });
        }
    }

    /// Walks a directory (non-recursive) for .txt/.md files and indexes each.
    pub fn build_from_dir(dir: &Path) -> io::Result<Index> {
        let mut index = Index::new();
        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && matches!(
                        p.extension().and_then(|e| e.to_str()),
                        Some("txt") | Some("md")
                    )
            })
            .collect();
        entries.sort();

        for path in entries {
            let text = fs::read_to_string(&path)?;
            index.add_document(&path.to_string_lossy(), &text);
        }
        Ok(index)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    pub fn load(path: &Path) -> io::Result<Index> {
        let json = fs::read_to_string(path)?;
        let index = serde_json::from_str(&json)?;
        Ok(index)
    }

    fn doc_frequency(&self, term: &str) -> usize {
        self.postings.get(term).map(|p| p.len()).unwrap_or(0)
    }

    /// idf = ln(N / df), with df=0 terms contributing zero score.
    fn idf(&self, term: &str) -> f64 {
        let n = self.docs.len();
        let df = self.doc_frequency(term);
        if n == 0 || df == 0 {
            return 0.0;
        }
        (n as f64 / df as f64).ln()
    }

    /// tf-idf for a single term in a single doc, tf normalized by doc length.
    fn tfidf_for_doc(&self, term: &str, doc_id: usize) -> f64 {
        let doc_len = self
            .docs
            .get(doc_id)
            .map(|d| d.token_count.max(1))
            .unwrap_or(1) as f64;
        let term_freq = self
            .postings
            .get(term)
            .and_then(|postings| postings.iter().find(|p| p.doc_id == doc_id))
            .map(|p| p.term_freq)
            .unwrap_or(0) as f64;
        if term_freq == 0.0 {
            return 0.0;
        }
        let tf = term_freq / doc_len;
        tf * self.idf(term)
    }

    /// Ranks documents by the sum of per-term TF-IDF scores over the query.
    /// Returns up to `top_n` (doc path, score) pairs, highest score first.
    pub fn search(&self, query: &str, top_n: usize) -> Vec<(String, f64)> {
        let terms = tokenize(query, true);
        let mut scores: HashMap<usize, f64> = HashMap::new();

        for term in &terms {
            if let Some(postings) = self.postings.get(term) {
                for posting in postings {
                    let score = self.tfidf_for_doc(term, posting.doc_id);
                    *scores.entry(posting.doc_id).or_insert(0.0) += score;
                }
            }
        }

        let mut ranked: Vec<(String, f64)> = scores
            .into_iter()
            .filter(|(_, score)| *score > 0.0)
            .map(|(doc_id, score)| (self.docs[doc_id].path.clone(), score))
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked.truncate(top_n);
        ranked
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_index() -> Index {
        let mut index = Index::new();
        index.add_document("doc0.txt", "the quick brown fox jumps over the lazy dog");
        index.add_document("doc1.txt", "fox fox fox everywhere in the forest");
        index.add_document("doc2.txt", "completely unrelated text about oceans");
        index
    }

    #[test]
    fn query_returns_expected_documents() {
        let index = sample_index();
        let results = index.search("fox", 10);
        let paths: Vec<&str> = results.iter().map(|(p, _)| p.as_str()).collect();
        assert!(paths.contains(&"doc0.txt"));
        assert!(paths.contains(&"doc1.txt"));
        assert!(!paths.contains(&"doc2.txt"));
    }

    #[test]
    fn more_occurrences_of_rarer_term_rank_higher() {
        let index = sample_index();
        let results = index.search("fox", 10);
        assert_eq!(results[0].0, "doc1.txt");
        assert!(results[0].1 > results[1].1);
    }

    #[test]
    fn absent_term_returns_no_results() {
        let index = sample_index();
        let results = index.search("xylophone", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn multi_term_query_combines_scores() {
        let index = sample_index();
        let single = index.search("fox", 10);
        let combined = index.search("fox forest", 10);

        let doc1_single = single.iter().find(|(p, _)| p == "doc1.txt").unwrap().1;
        let doc1_combined = combined.iter().find(|(p, _)| p == "doc1.txt").unwrap().1;
        assert!(doc1_combined > doc1_single);
    }

    #[test]
    fn save_and_load_round_trip() {
        let index = sample_index();
        let tmp = std::env::temp_dir().join("trailhead_test_index.json");
        index.save(&tmp).unwrap();
        let loaded = Index::load(&tmp).unwrap();
        assert_eq!(index, loaded);
        let _ = std::fs::remove_file(&tmp);
    }
}
