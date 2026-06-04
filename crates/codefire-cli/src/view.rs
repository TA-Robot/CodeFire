use super::http::{http_json, http_remote_path, parse_cf_http_url};
use super::remote::{
    ensure_object_dirs, load_remote_branch, parse_cf_url, remote_dirs, sha256_hex,
    write_object_records,
};
use super::*;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffAlgorithm {
    Myers,
    Patience,
    Histogram,
}

impl DiffAlgorithm {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "myers" => Some(Self::Myers),
            "patience" => Some(Self::Patience),
            "histogram" => Some(Self::Histogram),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiffOptions {
    pub(crate) algorithm: DiffAlgorithm,
    pub(crate) rename_detection: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            algorithm: DiffAlgorithm::Myers,
            rename_detection: false,
        }
    }
}

#[derive(Debug)]
struct ResolvedCommitish {
    objects: PathBuf,
    commit_id: String,
    label: String,
}

pub(crate) fn show_commitish(repo_root: Option<&Path>, value: &str) -> Result<String, CliError> {
    let resolved = resolve_commitish_any(repo_root, value)?;
    let commit = codefire_store::read_object(&resolved.objects, &resolved.commit_id)?;
    let manifest = root_object(&resolved.objects, &commit, "content_manifest")?;
    let atom_index = root_object(&resolved.objects, &commit, "atom_index")?;
    let files = manifest
        .get("entries")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let atoms = atom_index
        .get("atoms")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let message = commit.get("message").and_then(Value::as_str).unwrap_or("");
    let parents = commit
        .get("parents")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| "(none)".to_string());
    let certificate = commit
        .get("certificate")
        .and_then(|value| value.get("result"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let signature = commit.get("signature").and_then(Value::as_object);
    let signature = signature
        .map(|item| {
            format!(
                "{} ({})",
                item.get("signer")
                    .and_then(Value::as_str)
                    .unwrap_or("(unknown)"),
                item.get("algorithm")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            )
        })
        .unwrap_or_else(|| "(none)".to_string());

    Ok(format!(
        "Object: {}\nCommit: {}\nMessage: {message}\nParents: {parents}\nFiles: {files}\nAtoms: {atoms}\nCertificate: {certificate}\nSignature: {signature}\n",
        resolved.label, resolved.commit_id
    ))
}

pub(crate) fn diff_commitish_with_options(
    repo_root: Option<&Path>,
    left: &str,
    right: &str,
    options: &DiffOptions,
) -> Result<String, CliError> {
    let left = resolve_commitish_any(repo_root, left)?;
    let right = resolve_commitish_any(repo_root, right)?;
    let left_files = manifest_contents(&left.objects, &left.commit_id)?;
    let right_files = manifest_contents(&right.objects, &right.commit_id)?;
    Ok(render_manifest_diff(
        &left.label,
        &left_files,
        &right.label,
        &right_files,
        options,
    ))
}

fn resolve_local_commitish(repo_root: &Path, value: &str) -> Result<(String, String), CliError> {
    let objects = repo_root.join(".codefire").join("objects");
    if value.starts_with("CF-COMMIT-") {
        codefire_store::validate_sealed_commit(&objects, value)?;
        return Ok((value.to_string(), value.to_string()));
    }
    let branch = load_branch_record(repo_root, value)?;
    let head = required_string(&branch, &["head"])?;
    codefire_store::validate_sealed_commit(&objects, &head)?;
    Ok((head.clone(), format!("{value}@{head}")))
}

fn resolve_commitish_any(
    repo_root: Option<&Path>,
    value: &str,
) -> Result<ResolvedCommitish, CliError> {
    if value.starts_with("cf+http://") {
        let remote = parse_cf_http_url(value)?;
        let bundle = http_json(
            "GET",
            &http_remote_path(
                &remote.endpoint,
                &remote.org,
                &remote.app,
                &["branches", &remote.branch, "bundle"],
            ),
            None,
        )?;
        let branch = bundle.get("branch").ok_or_else(|| {
            CliError::InvalidRepository("HTTP remote returned an invalid branch bundle".to_string())
        })?;
        let records = bundle
            .get("objects")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CliError::InvalidRepository(
                    "HTTP remote returned an invalid branch bundle".to_string(),
                )
            })?;
        let temp_objects = env::temp_dir().join(format!(
            "codefire-http-objects-{}-{}",
            std::process::id(),
            stable_hash_48(value.as_bytes())
        ));
        if temp_objects.exists() {
            fs::remove_dir_all(&temp_objects)?;
        }
        ensure_object_dirs(&temp_objects)?;
        write_object_records(&temp_objects, records.iter().cloned())?;
        let head = required_string(branch, &["head"])?;
        codefire_store::validate_sealed_commit(&temp_objects, &head)?;
        return Ok(ResolvedCommitish {
            objects: temp_objects,
            commit_id: head.clone(),
            label: format!("{value}@{head}"),
        });
    }
    if value.starts_with("cf://") {
        let remote = parse_cf_url(value)?;
        let branch = load_remote_branch(&remote)?;
        let head = required_string(&branch, &["head"])?;
        let objects = remote_dirs(&remote.project_root).objects;
        codefire_store::validate_sealed_commit(&objects, &head)?;
        return Ok(ResolvedCommitish {
            objects,
            commit_id: head.clone(),
            label: format!("{value}@{head}"),
        });
    }
    let repo_root = repo_root.ok_or_else(|| {
        CliError::Usage(
            "local branch or commit requires running inside a CodeFire repository".to_string(),
        )
    })?;
    let (commit_id, label) = resolve_local_commitish(repo_root, value)?;
    Ok(ResolvedCommitish {
        objects: repo_root.join(".codefire").join("objects"),
        commit_id,
        label,
    })
}

