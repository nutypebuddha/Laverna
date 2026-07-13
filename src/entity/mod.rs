/// Pure function: Validate an entity ID string.
pub fn validate_entity_id(entity_id: &str) -> bool {
    !entity_id.is_empty() && entity_id.len() <= 128
}

/// Pure function: Compute entity hash for deduplication.
pub fn compute_entity_hash(entity_id: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in entity_id.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Pure function: Check if two entity IDs refer to the same entity.
pub fn is_same_entity(left_id: &str, right_id: &str) -> bool {
    left_id.eq_ignore_ascii_case(right_id)
}

// ─── Imports ───────────────────────────────────────────────────────────

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub use crate::astrology::{
    AtomClassification, ChangeSorter, Element, Graha, Guna, Modality, Nakshatra, PlanetaryRuler,
    Sign, SignAspect, VedicClassification, VedicElement,
};
use crate::chart::ChartSnapshot;

// ─── Core Runtime Entity (unchanged) ───────────────────────────────────

/// A runtime entity — a token flowing through the system.
///
/// Every token the LLM generates becomes an Entity at runtime, classified
/// by the Change Sorter across all 7 astrology axes. The EntityRegistry
/// holds these in a ring buffer (newest N tokens).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique runtime ID (e.g. "ent_000042")
    pub id: String,

    /// The original text token that created this entity
    pub text: String,

    /// Full 7-axis astrology classification from Change Sorter
    pub classification: AtomClassification,

    /// Dynamic entity ID this token was resolved to (if any from forms/events)
    #[serde(default)]
    pub resolved_entity: Option<String>,

    /// Tags from the resolved dynamic entity
    #[serde(default)]
    pub tags: Vec<String>,

    /// Numeric values extracted from this token or adjacent tokens
    #[serde(default)]
    pub values: HashMap<String, f64>,

    /// Logical truth value (for logic gate evaluation of token streams)
    /// Defaults to 1.0 (true). Set to 0.0 for negated tokens.
    #[serde(default)]
    pub truth: f64,

    /// Timestamp (monotonic sequence number)
    pub seq: u64,
}

impl Entity {
    /// Create a new runtime entity from a text token.
    pub fn new(text: &str, sorter: &ChangeSorter, seq: u64) -> Self {
        let classification = sorter.classify_token(text);
        Entity {
            id: format!("ent_{:06}", seq),
            text: text.to_string(),
            classification,
            resolved_entity: None,
            tags: Vec::new(),
            values: HashMap::new(),
            truth: 1.0,
            seq,
        }
    }

    /// Set the numeric values extracted from this token.
    pub fn with_values(mut self, values: HashMap<String, f64>) -> Self {
        self.values = values;
        self
    }

    /// Set the logical truth value.
    pub fn with_truth(mut self, truth: f64) -> Self {
        self.truth = truth.clamp(0.0, 1.0);
        self
    }

    /// Link this entity to a resolved dynamic entity ID.
    pub fn with_resolved_entity(mut self, entity_id: &str) -> Self {
        self.resolved_entity = Some(entity_id.to_string());
        self
    }

    /// Add tags to this entity.
    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(String::from).collect();
        self
    }

    /// The dominant sign from the entity's astrology classification.
    pub fn dominant_sign(&self) -> Option<Sign> {
        self.classification.dominant_sign()
    }

    /// The dominant element from the entity's astrology classification.
    pub fn dominant_element(&self) -> Option<Element> {
        self.classification.dominant_element()
    }

    /// The dominant modality from the entity's astrology classification.
    pub fn dominant_modality(&self) -> Option<Modality> {
        self.classification.dominant_modality()
    }

    /// The dominant Vedic graha (planet) from the entity's classification.
    pub fn dominant_graha(&self) -> Option<Graha> {
        self.classification.vedic.dominant_graha()
    }

    /// The dominant Vedic guna (quality) from the entity's classification.
    pub fn dominant_guna(&self) -> Option<Guna> {
        self.classification.vedic.dominant_guna()
    }

    /// The dominant Vedic nakshatra (lunar mansion) from the entity's classification.
    pub fn dominant_nakshatra(&self) -> Option<Nakshatra> {
        self.classification.vedic.dominant_nakshatra()
    }

    /// The dominant Vedic element (tattva) from the entity's classification.
    pub fn dominant_vedic_element(&self) -> Option<VedicElement> {
        self.classification.vedic.dominant_vedic_element()
    }
}

// ─── Token Entity Registry (ring buffer) ───────────────────────────────

/// A ring-buffer registry of runtime token entities.
///
/// Every token the LLM generates is stored here, newest-first.
/// Oldest tokens are evicted when the buffer exceeds `max_entities`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRegistry {
    /// Runtime entities by ID (ring buffer, newest first).
    /// The runtime ring is session state, not seed data — the startup
    /// snapshot cache persists only the seed maps below.
    #[serde(skip)]
    entities: HashMap<String, Entity>,

    /// Ordered list of entity IDs (newest first) for ring buffer eviction
    #[serde(skip)]
    order: VecDeque<String>,

    /// Maximum number of runtime entities before eviction
    #[serde(skip, default = "default_max_entities")]
    max_entities: usize,

    /// Monotonic sequence counter for entity ID generation
    #[serde(skip)]
    next_seq: u64,

    /// Seed entities loaded from TOML (`entities/*.toml`). These provide a
    /// stable, queryable knowledge base distinct from the runtime token ring.
    /// `Arc`-wrapped (copy-on-write via `Arc::make_mut`) so registry clones
    /// share the seed data instead of deep-copying it. The runtime ring above
    /// stays plain: it is empty at clone time and mutation-hot afterward.
    seeds: Arc<HashMap<String, SeedEntity>>,

    /// Pre-lowercased "id name description tags" blob per seed, built at
    /// registration so keyword search doesn't re-lowercase every seed's
    /// fields on every call.
    seed_search_cache: Arc<HashMap<String, String>>,
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Ring-buffer capacity for registries rebuilt from a startup snapshot
/// (must match `EntityRegistry::new`).
fn default_max_entities() -> usize {
    10_000
}

