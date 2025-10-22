use clap::Parser;
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Parser)]
struct Args {
    #[arg(default_value = ".")]
    path: String,

    #[arg(long)]
    include: Vec<String>,

    #[arg(long)]
    exclude: Vec<String>,

    #[arg(long, short)]
    summary: bool,

    #[arg(long)]
    max_lines: Option<usize>,

    #[arg(long, short)]
    complexity: bool,

    #[arg(long)]
    churn: bool,

    #[arg(long)]
    hotspots: bool,

    #[arg(long, default_value = "90")]
    days: u32,

    #[arg(long)]
    top: Option<usize>,

    #[arg(long)]
    min_value: Option<usize>,

    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=100))]
    threshold: Option<u8>,

    #[arg(long)]
    no_noise: bool,

    #[arg(long)]
    blame: bool,

    #[arg(long)]
    author: Option<String>,

    #[arg(long)]
    density: bool,

    #[arg(long)]
    emoji: bool,

    #[arg(long)]
    duplicates: bool,

    #[arg(long)]
    age: bool,

    #[arg(long)]
    ownership: bool,

}

fn main() {
    let args = Args::parse();
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let files: Vec<PathBuf> = WalkBuilder::new(&args.path)
        .build()
        .filter_map(|result| {
            if let Ok(entry) = result {
                let path = entry.path();
                if path.is_file()
                    && should_include(path, &args.include, &args.exclude)
                    && (!args.no_noise || !is_noise_file(path))
                {
                    Some(path.to_path_buf())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let results: Vec<(PathBuf, usize, String, String)> = files
        .par_iter()
        .filter_map(|path| {
            let (value, emoji_info) = if args.ownership {
                let owner_pct = calculate_ownership_percentage(path).unwrap_or(0);
                (owner_pct, format!("{}%", owner_pct))
            } else if args.age {
                let days_old = calculate_file_age_days(path).unwrap_or(0);
                (days_old, format!("{}d", days_old))
            } else if args.duplicates {
                let dup_pct = calculate_duplication_percentage(path, &files).unwrap_or(0);
                (dup_pct, format!("{}%", dup_pct))
            } else if args.emoji {
                let info = analyze_emojis(path).unwrap_or_default();
                (info.total, format!("{}u {}", info.unique, info.most_common))
            } else {
                let val = if args.density {
                    calculate_code_density(path).unwrap_or(0)
                } else if args.hotspots {
                    let complexity = calculate_complexity(path).unwrap_or(1);
                    let churn = calculate_churn(path, args.days).unwrap_or(0);
                    complexity * churn
                } else if args.churn {
                    calculate_churn(path, args.days).unwrap_or(0)
                } else if args.complexity {
                    calculate_complexity(path).unwrap_or(0)
                } else {
                    count_lines(path).unwrap_or(0)
                };
                (val, String::new())
            };

            let author = if args.blame || args.author.is_some() {
                get_primary_author(path).unwrap_or_else(|| "unknown".to_string())
            } else {
                String::new()
            };

            if let Some(filter_author) = &args.author {
                if !author
                    .to_lowercase()
                    .contains(&filter_author.to_lowercase())
                {
                    return None;
                }
            }

            if value > 0 {
                Some((path.clone(), value, author, emoji_info))
            } else {
                None
            }
        })
        .collect();

    let mut results = results;

    let has_filters = args.top.is_some() || args.min_value.is_some() || args.threshold.is_some() 
        || args.age || args.ownership || args.duplicates || args.complexity || args.churn || args.hotspots || args.density;

    if has_filters {
        results.sort_by(|a, b| b.1.cmp(&a.1));
    } else {
        results.sort_by(|a, b| a.0.cmp(&b.0));
    }

    if let Some(min_val) = args.min_value {
        results.retain(|(_, value, _, _)| *value >= min_val);
    }

    if let Some(threshold_pct) = args.threshold {
        if let Some(max_value) = results.iter().map(|(_, v, _, _)| v).max() {
            let threshold = (*max_value * threshold_pct as usize) / 100;
            results.retain(|(_, value, _, _)| *value >= threshold);
        }
    }

    if let Some(top_n) = args.top {
        results.truncate(top_n);
    }

    let total: usize = results.iter().map(|(_, count, _, _)| count).sum();
    let file_count = results.len();

    let max_lines_per_file = args.max_lines.unwrap_or_else(|| {
        let default_max = if args.ownership {
            100
        } else if args.age {
            365
        } else if args.duplicates {
            50
        } else if args.emoji {
            10
        } else if args.density {
            80
        } else if args.hotspots {
            200
        } else if args.churn {
            50
        } else if args.complexity {
            20
        } else {
            1000
        };
        results
            .iter()
            .map(|(_, count, _, _)| *count)
            .max()
            .unwrap_or(default_max)
    });

    if args.summary {
        let mut by_ext: HashMap<String, (usize, usize)> = HashMap::new();
        for (path, count, _, _) in &results {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("no_ext")
                .to_string();
            let entry = by_ext.entry(ext).or_insert((0, 0));
            entry.0 += count;
            entry.1 += 1;
        }

        let mut sorted_exts: Vec<_> = by_ext.into_iter().collect();
        sorted_exts.sort_by(|a, b| b.1.0.cmp(&a.1.0));

        for (ext, (lines, files)) in sorted_exts {
            print_colored_count(&mut stdout, lines, 1, max_lines_per_file);
            println!(" {ext} ({files} files)");
        }
    } else {
        for (path, count, author, extra_info) in results {
            print_colored_count(&mut stdout, count, 1, max_lines_per_file);
            if (args.emoji || args.duplicates || args.age || args.ownership) && !extra_info.is_empty() {
                if args.blame && !author.is_empty() {
                    println!(" {} [{}] ({})", path.display(), author, extra_info);
                } else {
                    println!(" {} ({})", path.display(), extra_info);
                }
            } else if args.blame && !author.is_empty() {
                println!(" {} [{}]", path.display(), author);
            } else {
                println!(" {}", path.display());
            }
        }
    }

    print_colored_count(&mut stdout, total, file_count, max_lines_per_file);
    if args.ownership {
        println!(" avg ownership %");
    } else if args.age {
        println!(" avg age (days)");
    } else if args.duplicates {
        println!(" avg duplication %");
    } else if args.emoji {
        println!(" total emojis");
    } else if args.density {
        println!(" total density score");
    } else if args.hotspots {
        println!(" total hotspot score");
    } else if args.churn {
        println!(" total changes");
    } else if args.complexity {
        println!(" total complexity");
    } else {
        println!(" total");
    }
}

fn print_colored_count(
    stdout: &mut StandardStream,
    count: usize,
    file_count: usize,
    max_lines_per_file: usize,
) {
    let max_lines = if file_count == 1 {
        max_lines_per_file
    } else {
        file_count * max_lines_per_file
    };
    let ratio = (count as f64 / max_lines as f64).min(1.0);
    let red = (255.0 * ratio) as u8;
    let green = (255.0 * (1.0 - ratio)) as u8;

    let mut color_spec = ColorSpec::new();
    color_spec.set_fg(Some(Color::Rgb(red, green, 0)));

    stdout.set_color(&color_spec).unwrap();
    print!("{count:>8}");
    stdout.reset().unwrap();
}

fn should_include(path: &Path, include: &[String], exclude: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    if !exclude.is_empty() {
        for pattern in exclude {
            if glob::Pattern::new(pattern).unwrap().matches(&path_str) {
                return false;
            }
        }
    }

    if !include.is_empty() {
        for pattern in include {
            if glob::Pattern::new(pattern).unwrap().matches(&path_str) {
                return true;
            }
        }
        return false;
    }

    true
}

fn count_lines(path: &Path) -> Result<usize, std::io::Error> {
    if is_binary(path)? {
        return Ok(0);
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

fn is_binary(path: &Path) -> Result<bool, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 512];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false);
    }

    let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
    Ok(null_count > bytes_read / 100)
}

fn calculate_complexity(path: &Path) -> Result<usize, std::io::Error> {
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

fn calculate_churn(path: &Path, days: u32) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args([
            "log",
            "--oneline",
            &format!("--since={days}days"),
            "--",
            &path_str,
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let lines = String::from_utf8_lossy(&output.stdout);
                Ok(lines.lines().count())
            } else {
                Ok(0)
            }
        }
        Err(_) => Ok(0),
    }
}

fn is_noise_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_lowercase();

    let noise_patterns = [
        // Config files
        ".json",
        ".xml",
        ".yaml",
        ".yml",
        ".toml",
        ".ini",
        ".cfg",
        ".conf",
        // Lock files
        "package-lock.json",
        "yarn.lock",
        "cargo.lock",
        "gemfile.lock",
        "composer.lock",
        // Generated/build files
        ".min.js",
        ".min.css",
        ".d.ts",
        ".map",
        // Documentation
        ".md",
        ".txt",
        ".rst",
        ".adoc",
        // Data files
        ".csv",
        ".sql",
        ".db",
        ".sqlite",
        // Assets
        ".png",
        ".jpg",
        ".jpeg",
        ".gif",
        ".svg",
        ".ico",
        ".woff",
        ".woff2",
        ".ttf",
        ".eot",
        // Vendor/dependencies
        "/vendor/",
        "/node_modules/",
        "/target/",
        "/build/",
        "/dist/",
        "/.git/",
        // Test fixtures
        "/fixtures/",
        "/mocks/",
        "/test/data/",
        "/testdata/",
    ];

    let noise_files = [
        "readme",
        "license",
        "changelog",
        "makefile",
        "dockerfile",
        "docker-compose",
        "vagrantfile",
        ".gitignore",
        ".dockerignore",
        ".eslintrc",
        ".prettierrc",
        "tsconfig.json",
        "jest.config.js",
        "webpack.config.js",
        "rollup.config.js",
    ];

    noise_patterns
        .iter()
        .any(|pattern| path_str.contains(pattern))
        || noise_files.iter().any(|file| filename.starts_with(file))
}

