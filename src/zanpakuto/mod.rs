// ─── Stopwords ───────────────────────────────────────────────────────────────

/// Common English stopwords that carry little semantic meaning.
/// Excluded from keyword extraction to reduce noise in classification.
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "can",
    "need", "dare", "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by", "from",
    "as", "into", "through", "during", "before", "after", "above", "below", "between", "out",
    "off", "over", "under", "again", "further", "then", "once", "here", "there", "when", "where",
    "why", "how", "all", "each", "every", "both", "few", "more", "most", "other", "some", "such",
    "no", "not", "only", "own", "same", "so", "than", "too", "very", "just", "and", "but", "or",
    "if", "while", "that", "this", "these", "those", "what", "which", "who", "whom", "it", "its",
];

/// Pure function: Check if a token is a stopword.
pub fn is_stopword(token: &str) -> bool {
    STOPWORDS.contains(&token.to_lowercase().as_str())
}

// ─── Normalization ──────────────────────────────────────────────────────────

/// Pure function: Normalize query text for NLP processing.
/// Lowercases, collapses whitespace. Preserves comparison operators
/// (<, <=, >, >=, =, !=) and decimal numbers (3.14).
pub fn normalize_query_text(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;
    let mut prev_was_space = true;

    while i < len {
        let ch = chars[i];

        if ch.is_alphanumeric() || ch == '.' || ch == '-' {
            result.push(ch.to_ascii_lowercase());
            prev_was_space = false;
        } else if ch == '<' || ch == '>' || ch == '=' {
            if !prev_was_space {
                result.push(' ');
            }
            // Check for two-char operator: <=, >=
            if i + 1 < len && chars[i + 1] == '=' {
                result.push(ch);
                result.push('=');
                i += 1; // skip the '='
            } else {
                result.push(ch);
            }
            prev_was_space = false;
        } else if ch == '!' && i + 1 < len && chars[i + 1] == '=' {
            // Only preserve != as operator, not standalone !
            if !prev_was_space {
                result.push(' ');
            }
            result.push('!');
            result.push('=');
            i += 1; // skip the '='
            prev_was_space = false;
        } else if ch.is_whitespace() && !prev_was_space {
            result.push(' ');
            prev_was_space = true;
        }
        // All other chars (punctuation like !, @, #) silently dropped

        i += 1;
    }

    let mut normalized = result.trim().to_string();
    // Collapse any remaining double spaces
    while normalized.contains("  ") {
        normalized = normalized.replace("  ", " ");
    }
    normalized
}

/// Pure function: Extract keywords from normalized query.
/// Filters stopwords and tokens <= 2 chars (same as before, but now with proper stopword check).
pub fn extract_keywords(normalized_query: &str) -> Vec<String> {
    normalized_query
        .split_whitespace()
        .filter(|word| word.len() > 2 && !is_stopword(word))
        .map(|word| word.to_string())
        .collect()
}

/// Pure function: Extract all tokens including operators and short words.
/// Used when full token fidelity is needed (e.g., validation constraints).
pub fn extract_all_tokens(normalized_query: &str) -> Vec<String> {
    normalized_query
        .split_whitespace()
        .map(|word| word.to_string())
        .collect()
}

// ─── Function Syntax Parsing ────────────────────────────────────────────────

/// Parsed function call from query syntax like `compute(code_efficiency, energy)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFunction {
    /// Function name (e.g., "compute", "validate", "search").
    pub name: String,
    /// Function arguments (e.g., ["code_efficiency", "energy"]).
    pub args: Vec<String>,
}