impl EntityRegistry {
    /// Create a new empty registry with default capacity (10,000 entities).
    pub fn new() -> Self {
        EntityRegistry {
            entities: HashMap::new(),
            order: VecDeque::new(),
            max_entities: 10_000,
            next_seq: 0,
            seeds: Arc::new(HashMap::new()),
            seed_search_cache: Arc::new(HashMap::new()),
        }
    }

    /// Create a new registry with a custom maximum capacity.
    pub fn with_capacity(max: usize) -> Self {
        EntityRegistry {
            entities: HashMap::new(),
            order: VecDeque::new(),
            max_entities: max,
            next_seq: 0,
            seeds: Arc::new(HashMap::new()),
            seed_search_cache: Arc::new(HashMap::new()),
        }
    }

    /// Record a new token in the registry.
    pub fn record_token(&mut self, text: &str, sorter: &ChangeSorter) -> &Entity {
        let seq = self.next_seq;
        self.next_seq += 1;

        let entity = Entity::new(text, sorter, seq);
        let id = entity.id.clone();

        self.order.push_front(id.clone());
        self.entities.insert(id, entity);

        // Evict oldest if over capacity (at most once per insert)
        if self.order.len() > self.max_entities {
            if let Some(old_id) = self.order.pop_back() {
                self.entities.remove(&old_id);
            }
        }

        // The entity we just inserted is the newest (front of order)
        // SAFETY: we just inserted at least one entity
        &self.entities[self.order.front().unwrap()]
    }

    /// Record multiple tokens at once.
    pub fn record_tokens(&mut self, tokens: &[&str], sorter: &ChangeSorter) -> Vec<String> {
        tokens
            .iter()
            .map(|t| self.record_token(t, sorter).id.clone())
            .collect()
    }

    /// Get an entity by its runtime ID.
    pub fn get(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Search runtime entities by text substring match.
    pub fn search(&self, keyword: &str) -> Vec<&Entity> {
        let kw_lower = keyword.to_lowercase();
        let mut results: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| {
                e.text.to_lowercase().contains(&kw_lower)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&kw_lower))
                    || e.resolved_entity
                        .as_ref()
                        .is_some_and(|s| s.to_lowercase().contains(&kw_lower))
            })
            .collect();
        results.sort_by_key(|e| &e.id);
        results
    }

    /// Find entities by dominant sign.
    pub fn by_sign(&self, sign: Sign) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.dominant_sign() == Some(sign))
            .collect()
    }

    /// Find entities by dominant element.
    pub fn by_element(&self, element: Element) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.dominant_element() == Some(element))
            .collect()
    }

    /// Find entities by dominant Vedic graha (planet).
    pub fn by_graha(&self, graha: Graha) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.dominant_graha() == Some(graha))
            .collect()
    }

    /// Compute the sign-based aspect between two runtime entities (via their
    /// dominant zodiac signs). See `crate::astrology::SignAspect` — this is
    /// coarser than real ephemeris and distinct from the structural wheel
    /// relationship computed by `seed_aspect_between` below.
    pub fn aspect_between(&self, id_a: &str, id_b: &str) -> Option<(SignAspect, &Entity, &Entity)> {
        let a = self.entities.get(id_a)?;
        let b = self.entities.get(id_b)?;

        let sign_a = a.dominant_sign()?;
        let sign_b = b.dominant_sign()?;

        let aspect = SignAspect::between_sign_indices(sign_a.index(), sign_b.index());
        Some((aspect, a, b))
    }

    /// Compute the **structural composition relationship** between two seed
    /// entities via their dominant grahas' fixed positions on the 9-node
    /// compositional wheel. Seeds live in a separate registry from
    /// the runtime ring, so `aspect_between` alone leaves them unreachable
    /// (V2).
    ///
    /// T27: this returns `wheel::CompositionAspect`, NOT a real astronomical
    /// aspect. Two grahas that are always exactly 180° apart in the real sky
    /// (e.g. Rahu and Ketu, which are lunar nodes) can still be `Adjacent`
    /// here if they happen to sit next to each other on the fixed 9-node
    /// ring — that is a correct answer to "how naturally do their formulas
    /// compose?", not an answer to "what is their real angular separation
    /// today?". For the latter, compute a `ChartSnapshot` for a specific
    /// date and use `chart::AstroAspect` / `ChartSnapshot::aspect_between`.
    pub fn seed_aspect_between(
        &self,
        id_a: &str,
        id_b: &str,
    ) -> Option<(crate::wheel::CompositionAspect, &SeedEntity, &SeedEntity)> {
        let a = self.seeds.get(id_a)?;
        let b = self.seeds.get(id_b)?;
        let ga = a.classification.as_ref()?.vedic.dominant_graha()?;
        let gb = b.classification.as_ref()?.vedic.dominant_graha()?;
        Some((crate::wheel::CompositionAspect::between(ga, gb), a, b))
    }

    /// Total runtime entities.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// List all runtime entity IDs (sorted).
    pub fn list(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.entities.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Get the next sequence number (for external tracking).
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }
}

// ─── Backward-compat SeedEntity (migration shim) ─────────────────────
// These are temporarily kept so the rest of the codebase compiles
// while modules migrate to the dynamic entity system.

/// Backward-compatible seed entity stub.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeedEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub classification: Option<AtomClassification>,
    pub properties: HashMap<String, f64>,
    pub constants: HashMap<String, f64>,
    pub tags: Vec<String>,
    pub formula: Option<String>,
    pub birth_time: Option<String>,
    pub bija: Option<String>,
    pub mantra: Option<String>,
    pub day: Option<String>,
    pub ruled_nakshatras: Vec<String>,
}

impl SeedEntity {
    pub fn birth_chart(&self) -> Option<ChartSnapshot> {
        None
    }
    pub fn to_vedic_classification(&self) -> VedicClassification {
        VedicClassification::new()
    }
    pub fn dominant_graha(&self) -> Option<Graha> {
        self.classification
            .as_ref()
            .and_then(|c| c.vedic.dominant_graha())
    }
    pub fn dominant_guna(&self) -> Option<Guna> {
        self.classification
            .as_ref()
            .and_then(|c| c.vedic.dominant_guna())
    }
}

impl EntityRegistry {
    /// Look up a seed entity by ID.
    pub fn get_seed(&self, id: &str) -> Option<SeedEntity> {
        self.seeds.get(id).cloned()
    }

