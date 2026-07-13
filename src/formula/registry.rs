//! # FormulaRegistry — loading, storing, and querying primitive formulas
//!
//! The registry loads formulas from TOML files (atomic primitives only)
//! and provides search by keyword, domain, and semantic similarity.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};

use crate::wheel::Domain;

use super::{Formula, FormulaError};

// ─── TF-IDF Semantic Search Index ───────────────────────────────────────────

/// Lightweight TF-IDF index for formula search.
///
/// Deterministic, zero external dependencies. Uses log-normalized term frequency
/// and inverse document frequency with cosine similarity scoring.
#[derive(Debug, Clone)]
pub struct TfIdfIndex {
    /// Number of indexed documents (formulas).
    num_docs: usize,
    /// Document frequency: term → number of formulas containing it.
    doc_freq: HashMap<String, usize>,
    /// Pre-computed IDF values: term → idf.
    idf: HashMap<String, f64>,
    /// TF-IDF vectors per formula: formula_id → {term → weight}.
    vectors: HashMap<String, HashMap<String, f64>>,
    /// Pre-computed vector magnitudes for cosine similarity.
    magnitudes: HashMap<String, f64>,
}

/// Build searchable text from a formula (space-separated fields).
fn formula_searchable_text(formula: &Formula) -> String {
    format!(
        "{} {} {} {} {} {}",
        formula.id,
        formula.description,
        formula.expression,
        formula.output,
        formula.inputs.join(" "),
        formula.zodiac.join(" "),
    )
}

impl TfIdfIndex {
    fn new() -> Self {
        TfIdfIndex {
            num_docs: 0,
            doc_freq: HashMap::new(),
            idf: HashMap::new(),
            vectors: HashMap::new(),
            magnitudes: HashMap::new(),
        }
    }

    /// Build a fresh index from a formula map.
    fn built_from(formulas: &HashMap<String, Formula>) -> Self {
        let mut idx = Self::new();
        idx.build(formulas);
        idx
    }

    /// Tokenize text into lowercase terms (split on non-alphanumeric).
    /// Filters out empty and single-character terms.
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty() && t.len() >= 2)
            .map(|t| t.to_string())
            .collect()
    }

    /// Compute sublinear term frequency: 1 + log10(count).
    fn term_frequency(terms: &[String]) -> HashMap<String, f64> {
        let mut tf = HashMap::new();
        for t in terms {
            *tf.entry(t.clone()).or_insert(0.0) += 1.0;
        }
        for v in tf.values_mut() {
            *v = 1.0 + f64::log10(*v);
        }
        tf
    }

    /// Build the index from a formula map.
    fn build(&mut self, formulas: &HashMap<String, Formula>) {
        self.num_docs = formulas.len();
        self.doc_freq.clear();
        self.vectors.clear();
        self.magnitudes.clear();

        if self.num_docs == 0 {
            return;
        }

        // Collect term frequencies and document frequencies
        let mut all_tf: HashMap<String, HashMap<String, f64>> = HashMap::new();
        let mut seen_terms: HashMap<String, HashSet<String>> = HashMap::new();

        for (fid, formula) in formulas {
            let text = formula_searchable_text(formula);
            let terms = Self::tokenize(&text);
            let tf = Self::term_frequency(&terms);
            all_tf.insert(fid.clone(), tf);

            let unique: HashSet<String> = terms.into_iter().collect();
            seen_terms.insert(fid.clone(), unique);
        }

        // Compute document frequency for each term: one pass over each
        // document's unique-term set (O(total terms) instead of
        // O(vocabulary × documents)).
        for terms in seen_terms.values() {
            for t in terms {
                *self.doc_freq.entry(t.clone()).or_insert(0) += 1;
            }
        }

        // Compute smoothed IDF.
        let n = self.num_docs as f64;
        for (t, df) in &self.doc_freq {
            let idf = (1.0 + n / *df as f64).log10();
            self.idf.insert(t.clone(), idf);
        }

        // Compute TF-IDF vectors and magnitudes
        for fid in formulas.keys() {
            let mut vec = HashMap::new();
            let mut mag_sq = 0.0;
            if let Some(tf) = all_tf.get(fid) {
                for (t, tf_val) in tf {
                    let idf = self.idf.get(t).copied().unwrap_or(0.0);
                    let weight = tf_val * idf;
                    vec.insert(t.clone(), weight);
                    mag_sq += weight * weight;
                }
            }
            self.vectors.insert(fid.clone(), vec);
            self.magnitudes.insert(fid.clone(), mag_sq.sqrt());
        }
    }

    /// Search formulas by cosine similarity to query text.
    /// Returns (formula_id, score) pairs sorted by descending score.
    pub fn search(&self, query: &str, top_n: usize) -> Vec<(String, f64)> {
        if self.num_docs == 0 {
            return Vec::new();
        }

        let query_terms = Self::tokenize(query);
        if query_terms.is_empty() {
            return Vec::new();
        }

        let q_tf = Self::term_frequency(&query_terms);
        let mut q_vec = HashMap::new();
        let mut q_mag_sq = 0.0;
        for (t, tf_val) in &q_tf {
            let idf = self.idf.get(t).copied().unwrap_or(0.0);
            let weight = tf_val * idf;
            q_vec.insert(t.clone(), weight);
            q_mag_sq += weight * weight;
        }
        let q_mag = q_mag_sq.sqrt();
        if q_mag == 0.0 {
            return Vec::new();
        }

        let mut scores: Vec<(String, f64)> = Vec::new();
        for (fid, vec) in &self.vectors {
            let mag = self.magnitudes.get(fid).copied().unwrap_or(0.0);
            if mag == 0.0 {
                continue;
            }
            let mut dot = 0.0;
            for (t, qw) in &q_vec {
                if let Some(fw) = vec.get(t) {
                    dot += qw * fw;
                }
            }
            let sim = dot / (q_mag * mag);
            if sim > 0.0 {
                scores.push((fid.clone(), sim));
            }
        }

        scores.sort_by(|a, b| {
            // Determinism: ties broken by formula ID (ascending) so the
            // output order is identical across process runs despite
            // `self.vectors` being a `HashMap` with `RandomState` seed.
            // Without this tie-break, two formulas with the same score
            // would appear in HashMap iteration order, which varies per
            // process. See the determinism audit (FIXES.md).
            b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0))
        });
        scores.truncate(top_n);
        scores
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.num_docs == 0
    }
}

