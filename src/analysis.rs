use crate::file_utils::is_binary;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn calculate_complexity(path: &Path) -> Result<usize, std::io::Error> {
    if is_binary(path)? {
        return Ok(0);
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match ext {
        "rs" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "java" | "js" | "ts" | "py" | "go"
        | "php" => calculate_code_complexity(path),
        _ => Ok(0),
    }
}

fn calculate_code_complexity(path: &Path) -> Result<usize, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut complexity = 1;
    let mut in_comment = false;
    let mut in_string = false;
    let mut escape_next = false;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' | '\'' if !in_comment => in_string = !in_string,
                '/' if !in_string && !in_comment => {
                    if chars.peek() == Some(&'/') {
                        break;
                    } else if chars.peek() == Some(&'*') {
                        in_comment = true;
                        chars.next();
                    }
                }
                '*' if in_comment => {
                    if chars.peek() == Some(&'/') {
                        in_comment = false;
                        chars.next();
                    }
                }
                _ if !in_string && !in_comment => {
                    if is_complexity_keyword(line, ch) {
                        complexity += 1;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(complexity)
}

fn is_complexity_keyword(line: &str, _ch: char) -> bool {
    let complexity_patterns = [
        "if ",
        "else if",
        "while ",
        "for ",
        "switch ",
        "case ",
        "catch ",
        "&&",
        "||",
        "?",
        "break ",
        "continue ",
        "return ",
        "throw ",
    ];

    complexity_patterns
        .iter()
        .any(|pattern| line.contains(pattern))
}

pub fn calculate_code_density(path: &Path) -> Result<usize, std::io::Error> {
    if is_binary(path)? {
        return Ok(0);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut total_chars = 0;
    let mut code_lines = 0;
    let mut nested_depth = 0;
    let mut max_depth = 0;
    let mut dense_lines = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        code_lines += 1;
        total_chars += trimmed.len();

        let depth_change = count_nesting_change(trimmed);
        nested_depth = (nested_depth as i32 + depth_change).max(0) as usize;
        max_depth = max_depth.max(nested_depth);

        let line_density = calculate_line_density(trimmed);
        if line_density > 50 {
            dense_lines += 1;
        }
    }

    if code_lines == 0 {
        return Ok(0);
    }

    let avg_line_length = total_chars / code_lines;
    let density_score = (avg_line_length * max_depth * dense_lines) / code_lines.max(1);

    Ok(density_score)
}

fn count_nesting_change(line: &str) -> i32 {
    let mut change = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for ch in line.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' | '\'' => in_string = !in_string,
            '{' | '(' | '[' if !in_string => change += 1,
            '}' | ')' | ']' if !in_string => change -= 1,
            _ => {}
        }
    }

    change
}

fn calculate_line_density(line: &str) -> usize {
    let operators = [
        "+", "-", "*", "/", "=", "!", "<", ">", "&", "|", "?", ":", ".", "->", "=>",
    ];
    let keywords = [
        "if", "else", "for", "while", "match", "switch", "case", "return", "throw",
    ];

    let mut density = line.len();

    for op in &operators {
        density += line.matches(op).count() * 2;
    }

    for kw in &keywords {
        density += line.matches(kw).count() * 3;
    }

    let parens = line.matches('(').count() + line.matches(')').count();
    density += parens * 2;

    density
}

#[derive(Default)]
pub struct EmojiAnalysis {
    pub total: usize,
    pub unique: usize,
    pub most_common: String,
}

pub fn analyze_emojis(path: &Path) -> Result<EmojiAnalysis, std::io::Error> {
    if is_binary(path)? {
        return Ok(EmojiAnalysis::default());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut emoji_counts: HashMap<char, usize> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        for c in line.chars() {
            if is_emoji(c) {
                *emoji_counts.entry(c).or_insert(0) += 1;
            }
        }
    }

    let total = emoji_counts.values().sum();
    let unique = emoji_counts.len();
    let most_common = emoji_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map_or_else(|| "none".to_string(), |(emoji, _)| emoji.to_string());

    Ok(EmojiAnalysis {
        total,
        unique,
        most_common,
    })
}

fn is_emoji(c: char) -> bool {
    match c as u32 {
        // Emoticons
        0x1F600..=0x1F64F => true,
        // Miscellaneous Symbols and Pictographs
        0x1F300..=0x1F5FF => true,
        // Transport and Map Symbols
        0x1F680..=0x1F6FF => true,
        // Flags (Regional indicator symbols)
        0x1F1E6..=0x1F1FF => true,
        // Supplemental Symbols and Pictographs
        0x1F900..=0x1F9FF => true,
        // Symbols and Pictographs Extended-A
        0x1FA70..=0x1FAFF => true,
        // Common emoji symbols
        0x2600..=0x26FF => true,
        0x2700..=0x27BF => true,
        // Additional common emojis
        0x1F004 => true,           // Mahjong tile
        0x1F0CF => true,           // Playing card
        0x1F170..=0x1F251 => true, // Enclosed characters
        // Variation selectors (emoji presentation)
        0xFE0F => true,
        // Zero-width joiner (for compound emojis)
        0x200D => true,
        _ => false,
    }
}

