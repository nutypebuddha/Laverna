use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=formulas/");
    println!("cargo:rerun-if-changed=entities/");
    println!("cargo:rerun-if-changed=shikai_form.toml");
    println!("cargo:rerun-if-changed=events.toml");

    let embedded = out_dir.join("embedded.rs");
    let mut src = String::new();

    // Seed corpus is always embedded so the binary is self-contained regardless
    // of CWD (fixes T35: entities/formulas/forms/events must load from anywhere).

    let formula_files = [
        "formulas/atomic_seed.toml",
        "formulas/atomic_dynamic.toml",
        "formulas/bridging_seed.toml",
        "formulas/bridging_dynamic.toml",
        "formulas/vortex_seed.toml",
        "formulas/vortex_dynamic.toml",
    ];

    let mut formulas = String::new();
    for rel in formula_files {
        let path = manifest.join(rel);
        if let Ok(content) = std::fs::read_to_string(&path) {
            formulas.push_str(&content);
            formulas.push('\n');
        }
    }

    let synonyms_path = manifest.join("formulas/search_synonyms.toml");
    let synonyms = std::fs::read_to_string(&synonyms_path).unwrap_or_default();

    let nonmath_files = [
        "formulas/nonmath_seed.toml",
        "formulas/nonmath_dynamic.toml",
    ];
    let mut nonmath = String::new();
    for rel in nonmath_files {
        let path = manifest.join(rel);
        if let Ok(content) = std::fs::read_to_string(&path) {
            nonmath.push_str(&content);
            nonmath.push('\n');
        }
    }

    let shikai_form_path = manifest.join("shikai_form.toml");
    let shikai_form = std::fs::read_to_string(&shikai_form_path).unwrap_or_default();

    let events_path = manifest.join("events.toml");
    let events_toml = std::fs::read_to_string(&events_path).unwrap_or_default();

    let entities_dir = manifest.join("entities");
    let mut entities = String::new();
    if let Ok(entries) = std::fs::read_dir(&entities_dir) {
        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "toml"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                entities.push_str(&content);
                entities.push('\n');
            }
        }
    }

    src.push_str(&format!(
        "pub const FORMULAS_TOML: &str = {:?};\n",
        formulas
    ));
    src.push_str(&format!(
        "pub const SYNONYMS_TOML: &str = {:?};\n",
        synonyms
    ));
    src.push_str(&format!(
        "pub const ENTITIES_TOML: &str = {:?};\n",
        entities
    ));
    src.push_str(&format!("pub const NONMATH_TOML: &str = {:?};\n", nonmath));
    src.push_str(&format!(
        "pub const SHIKAI_FORM_TOML: &str = {:?};\n",
        shikai_form
    ));
    src.push_str(&format!(
        "pub const EVENTS_TOML: &str = {:?};\n",
        events_toml
    ));

    std::fs::write(&embedded, src).expect("write embedded.rs");
}