// ─── TOML deserialization types ──────────────────────────────────────────

/// Intermediate struct for loading formulas from TOML.
#[derive(serde::Deserialize)]
struct FormulaToml {
    formula: Option<Vec<FormulaEntry>>,
}

#[derive(serde::Deserialize, Clone)]
struct FormulaEntry {
    id: String,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    formula_type: Option<String>,
    #[serde(default)]
    inputs: Vec<String>,
    #[serde(default)]
    output: String,
    #[serde(default)]
    expression: String,
    #[serde(default)]
    description: String,
    /// Optional evidence/provenance for this formula.
    #[serde(default)]
    evidence: Option<String>,
    /// Tags/zodiac keywords for domain classification and search indexing.
    /// TOML files may use `tags` (legacy) or `zodiac` (new) — both are accepted.
    #[serde(default, alias = "tags")]
    zodiac: Vec<String>,
    /// Source domain for bridging formulas (TOML `from` field).
    #[serde(default)]
    from: Option<String>,
    /// Target domain for bridging formulas (TOML `to` field).
    #[serde(default)]
    to: Option<String>,
    /// Aspect between domains for bridging formulas (TOML `aspect` field).
    #[serde(default)]
    aspect: Option<String>,
}

// ─── Synonym TOML deserialization types ────────────────────────────────

/// Intermediate struct for loading synonyms from TOML.
#[derive(serde::Deserialize)]
struct SynonymToml {
    group: HashMap<String, SynonymEntry>,
}

#[derive(serde::Deserialize, Clone)]
struct SynonymEntry {
    terms: Vec<String>,
}