fn root_object(objects: &Path, commit: &Value, root_name: &str) -> Result<Value, CliError> {
    let object_id = required_string(commit, &["roots", root_name])?;
    Ok(codefire_store::read_object(objects, &object_id)?)
}

pub(crate) fn manifest_contents(
    objects: &Path,
    commit_id: &str,
) -> Result<BTreeMap<String, Vec<u8>>, CliError> {
    let commit = codefire_store::read_object(objects, commit_id)?;
    let manifest = root_object(objects, &commit, "content_manifest")?;
    let entries = manifest
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CliError::InvalidRepository("content manifest entries must be a list".to_string())
        })?;
    let mut contents = BTreeMap::new();
    for entry in entries {
        if entry.get("kind").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let rel_path = required_string(entry, &["path"])?;
        let blob_id = required_string(entry, &["blob"])?;
        let blob = codefire_store::read_object(objects, &blob_id)?;
        let encoding = required_string(&blob, &["encoding"])?;
        if encoding != "base64" {
            return Err(CliError::InvalidRepository(format!(
                "unsupported blob encoding: {encoding}"
            )));
        }
        let content = required_string(&blob, &["content"])?;
        contents.insert(rel_path, decode_base64(&content)?);
    }
    Ok(contents)
}

fn render_manifest_diff(
    left_label: &str,
    left: &BTreeMap<String, Vec<u8>>,
    right_label: &str,
    right: &BTreeMap<String, Vec<u8>>,
    options: &DiffOptions,
) -> String {
    let file_moves = if options.rename_detection {
        detect_file_moves(left, right)
    } else {
        FileMoveDetection::default()
    };
    let paths = left
        .keys()
        .chain(right.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut output = String::new();
    let mut changed = false;

    for rename in &file_moves.renames {
        changed = true;
        output.push_str(&format!(
            "rename {} -> {} ({}% similarity)\n",
            rename.source, rename.target, rename.score
        ));
        render_file_delta(
            &format!("{left_label}/{}", rename.source),
            left.get(&rename.source),
            &format!("{right_label}/{}", rename.target),
            right.get(&rename.target),
            &mut output,
            options.algorithm,
        );
    }

    for copy in &file_moves.copies {
        changed = true;
        output.push_str(&format!(
            "copy {} -> {} ({}% similarity)\n",
            copy.source, copy.target, copy.score
        ));
        render_file_delta(
            &format!("{left_label}/{}", copy.source),
            left.get(&copy.source),
            &format!("{right_label}/{}", copy.target),
            right.get(&copy.target),
            &mut output,
            options.algorithm,
        );
    }

    for path in paths {
        if file_moves.skip_paths.contains(&path) {
            continue;
        }
        let left_data = left.get(&path);
        let right_data = right.get(&path);
        if left_data == right_data {
            continue;
        }
        changed = true;
        let left_path = if left_data.is_some() {
            format!("{left_label}/{path}")
        } else {
            format!("{left_label}/{path} (missing)")
        };
        let right_path = if right_data.is_some() {
            format!("{right_label}/{path}")
        } else {
            format!("{right_label}/{path} (missing)")
        };
        render_file_delta(
            &left_path,
            left_data,
            &right_path,
            right_data,
            &mut output,
            options.algorithm,
        );
    }
    if !changed {
        output.push_str("No differences.\n");
    }
    output
}

fn render_file_delta(
    left_path: &str,
    left_data: Option<&Vec<u8>>,
    right_path: &str,
    right_data: Option<&Vec<u8>>,
    output: &mut String,
    algorithm: DiffAlgorithm,
) {
    output.push_str(&format!("--- {left_path}\n+++ {right_path}\n"));
    let left_bytes = left_data.map(Vec::as_slice).unwrap_or(&[]);
    let right_bytes = right_data.map(Vec::as_slice).unwrap_or(&[]);
    if is_binary(left_bytes) || is_binary(right_bytes) {
        output.push_str(&format!(
            "Binary files differ: {} -> {}\n",
            binary_summary(left_data),
            binary_summary(right_data)
        ));
        return;
    }
    render_line_delta(left_bytes, right_bytes, output, algorithm);
}

#[derive(Debug, Default)]
struct FileMoveDetection {
    renames: Vec<FileMove>,
    copies: Vec<FileMove>,
    skip_paths: BTreeSet<String>,
}

#[derive(Debug)]
struct FileMove {
    source: String,
    target: String,
    score: u8,
}

fn detect_file_moves(
    left: &BTreeMap<String, Vec<u8>>,
    right: &BTreeMap<String, Vec<u8>>,
) -> FileMoveDetection {
    const MIN_SIMILARITY: u8 = 60;
    let deleted = left
        .keys()
        .filter(|path| !right.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    let added = right
        .keys()
        .filter(|path| !left.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();

    let mut detection = FileMoveDetection::default();
    let mut used_deleted = BTreeSet::new();
    let mut used_added = BTreeSet::new();
    let mut rename_candidates = Vec::new();
    for source in &deleted {
        for target in &added {
            let score = similarity_score(&left[source], &right[target]);
            if score >= MIN_SIMILARITY {
                rename_candidates.push((score, source.clone(), target.clone()));
            }
        }
    }
    rename_candidates.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    for (score, source, target) in rename_candidates {
        if used_deleted.insert(source.clone()) && used_added.insert(target.clone()) {
            detection.skip_paths.insert(source.clone());
            detection.skip_paths.insert(target.clone());
            detection.renames.push(FileMove {
                source,
                target,
                score,
            });
        }
    }

    for target in added {
        if used_added.contains(&target) {
            continue;
        }
        let Some((source, score)) = best_copy_source(left, right, &target, MIN_SIMILARITY) else {
            continue;
        };
        detection.skip_paths.insert(target.clone());
        detection.copies.push(FileMove {
            source,
            target,
            score,
        });
    }

    detection.renames.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.target.cmp(&right.target))
    });
    detection.copies.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.target.cmp(&right.target))
    });
    detection
}