/// Pure function: Parse function-call syntax from a query.
///
/// Recognizes patterns like `name(arg1, arg2)` and splits into name + args.
/// Returns `None` if the query doesn't contain function-call syntax.
pub fn parse_function_call(query: &str) -> Option<ParsedFunction> {
    let query = query.trim();

    // Find the opening parenthesis
    let paren_start = query.find('(')?;
    let name = query[..paren_start].trim().to_lowercase();

    // Find the closing parenthesis
    let args_str = query.get(paren_start + 1..)?;
    let paren_end = args_str.find(')')?;
    let args_raw = &args_str[..paren_end];

    if args_raw.is_empty() {
        return Some(ParsedFunction {
            name,
            args: Vec::new(),
        });
    }

    let args: Vec<String> = args_raw
        .split(',')
        .map(|a| a.trim().to_lowercase())
        .filter(|a| !a.is_empty())
        .collect();

    Some(ParsedFunction { name, args })
}

// ─── Intent Classification ──────────────────────────────────────────────────

/// Pure function: Parse natural language query into structured intent.
///
/// Supports:
/// - Function syntax: `compute(...)`, `validate(...)`, `search(...)`
/// - Natural language prefixes: "what is...", "how to...", "compute..."
/// - Default: "general"
pub fn parse_query_intent(query: &str) -> &'static str {
    // Check function-call syntax first (highest priority)
    if let Some(func) = parse_function_call(query) {
        return match func.name.as_str() {
            "compute" | "calculate" | "eval" | "evaluate" => "computation",
            "validate" | "check" | "verify" => "validation",
            "search" | "find" | "lookup" | "query" => "search",
            "entity" | "get" | "describe" => "entity",
            _ => "general",
        };
    }

    // Fall back to prefix matching
    let normalized = query.to_lowercase();
    if normalized.starts_with("what") || normalized.starts_with("how") {
        "question"
    } else if normalized.starts_with("compute") || normalized.starts_with("calculate") {
        "computation"
    } else if normalized.starts_with("validate") || normalized.starts_with("check") {
        "validation"
    } else if normalized.starts_with("find") || normalized.starts_with("search") {
        "search"
    } else {
        "general"
    }
}

/// Pure function: Extract numerical values from query text.
pub fn extract_numerical_values(query: &str) -> Vec<f64> {
    query
        .split_whitespace()
        .filter_map(|word| {
            word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                .parse::<f64>()
                .ok()
        })
        .collect()
}

/// Pure function: Determine domain from query keywords.
pub fn determine_query_domain(query: &str) -> &'static str {
    let normalized = query.to_lowercase();
    if normalized.contains("graha") || normalized.contains("planet") {
        "astrology"
    } else if normalized.contains("formula") || normalized.contains("equation") {
        "formula"
    } else if normalized.contains("entity") || normalized.contains("object") {
        "entity"
    } else if normalized.contains("test") || normalized.contains("validate") {
        "validation"
    } else {
        "general"
    }
}

// ─── Intent Score ───────────────────────────────────────────────────────────

/// Pure function: Compute query intent score based on keyword presence.
pub fn compute_intent_score(keywords: &[String], intent_markers: &[&str]) -> f64 {
    if keywords.is_empty() || intent_markers.is_empty() {
        return 0.0;
    }
    let matches = keywords
        .iter()
        .filter(|keyword| intent_markers.iter().any(|marker| keyword.contains(marker)))
        .count();
    matches as f64 / keywords.len() as f64
}

// ─── NLP Context ────────────────────────────────────────────────────────────

/// NLP context — pre-tokenized query from Zanpakuto preprocessing.
///
/// Holds the tokenized, stemmed tokens that the descent engine processes.
#[derive(Debug, Clone)]
pub struct NlpContext {
    /// Tokenized query terms (already cleaned and stemmed).
    pub tokens: Vec<String>,
    /// Original raw query text.
    pub raw_query: String,
    /// Parsed function call (if query used function syntax).
    pub function_call: Option<ParsedFunction>,
}

impl NlpContext {
    /// Create a new NLP context from a raw query string.
    pub fn from_query(query: &str) -> Self {
        let normalized = normalize_query_text(query);
        let tokens: Vec<String> = extract_keywords(&normalized)
            .into_iter()
            .map(|s| s.to_lowercase())
            .collect();
        let function_call = parse_function_call(query);
        NlpContext {
            tokens,
            raw_query: query.to_string(),
            function_call,
        }
    }

