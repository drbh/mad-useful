use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub fn calculate_churn(path: &Path, days: u32) -> Result<usize, std::io::Error> {
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

pub fn get_primary_author(path: &Path) -> Option<String> {
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

pub fn calculate_file_age_days(path: &Path) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args(["log", "-1", "--format=%ct", "--", &path_str])
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

pub fn calculate_ownership_percentage(path: &Path) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args(["log", "--format=%an", "--", &path_str])
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

pub fn calculate_isolation_percentage(path: &Path) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args(["log", "--format=%H", "--", &path_str])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                let commits_text = String::from_utf8_lossy(&output.stdout);
                let commit_hashes: Vec<&str> = commits_text
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect();

                if commit_hashes.is_empty() {
                    return Ok(0);
                }

                let mut single_file_commits = 0;

                for commit_hash in &commit_hashes {
                    let diff_output = Command::new("git")
                        .args([
                            "diff",
                            "--name-only",
                            &format!("{commit_hash}^"),
                            commit_hash,
                        ])
                        .output();

                    if let Ok(diff_result) = diff_output {
                        if diff_result.status.success() {
                            let changed_files = String::from_utf8_lossy(&diff_result.stdout);
                            let file_count = changed_files
                                .lines()
                                .filter(|line| !line.trim().is_empty())
                                .count();

                            if file_count == 1 {
                                single_file_commits += 1;
                            }
                        }
                    }
                }

                let isolation_percentage = (single_file_commits * 100) / commit_hashes.len();
                Ok(isolation_percentage)
            } else {
                Ok(0)
            }
        }
        Err(_) => Ok(0),
    }
}

pub fn calculate_rhythm_score(path: &Path) -> Result<usize, std::io::Error> {
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args(["log", "--format=%ct", "--", &path_str])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() && !output.stdout.is_empty() {
                let timestamps_text = String::from_utf8_lossy(&output.stdout);
                let mut timestamps: Vec<u64> = timestamps_text
                    .lines()
                    .filter_map(|line| line.trim().parse().ok())
                    .collect();

                if timestamps.len() < 2 {
                    return Ok(0);
                }

                timestamps.sort_unstable();
                timestamps.reverse(); // newest first

                let mut intervals: Vec<u64> = Vec::new();
                for window in timestamps.windows(2) {
                    let interval_seconds = window[0] - window[1];
                    let interval_days = interval_seconds / 86400; // seconds per day
                    intervals.push(interval_days);
                }

                if intervals.is_empty() {
                    return Ok(0);
                }

                // Calculate standard deviation of intervals
                let mean = intervals.iter().sum::<u64>() as f64 / intervals.len() as f64;
                let variance = intervals
                    .iter()
                    .map(|&x| {
                        let diff = x as f64 - mean;
                        diff * diff
                    })
                    .sum::<f64>()
                    / intervals.len() as f64;

                let std_dev = variance.sqrt();
                Ok(std_dev as usize)
            } else {
                Ok(0)
            }
        }
        Err(_) => Ok(0),
    }
}