fn best_copy_source(
    left: &BTreeMap<String, Vec<u8>>,
    right: &BTreeMap<String, Vec<u8>>,
    target: &str,
    min_similarity: u8,
) -> Option<(String, u8)> {
    left.iter()
        .filter(|(source, _)| right.contains_key(*source))
        .filter_map(|(source, source_data)| {
            let score = similarity_score(source_data, &right[target]);
            (score >= min_similarity).then(|| (source.clone(), score))
        })
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
}

fn similarity_score(left: &[u8], right: &[u8]) -> u8 {
    if left == right {
        return 100;
    }
    if left.is_empty() || right.is_empty() || is_binary(left) || is_binary(right) {
        return 0;
    }
    let left_lines = split_lines_lossy(left);
    let right_lines = split_lines_lossy(right);
    if left_lines.is_empty() || right_lines.is_empty() {
        return 0;
    }
    let common = lcs_pairs(&left_lines, &right_lines, None).len();
    let total = left_lines.len() + right_lines.len();
    ((common * 200 + total / 2) / total).min(100) as u8
}

fn is_binary(data: &[u8]) -> bool {
    data.contains(&0) || std::str::from_utf8(data).is_err()
}

fn binary_summary(data: Option<&Vec<u8>>) -> String {
    let Some(data) = data else {
        return "missing".to_string();
    };
    format!("{} bytes sha256:{}", data.len(), &sha256_hex(data)[..12])
}