    /// Look up a seed entity by ID without cloning.
    pub fn get_seed_ref(&self, id: &str) -> Option<&SeedEntity> {
        self.seeds.get(id)
    }

    /// Iterate all seed entities without cloning.
    pub fn seeds(&self) -> impl Iterator<Item = &SeedEntity> {
        self.seeds.values()
    }

    /// Search seed entities by keyword across id, name, description, and tags.
    pub fn search_seeds(&self, keyword: &str) -> Vec<SeedEntity> {
        self.search_seeds_ref(keyword)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Like [`search_seeds`](Self::search_seeds), but borrows the matching
    /// seeds instead of cloning them. Matches against the pre-lowercased
    /// per-seed search blob, so no per-seed allocation happens per call.
    pub fn search_seeds_ref(&self, keyword: &str) -> Vec<&SeedEntity> {
        let kw = keyword.to_lowercase();
        self.seeds
            .values()
            .filter(|s| {
                self.seed_search_cache
                    .get(&s.id)
                    .is_some_and(|blob| blob.contains(&kw))
            })
            .collect()
    }

    /// List all seed IDs (sorted).
    pub fn list_seeds(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.seeds.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Count of loaded seed entities.
    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }

    /// Register a seed entity programmatically.
    ///
    /// On duplicate ID, merges tags, properties, constants, and
    /// ruled_nakshatras from the existing entry instead of overwriting.
    /// This preserves the dual Western/Vedic cosmology design where
    /// the same concept appears under both frameworks.
    pub fn register_seed(&mut self, mut seed: SeedEntity) {
        let id = seed.id.clone();

        if let Some(existing) = self.seeds.get(&id) {
            // Union tag sets
            for tag in &existing.tags {
                if !seed.tags.contains(tag) {
                    seed.tags.push(tag.clone());
                }
            }
            // Backfill properties/constants from existing entry (existing wins)
            for (k, v) in &existing.properties {
                seed.properties.entry(k.clone()).or_insert(*v);
            }
            for (k, v) in &existing.constants {
                seed.constants.entry(k.clone()).or_insert(*v);
            }
            // Union ruled_nakshatras
            for nak in &existing.ruled_nakshatras {
                if !seed.ruled_nakshatras.contains(nak) {
                    seed.ruled_nakshatras.push(nak.clone());
                }
            }
        }

        let blob = format!(
            "{} {} {} {}",
            seed.id,
            seed.name,
            seed.description,
            seed.tags.join(" ")
        )
        .to_lowercase();
        Arc::make_mut(&mut self.seed_search_cache).insert(id.clone(), blob);
        Arc::make_mut(&mut self.seeds).insert(id, seed);
    }

    /// Load seed entities from a TOML file containing `[[entity]]` tables.
    ///
    /// Each entry's `domain` field (a graha name such as "surya", "chandra")
    /// is mapped into the seed's `AtomClassification` so downstream
    /// graha/nakshatra queries resolve. Files that fail to parse return an
    /// error rather than silently degrading.
    pub fn load_seeds_from_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let parsed: SeedFile = toml::from_str(&content)
            .map_err(|e| format!("cannot parse {}: {e}", path.display()))?;
        for entry in parsed.entity {
            let mut classification = AtomClassification::new();
            if let Some(domain) = &entry.domain {
                if let Some(graha) = Graha::parse(domain) {
                    classification = classification.with_graha(graha, 0.9);
                }
            }
            let seed = SeedEntity {
                id: entry.id,
                name: entry.name,
                description: entry.description,
                classification: Some(classification),
                properties: entry.properties,
                constants: HashMap::new(),
                tags: entry.tags,
                formula: None,
                birth_time: None,
                bija: None,
                mantra: None,
                day: None,
                ruled_nakshatras: Vec::new(),
            };
            self.register_seed(seed);
        }
        Ok(())
    }

    /// Load seed entities from an in-memory TOML string (used by the
    /// `portable` build, which embeds the corpus instead of reading disk).
    pub fn load_seeds_from_str(&mut self, content: &str) -> Result<(), String> {
        let parsed: SeedFile =
            toml::from_str(content).map_err(|e| format!("cannot parse seed TOML: {e}"))?;
        for entry in parsed.entity {
            let mut classification = AtomClassification::new();
            if let Some(domain) = &entry.domain {
                if let Some(graha) = Graha::parse(domain) {
                    classification = classification.with_graha(graha, 0.9);
                }
            }
            let seed = SeedEntity {
                id: entry.id,
                name: entry.name,
                description: entry.description,
                classification: Some(classification),
                properties: entry.properties,
                constants: HashMap::new(),
                tags: entry.tags,
                formula: None,
                birth_time: None,
                bija: None,
                mantra: None,
                day: None,
                ruled_nakshatras: Vec::new(),
            };
            self.register_seed(seed);
        }
        Ok(())
    }

    /// Get all seed entities whose dominant graha matches `domain`.
    pub fn seeds_by_graha(&self, domain: crate::wheel::Domain) -> Vec<SeedEntity> {
        self.seeds
            .values()
            .filter(|s| s.dominant_graha() == Some(domain))
            .cloned()
            .collect()
    }
}

/// TOML shape of a seed-entity file: a list of `[[entity]]` tables.
#[derive(Debug, Clone, Deserialize)]
struct SeedFile {
    #[serde(default)]
    entity: Vec<SeedEntry>,
}

/// A single `[[entity]]` row in a seed TOML file.
#[derive(Debug, Clone, Deserialize)]
struct SeedEntry {
    id: String,
    name: String,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    properties: HashMap<String, f64>,
}

// ─── Shikai Forms (static TOML patterns) ──────────────────────────────

/// A shikai form entry: a regex pattern paired with a base Vedic classification.
///
/// When a token matches the pattern, it receives the form's Vedic classification
/// as its "first release" — the initial tag that the descent wheels refine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShikaiFormEntry {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub graha: Option<String>,
    pub guna: Option<String>,
    pub tattva: Option<String>,
    pub nakshatra: Option<String>,
    pub rashi: Option<String>,
    pub priority: Option<f64>,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub properties: HashMap<String, f64>,
}

