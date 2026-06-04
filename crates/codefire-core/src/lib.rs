use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectId(String);

impl ObjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub enum CoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Config(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Io(error) => write!(f, "{error}"),
            CoreError::Json(error) => write!(f, "{error}"),
            CoreError::Config(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<std::io::Error> for CoreError {
    fn from(error: std::io::Error) -> Self {
        CoreError::Io(error)
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        CoreError::Json(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selector {
    #[serde(rename = "type")]
    pub selector_type: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Atom {
    pub atom_id: String,
    pub kind: String,
    pub artifact_path: String,
    pub selector: Selector,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomIndex {
    #[serde(rename = "type")]
    pub type_tag: String,
    pub version: u32,
    pub atoms: Vec<Atom>,
    pub duplicate_atom_ids: Vec<String>,
}

pub fn build_atom_index(open_dir: &Path) -> Result<AtomIndex, CoreError> {
    let config = Config::parse(open_dir)?;
    let mut atoms = Vec::new();
    for rel in rel_files(open_dir)? {
        let path = open_dir.join(&rel);
        let rel_posix = to_posix(&rel);
        if config.matches("requirements", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "requirement")?);
        } else if config.matches("designs", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "design")?);
        } else if config.matches("adrs", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "adr")?);
        } else if config.matches("ops", &rel_posix) {
            atoms.extend(extract_markdown_atoms(&path, &rel_posix, "ops")?);
        } else if config.matches("code", &rel_posix) {
            atoms.extend(extract_explicit_cf_atoms(&path, &rel_posix, "code")?);
        } else if config.matches("tests", &rel_posix) {
            atoms.extend(extract_explicit_cf_atoms(&path, &rel_posix, "test")?);
        }
    }
    atoms.sort_by(|left, right| {
        left.atom_id
            .cmp(&right.atom_id)
            .then_with(|| left.artifact_path.cmp(&right.artifact_path))
    });
    let mut counts = BTreeMap::new();
    for atom in &atoms {
        *counts.entry(atom.atom_id.clone()).or_insert(0usize) += 1;
    }
    let duplicate_atom_ids = counts
        .into_iter()
        .filter_map(|(atom_id, count)| (count > 1).then_some(atom_id))
        .collect();
    Ok(AtomIndex {
        type_tag: "atom_index".to_string(),
        version: VERSION,
        atoms,
        duplicate_atom_ids,
    })
}

pub fn extract_markdown_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut headings = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        if let Some((level, atom_id)) = markdown_atom_heading(line) {
            headings.push((line_index, level, atom_id));
        }
    }

    let mut atoms = Vec::with_capacity(headings.len());
    for (index, (start, level, atom_id)) in headings.iter().enumerate() {
        let mut end = lines.len();
        for (next_start, next_level, _) in &headings[index + 1..] {
            if next_level <= level {
                end = *next_start;
                break;
            }
        }
        let content = normalized_block_content(&lines[*start..end]);
        atoms.push(Atom {
            atom_id: atom_id.clone(),
            kind: kind_from_atom_id(atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: "markdown_heading".to_string(),
                value: atom_id.clone(),
            },
            content_hash: hash_text(&content),
        });
    }
    Ok(atoms)
}

pub fn extract_explicit_cf_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut atoms = Vec::new();
    for line in &lines {
        if let Some(atom_id) = explicit_cf_atom_id(line) {
            let content = format!("{}\n", line.trim());
            atoms.push(Atom {
                atom_id: atom_id.clone(),
                kind: kind_from_atom_id(&atom_id, fallback_kind).to_string(),
                artifact_path: artifact_path.to_string(),
                selector: Selector {
                    selector_type: "explicit_cf_atom".to_string(),
                    value: atom_id,
                },
                content_hash: hash_text(&content),
            });
        }
    }
    Ok(atoms)
}

pub fn kind_from_atom_id(atom_id: &str, fallback: &str) -> &'static str {
    if atom_id.starts_with("REQ-") {
        "requirement"
    } else if atom_id.starts_with("DES-") {
        "design"
    } else if atom_id.starts_with("TEST-") {
        "test"
    } else if atom_id.starts_with("CODE-") || atom_id.starts_with("CODE:") {
        "code"
    } else if atom_id.starts_with("ADR-") {
        "adr"
    } else if atom_id.starts_with("OPS-") {
        "ops"
    } else if atom_id.starts_with("API-") {
        "api"
    } else if atom_id.starts_with("DB-") {
        "db"
    } else {
        match fallback {
            "requirement" => "requirement",
            "design" => "design",
            "test" => "test",
            "code" => "code",
            "adr" => "adr",
            "ops" => "ops",
            "api" => "api",
            "db" => "db",
            _ => "unknown",
        }
    }
}

