use crate::analysis::{
    analyze_emojis, calculate_code_density, calculate_complexity, calculate_duplication_percentage,
    calculate_max_indent_level,
};
use crate::args::Args;
use crate::display::print_colored_count;
use crate::file_utils::{
    count_lines, count_nonwhitespace_chars, format_size, get_file_size, is_noise_file,
    should_include,
};
use crate::git::{
    calculate_churn, calculate_file_age_days, calculate_isolation_percentage,
    calculate_ownership_percentage, calculate_rhythm_score, get_primary_author,
};
use ignore::WalkBuilder;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use termcolor::{ColorChoice, StandardStream};

pub fn watch_mode(args: &Args, interval_secs: u64) {
    let mut _last_run = Instant::now();
    let watch_interval = Duration::from_secs(interval_secs);
    let mut last_values: HashMap<PathBuf, usize> = HashMap::new();
    let mut start_values: HashMap<PathBuf, usize> = HashMap::new();
    let start_time = Instant::now();
    let mut iteration_count = 0;

    // Set up signal handler to restore cursor on exit
    let _ = ctrlc::set_handler(move || {
        print!("\x1B[?25h"); // Show cursor
        std::process::exit(0);
    });

    println!(
        "Watching {} every {}s (Press Ctrl+C to stop)",
        args.path, interval_secs
    );
    println!();

    let mut first_run = true;

    loop {
        let loop_start = Instant::now();
        iteration_count += 1;

        if first_run {
            // Initial clear screen and hide cursor
            print!("\x1B[2J\x1B[1;1H\x1B[?25l");
        } else {
            // Clear screen to handle shorter lists
            print!("\x1B[2J\x1B[1;1H");
        }

        let elapsed_total = start_time.elapsed().as_secs();
        println!("Started: {elapsed_total}s ago | Iterations: {iteration_count}");
        println!(
            "Last update: {:2}s ago | Watching {} every {}s (Ctrl+C to stop)",
            0, args.path, interval_secs
        );
        run_analysis_with_changes(args, &mut last_values, &mut start_values, first_run);

        // Update timer and wait
        _last_run = loop_start;

        if first_run {
            first_run = false;
        }

        // Sleep with periodic updates to show elapsed time
        let sleep_duration = Duration::from_millis(100);
        let mut total_slept = Duration::new(0, 0);

        while total_slept < watch_interval {
            std::thread::sleep(sleep_duration);
            total_slept += sleep_duration;

            // Update timestamp display
            let elapsed = Instant::now().duration_since(_last_run).as_secs();
            let elapsed_total = start_time.elapsed().as_secs();
            print!("\x1B[1;1H");
            println!("Started: {elapsed_total}s ago | Iterations: {iteration_count}\x1B[K");
            println!(
                "Last update: {:2}s ago | Watching {} every {}s (Ctrl+C to stop)\x1B[K",
                elapsed, args.path, interval_secs
            );
        }
    }
}