    /// Create from pre-tokenized list.
    pub fn from_tokens(tokens: Vec<String>) -> Self {
        NlpContext {
            tokens,
            raw_query: String::new(),
            function_call: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_query_text_basic() {
        assert_eq!(normalize_query_text("Hello World!"), "hello world");
        assert_eq!(normalize_query_text("  SPACED  "), "spaced");
        assert_eq!(normalize_query_text("hello   world"), "hello world");
    }

    #[test]
    fn normalize_preserves_operators() {
        assert!(normalize_query_text("memory < 1MB").contains('<'));
        assert!(normalize_query_text("score >= 0.8").contains(">="));
        assert!(normalize_query_text("x != 0").contains("!="));
    }

    #[test]
    fn normalize_preserves_decimals() {
        assert!(normalize_query_text("0.36 ms").contains("0.36"));
        assert!(normalize_query_text("3.14159 pi").contains("3.14159"));
    }

    #[test]
    fn extract_keywords_filters_stopwords() {
        let keywords = extract_keywords("what is the meaning of life");
        assert!(!keywords.contains(&"what".to_string()));
        assert!(!keywords.contains(&"is".to_string()));
        assert!(!keywords.contains(&"the".to_string()));
        assert!(!keywords.contains(&"of".to_string()));
        assert!(keywords.contains(&"meaning".to_string()));
        assert!(keywords.contains(&"life".to_string()));
    }

    #[test]
    fn extract_keywords_basic() {
        assert!(extract_keywords("a b").is_empty());
        let kw = extract_keywords("energy efficiency in code");
        assert!(kw.contains(&"energy".to_string()));
        assert!(kw.contains(&"efficiency".to_string()));
        assert!(kw.contains(&"code".to_string()));
    }

    #[test]
    fn is_stopword_works() {
        assert!(is_stopword("the"));
        assert!(is_stopword("and"));
        assert!(is_stopword("in"));
        assert!(!is_stopword("energy"));
        assert!(!is_stopword("efficiency"));
    }

    #[test]
    fn parse_function_call_basic() {
        let f = parse_function_call("compute(code_efficiency, energy)").unwrap();
        assert_eq!(f.name, "compute");
        assert_eq!(f.args, vec!["code_efficiency", "energy"]);
    }

    #[test]
    fn parse_function_call_no_args() {
        let f = parse_function_call("search()").unwrap();
        assert_eq!(f.name, "search");
        assert!(f.args.is_empty());
    }

    #[test]
    fn parse_function_call_none() {
        assert!(parse_function_call("energy efficiency").is_none());
    }

    #[test]
    fn parse_query_intent_function_syntax() {
        assert_eq!(
            parse_query_intent("compute(code_efficiency)"),
            "computation"
        );
        assert_eq!(parse_query_intent("validate(memory < 1MB)"), "validation");
        assert_eq!(parse_query_intent("search(optimization)"), "search");
        assert_eq!(parse_query_intent("entity(binary_size)"), "entity");
    }

    #[test]
    fn parse_query_intent_natural_language() {
        assert_eq!(parse_query_intent("what is minimalism"), "question");
        assert_eq!(parse_query_intent("how to optimize code"), "question");
        assert_eq!(parse_query_intent("compute the answer"), "computation");
        assert_eq!(parse_query_intent("check this formula"), "validation");
        assert_eq!(parse_query_intent("find all entities"), "search");
        assert_eq!(parse_query_intent("energy efficiency in code"), "general");
    }

    #[test]
    fn compute_intent_score_basic() {
        let keywords = vec!["what".to_string(), "meaning".to_string(), "of".to_string()];
        let markers = ["what", "how"];
        assert!((compute_intent_score(&keywords, &markers) - 1.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn extract_all_tokens_preserves_operators() {
        let normalized = normalize_query_text("memory < 1MB");
        let tokens = extract_all_tokens(&normalized);
        assert!(tokens.contains(&"<".to_string()));
        assert!(tokens.contains(&"1mb".to_string()));
    }
}