fn markdown_atom_heading(line: &str) -> Option<(usize, String)> {
    let mut chars = line.chars().peekable();
    let mut level = 0usize;
    while chars.peek() == Some(&'#') && level < 6 {
        chars.next();
        level += 1;
    }
    if level == 0 || chars.next() != Some(' ') {
        return None;
    }
    let rest: String = chars.collect();
    let atom_id: String = rest
        .chars()
        .take_while(|ch| ch.is_ascii_uppercase() || is_atom_tail_char(*ch))
        .collect();
    if !is_valid_heading_atom_id(&atom_id) {
        return None;
    }
    let next = rest.chars().nth(atom_id.len());
    if matches!(next, None | Some(':') | Some(' ') | Some('\t')) {
        Some((level, atom_id))
    } else {
        None
    }
}

fn is_atom_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')
}

fn is_valid_heading_atom_id(atom_id: &str) -> bool {
    let Some((prefix, rest)) = atom_id.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_uppercase())
        && !rest.is_empty()
        && rest
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-'))
}

fn explicit_cf_atom_id(line: &str) -> Option<String> {
    let (_, after) = line.split_once("cf-atom:")?;
    let atom_id: String = after
        .trim_start()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | ':' | '-'))
        .collect();
    (!atom_id.is_empty()).then_some(atom_id)
}

fn normalized_block_content(lines: &[&str]) -> String {
    let joined = lines.join("\n");
    let trimmed = joined.trim();
    format!("{trimmed}\n")
}

fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("sha256:{}", hex_lower(&hasher.finalize()))
}

fn read_text_lossy(path: &Path) -> Result<String, CoreError> {
    Ok(String::from_utf8_lossy(&fs::read(path)?).into_owned())
}

fn split_lines_without_terminators(text: &str) -> Vec<&str> {
    text.lines().collect()
}

fn rel_files(open_dir: &Path) -> Result<Vec<PathBuf>, CoreError> {
    let mut files = Vec::new();
    collect_rel_files(open_dir, Path::new(""), &mut files)?;
    files.sort_by_key(|path| to_posix(path));
    Ok(files)
}

fn collect_rel_files(
    root: &Path,
    rel_dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), CoreError> {
    let dir = root.join(rel_dir);
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let rel = rel_dir.join(name.as_ref());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if matches!(name.as_ref(), ".codefire" | "__pycache__" | ".pytest_cache") {
                continue;
            }
            collect_rel_files(root, &rel, files)?;
        } else if file_type.is_file() && to_posix(&rel) != ".codefire-open" {
            files.push(rel);
        }
    }
    Ok(())
}

fn to_posix(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[derive(Debug, Clone)]
struct Config {
    patterns: HashMap<&'static str, Vec<String>>,
}

impl Config {
    fn parse(open_dir: &Path) -> Result<Self, CoreError> {
        let defaults = default_config_patterns();
        let path = open_dir.join("codefire.yaml");
        if !path.exists() {
            return Ok(Self { patterns: defaults });
        }

        let mut parsed: HashMap<&'static str, Vec<String>> =
            defaults.keys().map(|key| (*key, Vec::new())).collect();
        let mut current = None;
        for (line_index, raw_line) in read_text_lossy(&path)?.lines().enumerate() {
            let line_without_comment = raw_line.split('#').next().unwrap_or_default();
            let stripped = line_without_comment.trim();
            if stripped.is_empty() {
                continue;
            }
            if !raw_line.starts_with([' ', '\t']) {
                current = None;
            }
            if let Some(key) = config_section_key(stripped) {
                current = Some(key);
                continue;
            }
            if let Some(section) = current {
                if let Some(value) = stripped.strip_prefix("- path:") {
                    let value = unquote(value.trim());
                    if value.is_empty() {
                        return Err(CoreError::Config(format!(
                            "invalid codefire.yaml: empty path at line {}",
                            line_index + 1
                        )));
                    }
                    parsed.get_mut(section).expect("known section").push(value);
                    continue;
                }
                if stripped.starts_with("- ") {
                    return Err(CoreError::Config(format!(
                        "invalid codefire.yaml: artifact entry at line {} must start with 'path:'",
                        line_index + 1
                    )));
                }
            }
        }
        for (key, value) in defaults {
            if parsed.get(key).is_some_and(Vec::is_empty) {
                parsed.insert(key, value);
            }
        }
        Ok(Self { patterns: parsed })
    }

    fn matches(&self, section: &'static str, rel: &str) -> bool {
        self.patterns
            .get(section)
            .is_some_and(|patterns| patterns.iter().any(|pattern| path_matches(pattern, rel)))
    }
}

fn default_config_patterns() -> HashMap<&'static str, Vec<String>> {
    HashMap::from([
        (
            "requirements",
            vec![
                "docs/spec/**/*.md".to_string(),
                "docs/requirements/**/*.md".to_string(),
            ],
        ),
        ("designs", vec!["docs/design/**/*.md".to_string()]),
        (
            "adrs",
            vec![
                "adr/**/*.md".to_string(),
                "docs/adr/**/*.md".to_string(),
                "docs/adrs/**/*.md".to_string(),
            ],
        ),
        (
            "ops",
            vec![
                "docs/ops/**/*.md".to_string(),
                "docs/runbooks/**/*.md".to_string(),
                "runbooks/**/*.md".to_string(),
            ],
        ),
        ("apis", Vec::new()),
        ("db", Vec::new()),
        ("code", vec!["src/**/*.py".to_string()]),
        ("tests", vec!["tests/**/*.py".to_string()]),
    ])
}