fn render_line_delta(left: &[u8], right: &[u8], output: &mut String, algorithm: DiffAlgorithm) {
    let left_lines = split_lines_lossy(left);
    let right_lines = split_lines_lossy(right);
    for op in diff_lines(&left_lines, &right_lines, algorithm) {
        match op {
            DiffOp::Equal(_) => {}
            DiffOp::Delete(line) => push_diff_line(output, '-', line),
            DiffOp::Insert(line) => push_diff_line(output, '+', line),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiffOp<'a> {
    Equal(&'a str),
    Delete(&'a str),
    Insert(&'a str),
}

fn diff_lines<'a>(
    left: &'a [String],
    right: &'a [String],
    algorithm: DiffAlgorithm,
) -> Vec<DiffOp<'a>> {
    match algorithm {
        DiffAlgorithm::Myers => lcs_script(left, right),
        DiffAlgorithm::Patience => anchored_script(left, right, AnchorMode::Patience, 0),
        DiffAlgorithm::Histogram => anchored_script(left, right, AnchorMode::Histogram, 0),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnchorMode {
    Patience,
    Histogram,
}

fn anchored_script<'a>(
    left: &'a [String],
    right: &'a [String],
    mode: AnchorMode,
    depth: usize,
) -> Vec<DiffOp<'a>> {
    if left.is_empty() || right.is_empty() || depth > 128 {
        return lcs_script(left, right);
    }
    let anchors = match mode {
        AnchorMode::Patience => patience_anchors(left, right),
        AnchorMode::Histogram => histogram_anchors(left, right),
    };
    if anchors.is_empty() {
        return lcs_script(left, right);
    }

    let mut ops = Vec::new();
    let mut left_start = 0usize;
    let mut right_start = 0usize;
    for (left_anchor, right_anchor) in anchors {
        ops.extend(anchored_script(
            &left[left_start..left_anchor],
            &right[right_start..right_anchor],
            mode,
            depth + 1,
        ));
        ops.push(DiffOp::Equal(&left[left_anchor]));
        left_start = left_anchor + 1;
        right_start = right_anchor + 1;
    }
    ops.extend(anchored_script(
        &left[left_start..],
        &right[right_start..],
        mode,
        depth + 1,
    ));
    ops
}

fn lcs_script<'a>(left: &'a [String], right: &'a [String]) -> Vec<DiffOp<'a>> {
    let pairs = lcs_pairs(left, right, None);
    script_from_pairs(left, right, &pairs)
}

fn script_from_pairs<'a>(
    left: &'a [String],
    right: &'a [String],
    pairs: &[(usize, usize)],
) -> Vec<DiffOp<'a>> {
    let mut ops = Vec::with_capacity(left.len() + right.len());
    let mut left_cursor = 0usize;
    let mut right_cursor = 0usize;
    for &(left_match, right_match) in pairs {
        for line in &left[left_cursor..left_match] {
            ops.push(DiffOp::Delete(line));
        }
        for line in &right[right_cursor..right_match] {
            ops.push(DiffOp::Insert(line));
        }
        ops.push(DiffOp::Equal(&left[left_match]));
        left_cursor = left_match + 1;
        right_cursor = right_match + 1;
    }
    for line in &left[left_cursor..] {
        ops.push(DiffOp::Delete(line));
    }
    for line in &right[right_cursor..] {
        ops.push(DiffOp::Insert(line));
    }
    ops
}

fn patience_anchors(left: &[String], right: &[String]) -> Vec<(usize, usize)> {
    let left_counts = line_counts(left);
    let right_counts = line_counts(right);
    lcs_pairs(
        left,
        right,
        Some(&|line| {
            left_counts.get(line).copied().unwrap_or(0) == 1
                && right_counts.get(line).copied().unwrap_or(0) == 1
        }),
    )
}

fn histogram_anchors(left: &[String], right: &[String]) -> Vec<(usize, usize)> {
    let left_counts = line_counts(left);
    let right_counts = line_counts(right);
    let best_frequency = left_counts
        .iter()
        .filter_map(|(line, left_count)| {
            let right_count = right_counts.get(line)?;
            Some(left_count + right_count)
        })
        .filter(|count| *count <= 64)
        .min();
    let Some(best_frequency) = best_frequency else {
        return Vec::new();
    };
    lcs_pairs(
        left,
        right,
        Some(&|line| {
            left_counts.get(line).copied().unwrap_or(0)
                + right_counts.get(line).copied().unwrap_or(0)
                == best_frequency
        }),
    )
}

fn line_counts(lines: &[String]) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();
    for line in lines {
        *counts.entry(line.as_str()).or_insert(0) += 1;
    }
    counts
}

