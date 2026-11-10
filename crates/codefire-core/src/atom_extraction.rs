//! Atom extraction from Markdown and Python sources, and atom id rules.

use crate::{
    hash_text, normalized_block_content, read_text_lossy, split_lines_without_terminators, Atom,
    CoreError, Selector,
};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

pub fn extract_markdown_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut headings = Vec::new();
    let mut open_fence = None::<(char, usize)>;
    for (line_index, line) in lines.iter().enumerate() {
        if let Some((fence_char, fence_len)) = markdown_fence_marker(line) {
            if let Some((open_char, open_len)) = open_fence {
                if fence_char == open_char && fence_len >= open_len {
                    open_fence = None;
                }
            } else {
                open_fence = Some((fence_char, fence_len));
            }
            continue;
        }
        if open_fence.is_some() {
            continue;
        }
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

pub(crate) fn markdown_fence_marker(line: &str) -> Option<(char, usize)> {
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 3 {
        return None;
    }
    let trimmed = &line[indent..];
    let fence_char = trimmed.chars().next()?;
    if !matches!(fence_char, '`' | '~') {
        return None;
    }
    let fence_len = trimmed.chars().take_while(|ch| *ch == fence_char).count();
    (fence_len >= 3).then_some((fence_char, fence_len))
}

pub(crate) const PYTHON_EXPLICIT_MARKER_LOOKBACK_LINES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonSymbolKind {
    Class,
    Def,
    Method,
}

impl PythonSymbolKind {
    pub(crate) fn id_component(self) -> &'static str {
        match self {
            Self::Class => "class",
            Self::Def => "def",
            Self::Method => "method",
        }
    }

    pub(crate) fn selector_type(self) -> &'static str {
        match self {
            Self::Class => "python_class",
            Self::Def => "python_def",
            Self::Method => "python_method",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PythonSymbolBlock {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) kind: PythonSymbolKind,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PythonStringState {
    Continued(u8),
    Triple(u8),
}

impl PythonStringState {
    pub(crate) fn quote(self) -> u8 {
        match self {
            Self::Continued(quote) | Self::Triple(quote) => quote,
        }
    }

    pub(crate) fn width(self) -> usize {
        match self {
            Self::Continued(_) => 1,
            Self::Triple(_) => 3,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PythonLexState {
    pub(crate) string: Option<PythonStringState>,
}

#[derive(Debug)]
pub(crate) struct PythonLexLine {
    pub(crate) code: String,
    pub(crate) comment_start: Option<usize>,
}

/// Extracts the supported Python symbol profile used by the reference implementation.
///
/// Explicit `cf-atom` markers in the three lines immediately preceding a symbol (or on the
/// symbol line) own that symbol once and replace its derived ID. Other markers remain standalone
/// comment atoms. Symbol and marker scans are linear in the source size; only the constant-size
/// marker ownership window is searched per symbol.
pub fn extract_python_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let lexical_lines = python_lexical_lines(&lines);
    let mut explicit_by_line = BTreeMap::new();
    for (line_index, (line, lexical)) in lines.iter().zip(&lexical_lines).enumerate() {
        let Some(comment_start) = lexical.comment_start else {
            continue;
        };
        if let Some(atom_id) =
            explicit_cf_atom_id(&line[comment_start..], artifact_path, line_index + 1)?
        {
            explicit_by_line.insert(line_index, atom_id);
        }
    }

    let blocks = python_symbol_blocks(&lines, &lexical_lines);
    let mut used_explicit = HashSet::with_capacity(explicit_by_line.len());
    let mut atoms = Vec::with_capacity(blocks.len().saturating_add(explicit_by_line.len()));
    for block in blocks {
        let first_candidate = block
            .start
            .saturating_sub(PYTHON_EXPLICIT_MARKER_LOOKBACK_LINES);
        let explicit_line = (first_candidate..=block.start).rev().find(|line_index| {
            explicit_by_line.contains_key(line_index) && !used_explicit.contains(line_index)
        });
        let atom_id = if let Some(line_index) = explicit_line {
            used_explicit.insert(line_index);
            explicit_by_line
                .get(&line_index)
                .expect("selected explicit marker exists")
                .clone()
        } else {
            let prefix = if fallback_kind == "test" {
                "TEST"
            } else {
                "CODE"
            };
            format!(
                "{prefix}:{artifact_path}::{}:{}",
                block.kind.id_component(),
                block.name
            )
        };
        let content_start = explicit_line.unwrap_or(block.start);
        let content = normalized_block_content(&lines[content_start..block.end]);
        atoms.push(Atom {
            kind: kind_from_atom_id(&atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: block.kind.selector_type().to_string(),
                value: block.name,
            },
            content_hash: hash_text(&content),
            atom_id,
        });
    }

    for (line_index, atom_id) in explicit_by_line {
        if used_explicit.contains(&line_index) {
            continue;
        }
        let content = normalized_block_content(&lines[line_index..line_index + 1]);
        atoms.push(Atom {
            kind: kind_from_atom_id(&atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: "python_comment".to_string(),
                value: atom_id.clone(),
            },
            content_hash: hash_text(&content),
            atom_id,
        });
    }
    Ok(atoms)
}

pub(crate) fn python_lexical_lines(lines: &[&str]) -> Vec<PythonLexLine> {
    let mut state = PythonLexState::default();
    lines
        .iter()
        .map(|line| python_lex_line(line, &mut state))
        .collect()
}

pub(crate) fn python_lex_line(line: &str, state: &mut PythonLexState) -> PythonLexLine {
    let bytes = line.as_bytes();
    let mut masked = bytes.to_vec();
    let mut index = 0usize;
    let mut comment_start = None;

    if let Some(open) = state.string {
        let quote = open.quote();
        let width = open.width();
        if let Some(end) = find_python_quote_end(bytes, 0, quote, width) {
            mask_python_bytes(&mut masked, 0, end);
            index = end;
            state.string = None;
        } else {
            mask_python_bytes(&mut masked, 0, bytes.len());
            if matches!(open, PythonStringState::Continued(_))
                && !python_line_continues_string(bytes)
            {
                state.string = None;
            }
            return PythonLexLine {
                code: String::from_utf8(masked).expect("masking preserves UTF-8"),
                comment_start,
            };
        }
    }

    while index < bytes.len() {
        match bytes[index] {
            b'#' => {
                comment_start = Some(index);
                mask_python_bytes(&mut masked, index, bytes.len());
                break;
            }
            quote @ (b'\'' | b'"') => {
                let width = if bytes.get(index + 1) == Some(&quote)
                    && bytes.get(index + 2) == Some(&quote)
                {
                    3
                } else {
                    1
                };
                let prefix_start = python_string_prefix_start(bytes, index);
                if let Some(end) = find_python_quote_end(bytes, index + width, quote, width) {
                    mask_python_bytes(&mut masked, prefix_start, end);
                    index = end;
                } else {
                    mask_python_bytes(&mut masked, prefix_start, bytes.len());
                    state.string = match width {
                        3 => Some(PythonStringState::Triple(quote)),
                        1 if python_line_continues_string(bytes) => {
                            Some(PythonStringState::Continued(quote))
                        }
                        _ => None,
                    };
                    break;
                }
            }
            _ => index += 1,
        }
    }

    PythonLexLine {
        code: String::from_utf8(masked).expect("masking preserves UTF-8"),
        comment_start,
    }
}

pub(crate) fn find_python_quote_end(
    bytes: &[u8],
    mut index: usize,
    quote: u8,
    width: usize,
) -> Option<usize> {
    while index.saturating_add(width) <= bytes.len() {
        if bytes[index..index + width]
            .iter()
            .all(|candidate| *candidate == quote)
            && !python_quote_is_escaped(bytes, index)
        {
            return Some(index + width);
        }
        index += 1;
    }
    None
}

pub(crate) fn python_quote_is_escaped(bytes: &[u8], index: usize) -> bool {
    let mut cursor = index;
    let mut backslashes = 0usize;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        cursor -= 1;
        backslashes += 1;
    }
    backslashes % 2 == 1
}

pub(crate) fn python_line_continues_string(bytes: &[u8]) -> bool {
    let mut backslashes = 0usize;
    for byte in bytes.iter().rev() {
        if *byte != b'\\' {
            break;
        }
        backslashes += 1;
    }
    backslashes % 2 == 1
}

pub(crate) fn python_string_prefix_start(bytes: &[u8], quote_index: usize) -> usize {
    let mut start = quote_index;
    while start > 0
        && quote_index - start < 2
        && matches!(
            bytes[start - 1],
            b'r' | b'R' | b'b' | b'B' | b'u' | b'U' | b'f' | b'F'
        )
    {
        start -= 1;
    }
    if start == quote_index {
        return start;
    }
    if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
        quote_index
    } else {
        start
    }
}

pub(crate) fn mask_python_bytes(bytes: &mut [u8], start: usize, end: usize) {
    bytes[start..end].fill(b' ');
}

pub(crate) fn python_symbol_blocks(
    lines: &[&str],
    lexical_lines: &[PythonLexLine],
) -> Vec<PythonSymbolBlock> {
    let mut blocks = Vec::<PythonSymbolBlock>::new();
    let mut open_blocks = Vec::<(usize, usize)>::new();
    let mut class_stack = Vec::<(usize, String)>::new();

    for (line_index, lexical) in lexical_lines.iter().enumerate() {
        let code = lexical.code.as_str();
        if python_block_boundary(code) {
            let boundary_indent = leading_space_count(code);
            while open_blocks
                .last()
                .is_some_and(|(indent, _)| boundary_indent <= *indent)
            {
                let (_, block_index) = open_blocks.pop().expect("checked as non-empty");
                blocks[block_index].end = line_index;
            }
        }

        let Some((indent, is_class, symbol_name)) = python_symbol_declaration(code) else {
            continue;
        };
        while class_stack
            .last()
            .is_some_and(|(class_indent, _)| indent <= *class_indent)
        {
            class_stack.pop();
        }
        let (kind, name) = if is_class {
            (PythonSymbolKind::Class, symbol_name.clone())
        } else if let Some((_, class_name)) = class_stack.last() {
            (
                PythonSymbolKind::Method,
                format!("{class_name}.{symbol_name}"),
            )
        } else {
            (PythonSymbolKind::Def, symbol_name.clone())
        };
        let block_index = blocks.len();
        blocks.push(PythonSymbolBlock {
            start: line_index,
            end: lines.len(),
            kind,
            name,
        });
        open_blocks.push((indent, block_index));
        if is_class {
            class_stack.push((indent, symbol_name));
        }
    }
    debug_assert_eq!(lines.len(), lexical_lines.len());
    blocks
}

pub(crate) fn python_block_boundary(line: &str) -> bool {
    let trimmed = line.trim_start();
    !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with('@')
}

pub(crate) fn python_symbol_declaration(line: &str) -> Option<(usize, bool, String)> {
    let indent = line.chars().take_while(|ch| ch.is_whitespace()).count();
    let trimmed = line.trim_start();
    let (is_class, after_keyword) = python_keyword_tail(trimmed, "class")
        .map(|tail| (true, tail))
        .or_else(|| python_keyword_tail(trimmed, "def").map(|tail| (false, tail)))
        .or_else(|| {
            python_keyword_tail(trimmed, "async")
                .and_then(|tail| python_keyword_tail(tail, "def"))
                .map(|tail| (false, tail))
        })?;
    let mut chars = after_keyword.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    let mut name = String::from(first);
    name.extend(chars.take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_'));
    Some((indent, is_class, name))
}

pub(crate) fn python_keyword_tail<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let tail = line.strip_prefix(keyword)?;
    let first = tail.chars().next()?;
    first.is_whitespace().then(|| tail.trim_start())
}

pub fn extract_explicit_cf_atoms(
    path: &Path,
    artifact_path: &str,
    fallback_kind: &str,
) -> Result<Vec<Atom>, CoreError> {
    let text = read_text_lossy(path)?;
    let lines = split_lines_without_terminators(&text);
    let mut markers = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        if let Some(atom_id) = explicit_cf_atom_id(line, artifact_path, line_index + 1)? {
            markers.push((line_index, atom_id));
        }
    }
    let mut atoms = Vec::with_capacity(markers.len());
    for (index, (start, atom_id)) in markers.iter().enumerate() {
        let marker_end = markers
            .get(index + 1)
            .map(|(next_start, _)| *next_start)
            .unwrap_or(lines.len());
        let end = explicit_cf_atom_span_end(&lines, *start, marker_end, artifact_path);
        let content = normalized_block_content(&lines[*start..end]);
        atoms.push(Atom {
            atom_id: atom_id.clone(),
            kind: kind_from_atom_id(atom_id, fallback_kind).to_string(),
            artifact_path: artifact_path.to_string(),
            selector: Selector {
                selector_type: "explicit_cf_atom".to_string(),
                value: atom_id.clone(),
            },
            content_hash: hash_text(&content),
        });
    }
    Ok(atoms)
}

pub(crate) fn explicit_cf_atom_span_end(
    lines: &[&str],
    marker_start: usize,
    marker_end: usize,
    artifact_path: &str,
) -> usize {
    if artifact_path.ends_with(".py") {
        python_explicit_cf_atom_span_end(lines, marker_start, marker_end).unwrap_or(marker_end)
    } else {
        marker_end
    }
}

pub(crate) fn python_explicit_cf_atom_span_end(
    lines: &[&str],
    marker_start: usize,
    marker_end: usize,
) -> Option<usize> {
    let mut index = marker_start + 1;
    while index < marker_end {
        let trimmed = lines[index].trim_start();
        if trimmed.trim().is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }
        if trimmed.starts_with('@') {
            let decorator_indent = leading_space_count(lines[index]);
            while index < marker_end && lines[index].trim_start().starts_with('@') {
                index += 1;
            }
            if index < marker_end && is_python_symbol_line(lines[index].trim_start()) {
                let symbol_indent = leading_space_count(lines[index]).min(decorator_indent);
                return Some(python_block_end(lines, index, marker_end, symbol_indent));
            }
            return None;
        }
        if is_python_symbol_line(trimmed) {
            return Some(python_block_end(
                lines,
                index,
                marker_end,
                leading_space_count(lines[index]),
            ));
        }
        return None;
    }
    None
}