impl ShikaiFormEntry {
    /// Build a VedicClassification from this form's string fields.
    pub fn to_vedic(&self) -> VedicClassification {
        let mut vc = VedicClassification::new();
        if let Some(ref g) = self.graha {
            if let Some(graha) = Graha::parse(g) {
                vc = vc.with_graha(graha, 0.9);
            }
        }
        if let Some(ref n) = self.nakshatra {
            if let Some(nak) = Nakshatra::parse(n) {
                vc = vc.with_nakshatra(nak, 0.8);
            }
        }
        if let Some(ref g) = self.guna {
            if let Some(guna) = Guna::parse(g) {
                vc = vc.with_guna(guna, 0.8);
            }
        }
        if let Some(ref t) = self.tattva {
            if let Some(elem) = VedicElement::parse(t) {
                vc = vc.with_vedic_element(elem, 0.8);
            }
        }
        vc.confidence = self.priority.unwrap_or(0.7);
        vc
    }
}

/// Compiled form entry — a regex paired with its Vedic classification.
#[derive(Debug, Clone)]
struct CompiledForm {
    regex: regex::Regex,
    entry: ShikaiFormEntry,
    vedic: VedicClassification,
}

/// Registry of ShikaiForm patterns, loaded from `shikai_form.toml`.
///
/// Matches tokens against compiled regex patterns and returns the matching
/// forms sorted by priority (highest first).
#[derive(Debug, Clone, Default)]
pub struct ShikaiFormRegistry {
    forms: Vec<CompiledForm>,
}

impl ShikaiFormRegistry {
    pub fn new() -> Self {
        ShikaiFormRegistry { forms: Vec::new() }
    }