/// The central formula registry for primitive formulas.
///
/// Loads atomic primitive formulas from disk and provides query methods.
/// The heavy maps are `Arc`-wrapped so `FormulaRegistry::clone()` is a
/// refcount bump, not a deep copy — `main()` hands snapshots to four engines
/// per startup. Mutation goes through `Arc::make_mut` (copy-on-write): the
/// first write after a clone pays the copy for that field only, so engines
/// that never mutate (Shikai, NLP, descent) never pay it at all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaRegistry {
    /// All formulas indexed by ID.
    formulas: Arc<HashMap<String, Formula>>,

    /// Formulas indexed by domain.
    by_domain: Arc<HashMap<Domain, Vec<String>>>,

    /// Formulas indexed by output variable name.
    by_output: Arc<HashMap<String, Vec<String>>>,

    /// Word index: lowercase token → set of formula IDs containing that token.
    word_index: Arc<HashMap<String, BTreeSet<String>>>,

    /// Lowercased searchable text per formula for O(1) substring matching.
    search_text_cache: Arc<HashMap<String, String>>,

    /// TF-IDF semantic search index. Built lazily on first semantic search
    /// (most CLI invocations never need it); the cell is shared across
    /// registry clones, so one build serves every engine. Registering a
    /// formula swaps in a fresh cell (see `invalidate_tfidf`).
    /// Skipped by the startup snapshot cache — a deserialized registry
    /// starts with an empty cell and rebuilds on first semantic search.
    #[serde(skip)]
    tfidf: Arc<OnceLock<TfIdfIndex>>,

    /// Synonym dictionary for query expansion. Maps each term to all terms
    /// in its synonym group (loaded from search_synonyms.toml). Used by
    /// `search()` and `search_semantic()` to find formulas even when the
    /// user's exact words don't appear in formula text (e.g. "gravity" →
    /// also matches "gravitational", "g_force").
    #[serde(default)]
    synonyms: Arc<HashMap<String, Vec<String>>>,
}

