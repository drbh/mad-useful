use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

pub fn should_include(path: &Path, include: &[String], exclude: &[String]) -> bool {
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

pub fn count_lines(path: &Path) -> Result<usize, std::io::Error> {
    if is_binary(path)? {
        return Ok(0);
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}

pub fn is_binary(path: &Path) -> Result<bool, std::io::Error> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 512];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(false);
    }

    let null_count = buffer[..bytes_read].iter().filter(|&&b| b == 0).count();
    Ok(null_count > bytes_read / 100)
}

pub fn is_noise_file(path: &Path) -> bool {
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

pub fn count_nonwhitespace_chars(path: &Path) -> Result<usize, std::io::Error> {
    if is_binary(path)? {
        return Ok(0);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut char_count = 0;

    for line in reader.lines() {
        let line = line?;
        char_count += line.chars().filter(|&c| !c.is_whitespace()).count();
    }

    Ok(char_count)
}

pub fn get_file_size(path: &Path) -> Result<usize, std::io::Error> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len() as usize)
}

pub fn format_size(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes}B")
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}