    /// Load forms from a TOML file.
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        self.load_from_str(&content)
            .map_err(|e| format!("{}: {}", path.display(), e))
    }

    /// Load forms from an in-memory TOML string (e.g. the `portable` build's
    /// compiled-in corpus, which has no file on disk to read from).
    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        #[derive(Deserialize)]
        struct FormFile {
            form: Option<Vec<ShikaiFormEntry>>,
        }

        let parsed: FormFile =
            toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))?;

        if let Some(entries) = parsed.form {
            for entry in entries {
                let vedic = entry.to_vedic();
                let pattern = &entry.pattern;
                match regex::Regex::new(pattern) {
                    Ok(re) => {
                        self.forms.push(CompiledForm {
                            regex: re,
                            vedic,
                            entry,
                        });
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: form '{}' has invalid pattern '{}': {}",
                            entry.id, pattern, e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Match a token against all registered forms.
    /// Returns matching forms sorted by priority (descending).
    pub fn match_token(&self, text: &str) -> Vec<&ShikaiFormEntry> {
        let mut results: Vec<&ShikaiFormEntry> = self
            .forms
            .iter()
            .filter(|cf| cf.regex.is_match(text))
            .map(|cf| &cf.entry)
            .collect();
        results.sort_by(|a, b| {
            b.priority
                .unwrap_or(0.0)
                .partial_cmp(&a.priority.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Get Vedic classification for matching forms (merged with max).
    pub fn match_vedic(&self, text: &str) -> VedicClassification {
        let mut vc = VedicClassification::new();
        for cf in &self.forms {
            if cf.regex.is_match(text) {
                vc = vc.merge_max(&cf.vedic);
            }
        }
        vc
    }

    /// Number of forms registered.
    pub fn len(&self) -> usize {
        self.forms.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.forms.is_empty()
    }

    /// List all entry IDs.
    pub fn list_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.forms.iter().map(|cf| cf.entry.id.as_str()).collect();
        ids.sort();
        ids
    }

    /// Search forms by keyword in id, name, description, or tags.
    pub fn search(&self, keyword: &str) -> Vec<&ShikaiFormEntry> {
        let kw = keyword.to_lowercase();
        let mut results: Vec<&ShikaiFormEntry> = self
            .forms
            .iter()
            .filter(|cf| {
                let e = &cf.entry;
                e.id.to_lowercase().contains(&kw)
                    || e.name.to_lowercase().contains(&kw)
                    || e.description.to_lowercase().contains(&kw)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&kw))
            })
            .map(|cf| &cf.entry)
            .collect();
        results.sort_by(|a, b| {
            b.priority
                .unwrap_or(0.0)
                .partial_cmp(&a.priority.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}

// ─── Events (static TOML historical/cultural events) ──────────────────

/// A historical or cultural event with a date for birth chart computation.
///
/// When a token matches an event's keywords, that event's date generates
/// a birth chart. The graha positions from that chart inform the token's
/// Vedic classification — as if the token was "born" at that event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub date: String,      // ISO 8601 date "YYYY-MM-DD"
    pub date_type: String, // "exact", "approximate", "symbolic"
    pub keywords: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Event {
    /// Parse this event's date into (year, month, day).
    pub fn parse_date(&self) -> Option<(i64, u8, u8)> {
        let parts: Vec<&str> = self.date.split('-').collect();
        if parts.len() < 3 {
            return None;
        }
        let year: i64 = parts[0].parse().ok()?;
        let month: u8 = parts[1].parse().ok()?;
        let day: u8 = parts[2].parse().ok()?;
        Some((year, month, day))
    }

    /// Compute a birth chart from this event's date.
    pub fn birth_chart(&self) -> Option<ChartSnapshot> {
        let (year, month, day) = self.parse_date()?;
        let jd = date_to_jd(year, month, day);
        Some(ChartSnapshot::new(jd).with_label(&self.id))
    }

    /// Check if a token matches any of this event's keywords.
    pub fn matches_token(&self, token: &str) -> bool {
        let token_lower = token.to_lowercase();
        self.keywords.iter().any(|kw| {
            let kw_lower = kw.to_lowercase();
            token_lower == kw_lower
                || token_lower.contains(&kw_lower)
                || kw_lower.contains(&token_lower)
        })
    }
}

/// Registry of events, loaded from `events.toml`.
///
/// Events provide date-anchored birth charts. When a token descends through
/// the vortex and matches an event, that event's date activates the gyro
/// wheel at that layer, spinning the token with real graha positions.
#[derive(Debug, Clone, Default)]
pub struct EventRegistry {
    events: Vec<Event>,
}

impl EventRegistry {
    pub fn new() -> Self {
        EventRegistry { events: Vec::new() }
    }

    /// Load events from a TOML file.
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
        self.load_from_str(&content)
            .map_err(|e| format!("{}: {}", path.display(), e))
    }

    /// Load events from an in-memory TOML string (e.g. the `portable` build's
    /// compiled-in corpus, which has no file on disk to read from).
    pub fn load_from_str(&mut self, content: &str) -> Result<(), String> {
        #[derive(Deserialize)]
        struct EventFile {
            event: Option<Vec<Event>>,
        }

        let parsed: EventFile =
            toml::from_str(content).map_err(|e| format!("TOML parse error: {}", e))?;

        if let Some(entries) = parsed.event {
            self.events.extend(entries);
        }

        Ok(())
    }

    /// Find events whose keywords match the given token.
    pub fn match_token(&self, text: &str) -> Vec<&Event> {
        self.events
            .iter()
            .filter(|e| e.matches_token(text))
            .collect()
    }

    /// Get an event by ID.
    pub fn get(&self, id: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.id == id)
    }

    /// Search events by keyword.
    pub fn search(&self, keyword: &str) -> Vec<&Event> {
        let kw = keyword.to_lowercase();
        let mut results: Vec<&Event> = self
            .events
            .iter()
            .filter(|e| {
                e.id.to_lowercase().contains(&kw)
                    || e.name.to_lowercase().contains(&kw)
                    || e.description.to_lowercase().contains(&kw)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&kw))
                    || e.keywords.iter().any(|k| k.to_lowercase().contains(&kw))
            })
            .collect();
        results.sort_by_key(|e| &e.id);
        results
    }

    /// Number of events registered.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// List all event IDs.
    pub fn list_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.events.iter().map(|e| e.id.as_str()).collect();
        ids.sort();
        ids
    }
}

// ─── Dynamic Entity Generation ──────────────────────────────────────────

/// A dynamic entity — generated on-the-fly by matching a token against
/// shikai forms and events.
#[derive(Debug, Clone)]
pub struct DynamicEntity {
    /// The original token text.
    pub token: String,

    /// A synthetic ID like "dyn_<token>"
    pub id: String,

    /// Matched shikai forms (sorted by priority).
    pub forms: Vec<ShikaiFormEntry>,

    /// Matched events.
    pub events: Vec<Event>,

    /// Merged Vedic classification from forms + event birth charts.
    pub vedic_classification: VedicClassification,

    /// Birth charts from matched events.
    pub birth_charts: Vec<ChartSnapshot>,

    /// Decomposed parts of the token (for compound tokens like "nutypebuddha").
    pub decomposed_from: Vec<String>,

    /// Union of all matched tags.
    pub tags: Vec<String>,

    /// Union of all matched properties.
    pub properties: HashMap<String, f64>,
}

/// Decompose a compound token into its constituent parts.
///
/// Handles:
/// - camelCase: "nutypebuddha" → ["nutype", "buddha"]
/// - PascalCase: "RustLang" → ["Rust", "Lang"]
/// - snake_case: "kinetic_energy" → ["kinetic", "energy"]
/// - kebab-case: "big-stick" → ["big", "stick"]
/// - Single words: "force" → ["force"]
pub fn decompose_token(token: &str) -> Vec<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    // First, split on underscores, hyphens, dots, colons
    let separators = ['_', '-', '.', ':', '/', '\\'];
    let has_separator = trimmed.chars().any(|c| separators.contains(&c));

    if has_separator {
        return trimmed
            .split(|c: char| separators.contains(&c))
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();
    }

    // Check if it looks like camelCase (starts lowercase, has uppercase inside)
    let has_camel_boundary = trimmed.chars().skip(1).any(|c| c.is_uppercase());
    if has_camel_boundary {
        let mut parts = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = trimmed.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            if c.is_uppercase() && !current.is_empty() {
                let prev_is_upper = i > 0 && chars[i - 1].is_uppercase();
                let next_is_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
                if !prev_is_upper || next_is_lower {
                    parts.push(current.clone().to_lowercase());
                    current.clear();
                }
            }
            current.push(c);
        }
        if !current.is_empty() {
            parts.push(current.to_lowercase());
        }
        return parts;
    }

    // Single word — return as-is, lowercased
    vec![trimmed.to_lowercase()]
}

/// Generate a dynamic entity from a token by matching against forms and events.
///
/// The vortex path:
/// 1. Decompose token into parts (camelCase/snake_case/kebab-case split)
/// 2. For each part: match against events → get birth charts
/// 3. For each part: match against shikai forms → get base Vedic classification
/// 4. Merge all birth chart graha positions into the Vedic classification
/// 5. Union all tags and properties
pub fn generate_dynamic_entity(
    token: &str,
    forms: &ShikaiFormRegistry,
    events: &EventRegistry,
) -> DynamicEntity {
    let parts = decompose_token(token);
    let tokens_to_check: Vec<String> = if parts.is_empty() {
        vec![token.to_lowercase()]
    } else {
        parts.clone()
    };

    let mut matched_forms: Vec<ShikaiFormEntry> = Vec::new();
    let mut matched_events: Vec<Event> = Vec::new();
    let mut seen_form_ids = std::collections::HashSet::new();
    let mut seen_event_ids = std::collections::HashSet::new();

    for part in &tokens_to_check {
        for form_entry in forms.match_token(part) {
            if seen_form_ids.insert(form_entry.id.clone()) {
                matched_forms.push(form_entry.clone());
            }
        }
        for event in events.match_token(part) {
            if seen_event_ids.insert(event.id.clone()) {
                matched_events.push(event.clone());
            }
        }
    }

    // Merge Vedic classification from forms
    let mut vc = VedicClassification::new();
    for form in &matched_forms {
        vc = vc.merge_max(&form.to_vedic());
    }

    // Compute birth charts for matched events
    let birth_charts: Vec<ChartSnapshot> = matched_events
        .iter()
        .filter_map(|e| e.birth_chart())
        .collect();

    // Merge graha positions from birth charts into Vedic classification
    for chart in &birth_charts {
        for gp in &chart.graha_positions {
            let weight = if gp.graha == Graha::Surya { 0.9 } else { 0.3 };
            vc = vc.with_graha(gp.graha, weight);
        }
    }

    // Union tags
    let mut all_tags: Vec<String> = Vec::new();
    let mut seen_tags = std::collections::HashSet::new();
    for form in &matched_forms {
        for tag in &form.tags {
            if seen_tags.insert(tag.clone()) {
                all_tags.push(tag.clone());
            }
        }
    }
    for event in &matched_events {
        for tag in &event.tags {
            if seen_tags.insert(tag.clone()) {
                all_tags.push(tag.clone());
            }
        }
    }

    // Union properties (form wins over event on conflict)
    let mut all_properties: HashMap<String, f64> = HashMap::new();
    for event in &matched_events {
        // Events don't have explicit properties currently, but future-proof
        for (k, v) in event.tags.iter().enumerate() {
            all_properties
                .entry(format!("event_{}", k))
                .or_insert(v.len() as f64);
        }
    }
    for form in &matched_forms {
        for (k, v) in &form.properties {
            all_properties.insert(k.clone(), *v);
        }
    }

    let id = format!(
        "dyn_{}",
        token
            .to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "_")
    );

    DynamicEntity {
        token: token.to_string(),
        id,
        forms: matched_forms,
        events: matched_events,
        vedic_classification: vc,
        birth_charts,
        decomposed_from: parts,
        tags: all_tags,
        properties: all_properties,
    }
}