fn lcs_pairs(
    left: &[String],
    right: &[String],
    allowed: Option<&dyn Fn(&str) -> bool>,
) -> Vec<(usize, usize)> {
    let mut right_positions: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, line) in right.iter().enumerate() {
        if predicate_allows(allowed, line) {
            right_positions
                .entry(line.as_str())
                .or_default()
                .push(index);
        }
    }

    let mut tails: Vec<usize> = Vec::new();
    let mut tail_nodes: Vec<usize> = Vec::new();
    let mut nodes: Vec<LcsNode> = Vec::new();
    for (left_index, line) in left.iter().enumerate() {
        if !predicate_allows(allowed, line) {
            continue;
        }
        let Some(positions) = right_positions.get(line.as_str()) else {
            continue;
        };
        for &right_index in positions.iter().rev() {
            let length_index = tails.partition_point(|&value| value < right_index);
            let previous = length_index
                .checked_sub(1)
                .and_then(|index| tail_nodes.get(index).copied());
            let node_index = nodes.len();
            nodes.push(LcsNode {
                left: left_index,
                right: right_index,
                previous,
            });
            if length_index == tails.len() {
                tails.push(right_index);
                tail_nodes.push(node_index);
            } else if right_index < tails[length_index] {
                tails[length_index] = right_index;
                tail_nodes[length_index] = node_index;
            }
        }
    }

    let Some(mut node_index) = tail_nodes.last().copied() else {
        return Vec::new();
    };
    let mut pairs = Vec::with_capacity(tails.len());
    loop {
        let node = nodes[node_index];
        pairs.push((node.left, node.right));
        let Some(previous) = node.previous else {
            break;
        };
        node_index = previous;
    }
    pairs.reverse();
    pairs
}

