// H²CE Semantic Search Adapter
// Integrates H²CE multi-resolution retrieval with Markovian reasoning

use serde::{Deserialize, Serialize};

#[cfg(feature = "h2ce-integration")]
use anyhow::{Context, Result};

/// H²CE search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2CEConfig {
    /// Enable H²CE integration
    pub enabled: bool,

    /// Index path for H²CE
    #[serde(default = "default_index_path")]
    pub index_path: String,

    /// Corpus path to search
    #[serde(default = "default_corpus_path")]
    pub corpus_path: String,

    /// Maximum number of search results
    #[serde(default = "default_max_results")]
    pub max_results: usize,

    /// Minimum similarity threshold (0.0-1.0)
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,

    /// Resolution level: L0 (atomic), L1 (paragraph), L2 (summary), L4 (document), or "all"
    #[serde(default = "default_resolution_level")]
    pub resolution_level: String,
}

fn default_index_path() -> String {
    ".h2ce_index".to_string()
}

fn default_corpus_path() -> String {
    "./corpus".to_string()
}

fn default_max_results() -> usize {
    5
}

fn default_similarity_threshold() -> f32 {
    0.5
}

fn default_resolution_level() -> String {
    "all".to_string()
}

impl Default for H2CEConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            index_path: default_index_path(),
            corpus_path: default_corpus_path(),
            max_results: default_max_results(),
            similarity_threshold: default_similarity_threshold(),
            resolution_level: default_resolution_level(),
        }
    }
}

/// H²CE search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Matched text chunk
    pub text: String,

    /// Relevance score
    pub score: f32,

    /// Source file path
    pub source: String,

    /// Resolution level (L0, L1, L2, L4)
    pub level: String,

    /// Chunk metadata
    pub metadata: serde_json::Value,
}

/// H²CE Adapter for semantic search during reasoning
#[cfg(feature = "h2ce-integration")]
pub struct H2CEAdapter {
    config: H2CEConfig,
    // Will hold H²CE engine when integrated
    // engine: Option<CachedBM25Engine>,
}

#[cfg(feature = "h2ce-integration")]
impl H2CEAdapter {
    /// Create new H²CE adapter
    pub fn new(config: H2CEConfig) -> Self {
        Self {
            config,
            // engine: None,  // Will initialize in Phase 8
        }
    }

    /// Search corpus with query
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        tracing::debug!(
            "H²CE search: query='{}', top_k={}, level={}",
            query,
            self.config.max_results,
            self.config.resolution_level
        );

        // TODO: Implement actual H²CE search using MCP client or direct engine
        // For now, return empty results (Phase 8 Task 1)

        tracing::warn!("H²CE search not yet implemented - returning empty results");
        Ok(Vec::new())
    }

}

/// Stub implementation when H²CE feature is disabled
#[cfg(not(feature = "h2ce-integration"))]
pub struct H2CEAdapter {
    _config: H2CEConfig,
}

#[cfg(not(feature = "h2ce-integration"))]
impl H2CEAdapter {
    pub fn new(config: H2CEConfig) -> Self {
        Self { _config: config }
    }

    pub async fn search(&self, _query: &str) -> anyhow::Result<Vec<SearchResult>> {
        Ok(Vec::new())
    }
}

/// Format search results for injection into carryover (always available)
impl H2CEAdapter {
    pub fn format_results(results: &[SearchResult]) -> String {
        if results.is_empty() {
            return String::new();
        }

        let mut formatted = String::from("\n\n[SEMANTIC SEARCH RESULTS]\n");

        for (idx, result) in results.iter().enumerate() {
            formatted.push_str(&format!(
                "\nResult {}: (score: {:.2}, level: {}, source: {})\n{}\n",
                idx + 1,
                result.score,
                result.level,
                result.source,
                result.text
            ));
        }

        formatted.push_str("\n[END SEARCH RESULTS]\n\n");
        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h2ce_config_default() {
        let config = H2CEConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_results, 5);
        assert_eq!(config.similarity_threshold, 0.5);
    }

    #[test]
    fn test_format_results_empty() {
        let results: Vec<SearchResult> = vec![];
        let formatted = H2CEAdapter::format_results(&results);
        assert!(formatted.is_empty());
    }

    #[test]
    fn test_format_results() {
        let results = vec![
            SearchResult {
                text: "Test content 1".to_string(),
                score: 0.95,
                source: "file1.rs".to_string(),
                level: "L1".to_string(),
                metadata: serde_json::json!({}),
            },
            SearchResult {
                text: "Test content 2".to_string(),
                score: 0.85,
                source: "file2.rs".to_string(),
                level: "L0".to_string(),
                metadata: serde_json::json!({}),
            },
        ];

        let formatted = H2CEAdapter::format_results(&results);
        assert!(formatted.contains("[SEMANTIC SEARCH RESULTS]"));
        assert!(formatted.contains("Result 1"));
        assert!(formatted.contains("Result 2"));
        assert!(formatted.contains("score: 0.95"));
        assert!(formatted.contains("file1.rs"));
    }
}