pub(crate) fn is_python_symbol_line(trimmed: &str) -> bool {
    trimmed.starts_with("class ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("async def ")
}

pub(crate) fn python_block_end(
    lines: &[&str],
    symbol_start: usize,
    marker_end: usize,
    block_indent: usize,
) -> usize {
    let mut end = symbol_start + 1;
    for (index, line) in lines
        .iter()
        .enumerate()
        .take(marker_end)
        .skip(symbol_start + 1)
    {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            end = index + 1;
            continue;
        }
        let indent = leading_space_count(line);
        if indent <= block_indent {
            return trim_trailing_blank_lines(lines, end);
        }
        end = index + 1;
    }
    trim_trailing_blank_lines(lines, end)
}

pub(crate) fn trim_trailing_blank_lines(lines: &[&str], mut end: usize) -> usize {
    while end > 0 && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    end
}

pub(crate) fn leading_space_count(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
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

pub(crate) fn markdown_atom_heading(line: &str) -> Option<(usize, String)> {
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
        .take_while(|ch| ch.is_ascii_uppercase() || is_dash_atom_tail_char(*ch))
        .collect();
    if !is_valid_atom_id(&atom_id) {
        return None;
    }
    if !is_auto_markdown_atom_id(&atom_id) {
        return None;
    }
    let next = rest.chars().nth(atom_id.len());
    if matches!(next, None | Some(':') | Some(' ') | Some('\t')) {
        Some((level, atom_id))
    } else {
        None
    }
}

pub(crate) fn is_auto_markdown_atom_id(atom_id: &str) -> bool {
    if atom_id.starts_with("PROC-") || atom_id.contains("-PROC-") {
        return false;
    }
    let Some((prefix, _)) = atom_id.split_once('-') else {
        return atom_id.starts_with("CODE:");
    };
    matches!(
        prefix,
        "REQ" | "DES" | "TEST" | "ADR" | "OPS" | "API" | "DB"
    )
}

pub(crate) fn is_atom_id_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-' | ':' | '/')
}