pub fn calculate_duplication_percentage(
    target_path: &Path,
    all_files: &[PathBuf],
) -> Result<usize, std::io::Error> {
    if is_binary(target_path)? {
        return Ok(0);
    }

    let target_content = read_normalized_content(target_path)?;
    if target_content.len() < 100 {
        return Ok(0);
    }

    let target_chunks = extract_chunks(&target_content);
    if target_chunks.is_empty() {
        return Ok(0);
    }

    let mut all_chunks = HashSet::new();
    for other_path in all_files {
        if other_path == target_path || is_binary(other_path).unwrap_or(true) {
            continue;
        }

        if let Ok(other_content) = read_normalized_content(other_path) {
            let other_chunks = extract_chunks(&other_content);
            all_chunks.extend(other_chunks);
        }
    }

    let duplicate_chunks = target_chunks
        .iter()
        .filter(|chunk| all_chunks.contains(*chunk))
        .count();

    let duplication_percentage = if target_chunks.is_empty() {
        0
    } else {
        (duplicate_chunks * 100) / target_chunks.len()
    };

    Ok(duplication_percentage.min(100))
}

pub fn read_normalized_content(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut normalized = String::new();
    for line in reader.lines() {
        let line = line?;
        let cleaned = normalize_line(&line);
        if !cleaned.is_empty() {
            normalized.push_str(&cleaned);
            normalized.push('\n');
        }
    }

    Ok(normalized)
}

fn normalize_line(line: &str) -> String {
    line.trim()
        .replace([' ', '\t'], "")
        .replace("//", "")
        .replace("/*", "")
        .replace("*/", "")
        .replace('#', "")
        .to_lowercase()
}

fn extract_chunks(content: &str) -> Vec<u64> {
    let bytes = content.as_bytes();
    let mut chunks = Vec::new();

    // Simple rolling hash with fixed-size sliding window
    let window_size = 32;
    if bytes.len() < window_size {
        return chunks;
    }

    let mut rolling_hash = 0u64;
    let base = 257u64;
    let modulus = 1_000_000_007u64;

    // Calculate initial hash
    for i in 0..window_size {
        rolling_hash = (rolling_hash * base + u64::from(bytes[i])) % modulus;
    }

    let mut boundaries = Vec::new();

    // Roll the hash and find gear pattern (last 8 bits are zero)
    for i in window_size..bytes.len() {
        if rolling_hash & 0xFF == 0 {
            boundaries.push(i - window_size);
        }

        // Remove leftmost character and add rightmost character
        let power = fast_pow(base, window_size - 1, modulus);
        rolling_hash = (rolling_hash + modulus
            - (u64::from(bytes[i - window_size]) * power) % modulus)
            % modulus;
        rolling_hash = (rolling_hash * base + u64::from(bytes[i])) % modulus;
    }

    // Create chunks from boundaries
    let mut start = 0;
    for &boundary in &boundaries {
        if boundary > start && boundary - start >= 20 {
            let chunk_bytes = &bytes[start..boundary];
            let chunk_hash = simple_hash(chunk_bytes);
            chunks.push(chunk_hash);
        }
        start = boundary;
    }

    // Handle the last chunk
    if bytes.len() > start && bytes.len() - start >= 20 {
        let chunk_bytes = &bytes[start..];
        let chunk_hash = simple_hash(chunk_bytes);
        chunks.push(chunk_hash);
    }

    chunks
}

fn fast_pow(base: u64, exp: usize, modulus: u64) -> u64 {
    let mut result = 1u64;
    let mut b = base % modulus;
    let mut e = exp;

    while e > 0 {
        if e % 2 == 1 {
            result = (result * b) % modulus;
        }
        e /= 2;
        b = (b * b) % modulus;
    }

    result
}

fn simple_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0u64;
    for &byte in bytes {
        hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
    }
    hash
}

pub fn calculate_max_indent_level(path: &Path) -> Result<usize, std::io::Error> {
    if is_binary(path)? {
        return Ok(0);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut max_indent = 0;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let indent_level = calculate_line_indent(&line);
        max_indent = max_indent.max(indent_level);
    }

    Ok(max_indent)
}

fn calculate_line_indent(line: &str) -> usize {
    let mut indent = 0;
    for ch in line.chars() {
        match ch {
            ' ' => indent += 1,
            '\t' => indent += 4, // Count tabs as 4 spaces
            _ => break,
        }
    }
    indent / 4 // Convert to logical indent levels (assuming 4-space indents)
}
