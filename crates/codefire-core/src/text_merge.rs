//! Line-level three-way merge of text files.
//!
//! Given a common base and two descendants, lines changed on only one side
//! are taken from that side. A region changed differently on both sides is a
//! conflict and is written between markers so the author resolves just that
//! region. The merge is purely textual: it does not decide whether the result
//! is consistent. Fires, semantic candidates and verification still do that.
//!
//! Work is bounded. Each base-to-side comparison may spend at most
//! [`MAX_EDIT_DISTANCE`] edits after the common prefix and suffix are removed;
//! past that the function declines and the caller keeps the file-level result.

/// Largest edit distance one side may have from the base.
///
/// The search keeps `O(d^2)` 32-bit entries, about 16 MiB at this limit.
pub const MAX_EDIT_DISTANCE: usize = 2_000;

pub const TARGET_MARKER: &str = "<<<<<<< target";
pub const SEPARATOR_MARKER: &str = "=======";
pub const SOURCE_MARKER: &str = ">>>>>>> source";

/// Result of a line-level three-way merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextMerge {
    /// Both sides combined without overlap.
    Clean(Vec<u8>),
    /// At least one region differs on both sides. `content` holds the merged
    /// text with each such region between conflict markers.
    Conflict {
        content: Vec<u8>,
        conflict_regions: usize,
    },
}

/// Merges `target` and `source`, both derived from `base`.
///
/// Returns `None` when either side is too far from the base to compare within
/// the work bound, or when a side already contains a marker line, which would
/// make the written markers ambiguous.
pub fn merge_text_three_way(base: &[u8], target: &[u8], source: &[u8]) -> Option<TextMerge> {
    let base_lines = split_lines(base);
    let target_lines = split_lines(target);
    let source_lines = split_lines(source);
    if target_lines
        .iter()
        .chain(&source_lines)
        .any(|line| is_marker_line(line))
    {
        return None;
    }
    let to_target = base_matches(&base_lines, &target_lines)?;
    let to_source = base_matches(&base_lines, &source_lines)?;

    let mut merged = Vec::with_capacity(target.len().max(source.len()));
    let mut conflict_regions = 0usize;
    let (mut base_at, mut target_at, mut source_at) = (0usize, 0usize, 0usize);
    loop {
        // The next base line that both sides kept ends the current region.
        let stable = (base_at..base_lines.len())
            .find_map(|index| Some((index, to_target[index]?, to_source[index]?)));
        let (base_end, target_end, source_end) =
            stable.unwrap_or((base_lines.len(), target_lines.len(), source_lines.len()));
        conflict_regions += usize::from(push_region(
            &mut merged,
            &base_lines[base_at..base_end],
            &target_lines[target_at..target_end],
            &source_lines[source_at..source_end],
        ));
        if stable.is_none() {
            break;
        }
        merged.extend_from_slice(target_lines[target_end]);
        base_at = base_end + 1;
        target_at = target_end + 1;
        source_at = source_end + 1;
    }

    Some(if conflict_regions == 0 {
        TextMerge::Clean(merged)
    } else {
        TextMerge::Conflict {
            content: merged,
            conflict_regions,
        }
    })
}

/// Appends one region between stable lines. Returns whether it conflicted.
fn push_region(merged: &mut Vec<u8>, base: &[&[u8]], target: &[&[u8]], source: &[&[u8]]) -> bool {
    let chosen = if target == source || source == base {
        target
    } else if target == base {
        source
    } else {
        push_marker(merged, TARGET_MARKER);
        push_lines(merged, target);
        push_marker(merged, SEPARATOR_MARKER);
        push_lines(merged, source);
        push_marker(merged, SOURCE_MARKER);
        return true;
    };
    for line in chosen {
        merged.extend_from_slice(line);
    }
    false
}

fn push_marker(merged: &mut Vec<u8>, marker: &str) {
    end_line(merged);
    merged.extend_from_slice(marker.as_bytes());
    merged.push(b'\n');
}

fn push_lines(merged: &mut Vec<u8>, lines: &[&[u8]]) {
    for line in lines {
        merged.extend_from_slice(line);
    }
    end_line(merged);
}

/// A final line without a newline must not swallow the marker after it.
fn end_line(merged: &mut Vec<u8>) {
    if merged.last().is_some_and(|byte| *byte != b'\n') {
        merged.push(b'\n');
    }
}

fn is_marker_line(line: &[u8]) -> bool {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    // The separator alone is not ambiguous (and is a common Markdown
    // heading underline); unresolved-conflict detection keys on these two.
    [TARGET_MARKER, SOURCE_MARKER]
        .iter()
        .any(|marker| line == marker.as_bytes())
}