impl Default for FormulaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FormulaRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        FormulaRegistry {
            formulas: Arc::new(HashMap::new()),
            by_domain: Arc::new(HashMap::new()),
            by_output: Arc::new(HashMap::new()),
            word_index: Arc::new(HashMap::new()),
            search_text_cache: Arc::new(HashMap::new()),
            tfidf: Arc::new(OnceLock::new()),
            synonyms: Arc::new(HashMap::new()),
        }
    }

    /// Register a single formula.
    pub fn register(&mut self, mut formula: Formula) -> Result<(), FormulaError> {
        let id = formula.id.clone();
        let domain = formula.domain;
        let output = formula.output.clone();

        // On duplicate ID: merge domains instead of silently dropping the
        // earlier entry's domain. Every dual sign/graha formula pair (e.g.
        // "add" in aries_math.toml vs. mangala_math.toml) declares the same
        // content under a different `domain`; the second load must keep both.
        if let Some(existing) = self.formulas.get(&id) {
            // Only warn when the duplicate carries a *different* domain — that's
            // a genuine cross-domain overload worth surfacing. A same-domain
            // redefinition (e.g. two `price_elasticity` entries) is a redundant
            // copy and merging it is silent and harmless, so stay quiet.
            if existing.domain != domain {
                eprintln!(
                    "Warning: duplicate formula id '{}' — merging domains ({:?} + {:?})",
                    id, existing.domain, domain
                );
            }

            let mut merged = existing.also_domains.clone();
            merged.push(existing.domain);
            merged.extend(formula.also_domains.iter().copied());
            merged.retain(|d| *d != domain);
            merged.sort_by_key(|d| *d as usize);
            merged.dedup();
            formula.also_domains = merged;

            Arc::make_mut(&mut self.word_index)
                .values_mut()
                .for_each(|ids| {
                    ids.remove(&id);
                });
            Arc::make_mut(&mut self.search_text_cache).remove(&id);
            // Drop stale by_domain entries for this id from every domain it
            // was previously indexed under, so re-indexing below is clean.
            for ids in Arc::make_mut(&mut self.by_domain).values_mut() {
                ids.retain(|existing_id| existing_id != &id);
            }
            // Also drop stale by_output entries
            for ids in Arc::make_mut(&mut self.by_output).values_mut() {
                ids.retain(|existing_id| existing_id != &id);
            }
        }

        // Build word index from id, description, expression, inputs, output, zodiac
        // Pre-lowercase once, reuse for both word index and search cache
        let searchable_text = formula_searchable_text(&formula);
        let searchable_lower = searchable_text.to_lowercase();
        let word_index = Arc::make_mut(&mut self.word_index);
        for word in searchable_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
        {
            word_index
                .entry(word.to_string())
                .or_default()
                .insert(id.clone());
        }
        Arc::make_mut(&mut self.search_text_cache).insert(id.clone(), searchable_lower);

        // Index the id under its primary domain *and* every merged domain,
        // so lookups like `by_domain[Domain::Mangala]` still find "add" even
        // when its canonical `domain` field reads a different graha.
        let by_domain = Arc::make_mut(&mut self.by_domain);
        for d in std::iter::once(domain).chain(formula.also_domains.iter().copied()) {
            by_domain.entry(d).or_default().push(id.clone());
        }
        // Index by output variable
        Arc::make_mut(&mut self.by_output)
            .entry(output)
            .or_default()
            .push(id.clone());

        Arc::make_mut(&mut self.formulas).insert(id.clone(), formula);
        self.invalidate_tfidf();

        Ok(())
    }

    /// Force an eager TF-IDF rebuild. Normally unnecessary — the index builds
    /// itself lazily on the first semantic search — but tests and benchmarks
    /// call this to keep their timings free of one-off build cost.
    pub fn rebuild_tfidf(&mut self) {
        let cell = OnceLock::new();
        let _ = cell.set(TfIdfIndex::built_from(&self.formulas));
        self.tfidf = Arc::new(cell);
    }

    /// Swap in a fresh (unbuilt) TF-IDF cell after a mutation, so the next
    /// semantic search rebuilds against current formulas. Clones holding the
    /// old cell keep their consistent snapshot.
    fn invalidate_tfidf(&mut self) {
        if self.tfidf.get().is_some() {
            self.tfidf = Arc::new(OnceLock::new());
        }
    }

    /// Register a batch of formulas, then rebuild TF-IDF index once.
    pub fn register_all(&mut self, formulas: Vec<Formula>) -> Result<(), FormulaError> {
        for f in formulas {
            self.register(f)?;
        }
        // TF-IDF builds lazily on first semantic search — no eager rebuild.
        Ok(())
    }

    /// Get a formula by ID.
    pub fn get(&self, id: &str) -> Option<&Formula> {
        self.formulas.get(id)
    }

    /// Get a mutable reference to a formula by ID.
    /// Copy-on-write: detaches this registry's formula map from any clones.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Formula> {
        Arc::make_mut(&mut self.formulas).get_mut(id)
    }

    /// Find formulas by domain.
    pub fn by_domain(&self, domain: Domain) -> Vec<&Formula> {
        self.by_domain
            .get(&domain)
            .map(|ids| ids.iter().filter_map(|id| self.formulas.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find formulas by output variable name.
    ///
    /// Returns all formulas whose `output` field matches the given variable name.
    /// Useful for reverse lookup: "what formulas produce `kinetic_energy`?"
    pub fn by_output(&self, output: &str) -> Vec<&Formula> {
        self.by_output
            .get(output)
            .map(|ids| ids.iter().filter_map(|id| self.formulas.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find formulas by keyword in ID, description, expression, or zodiac.
    /// Query is expanded with synonyms before searching (e.g. "gravity" also
    /// matches "gravitational", "g_force").
    pub fn search(&self, keyword: &str) -> Vec<&Formula> {
        let expanded = self.expand_query(keyword);
        let kw = expanded.to_lowercase();

        let tokens: Vec<&str> = kw
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        // Single-token: word index fast path
        if tokens.len() == 1 {
            if let Some(ids) = self.word_index.get(&kw) {
                return ids.iter().filter_map(|id| self.formulas.get(id)).collect();
            }
        }

        // Multi-token: check each token against word index, union results
        if tokens.len() > 1 {
            let mut seen = BTreeSet::new();
            for token in &tokens {
                if let Some(ids) = self.word_index.get(*token) {
                    seen.extend(ids.iter().cloned());
                }
            }
            if !seen.is_empty() {
                return seen.iter().filter_map(|id| self.formulas.get(id)).collect();
            }
        }

        // Slow path: linear substring scan using pre-cached lowercased text.
        // Also try the original (unexpanded) keyword for substring matching
        // in case the expansion lost precision.
        //
        // Determinism: collect into a Vec, then sort by `f.id` so the
        // output order is identical across process runs. `self.formulas`
        // is a `HashMap` with `RandomState` seed, so iterating it in
        // insertion order is non-deterministic across runs.
        let original_lower = keyword.to_lowercase();
        let mut hits: Vec<&Formula> = self
            .formulas
            .values()
            .filter(|f| {
                self.search_text_cache
                    .get(&f.id)
                    .is_some_and(|cached| cached.contains(&kw) || cached.contains(&original_lower))
            })
            .collect();
        hits.sort_by(|a, b| a.id.cmp(&b.id));
        hits
    }

    /// Expand a query string with synonyms from the loaded dictionary.
    /// Each token is replaced by the union of itself and all its synonyms.
    /// This is the zero-dependency alternative to embedding-based search.
    fn expand_query(&self, query: &str) -> String {
        let tokens: Vec<&str> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();
        if tokens.is_empty() {
            return query.to_string();
        }
        let mut expanded: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for token in &tokens {
            let lower = token.to_lowercase();
            // Always include the original token (dedup via seen set)
            if seen.insert(lower.clone()) {
                expanded.push(lower.clone());
            }
            // Add all synonyms for this token
            if let Some(synonyms) = self.synonyms.get(&lower) {
                for syn in synonyms {
                    if seen.insert(syn.clone()) {
                        expanded.push(syn.clone());
                    }
                }
            }
        }
        expanded.join(" ")
    }

    /// Search formulas by semantic similarity to query text.
    /// Builds the TF-IDF index on first use (shared across registry clones).
    /// Query is expanded with synonyms before searching.
    pub fn search_semantic(&self, query: &str, top_n: usize) -> Vec<&Formula> {
        let tfidf = self
            .tfidf
            .get_or_init(|| TfIdfIndex::built_from(&self.formulas));
        if tfidf.is_empty() {
            return self.search(query);
        }
        let expanded = self.expand_query(query);
        let results = tfidf.search(&expanded, top_n);
        results
            .into_iter()
            .filter_map(|(id, _score)| self.formulas.get(&id))
            .collect()
    }

    /// Load formulas from a TOML file at `path`.
    ///
    /// Does **not** rebuild the TF-IDF index — callers loading a batch of
    /// files should call [`rebuild_tfidf`](Self::rebuild_tfidf) once after
    /// the last file instead of paying a full rebuild per file.
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), FormulaError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            FormulaError::SerdeError(format!("cannot read {}: {}", path.display(), e))
        })?;
        self.load_from_toml_str_at(&content, Some(path))
    }

    /// Load formulas from a TOML string, with an optional source path for error reporting.
    /// The TF-IDF index builds lazily on first semantic search, so the
    /// registry is immediately searchable with no eager rebuild here.
    pub fn load_from_toml_str(&mut self, toml_str: &str) -> Result<(), FormulaError> {
        self.load_from_toml_str_at(toml_str, None)?;
        Ok(())
    }

    /// Load formulas from a TOML string with an associated source path.
    fn load_from_toml_str_at(
        &mut self,
        toml_str: &str,
        source: Option<&std::path::Path>,
    ) -> Result<(), FormulaError> {
        let source_label =
            source.map_or_else(|| "<unknown>".to_string(), |p| p.display().to_string());
        let parsed: FormulaToml = toml::from_str(toml_str).map_err(|e| {
            FormulaError::SerdeError(format!("{}: TOML parse error: {}", source_label, e))
        })?;

        // Load primitive formulas
        if let Some(formulas) = parsed.formula {
            for (i, entry) in formulas.iter().enumerate() {
                let formula = self.entry_to_formula(entry.clone()).map_err(|e| {
                    FormulaError::SerdeError(format!(
                        "{}: formula #{} ('{}'): {}",
                        source_label,
                        i + 1,
                        entry.id,
                        e
                    ))
                })?;
                self.register(formula)?;
            }
        }

        Ok(())
    }

    /// Load synonym dictionary from a TOML file.
    ///
    /// Expected format:
    /// ```toml
    /// [group.gravity]
    /// terms = ["gravity", "gravitational", "gravitation", "g_force"]
    /// ```
    ///
    /// Every term in each group is mapped bidirectionally to all other terms
    /// in the same group. Loading is additive — calling this multiple times
    /// merges groups.
    pub fn load_synonyms_from_file(&mut self, path: &std::path::Path) -> Result<(), FormulaError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            FormulaError::SerdeError(format!("cannot read synonyms {}: {}", path.display(), e))
        })?;
        let parsed: SynonymToml = toml::from_str(&content)
            .map_err(|e| FormulaError::SerdeError(format!("synonyms TOML parse error: {}", e)))?;

        let synonyms = Arc::make_mut(&mut self.synonyms);
        for (_key, entry) in parsed.group {
            let terms: Vec<String> = entry.terms.iter().map(|t| t.to_lowercase()).collect();
            if terms.is_empty() {
                continue;
            }
            // Build bidirectional mapping: every term maps to all others in its group
            for term in &terms {
                let others: Vec<String> = terms.iter().filter(|t| *t != term).cloned().collect();
                synonyms.entry(term.clone()).or_default().extend(others);
            }
        }
        Ok(())
    }

    /// Load synonym groups from an in-memory TOML string (used by the
    /// `portable` build, which embeds the corpus instead of reading disk).
    pub fn load_synonyms_from_str(&mut self, content: &str) -> Result<(), FormulaError> {
        let parsed: SynonymToml = toml::from_str(content)
            .map_err(|e| FormulaError::SerdeError(format!("synonyms TOML parse error: {}", e)))?;

        let synonyms = Arc::make_mut(&mut self.synonyms);
        for (_key, entry) in parsed.group {
            let terms: Vec<String> = entry.terms.iter().map(|t| t.to_lowercase()).collect();
            if terms.is_empty() {
                continue;
            }
            for term in &terms {
                let others: Vec<String> = terms.iter().filter(|t| *t != term).cloned().collect();
                synonyms.entry(term.clone()).or_default().extend(others);
            }
        }
        Ok(())
    }

    /// Get the synonym map (public read-only access).
    pub fn synonyms(&self) -> &HashMap<String, Vec<String>> {
        &self.synonyms
    }

    /// Check if a word has synonyms in the dictionary.
    pub fn has_synonyms(&self, word: &str) -> bool {
        self.synonyms.contains_key(&word.to_lowercase())
    }

    /// Convert a TOML formula entry to a Formula.
    fn entry_to_formula(&self, entry: FormulaEntry) -> Result<Formula, FormulaError> {
        // Determine primary domain: explicit `domain` field takes precedence,
        // fall back to `from` (for bridging formulas), or default to Mangala.
        let domain = match &entry.domain {
            Some(d) => Domain::parse(d)
                .ok_or_else(|| FormulaError::SerdeError(format!("unknown domain: {}", d)))?,
            None => match &entry.from {
                Some(f) => Domain::parse(f).ok_or_else(|| {
                    FormulaError::SerdeError(format!("unknown from domain: {}", f))
                })?,
                None => Domain::Mangala,
            },
        };

        // Parse bridging metadata: from_domain, to_domain, aspect
        let from_domain = entry.from.as_ref().and_then(|f| Domain::parse(f));
        let to_domain = entry.to.as_ref().and_then(|t| Domain::parse(t));

        let formula_type = match entry.formula_type.as_deref() {
            Some("math") | None => super::FormulaType::Math,
            Some("logic") => super::FormulaType::Logic,
            Some("llm") => super::FormulaType::Llm,
            Some(other) => {
                // No silent degradation: a typo like "logik" must not
                // quietly lose the Logic domain contract.
                eprintln!(
                    "Warning: formula '{}' has unknown formula_type '{}' — treating as math",
                    entry.id, other
                );
                super::FormulaType::Math
            }
        };

        let inputs: Vec<&str> = entry.inputs.iter().map(|s| s.as_str()).collect();

        let mut formula = Formula::new(
            &entry.id,
            formula_type,
            domain,
            inputs,
            &entry.output,
            &entry.expression,
            &entry.description,
        );

        formula.zodiac = entry.zodiac;
        formula.evidence = entry.evidence;
        formula.from_domain = from_domain;
        formula.to_domain = to_domain;
        formula.bridge_aspect = entry.aspect;

        Ok(formula)
    }

    /// Iterate the pre-lowercased searchable text of every formula.
    ///
    /// This is the same cached text `search()` uses for its substring
    /// fallback — callers doing per-query token matching should use this
    /// instead of re-building and re-lowercasing formula text themselves.
    pub fn search_texts(&self) -> impl Iterator<Item = &str> {
        self.search_text_cache.values().map(|s| s.as_str())
    }

    /// Total number of registered formulas.
    pub fn len(&self) -> usize {
        self.formulas.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.formulas.is_empty()
    }

    /// Get all formulas in the registry (sorted by ID for deterministic order).
    pub fn all(&self) -> Vec<&Formula> {
        let mut list: Vec<&Formula> = self.formulas.values().collect();
        list.sort_by_key(|f| &f.id);
        list
    }

    /// Find up to `n` formula IDs similar to `query` (V7: did-you-mean).
    /// Uses TF-IDF semantic search; falls back to keyword substring search
    /// if the index isn't built yet. Returns formula IDs with scores.
    pub fn suggest_similar(&self, query: &str, n: usize) -> Vec<(String, f64)> {
        if self.formulas.is_empty() {
            return Vec::new();
        }
        // Try TF-IDF first (may build the index lazily)
        let tfidf = self
            .tfidf
            .get_or_init(|| TfIdfIndex::built_from(&self.formulas));
        if !tfidf.is_empty() {
            let expanded = self.expand_query(query);
            let results = tfidf.search(&expanded, n);
            if !results.is_empty() {
                return results;
            }
        }
        // Fallback: Levenshtein-like via keyword search
        let kw_results = self.search(query);
        kw_results
            .into_iter()
            .take(n)
            .map(|f| (f.id.clone(), 0.5))
            .collect()
    }

    /// Find up to `n` output variable names similar to `query` — the
    /// did-you-mean hint for `reason`/`find_path` targets (rep-2 audit:
    /// `--want energy` found nothing despite `kinetic_energy` being
    /// loaded, with no pointer to the near-miss).
    pub fn suggest_outputs(&self, query: &str, n: usize) -> Vec<String> {
        let q = query.to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let mut matches: Vec<(usize, &str)> = self
            .by_output
            .iter()
            // Re-registration leaves stale keys with emptied id lists behind
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(out, _)| out)
            .filter(|out| {
                let o = out.to_lowercase();
                // Containment only counts when the contained side is a
                // meaningful fragment — otherwise 1-letter outputs like
                // `r` match every query
                o != q && ((q.len() >= 3 && o.contains(&q)) || (o.len() >= 3 && q.contains(&o)))
            })
            .map(|out| (out.len(), out.as_str()))
            .collect();
        // Shortest names sort first (closest to the query), ties lexicographic
        matches.sort();
        matches
            .into_iter()
            .take(n)
            .map(|(_, o)| o.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_formulas() -> Vec<Formula> {
        vec![
            Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition"),
            Formula::math("sqrt", vec!["x"], "root", "sqrt(x)", "Square root"),
            Formula::logic("nand", vec!["a", "b"], "out", "1 - a*b", "NAND gate"),
        ]
    }

    #[test]
    fn test_registry_create() {
        let r = FormulaRegistry::new();
        assert!(r.is_empty());
    }

    #[test]
    fn test_suggest_outputs_near_miss() {
        // Rep-2 audit: reasoning toward 'energy' with only 'kinetic_energy'
        // loaded gave no hint about the near-miss output name.
        let mut r = FormulaRegistry::new();
        r.register_all(sample_formulas()).unwrap();
        r.register(Formula::math(
            "kinetic_energy",
            vec!["mass", "velocity"],
            "kinetic_energy",
            "0.5 * mass * velocity^2",
            "Kinetic energy",
        ))
        .unwrap();

        assert_eq!(r.suggest_outputs("energy", 5), vec!["kinetic_energy"]);
        // Exact matches are excluded — the hint is only for misses
        assert!(r.suggest_outputs("kinetic_energy", 5).is_empty());
        // No overlap → no suggestions
        assert!(r.suggest_outputs("unicorn", 5).is_empty());
        assert!(r.suggest_outputs("", 5).is_empty());
    }

    #[test]
    fn test_registry_register_and_query() {
        let mut r = FormulaRegistry::new();
        r.register_all(sample_formulas()).unwrap();
        assert_eq!(r.len(), 3);

        let f = r.get("add").unwrap();
        assert_eq!(f.domain, Domain::Mangala);

        let aries = r.by_domain(Domain::Mangala);
        assert_eq!(aries.len(), 2);
    }

    #[test]
    fn test_load_llm_kind_from_toml() {
        let mut r = FormulaRegistry::new();
        r.load_from_toml_str(
            r#"
            [[formula]]
            id = "estimate_reading_minutes"
            formula_type = "llm"
            domain = "budha"
            inputs = ["word_count"]
            output = "reading_minutes"
            expression = "Estimate how many minutes an average adult needs to read {word_count} words."
            description = "LLM-estimated reading time"
            "#,
        )
        .unwrap();
        let f = r.get("estimate_reading_minutes").unwrap();
        assert_eq!(f.formula_type, crate::formula::FormulaType::Llm);
        assert_eq!(f.domain, Domain::Budha);
    }

    #[test]
    fn test_registry_search() {
        let mut r = FormulaRegistry::new();
        r.register_all(sample_formulas()).unwrap();

        let found = r.search("nand");
        assert_eq!(found.len(), 1);

        let found2 = r.search("a");
        assert_eq!(found2.len(), 2);
    }

    #[test]
    fn test_search_matches_zodiac() {
        let mut r = FormulaRegistry::new();
        let mut f = Formula::logic("nand", vec!["a", "b"], "out", "1 - a*b", "NAND gate");
        f.zodiac = vec!["♏".to_string(), "scorpio".to_string(), "logic".to_string()];
        r.register(f).unwrap();

        let found = r.search("scorpio");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "nand");
    }

    #[test]
    fn test_tfidf_search_semantic() {
        let mut r = FormulaRegistry::new();
        r.register_all(sample_formulas()).unwrap();

        let results = r.search_semantic("addition sum", 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_duplicate_id_merges_domains() {
        let mut r = FormulaRegistry::new();
        let mut first = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition (math)");
        first.domain = Domain::Mangala;
        let mut second = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition (dup)");
        second.domain = Domain::Budha;
        r.register(first).unwrap();
        r.register(second).unwrap();

        // One formula, not two — and the earlier domain survives as a tag.
        assert_eq!(r.len(), 1);
        let f = r.get("add").unwrap();
        assert_eq!(f.domain, Domain::Budha);
        assert_eq!(f.also_domains, vec![Domain::Mangala]);

        // by_domain finds the formula under both domains, without duplicates.
        let budha = r.by_domain(Domain::Budha);
        assert_eq!(budha.len(), 1);
        let mangala = r.by_domain(Domain::Mangala);
        assert_eq!(mangala.len(), 1);
        assert_eq!(mangala[0].id, "add");
    }

    #[test]
    fn test_triple_duplicate_accumulates_domains() {
        let mut r = FormulaRegistry::new();
        for domain in [Domain::Mangala, Domain::Budha, Domain::Shani] {
            let mut f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition");
            f.domain = domain;
            r.register(f).unwrap();
        }

        assert_eq!(r.len(), 1);
        let f = r.get("add").unwrap();
        assert_eq!(f.domain, Domain::Shani);
        assert_eq!(f.also_domains, vec![Domain::Mangala, Domain::Budha]);
        for domain in [Domain::Mangala, Domain::Budha, Domain::Shani] {
            assert_eq!(r.by_domain(domain).len(), 1, "missing under {:?}", domain);
        }
    }

    #[test]
    fn test_all_formulas_sorted() {
        let mut r = FormulaRegistry::new();
        r.register_all(sample_formulas()).unwrap();
        let all = r.all();
        assert_eq!(all.len(), 3);
        // Should be sorted by id: add, nand, sqrt
        assert_eq!(all[0].id, "add");
        assert_eq!(all[1].id, "nand");
        assert_eq!(all[2].id, "sqrt");
    }

    #[test]
    fn test_by_output() {
        let mut r = FormulaRegistry::new();
        r.register_all(sample_formulas()).unwrap();

        // add produces "sum"
        let sum_formulas = r.by_output("sum");
        assert_eq!(sum_formulas.len(), 1);
        assert_eq!(sum_formulas[0].id, "add");

        // sqrt produces "root"
        let root_formulas = r.by_output("root");
        assert_eq!(root_formulas.len(), 1);
        assert_eq!(root_formulas[0].id, "sqrt");

        // nand produces "out"
        let out_formulas = r.by_output("out");
        assert_eq!(out_formulas.len(), 1);
        assert_eq!(out_formulas[0].id, "nand");

        // Non-existent output
        let none = r.by_output("nonexistent");
        assert!(none.is_empty());
    }

    #[test]
    fn test_load_all_formula_toml_files() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("formulas");
        if !dir.exists() {
            eprintln!("Skipping: formulas/ directory not found");
            return;
        }
        let mut r = FormulaRegistry::new();
        let mut loaded = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map_or(false, |e| e == "toml") {
                r.load_from_file(&path)
                    .unwrap_or_else(|e| panic!("Failed to load {}: {}", path.display(), e));
                loaded += 1;
            }
        }
        assert!(loaded > 0, "Should have loaded at least one TOML file");
        assert!(
            r.len() > 100,
            "Should have loaded many formulas, got {}",
            r.len()
        );
        // Spot-check a few known formulas
        assert!(r.get("add").is_some(), "Missing 'add' formula");
        assert!(r.get("nand").is_some(), "Missing 'nand' formula");
        assert!(
            r.get("kleiber_law").is_some(),
            "Missing 'kleiber_law' from atomic_seed"
        );
        assert!(
            r.get("clinical_severity").is_some(),
            "Missing 'clinical_severity' from bridging_seed"
        );
        assert!(
            r.get("climate_anxiety_vortex").is_some(),
            "Missing 'climate_anxiety_vortex' from vortex_seed"
        );
    }
}
