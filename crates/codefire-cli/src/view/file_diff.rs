use super::{DiffAlgorithm, DiffOptions};
use crate::remote::sha256_hex;
use std::collections::{BTreeMap, BTreeSet, HashMap};

const MAX_RENAME_CANDIDATE_PAIRS: usize = 4_096;
const MAX_SIMILARITY_BYTES: usize = 512 * 1024;
const MAX_SIMILARITY_LINES: usize = 10_000;
const MAX_LCS_MATCH_CANDIDATES: usize = 1_000_000;
const MAX_TEXT_DIFF_BYTES_PER_FILE: usize = 256 * 1024;

pub(super) fn render_manifest_diff(
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

    for warning in &file_moves.warnings {
        changed = true;
        output.push_str(&format!("warning: {warning}\n"));
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
    warnings: Vec<String>,
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

    detect_exact_renames_by_hash(
        left,
        right,
        &deleted,
        &added,
        &mut detection,
        &mut used_deleted,
        &mut used_added,
    );

    let remaining_deleted = deleted
        .iter()
        .filter(|source| !used_deleted.contains(*source))
        .collect::<Vec<_>>();
    let remaining_added = added
        .iter()
        .filter(|target| !used_added.contains(*target))
        .collect::<Vec<_>>();
    let candidate_pairs = remaining_deleted
        .len()
        .saturating_mul(remaining_added.len());
    if candidate_pairs > MAX_RENAME_CANDIDATE_PAIRS {
        detection.warnings.push(format!(
            "rename similarity detection skipped {candidate_pairs} candidate pairs above limit {MAX_RENAME_CANDIDATE_PAIRS}; exact hash renames were still detected"
        ));
    } else {
        for source in &remaining_deleted {
            for target in &remaining_added {
                let score = similarity_score(&left[*source], &right[*target]);
                if score >= MIN_SIMILARITY {
                    rename_candidates.push((score, (*source).clone(), (*target).clone()));
                }
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

    let copy_candidate_pairs = left.len().saturating_mul(added.len());
    let skip_inexact_copies = copy_candidate_pairs > MAX_RENAME_CANDIDATE_PAIRS;
    if skip_inexact_copies {
        detection.warnings.push(format!(
            "copy similarity detection skipped {copy_candidate_pairs} candidate pairs above limit {MAX_RENAME_CANDIDATE_PAIRS}"
        ));
    }
    for target in added {
        if used_added.contains(&target) {
            continue;
        }
        if skip_inexact_copies {
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

fn detect_exact_renames_by_hash(
    left: &BTreeMap<String, Vec<u8>>,
    right: &BTreeMap<String, Vec<u8>>,
    deleted: &[String],
    added: &[String],
    detection: &mut FileMoveDetection,
    used_deleted: &mut BTreeSet<String>,
    used_added: &mut BTreeSet<String>,
) {
    let mut deleted_by_hash: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for source in deleted {
        deleted_by_hash
            .entry(sha256_hex(&left[source]))
            .or_default()
            .push(source);
    }
    for sources in deleted_by_hash.values_mut() {
        sources.sort();
    }
    for target in added {
        let hash = sha256_hex(&right[target]);
        let Some(sources) = deleted_by_hash.get(&hash) else {
            continue;
        };
        let Some(source) = sources
            .iter()
            .copied()
            .find(|source| !used_deleted.contains(*source))
        else {
            continue;
        };
        used_deleted.insert(source.clone());
        used_added.insert(target.clone());
        detection.skip_paths.insert(source.clone());
        detection.skip_paths.insert(target.clone());
        detection.renames.push(FileMove {
            source: source.clone(),
            target: target.clone(),
            score: 100,
        });
    }
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
    if left.len() > MAX_SIMILARITY_BYTES || right.len() > MAX_SIMILARITY_BYTES {
        return 0;
    }
    let left_lines = split_lines_lossy(left);
    let right_lines = split_lines_lossy(right);
    if left_lines.is_empty() || right_lines.is_empty() {
        return 0;
    }
    if left_lines.len() > MAX_SIMILARITY_LINES || right_lines.len() > MAX_SIMILARITY_LINES {
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
    let start_len = output.len();
    let mut omitted_changed_lines = 0usize;
    for op in diff_lines(&left_lines, &right_lines, algorithm) {
        match op {
            DiffOp::Equal(_) => {}
            DiffOp::Delete(line) => {
                push_diff_line_bounded(output, '-', line, start_len, &mut omitted_changed_lines)
            }
            DiffOp::Insert(line) => {
                push_diff_line_bounded(output, '+', line, start_len, &mut omitted_changed_lines)
            }
        }
    }
    if omitted_changed_lines > 0 {
        output.push_str(&format!(
            "... diff output truncated, omitted {omitted_changed_lines} changed lines\n"
        ));
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
    let mut match_candidates = 0usize;
    for (left_index, line) in left.iter().enumerate() {
        if !predicate_allows(allowed, line) {
            continue;
        }
        let Some(positions) = right_positions.get(line.as_str()) else {
            continue;
        };
        match_candidates = match_candidates.saturating_add(positions.len());
        if match_candidates > MAX_LCS_MATCH_CANDIDATES {
            return Vec::new();
        }
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

fn push_diff_line_bounded(
    output: &mut String,
    prefix: char,
    line: &str,
    start_len: usize,
    omitted_changed_lines: &mut usize,
) {
    if output.len().saturating_sub(start_len) >= MAX_TEXT_DIFF_BYTES_PER_FILE {
        *omitted_changed_lines += 1;
        return;
    }
    push_diff_line(output, prefix, line);
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
                ..DiffOptions::default()
            },
        );
        assert!(output.contains("rename src/old.rs -> src/new.rs"));
        assert!(output.contains("-    old();"));
        assert!(output.contains("+    new();"));
        assert!(!output.contains("src/old.rs (missing)"));
    }

    #[test]
    fn rename_detection_uses_hash_for_exact_renames_above_candidate_limit() {
        let mut left_items = Vec::new();
        let mut right_items = Vec::new();
        left_items.push(("src/exact_old.rs".to_string(), b"same\nbody\n".to_vec()));
        right_items.push(("src/exact_new.rs".to_string(), b"same\nbody\n".to_vec()));
        for index in 0..70 {
            left_items.push((
                format!("old/{index}.txt"),
                format!("left {index}\n").into_bytes(),
            ));
            right_items.push((
                format!("new/{index}.txt"),
                format!("right {index}\n").into_bytes(),
            ));
        }
        let left = left_items.into_iter().collect::<BTreeMap<_, _>>();
        let right = right_items.into_iter().collect::<BTreeMap<_, _>>();

        let output = render_manifest_diff(
            "left",
            &left,
            "right",
            &right,
            &DiffOptions {
                algorithm: DiffAlgorithm::Myers,
                rename_detection: true,
                ..DiffOptions::default()
            },
        );

        assert!(output.contains("rename src/exact_old.rs -> src/exact_new.rs (100% similarity)"));
        assert!(output.contains("warning: rename similarity detection skipped"));
    }

    #[test]
    fn similarity_score_skips_huge_non_identical_files() {
        let left = "left\n".repeat(MAX_SIMILARITY_LINES + 1);
        let right = "right\n".repeat(MAX_SIMILARITY_LINES + 1);

        assert_eq!(similarity_score(left.as_bytes(), right.as_bytes()), 0);
    }

    #[test]
    fn lcs_pairs_returns_empty_when_match_candidates_exceed_limit() {
        let repeated = "same\n".to_string();
        let line_count = 1_100usize;
        let left = vec![repeated.clone(); line_count];
        let right = vec![repeated; line_count];

        assert!(lcs_pairs(&left, &right, None).is_empty());
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
                ..DiffOptions::default()
            },
        );
        assert!(output.contains("copy src/base.rs -> src/copied.rs (100% similarity)"));
        assert!(!output.contains("src/copied.rs (missing)"));
    }

    #[test]
    fn text_diff_output_is_bounded_with_omission_summary() {
        let long_line = format!("{}\n", "x".repeat(1024));
        let left = long_line.repeat(400);
        let right = String::new();

        let diff = diff_text(&left, &right, DiffAlgorithm::Myers);

        assert!(diff.len() < MAX_TEXT_DIFF_BYTES_PER_FILE + 4096);
        assert!(diff.contains("diff output truncated"));
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