/// Splits into lines that keep their terminator, so joining them is lossless.
fn split_lines(data: &[u8]) -> Vec<&[u8]> {
    data.split_inclusive(|byte| *byte == b'\n').collect()
}

/// For each base line, the index of the line it corresponds to in `other`
/// under a longest common subsequence, or `None` when the line was removed.
fn base_matches(base: &[&[u8]], other: &[&[u8]]) -> Option<Vec<Option<usize>>> {
    let prefix = base
        .iter()
        .zip(other)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = base[prefix..]
        .iter()
        .rev()
        .zip(other[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let mut matches = vec![None; base.len()];
    for (index, slot) in matches.iter_mut().enumerate().take(prefix) {
        *slot = Some(index);
    }
    for offset in 0..suffix {
        matches[base.len() - 1 - offset] = Some(other.len() - 1 - offset);
    }
    let base_middle = &base[prefix..base.len() - suffix];
    let other_middle = &other[prefix..other.len() - suffix];
    for (left, right) in shortest_edit_matches(base_middle, other_middle)? {
        matches[prefix + left] = Some(prefix + right);
    }
    Some(matches)
}

/// Myers' shortest edit script, returning the matched index pairs.
///
/// `frontiers[d]` is the furthest `x` reached on each diagonal after `d - 1`
/// edits, stored for diagonals `-d..=d` only.
fn shortest_edit_matches(left: &[&[u8]], right: &[&[u8]]) -> Option<Vec<(usize, usize)>> {
    let (left_len, right_len) = (left.len() as isize, right.len() as isize);
    if left_len == 0 || right_len == 0 {
        return Some(Vec::new());
    }
    let limit = MAX_EDIT_DISTANCE.min(left.len() + right.len()) as isize;
    // Index `k + limit + 1` holds diagonal `k`; one spare slot on each end.
    let width = (2 * limit + 3) as usize;
    let slot = |diagonal: isize| (diagonal + limit + 1) as usize;
    let mut furthest = vec![0u32; width];
    let mut frontiers: Vec<Vec<u32>> = Vec::new();
    let mut found = None;
    'search: for edits in 0..=limit {
        frontiers.push(furthest[slot(-edits - 1)..=slot(edits + 1)].to_vec());
        let mut diagonal = -edits;
        while diagonal <= edits {
            let down = diagonal == -edits
                || (diagonal != edits
                    && furthest[slot(diagonal - 1)] < furthest[slot(diagonal + 1)]);
            let mut x = if down {
                furthest[slot(diagonal + 1)] as isize
            } else {
                furthest[slot(diagonal - 1)] as isize + 1
            };
            let mut y = x - diagonal;
            while x < left_len && y < right_len && left[x as usize] == right[y as usize] {
                x += 1;
                y += 1;
            }
            furthest[slot(diagonal)] = x as u32;
            if x >= left_len && y >= right_len {
                found = Some(edits);
                break 'search;
            }
            diagonal += 2;
        }
    }
    let total_edits = found?;

    let mut pairs = Vec::new();
    let (mut x, mut y) = (left_len, right_len);
    for edits in (1..=total_edits).rev() {
        let frontier = &frontiers[edits as usize];
        // `frontier` covers diagonals `-edits - 1..=edits + 1`.
        let at = |diagonal: isize| frontier[(diagonal + edits + 1) as usize] as isize;
        let diagonal = x - y;
        let down = diagonal == -edits || (diagonal != edits && at(diagonal - 1) < at(diagonal + 1));
        let previous_diagonal = if down { diagonal + 1 } else { diagonal - 1 };
        let previous_x = at(previous_diagonal);
        let previous_y = previous_x - previous_diagonal;
        while x > previous_x && y > previous_y {
            x -= 1;
            y -= 1;
            pairs.push((x as usize, y as usize));
        }
        x = previous_x;
        y = previous_y;
    }
    while x > 0 && y > 0 {
        x -= 1;
        y -= 1;
        pairs.push((x as usize, y as usize));
    }
    Some(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: &[&str]) -> Vec<u8> {
        lines
            .iter()
            .flat_map(|line| format!("{line}\n").into_bytes())
            .collect()
    }

    fn clean(base: &[u8], target: &[u8], source: &[u8]) -> Vec<u8> {
        match merge_text_three_way(base, target, source) {
            Some(TextMerge::Clean(content)) => content,
            other => panic!("expected a clean merge, got {other:?}"),
        }
    }

    #[test]
    fn changes_to_different_lines_merge_cleanly() {
        let base = text(&["1", "2", "3", "4", "5", "6", "7", "8"]);
        let target = text(&["1", "2", "3", "4", "5", "6", "7 main", "8"]);
        let source = text(&["1", "2 feat", "3", "4", "5", "6", "7", "8"]);
        assert_eq!(
            clean(&base, &target, &source),
            text(&["1", "2 feat", "3", "4", "5", "6", "7 main", "8"])
        );
    }

    #[test]
    fn insertions_deletions_and_identical_edits_combine() {
        let base = text(&["a", "b", "c", "d", "e"]);
        let target = text(&["a", "b", "c", "d", "e", "target tail"]);
        let source = text(&["source head", "a", "c", "d", "e"]);
        assert_eq!(
            clean(&base, &target, &source),
            text(&["source head", "a", "c", "d", "e", "target tail"])
        );

        let both = text(&["a", "B", "c", "d", "e"]);
        assert_eq!(clean(&base, &both, &both), both);
        assert_eq!(clean(&base, &base, &source), source);
        assert_eq!(clean(&base, &target, &base), target);
    }

    #[test]
    fn overlapping_edits_conflict_only_in_their_region() {
        let base = text(&["keep 1", "old", "keep 2", "tail"]);
        let target = text(&["keep 1", "target value", "keep 2", "tail"]);
        let source = text(&["keep 1", "source value", "keep 2", "tail changed"]);
        let merged = merge_text_three_way(&base, &target, &source).unwrap();
        assert_eq!(
            merged,
            TextMerge::Conflict {
                content: text(&[
                    "keep 1",
                    "<<<<<<< target",
                    "target value",
                    "=======",
                    "source value",
                    ">>>>>>> source",
                    "keep 2",
                    "tail changed",
                ]),
                conflict_regions: 1,
            }
        );
    }

    #[test]
    fn missing_final_newline_does_not_join_marker_lines() {
        let base = b"a\nold".to_vec();
        let target = b"a\ntarget".to_vec();
        let source = b"a\nsource".to_vec();
        let Some(TextMerge::Conflict { content, .. }) =
            merge_text_three_way(&base, &target, &source)
        else {
            panic!("expected a conflict");
        };
        assert_eq!(
            content,
            b"a\n<<<<<<< target\ntarget\n=======\nsource\n>>>>>>> source\n".to_vec()
        );
        // A clean merge keeps the missing final newline as written.
        assert_eq!(clean(b"a\nb", b"a\nb", b"x\na\nb"), b"x\na\nb".to_vec());
    }

    #[test]
    fn sides_that_already_contain_marker_lines_are_declined() {
        let base = text(&["a", "b"]);
        let target = text(&["a", ">>>>>>> source", "b"]);
        assert_eq!(
            merge_text_three_way(&base, &target, &text(&["a", "b", "c"])),
            None
        );
    }

    #[test]
    fn distant_sides_are_declined_instead_of_searched_without_bound() {
        let base: Vec<u8> = (0..MAX_EDIT_DISTANCE * 2)
            .flat_map(|index| format!("base {index}\n").into_bytes())
            .collect();
        let target: Vec<u8> = (0..MAX_EDIT_DISTANCE * 2)
            .flat_map(|index| format!("target {index}\n").into_bytes())
            .collect();
        assert_eq!(merge_text_three_way(&base, &target, &base), None);
    }

    #[test]
    fn merge_agrees_with_a_reference_on_generated_edits() {
        // Deterministic pseudo-random edits: each side changes disjoint lines,
        // so the expected result is known by construction.
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for round in 0..200 {
            let length = 20 + (next() % 60) as usize;
            let base_lines: Vec<String> =
                (0..length).map(|index| format!("line {index}")).collect();
            let mut target_lines = base_lines.clone();
            let mut source_lines = base_lines.clone();
            let mut expected = base_lines.clone();
            // Edit every sixth line at most, alternating sides, so two edits
            // are never adjacent and never overlap.
            for index in (0..length).step_by(6) {
                match next() % 3 {
                    0 => {
                        target_lines[index] = format!("target {round} {index}");
                        expected[index] = target_lines[index].clone();
                    }
                    1 => {
                        source_lines[index] = format!("source {round} {index}");
                        expected[index] = source_lines[index].clone();
                    }
                    _ => {}
                }
            }
            let join = |lines: &[String]| -> Vec<u8> {
                lines
                    .iter()
                    .flat_map(|line| format!("{line}\n").into_bytes())
                    .collect()
            };
            assert_eq!(
                clean(
                    &join(&base_lines),
                    &join(&target_lines),
                    &join(&source_lines)
                ),
                join(&expected),
                "round {round}"
            );
        }
    }
}