fn get_primary_author(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args(["log", "--format=%an", "--", &path_str])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let authors_text = String::from_utf8_lossy(&output.stdout);
                let mut author_counts: HashMap<String, usize> = HashMap::new();

                for author in authors_text.lines() {
                    let author = author.trim().to_string();
                    if !author.is_empty() {
                        *author_counts.entry(author).or_insert(0) += 1;
                    }
                }

                author_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(author, _)| author)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn calculate_code_density(path: &Path) -> Result<usize, std::io::Error> {
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

        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#") {
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
struct EmojiAnalysis {
    total: usize,
    unique: usize,
    most_common: String,
}

fn analyze_emojis(path: &Path) -> Result<EmojiAnalysis, std::io::Error> {
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
        .map(|(emoji, _)| emoji.to_string())
        .unwrap_or_else(|| "none".to_string());

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

fn calculate_duplication_percentage(target_path: &Path, all_files: &[PathBuf]) -> Result<usize, std::io::Error> {
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

    let duplicate_chunks = target_chunks.iter()
        .filter(|chunk| all_chunks.contains(*chunk))
        .count();

    let duplication_percentage = if target_chunks.is_empty() {
        0
    } else {
        (duplicate_chunks * 100) / target_chunks.len()
    };

    Ok(duplication_percentage.min(100))
}

fn read_normalized_content(path: &Path) -> Result<String, std::io::Error> {
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
        .replace("#", "")
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
        rolling_hash = (rolling_hash * base + bytes[i] as u64) % modulus;
    }
    
    let mut boundaries = Vec::new();
    
    // Roll the hash and find gear pattern (last 8 bits are zero)
    for i in window_size..bytes.len() {
        if rolling_hash & 0xFF == 0 {
            boundaries.push(i - window_size);
        }
        
        // Remove leftmost character and add rightmost character
        let power = fast_pow(base, window_size - 1, modulus);
        rolling_hash = (rolling_hash + modulus - (bytes[i - window_size] as u64 * power) % modulus) % modulus;
        rolling_hash = (rolling_hash * base + bytes[i] as u64) % modulus;
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
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

fn calculate_file_age_days(path: &Path) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();
    
    let output = Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%ct",
            "--",
            &path_str,
        ])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let timestamp_str = String::from_utf8_lossy(&output.stdout);
                if let Ok(timestamp) = timestamp_str.trim().parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    
                    let age_seconds = now.saturating_sub(timestamp);
                    let age_days = age_seconds / 86400; // seconds per day
                    
                    Ok(age_days as usize)
                } else {
                    // No valid timestamp, fallback to file modification time
                    file_system_age_days(path)
                }
            } else {
                // Git command failed, fallback to file modification time
                file_system_age_days(path)
            }
        }
        Err(_) => file_system_age_days(path),
    }
}