pub fn is_valid_atom_id(atom_id: &str) -> bool {
    is_valid_dash_atom_id(atom_id) || is_valid_code_path_atom_id(atom_id)
}

pub(crate) fn is_valid_dash_atom_id(atom_id: &str) -> bool {
    let Some((prefix, rest)) = atom_id.split_once('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.chars().all(|ch| ch.is_ascii_uppercase())
        && !rest.is_empty()
        && rest.chars().all(is_dash_atom_tail_char)
}

pub(crate) fn is_valid_code_path_atom_id(atom_id: &str) -> bool {
    let Some(rest) = atom_id.strip_prefix("CODE:") else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(is_code_path_atom_tail_char)
}

pub(crate) fn is_dash_atom_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')
}

pub(crate) fn is_code_path_atom_tail_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-' | ':' | '/')
}

pub(crate) fn explicit_cf_atom_id(
    line: &str,
    artifact_path: &str,
    line_number: usize,
) -> Result<Option<String>, CoreError> {
    let Some((_, after)) = line.split_once("cf-atom:") else {
        return Ok(None);
    };
    let atom_id: String = after
        .trim_start()
        .chars()
        .take_while(|ch| is_atom_id_char(*ch))
        .collect();
    if atom_id.is_empty() {
        return Err(CoreError::Config(format!(
            "invalid Atom ID in {artifact_path}:{line_number}: cf-atom marker has no Atom ID"
        )));
    }
    if !is_valid_atom_id(&atom_id) {
        return Err(CoreError::Config(format!(
            "invalid Atom ID in {artifact_path}:{line_number}: {atom_id}"
        )));
    }
    Ok(Some(atom_id))
}