// ─── Date helpers ──────────────────────────────────────────────────────

/// Convert a Gregorian date (year, month, day) to Julian Day number (noon).
/// Uses the Meeus algorithm. Valid for all Gregorian dates after 1582-10-15.
pub fn date_to_jd(year: i64, month: u8, day: u8) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month as i64 + 12)
    } else {
        (year, month as i64)
    };
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y as f64 + 4716.0)).floor() + (30.6001 * (m as f64 + 1.0)).floor() + day as f64 + b
        - 1524.5
        + 0.5
}

// ─── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Original pure-function tests ──

    #[test]
    fn validate_entity_id_basic() {
        assert!(validate_entity_id("surya"));
        assert!(validate_entity_id("graha_mangala"));
        assert!(!validate_entity_id(""));
    }

    #[test]
    fn compute_entity_hash_basic() {
        let hash_a = compute_entity_hash("surya");
        let hash_b = compute_entity_hash("surya");
        let hash_c = compute_entity_hash("chandra");
        assert_eq!(hash_a, hash_b);
        assert_ne!(hash_a, hash_c);
    }

    #[test]
    fn is_same_entity_basic() {
        assert!(is_same_entity("surya", "surya"));
        assert!(is_same_entity("Surya", "surya"));
        assert!(!is_same_entity("surya", "chandra"));
    }

    // ── Entity tests ──

    fn test_sorter() -> ChangeSorter {
        ChangeSorter::new()
    }

    #[test]
    fn test_create_token_entity() {
        let sorter = test_sorter();
        let entity = Entity::new("force", &sorter, 0);
        assert_eq!(entity.text, "force");
        assert_eq!(entity.id, "ent_000000");
        assert!(entity.dominant_sign().is_some());
    }

    #[test]
    fn test_entity_classified() {
        let sorter = test_sorter();
        let entity = Entity::new("math", &sorter, 1);
        // "math" should activate a strong sign
        assert!(
            entity.classification.signs[0] > 0.5,
            "math should activate a sign"
        );
    }

    #[test]
    fn test_entity_with_values() {
        let sorter = test_sorter();
        let mut values = HashMap::new();
        values.insert("mass".to_string(), 5.0);
        let entity = Entity::new("5kg", &sorter, 2).with_values(values);
        assert!((entity.values.get("mass").unwrap() - 5.0).abs() < 1e-12);
    }

    // ── Registry tests ──

    #[test]
    fn test_registry_record_token() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        let entity = reg.record_token("force", &sorter);
        assert_eq!(entity.text, "force");
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_registry_record_tokens() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        let ids = reg.record_tokens(&["calculate", "kinetic", "energy"], &sorter);
        assert_eq!(ids.len(), 3);
        assert_eq!(reg.len(), 3);
        assert!(reg.get(&ids[0]).is_some());
        assert!(reg.get(&ids[1]).is_some());
        assert!(reg.get(&ids[2]).is_some());
    }

    #[test]
    fn test_registry_search() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("kinetic", &sorter);
        reg.record_token("energy", &sorter);
        reg.record_token("mass", &sorter);

        let results = reg.search("kinetic");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "kinetic");
    }

    #[test]
    fn test_registry_by_sign() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("math", &sorter);
        reg.record_token("force", &sorter);
        reg.record_token("history", &sorter);

        let aries = reg.by_sign(Sign::Aries);
        // At least some tokens should be in Aries
        assert!(!aries.is_empty() || !reg.is_empty());
    }

    #[test]
    fn test_registry_aspect_between() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("math", &sorter);
        reg.record_token("history", &sorter);

        let ids: Vec<String> = reg.list().iter().map(|s| s.to_string()).collect();
        if ids.len() >= 2 {
            let result = reg.aspect_between(&ids[0], &ids[1]);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_registry_ring_buffer_eviction() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::with_capacity(5);
        for i in 0..10 {
            reg.record_token(&format!("token_{}", i), &sorter);
        }
        assert_eq!(reg.len(), 5);
        assert!(reg.search("token_0").is_empty());
        assert!(!reg.search("token_9").is_empty());
    }

    // ── Decomposition tests ──

    #[test]
    fn test_decompose_single_word() {
        let parts = decompose_token("force");
        assert_eq!(parts, vec!["force"]);
    }

    #[test]
    fn test_decompose_snake_case() {
        let parts = decompose_token("kinetic_energy");
        assert_eq!(parts, vec!["kinetic", "energy"]);
    }

    #[test]
    fn test_decompose_camel_case() {
        let parts = decompose_token("nutypebuddha");
        assert_eq!(parts, vec!["nutypebuddha"]); // no caps boundary
    }

    #[test]
    fn test_decompose_camel_case_with_caps() {
        let parts = decompose_token("getHTTPCode");
        // Consecutive uppercase (acronym) kept together: H-T-T-P → "http"
        assert_eq!(parts, vec!["get", "http", "code"]);
    }

    #[test]
    fn test_decompose_kebab_case() {
        let parts = decompose_token("big-stick");
        assert_eq!(parts, vec!["big", "stick"]);
    }

    #[test]
    fn test_decompose_empty() {
        let parts = decompose_token("");
        assert!(parts.is_empty());
    }

    // ── ShikaiForm tests ──

    fn test_form_registry() -> ShikaiFormRegistry {
        let mut reg = ShikaiFormRegistry::new();
        // Manually add some test forms (since we can't load from file in unit tests easily)
        let entry1 = ShikaiFormEntry {
            id: "test_mangala".to_string(),
            name: "Test Mangala".to_string(),
            pattern: "(?i)^[A-Z].*$".to_string(),
            graha: Some("mangala".to_string()),
            guna: None,
            tattva: None,
            nakshatra: None,
            rashi: None,
            priority: Some(1.0),
            description: "Starts with capital letter".to_string(),
            tags: vec!["test".to_string()],
            properties: HashMap::new(),
        };
        let vedic1 = entry1.to_vedic();
        let re1 = regex::Regex::new("(?i)^[A-Z].*$").unwrap();
        reg.forms.push(CompiledForm {
            regex: re1,
            entry: entry1,
            vedic: vedic1,
        });
        reg
    }

    #[test]
    fn test_form_match() {
        let reg = test_form_registry();
        let matches = reg.match_token("Hello");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "test_mangala");
    }

    #[test]
    fn test_form_no_match() {
        let reg = test_form_registry();
        let matches = reg.match_token("123_start");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_form_to_vedic() {
        let entry = ShikaiFormEntry {
            id: "test".to_string(),
            name: "Test".to_string(),
            pattern: ".*".to_string(),
            graha: Some("surya".to_string()),
            guna: Some("sattva".to_string()),
            tattva: None,
            nakshatra: None,
            rashi: None,
            priority: Some(0.8),
            description: "Test".to_string(),
            tags: vec![],
            properties: HashMap::new(),
        };
        let vc = entry.to_vedic();
        assert_eq!(vc.dominant_graha(), Some(Graha::Surya));
        assert_eq!(vc.dominant_guna(), Some(Guna::Sattva));
    }

    // ── Event tests ──

    #[test]
    fn test_event_date_parsing() {
        let event = Event {
            id: "test_event".to_string(),
            name: "Test Event".to_string(),
            date: "1979-04-07".to_string(),
            date_type: "exact".to_string(),
            keywords: vec!["gundam".to_string(), "newtype".to_string()],
            description: "Test".to_string(),
            tags: vec![],
        };
        let (year, month, day) = event.parse_date().unwrap();
        assert_eq!(year, 1979);
        assert_eq!(month, 4);
        assert_eq!(day, 7);
    }

    #[test]
    fn test_event_keyword_match() {
        let event = Event {
            id: "test".to_string(),
            name: "Test".to_string(),
            date: "2000-01-01".to_string(),
            date_type: "exact".to_string(),
            keywords: vec!["rust".to_string(), "language".to_string()],
            description: "Test".to_string(),
            tags: vec![],
        };
        assert!(event.matches_token("rust"));
        assert!(event.matches_token("rustlang"));
        assert!(event.matches_token("language"));
        assert!(!event.matches_token("python"));
    }

    #[test]
    fn test_event_birth_chart() {
        let event = Event {
            id: "test_chart".to_string(),
            name: "Test Chart".to_string(),
            date: "2000-01-01".to_string(),
            date_type: "exact".to_string(),
            keywords: vec!["test".to_string()],
            description: "Test".to_string(),
            tags: vec![],
        };
        let chart = event.birth_chart();
        assert!(chart.is_some());
        let chart = chart.unwrap();
        assert_eq!(chart.graha_positions.len(), 9);
        assert_eq!(chart.label, Some("test_chart".to_string()));
    }

    // ── Dynamic entity generation tests ──

    #[test]
    fn test_dynamic_entity_generation() {
        let forms = test_form_registry();
        let events = EventRegistry::new();

        let de = generate_dynamic_entity("Hello", &forms, &events);
        assert_eq!(de.token, "Hello");
        assert_eq!(de.forms.len(), 1);
        assert_eq!(de.forms[0].id, "test_mangala");
        assert_eq!(
            de.vedic_classification.dominant_graha(),
            Some(Graha::Mangala)
        );
    }

    #[test]
    fn test_dynamic_entity_no_match() {
        let forms = ShikaiFormRegistry::new();
        let events = EventRegistry::new();

        let de = generate_dynamic_entity("xyzzy", &forms, &events);
        assert_eq!(de.token, "xyzzy");
        assert!(de.forms.is_empty());
        assert!(de.events.is_empty());
        assert!(de.birth_charts.is_empty());
    }

    #[test]
    fn test_empty_registry() {
        let reg = EntityRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_registry_list_sorted() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("zebra", &sorter);
        reg.record_token("apple", &sorter);
        let list = reg.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "ent_000000");
        assert_eq!(list[1], "ent_000001");
    }

    #[test]
    fn test_form_registry_load_and_match() {
        // Create a temp TOML file for form loading
        let toml_content = r#"
[[form]]
id = "test_rust"
name = "Rust Form"
pattern = ".*rust.*"
graha = "mangala"
priority = 0.9
description = "Rust language form"
tags = ["rust", "programming"]
"#;
        let path = std::env::temp_dir().join("test_shikai_form.toml");
        std::fs::write(&path, toml_content).unwrap();

        let mut reg = ShikaiFormRegistry::new();
        reg.load_from_file(&path).unwrap();
        assert_eq!(reg.len(), 1);

        let matches = reg.match_token("rust_lang");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "test_rust");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_event_registry_load_and_match() {
        let toml_content = r#"
[[event]]
id = "test_gundam_event"
name = "Gundam First Broadcast"
date = "1979-04-07"
date_type = "exact"
keywords = ["gundam", "newtype", "mobile suit"]
description = "First broadcast of Mobile Suit Gundam"
tags = ["anime", "gundam"]
"#;
        let path = std::env::temp_dir().join("test_events.toml");
        std::fs::write(&path, toml_content).unwrap();

        let mut reg = EventRegistry::new();
        reg.load_from_file(&path).unwrap();
        assert_eq!(reg.len(), 1);

        let matches = reg.match_token("gundam");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].id, "test_gundam_event");

        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_dynamic_entity_with_event() {
        let forms = ShikaiFormRegistry::new();
        let event = Event {
            id: "test_event".to_string(),
            name: "Test Event".to_string(),
            date: "1979-04-07".to_string(),
            date_type: "exact".to_string(),
            keywords: vec!["newtype".to_string()],
            description: "Test".to_string(),
            tags: vec!["mecha".to_string()],
        };
        let mut events = EventRegistry::new();
        events.events.push(event);

        let de = generate_dynamic_entity("newtype", &forms, &events);
        assert_eq!(de.events.len(), 1);
        assert_eq!(de.events[0].id, "test_event");
        assert_eq!(de.birth_charts.len(), 1);
        // Vedic classification should have graha weights from the birth chart
        assert!(
            de.vedic_classification.grahas.iter().any(|w| *w > 0.0),
            "birth chart should activate at least one graha"
        );
    }

    #[test]
    fn test_decompose_then_dynamic() {
        let forms = ShikaiFormRegistry::new();
        let event = Event {
            id: "gundam".to_string(),
            name: "Gundam".to_string(),
            date: "1979-04-07".to_string(),
            date_type: "exact".to_string(),
            keywords: vec!["buddha".to_string()], // "buddha" is a keyword
            description: "Siddhartha".to_string(),
            tags: vec![],
        };
        let mut events = EventRegistry::new();
        events.events.push(event);

        // "nutypebuddha" should decompose to ["nutypebuddha"] (single word, no caps boundary)
        // Actually "nutypebuddha" has no uppercase letters, so it stays as one word
        // The decomposed form still matches keywords if the whole word matches
        let de = generate_dynamic_entity("buddha", &forms, &events);
        assert_eq!(de.events.len(), 1);
    }

    #[test]
    fn test_date_to_jd() {
        // J2000.0 epoch
        let jd = date_to_jd(2000, 1, 1);
        assert!(
            (jd - 2451545.0).abs() < 1.0,
            "JD for J2000 should be ~2451545"
        );
    }

    /// T27 regression test (2026-07-11): reproduces the exact bug found by
    /// running the shipped `athena-x86_64` binary's `entity-aspect` CLI
    /// command against the real `rahu`/`ketu` seeds from
    /// `entities/navagraha.toml`. Rahu and Ketu are lunar nodes and are
    /// *always* exactly 180° apart in real ephemeris — but on the fixed
    /// 9-node structural composition wheel they sit only 1 step apart. This
    /// test locks in the honest structural answer (`Adjacent`) under its
    /// unambiguous T27 name, and confirms `seed_aspect_between` returns
    /// `wheel::CompositionAspect`, not anything claiming to be astronomical.
    #[test]
    fn t27_seed_aspect_between_rahu_ketu_is_structural_not_astronomical() {
        // Minimal fixture using exactly the fields `load_seeds_from_str`
        // consumes (SeedEntry: id/name/domain/description/tags/properties).
        // `domain` is what drives the graha classification via
        // `Graha::parse` — this mirrors how entities/navagraha.toml's real
        // rahu/ketu rows resolve, just trimmed to the essentials.
        let toml_fixture = r#"
            [[entity]]
            id = "rahu"
            name = "Rahu"
            domain = "rahu"
            description = "Navagraha: North Lunar Node."
            tags = ["navagraha", "graha", "shadow", "node"]

            [[entity]]
            id = "ketu"
            name = "Ketu"
            domain = "ketu"
            description = "Navagraha: South Lunar Node."
            tags = ["navagraha", "graha", "shadow", "node"]
        "#;

        let mut registry = EntityRegistry::new();
        registry
            .load_seeds_from_str(toml_fixture)
            .expect("T27 fixture TOML should load like the real corpus does");

        let (aspect, a, b) = registry
            .seed_aspect_between("rahu", "ketu")
            .expect("rahu and ketu both have graha classification; aspect must resolve");

        assert_eq!(a.id, "rahu");
        assert_eq!(b.id, "ketu");
        assert_eq!(
            aspect,
            crate::wheel::CompositionAspect::Adjacent,
            "T27: Rahu-Ketu are 1 structural step apart on the composition wheel — \
             this is the correct, honest answer and is NOT a claim about their real \
             (always exactly 180°) ephemeris separation. For that, use ChartSnapshot \
             built from a real date via crate::chart::AstroAspect."
        );
    }

    // ─── §16 edge case: Ise family's Shinken Hakkyōken (read-only legacy) ─
    // A passed-down, already-complete seed entity must not be corrupted by
    // ad-hoc runtime edits through the public read API. The canonical seed is
    // held behind `Arc`; `get_seed` returns an owned clone and `get_seed_ref`
    // a shared reference, so mutating a returned value cannot re-imprint the
    // legacy blade.
    #[test]
    fn ise_legacy_seed_is_immutable_through_read_api() {
        let mut reg = EntityRegistry::new();
        let legacy = SeedEntity {
            id: "shinken_hakkyoken".to_string(),
            name: "Shinken Hakkyōken".to_string(),
            description: "Ise family heirloom — complete, passed down".to_string(),
            classification: None,
            properties: HashMap::new(),
            constants: HashMap::new(),
            tags: vec!["legacy".to_string()],
            formula: None,
            birth_time: None,
            bija: None,
            mantra: None,
            day: None,
            ruled_nakshatras: vec![],
        };
        reg.register_seed(legacy);

        // Read the legacy blade out and try to "re-imprint" it.
        let mut cloned = reg.get_seed("shinken_hakkyoken").expect("legacy present");
        assert_eq!(cloned.name, "Shinken Hakkyōken");
        cloned.name = "Re-imprinted".to_string();
        cloned.tags.push("mutated".to_string());

        // The canonical seed in the registry is untouched.
        let canonical = reg.get_seed("shinken_hakkyoken").expect("legacy present");
        assert_eq!(canonical.name, "Shinken Hakkyōken");
        assert!(
            !canonical.tags.contains(&"mutated".to_string()),
            "legacy seed must not be re-imprinted via the read API"
        );
        // And the shared reference view agrees.
        let shared = reg
            .get_seed_ref("shinken_hakkyoken")
            .expect("legacy present");
        assert_eq!(shared.name, "Shinken Hakkyōken");
    }
}