fn run_analysis_with_changes(
    args: &Args,
    last_values: &mut HashMap<PathBuf, usize>,
    start_values: &mut HashMap<PathBuf, usize>,
    is_first_run: bool,
) {
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
            let (value, emoji_info) = if args.size {
                let file_size = get_file_size(path).unwrap_or(0);
                (file_size, format_size(file_size))
            } else if args.chars {
                let char_count = count_nonwhitespace_chars(path).unwrap_or(0);
                (char_count, String::new())
            } else if args.indent {
                let max_indent = calculate_max_indent_level(path).unwrap_or(0);
                (max_indent, format!("{max_indent}↓"))
            } else if args.isolation {
                let isolation_pct = calculate_isolation_percentage(path).unwrap_or(0);
                (isolation_pct, format!("{isolation_pct}%"))
            } else if args.rhythm {
                let rhythm_score = calculate_rhythm_score(path).unwrap_or(0);
                (rhythm_score, format!("{rhythm_score}d"))
            } else if args.ownership {
                let owner_pct = calculate_ownership_percentage(path).unwrap_or(0);
                (owner_pct, format!("{owner_pct}%"))
            } else if args.age {
                let days_old = calculate_file_age_days(path).unwrap_or(0);
                (days_old, format!("{days_old}d"))
            } else if args.duplicates {
                let dup_pct = calculate_duplication_percentage(path, &files).unwrap_or(0);
                (dup_pct, format!("{dup_pct}%"))
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
                } else if args.chars {
                    count_nonwhitespace_chars(path).unwrap_or(0)
                } else if args.size {
                    get_file_size(path).unwrap_or(0)
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

    // Add previously tracked files that no longer exist (show them with value 0)
    if !is_first_run {
        let current_files: HashSet<PathBuf> =
            results.iter().map(|(path, _, _, _)| path.clone()).collect();

        for (path, _) in last_values.iter() {
            if !current_files.contains(path) {
                // File was removed, add it with value 0
                let author = if args.blame || args.author.is_some() {
                    get_primary_author(path).unwrap_or_else(|| "unknown".to_string())
                } else {
                    String::new()
                };

                // Skip if author filter doesn't match
                if let Some(filter_author) = &args.author {
                    if !author
                        .to_lowercase()
                        .contains(&filter_author.to_lowercase())
                    {
                        continue;
                    }
                }

                results.push((path.clone(), 0, author, String::new()));
            }
        }
    }

    // Aggregate by directory if --dirs flag is set
    if args.dirs {
        let mut dir_aggregates: HashMap<PathBuf, (usize, usize, Vec<String>)> = HashMap::new();

        // Get base path to calculate relative depth
        let base_path = Path::new(&args.path);
        let base_components_count = base_path.components().count();

        for (path, value, author, _extra_info) in results {
            let mut dir = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();

            // If depth is specified, truncate directory path to that depth relative to base
            if let Some(target_depth) = args.depth {
                let components: Vec<_> = dir.components().collect();
                let relative_depth = base_components_count + target_depth;
                if components.len() > relative_depth {
                    dir = components.iter().take(relative_depth).collect();
                }
            }

            let entry = dir_aggregates.entry(dir).or_insert((0, 0, Vec::new()));
            entry.0 += value; // Sum values
            entry.1 += 1; // Count files
            if !author.is_empty() && !entry.2.contains(&author) {
                entry.2.push(author); // Collect unique authors
            }
        }

        results = dir_aggregates
            .into_iter()
            .map(|(dir, (total_value, file_count, authors))| {
                let author_info = if authors.is_empty() {
                    String::new()
                } else {
                    authors.join(",")
                };
                let extra_info = if args.size {
                    format!("{} ({}f)", format_size(total_value), file_count)
                } else if args.emoji
                    || args.duplicates
                    || args.age
                    || args.ownership
                    || args.isolation
                    || args.rhythm
                    || args.indent
                {
                    format!("{file_count}f")
                } else {
                    String::new()
                };
                (dir, total_value, author_info, extra_info)
            })
            .collect();
    }

    let has_filters = args.top.is_some()
        || args.skip.is_some()
        || args.min_value.is_some()
        || args.threshold.is_some()
        || args.age
        || args.ownership
        || args.duplicates
        || args.complexity
        || args.churn
        || args.hotspots
        || args.density
        || args.isolation
        || args.rhythm
        || args.indent
        || args.chars
        || args.dirs
        || args.depth.is_some()
        || args.size;

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

    if let Some(skip_n) = args.skip {
        if skip_n < results.len() {
            results = results.into_iter().skip(skip_n).collect();
        } else {
            results.clear();
        }
    }

    if let Some(top_n) = args.top {
        results.truncate(top_n);
    }

    let total: usize = results.iter().map(|(_, count, _, _)| count).sum();
    let file_count = results.len();

    let max_lines_per_file = args.max_lines.unwrap_or_else(|| {
        let default_max = if args.size {
            1000000
        } else if args.chars {
            50000
        } else if args.indent {
            20
        } else if args.isolation {
            100
        } else if args.rhythm {
            50
        } else if args.ownership {
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
            print_colored_count(&mut stdout, lines, 1, max_lines_per_file, args.no_color);
            println!(" {ext} ({files} files)");
        }
    } else {
        for (path, count, author, extra_info) in &results {
            print_colored_count(&mut stdout, *count, 1, max_lines_per_file, args.no_color);

            // Calculate and display change deltas
            let mut change_parts = Vec::new();

            // Delta since last interval
            if let Some(&last_value) = last_values.get(path) {
                if *count != last_value {
                    let delta = *count as i32 - last_value as i32;
                    if delta > 0 {
                        change_parts.push(format!("+{delta}"));
                    } else {
                        change_parts.push(format!("{delta}"));
                    }
                }
            }

            // Delta since start (only if not first run)
            if !is_first_run {
                let start_value = start_values.get(path).copied().unwrap_or(0);
                if *count != start_value {
                    let total_delta = *count as i32 - start_value as i32;
                    if total_delta > 0 {
                        change_parts.push(format!("Δ+{total_delta}"));
                    } else {
                        change_parts.push(format!("Δ{total_delta}"));
                    }
                }
            }

            let change_str = if change_parts.is_empty() {
                String::new()
            } else {
                format!(" \x1B[90m({})\x1B[0m", change_parts.join(" "))
            };

            if (args.emoji
                || args.duplicates
                || args.age
                || args.ownership
                || args.isolation
                || args.rhythm
                || args.indent
                || args.dirs
                || args.size)
                && !extra_info.is_empty()
            {
                if args.blame && !author.is_empty() {
                    println!(
                        " {}{} [{}] ({})",
                        path.display(),
                        change_str,
                        author,
                        extra_info
                    );
                } else {
                    println!(" {}{} ({})", path.display(), change_str, extra_info);
                }
            } else if args.blame && !author.is_empty() {
                println!(" {}{} [{}]", path.display(), change_str, author);
            } else {
                println!(" {}{}", path.display(), change_str);
            }
        }

        // Update tracking values for next iteration
        for (path, count, _, _) in &results {
            // Store start values on first run
            if is_first_run {
                start_values.insert(path.clone(), *count);
            }
            last_values.insert(path.clone(), *count);
        }
    }

    print_colored_count(
        &mut stdout,
        total,
        file_count,
        max_lines_per_file,
        args.no_color,
    );
    if args.dirs {
        println!(" total dirs");
    } else if args.size {
        println!(" total bytes");
    } else if args.chars {
        println!(" total chars");
    } else if args.indent {
        println!(" max indent depth");
    } else if args.isolation {
        println!(" avg isolation %");
    } else if args.rhythm {
        println!(" avg rhythm score");
    } else if args.ownership {
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