fn config_section_key(stripped: &str) -> Option<&'static str> {
    let section = stripped.strip_suffix(':')?;
    match section {
        "requirements" => Some("requirements"),
        "designs" => Some("designs"),
        "adrs" => Some("adrs"),
        "ops" => Some("ops"),
        "apis" => Some("apis"),
        "db" => Some("db"),
        "code" => Some("code"),
        "tests" => Some("tests"),
        _ => None,
    }
}

fn path_matches(pattern: &str, rel: &str) -> bool {
    wildcard_match(pattern, rel)
        || pattern.split_once("**/").is_some_and(|(prefix, suffix)| {
            rel.starts_with(prefix) && wildcard_match(suffix, &rel[prefix.len()..])
                || wildcard_match(&format!("{prefix}{suffix}"), rel)
        })
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let (mut p, mut t) = (0usize, 0usize);
    let mut star = None;
    let mut star_text = 0usize;
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == text[t] || pattern[p] == b'?') {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            star_text = t;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            star_text += 1;
            t = star_text;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn markdown_atoms_match_heading_scope_and_python_hash_format() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("spec.md");
        fs::write(
            &path,
            "# Intro\n\n## REQ-AUTH-001: Session expiration\n\nBody\n\n### DES-NESTED-IGNORED: detail\n\nNested\n\n## REQ-AUTH-002 Next\nTail\n",
        )
        .unwrap();

        let atoms = extract_markdown_atoms(&path, "docs/spec/auth.md", "requirement").unwrap();

        assert_eq!(atoms.len(), 3);
        assert_eq!(atoms[0].atom_id, "REQ-AUTH-001");
        assert_eq!(atoms[0].kind, "requirement");
        assert_eq!(atoms[0].selector.selector_type, "markdown_heading");
        assert_eq!(
            atoms[0].content_hash,
            "sha256:c0bf4a8b5b28a4f811bea3dad3ed81c94931b3b93b7b300eed250531f09aab10"
        );
        assert_eq!(atoms[1].atom_id, "DES-NESTED-IGNORED");
        assert_eq!(atoms[1].kind, "design");
        assert_eq!(atoms[2].atom_id, "REQ-AUTH-002");
    }

    #[test]
    fn explicit_cf_atoms_extract_comment_markers_without_symbol_parsing() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("app.py");
        fs::write(
            &path,
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n",
        )
        .unwrap();

        let atoms = extract_explicit_cf_atoms(&path, "src/app.py", "code").unwrap();

        assert_eq!(atoms.len(), 1);
        assert_eq!(atoms[0].atom_id, "CODE-SessionPolicy");
        assert_eq!(atoms[0].kind, "code");
        assert_eq!(atoms[0].selector.selector_type, "explicit_cf_atom");
        assert_eq!(
            atoms[0].content_hash,
            "sha256:e77db9516bb440255ebb1177be2f1aa3b5fcba7f46e43dd86871783fdf97cee4"
        );
    }

    #[test]
    fn build_atom_index_uses_defaults_config_and_detects_duplicates() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("docs/spec/auth.md"),
            "## REQ-AUTH-001: Session expiration\n\nRequirement.\n",
        );
        write_file(
            &temp.path().join("docs/design/auth.md"),
            "## DES-AUTH-001: Session design\n\nDesign.\n",
        );
        write_file(
            &temp.path().join("src/app.py"),
            "# cf-atom: CODE-SessionPolicy\nclass SessionPolicy:\n    pass\n",
        );
        write_file(
            &temp.path().join("tests/test_app.py"),
            "# cf-atom: TEST-session-policy\ndef test_session_policy():\n    pass\n",
        );
        write_file(
            &temp.path().join("src/duplicate.py"),
            "# cf-atom: CODE-SessionPolicy\n",
        );

        let index = build_atom_index(temp.path()).unwrap();
        let ids = index
            .atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(index.type_tag, "atom_index");
        assert_eq!(
            ids,
            vec![
                "CODE-SessionPolicy",
                "CODE-SessionPolicy",
                "DES-AUTH-001",
                "REQ-AUTH-001",
                "TEST-session-policy",
            ]
        );
        assert_eq!(index.duplicate_atom_ids, vec!["CODE-SessionPolicy"]);
    }

    #[test]
    fn codefire_yaml_overrides_artifact_patterns() {
        let temp = tempdir().unwrap();
        write_file(
            &temp.path().join("codefire.yaml"),
            "requirements:\n  - path: product/*.md\ncode:\n  - path: lib/*.rs\n",
        );
        write_file(
            &temp.path().join("product/auth.md"),
            "## REQ-CUSTOM-001: Custom requirement\n",
        );
        write_file(
            &temp.path().join("lib/main.rs"),
            "// cf-atom: CODE-RustMain\n",
        );

        let index = build_atom_index(temp.path()).unwrap();
        let ids = index
            .atoms
            .iter()
            .map(|atom| atom.atom_id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["CODE-RustMain", "REQ-CUSTOM-001"]);
    }

    fn write_file(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = fs::File::create(path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }
}