fn predicate_allows(allowed: Option<&dyn Fn(&str) -> bool>, line: &str) -> bool {
    allowed.map_or(true, |predicate| predicate(line))
}

#[derive(Debug, Clone, Copy)]
struct LcsNode {
    left: usize,
    right: usize,
    previous: Option<usize>,
}

fn split_lines_lossy(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .split_inclusive('\n')
        .map(str::to_string)
        .collect()
}

fn push_diff_line(output: &mut String, prefix: char, line: &str) {
    output.push(prefix);
    output.push_str(line);
    if !line.ends_with('\n') {
        output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff_text(left: &str, right: &str, algorithm: DiffAlgorithm) -> String {
        let mut output = String::new();
        render_line_delta(left.as_bytes(), right.as_bytes(), &mut output, algorithm);
        output
    }

    fn manifest(items: &[(&str, &[u8])]) -> BTreeMap<String, Vec<u8>> {
        items
            .iter()
            .map(|(path, data)| ((*path).to_string(), (*data).to_vec()))
            .collect()
    }

    #[test]
    fn myers_diff_keeps_middle_common_lines() {
        let diff = diff_text(
            "alpha\nkeep\nold\nomega\n",
            "alpha\nkeep\nnew\nomega\n",
            DiffAlgorithm::Myers,
        );
        assert_eq!(diff, "-old\n+new\n");
    }

    #[test]
    fn patience_diff_uses_unique_anchors_around_repeated_lines() {
        let diff = diff_text(
            "header\nsame\nleft only\nsame\nfooter\n",
            "header\nsame\nright only\nsame\nfooter\n",
            DiffAlgorithm::Patience,
        );
        assert_eq!(diff, "-left only\n+right only\n");
    }

    #[test]
    fn histogram_diff_handles_low_frequency_anchors() {
        let diff = diff_text(
            "block\nanchor\nold\nblock\n",
            "block\nanchor\nnew\nblock\n",
            DiffAlgorithm::Histogram,
        );
        assert_eq!(diff, "-old\n+new\n");
    }

    #[test]
    fn rename_detection_reports_similar_deleted_and_added_file() {
        let left = manifest(&[("src/old.rs", b"fn main() {\n    old();\n}\n")]);
        let right = manifest(&[("src/new.rs", b"fn main() {\n    new();\n}\n")]);
        let output = render_manifest_diff(
            "left",
            &left,
            "right",
            &right,
            &DiffOptions {
                algorithm: DiffAlgorithm::Myers,
                rename_detection: true,
            },
        );
        assert!(output.contains("rename src/old.rs -> src/new.rs"));
        assert!(output.contains("-    old();"));
        assert!(output.contains("+    new();"));
        assert!(!output.contains("src/old.rs (missing)"));
    }

    #[test]
    fn copy_detection_reports_added_file_from_existing_source() {
        let left = manifest(&[("src/base.rs", b"shared\nbody\n")]);
        let right = manifest(&[
            ("src/base.rs", b"shared\nbody\n"),
            ("src/copied.rs", b"shared\nbody\n"),
        ]);
        let output = render_manifest_diff(
            "left",
            &left,
            "right",
            &right,
            &DiffOptions {
                algorithm: DiffAlgorithm::Myers,
                rename_detection: true,
            },
        );
        assert!(output.contains("copy src/base.rs -> src/copied.rs (100% similarity)"));
        assert!(!output.contains("src/copied.rs (missing)"));
    }

    #[test]
    fn binary_diff_uses_summary_instead_of_payload_lines() {
        let left = manifest(&[("asset.bin", b"\x00\x01\x02old")]);
        let right = manifest(&[("asset.bin", b"\x00\x01\x02new")]);
        let output = render_manifest_diff("left", &left, "right", &right, &DiffOptions::default());
        assert!(output.contains("Binary files differ: 6 bytes sha256:"));
        assert!(!output.contains("-\u{0}\u{1}"));
    }
}
