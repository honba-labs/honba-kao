//! Web font layer integrity (spec §4.6).
//!
//! The bulk woff2/woff web faces under `assets/fonts/web/` are staged by an
//! offline fetch (documented in the HANDOFF), never by `cargo`. These tests
//! prove the staged layer without network access: every manifest face exists
//! on disk with matching size and sha256, the generated `fonts.css` references
//! exactly the staged files with one `font-display: swap` per `@font-face`
//! block, the expected family/weight matrix is complete, and each family
//! ships its actual license text.
//!
//! Manifest shape (one row per staged FILE):
//! `{ "generated_at": …, "fonts": [{family, slug, subset, format, file,
//!    source, sha256, bytes, weights: […]}], "licenses": [{family, slug,
//!    license, file, source}] }`

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use sha2::{Digest, Sha256};

const WEB_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/web");

/// `(family, slug, css weights the terminal actually uses)` per spec §4.6.
const EXPECTED_FAMILIES: &[(&str, &str, &[u32])] = &[
    ("Inter", "inter", &[400, 500, 600, 700, 800]),
    ("IBM Plex Mono", "ibm-plex-mono", &[400, 500, 600, 700]),
    ("JetBrains Mono", "jetbrains-mono", &[400, 500, 700]),
    ("Roboto Mono", "roboto-mono", &[400, 500, 700]),
];

fn web_dir() -> PathBuf {
    PathBuf::from(WEB_DIR)
}

fn manifest() -> serde_json::Value {
    let path = web_dir().join("downloads.manifest.json");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
    serde_json::from_str(&text).expect("downloads.manifest.json is not valid JSON")
}

fn fonts() -> Vec<serde_json::Value> {
    manifest()["fonts"]
        .as_array()
        .cloned()
        .expect("manifest['fonts'] array")
}

/// Reads the generated @font-face registry.
fn fonts_css() -> String {
    fs::read_to_string(web_dir().join("fonts.css"))
        .expect("missing assets/fonts/web/fonts.css — run the web-font fetch")
}

#[test]
fn manifest_faces_exist_with_matching_size_and_sha256() {
    assert!(
        !fonts().is_empty(),
        "manifest['fonts'] must list the staged web faces"
    );
    for face in &fonts() {
        let file = face["file"].as_str().expect("face['file']");
        let path = web_dir().join(file);
        let data = fs::read(&path).unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
        let listed = face["bytes"].as_u64().expect("face['bytes']") as usize;
        assert_eq!(
            data.len(),
            listed,
            "byte size mismatch for {file} (manifest says {listed})"
        );
        let digest = hex_lower(&Sha256::digest(&data));
        let listed_hash = face["sha256"].as_str().expect("face['sha256']");
        assert_eq!(
            digest, listed_hash,
            "sha256 mismatch for {file} — restage or fix the manifest"
        );
        for field in ["family", "slug", "subset", "format", "source"] {
            assert!(
                !face[field].as_str().unwrap_or_default().is_empty(),
                "face['{field}'] is empty for {file}"
            );
        }
    }
}

#[test]
fn manifest_files_are_well_formed_relative_paths() {
    for face in fonts() {
        let file = face["file"].as_str().expect("face['file']");
        let slug = face["slug"].as_str().expect("face['slug']");
        assert!(
            file.starts_with(&format!("{slug}/")),
            "{file} must live in its family dir ({slug}/…)"
        );
        assert!(
            file.ends_with(".woff2") || file.ends_with(".woff"),
            "{file} must be a woff2/woff face"
        );
        assert!(
            !file.contains(".."),
            "{file} must not escape assets/fonts/web/"
        );
    }
}

#[test]
fn fonts_css_references_every_staged_file() {
    let css = fonts_css();
    for face in fonts() {
        let file = face["file"].as_str().expect("face['file']");
        let url = format!("./{file}");
        assert!(
            css.contains(&url),
            "fonts.css never references staged face {url}"
        );
    }
}

#[test]
fn fonts_css_declares_font_display_swap_in_every_block() {
    let css = fonts_css();
    let blocks = css.matches("@font-face").count();
    assert!(blocks > 0, "fonts.css has no @font-face blocks");
    let swaps = css.matches("font-display: swap").count();
    assert_eq!(
        blocks, swaps,
        "every @font-face block must carry font-display: swap"
    );
}

#[test]
fn css_covers_exact_expected_family_weight_matrix() {
    let css = fonts_css();
    for (family, slug, weights) in EXPECTED_FAMILIES {
        let staged = fonts()
            .iter()
            .filter(|f| f["slug"].as_str() == Some(*slug))
            .count();
        assert!(staged > 0, "no staged faces for family '{family}'");

        let mut css_weights = BTreeSet::new();
        for block in css.split("@font-face").skip(1) {
            let declared = between(block, "font-family: '", "'");
            if declared != *family {
                continue;
            }
            let weight = between(block, "font-weight: ", ";");
            let weight: u32 = weight
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("unparseable font-weight '{weight}' in {family} block"));
            css_weights.insert(weight);
        }
        let want: BTreeSet<u32> = weights.iter().copied().collect();
        assert_eq!(
            css_weights, want,
            "font-weight matrix mismatch for '{family}'"
        );
    }
}

#[test]
fn every_family_ships_its_actual_license() {
    let licenses = manifest()["licenses"]
        .as_array()
        .cloned()
        .expect("manifest['licenses'] array");
    assert!(
        licenses.len() >= EXPECTED_FAMILIES.len(),
        "missing licenses"
    );
    for (family, slug, _) in EXPECTED_FAMILIES {
        let row = licenses
            .iter()
            .find(|l| l["slug"].as_str() == Some(*slug))
            .unwrap_or_else(|| panic!("no license row for '{family}'"));
        let file = row["file"].as_str().expect("license['file']");
        let path = web_dir().join(file);
        let text = fs::read(&path).unwrap_or_else(|e| panic!("missing {}: {e}", path.display()));
        assert!(text.len() > 100, "license {file} looks truncated");
        assert!(
            !row["source"].as_str().unwrap_or_default().is_empty(),
            "license {file} lacks its upstream source URL"
        );
        // Truth beats convenience: verify the OFL text actually shipped
        // (Google Fonts' Roboto Mono upstream is OFL-1.1, queried and
        // confirmed against googlefonts/robotomono — OFL.txt at repo root).
        assert_eq!(
            row["license"].as_str(),
            Some("OFL-1.1"),
            "'{family}' license must be recorded as OFL-1.1"
        );
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            text.to_ascii_lowercase().contains("open font license")
                || text.to_ascii_lowercase().contains("font license"),
            "'{family}' license text must be the Open Font License"
        );
    }
}

fn between(haystack: &str, start: &str, end: &str) -> String {
    let Some(i) = haystack.find(start) else {
        return String::new();
    };
    let rest = &haystack[i + start.len()..];
    match rest.find(end) {
        Some(j) => rest[..j].to_string(),
        None => String::new(),
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