fn file_system_age_days(path: &Path) -> Result<usize, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    let modified = metadata.modified()?;
    let now = std::time::SystemTime::now();
    
    match now.duration_since(modified) {
        Ok(duration) => {
            let days = duration.as_secs() / 86400;
            Ok(days as usize)
        }
        Err(_) => Ok(0),
    }
}

fn calculate_ownership_percentage(path: &Path) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();
    
    let output = Command::new("git")
        .args([
            "log",
            "--format=%an",
            "--",
            &path_str,
        ])
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                let authors_text = String::from_utf8_lossy(&output.stdout);
                let mut author_counts: HashMap<String, usize> = HashMap::new();
                let mut total_commits = 0;
                
                for author in authors_text.lines() {
                    let author = author.trim().to_string();
                    if !author.is_empty() {
                        *author_counts.entry(author).or_insert(0) += 1;
                        total_commits += 1;
                    }
                }
                
                if total_commits == 0 {
                    return Ok(0);
                }
                
                // Find the author with the most commits
                let max_commits = author_counts.values().max().unwrap_or(&0);
                let ownership_percentage = (*max_commits * 100) / total_commits;
                
                Ok(ownership_percentage)
            } else {
                // No git history, return 0 to indicate no ownership data
                Ok(0)
            }
        }
        Err(_) => Ok(0),
    }
}

