# Mad Useful (madu) - Code Analysis Tool

> [!WARNING]
> This tool is always under active development and is just a mash up of various tiny static analysis features tools that have been useful to me over the years. Use at your own risk!


A fast, parallel code analysis tool that provides comprehensive codebase metrics and tracks changes over time. Built for speed with parallel processing and designed for developers who need quick insights into code complexity, file size, git history, and development patterns.

## Key Features

- **Lightning fast** - Parallel processing with Rayon for instant results
- **Comprehensive metrics** - Lines, complexity, density, size, indentation depth
- **Git integration** - Churn analysis, ownership tracking, commit patterns  
- **Smart filtering** - Regex patterns, thresholds, noise reduction
- **Real-time monitoring** - Watch mode for continuous analysis
- **Clean output** - Colored visualization with plain text option

## Installation

```bash
# Clone and build
git clone https://github.com/drbh/mad-useful.git
cd mad-useful
cargo build --release

# Or install directly
cargo install --path .
```

## Quick Start

```bash
# Analyze current directory
madu

# Analyze specific directory with top 10 results
madu --top 10 src/

# Find complexity hotspots
madu --hotspots --top 5 src/

# Monitor changes in real-time
madu --watch 5 src/
```

### Help Output

```bash
$ ./target/release/madu --help
```

```
A fast, parallel code analysis tool for understanding codebase metrics and changes over time

Usage: madu [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory or file to analyze [env: MADU_PATH=] [default: .]

Options:
      --include <INCLUDE>      [FILTER] File filter - include files by glob pattern [env: MADU_INCLUDE=]
      --exclude <EXCLUDE>      [FILTER] File filter - exclude files by glob pattern [env: MADU_EXCLUDE=]
      --no-noise               [FILTER] File filter - exclude common noise files (configs, locks, generated) [env: MADU_NO_NOISE=]
  -c, --complexity             [ANALYSIS] Calculate cyclomatic complexity - control flow complexity score [env: MADU_COMPLEXITY=]
      --density                [ANALYSIS] Code density calculation - operator/keyword density score [env: MADU_DENSITY=]
      --indent                 [ANALYSIS] Nesting depth analysis - maximum indentation level [env: MADU_INDENT=]
      --chars                  [ANALYSIS] Character counting - non-whitespace character count [env: MADU_CHARS=]
      --size                   [ANALYSIS] File size measurement - bytes with human-readable units [env: MADU_SIZE=]
      --duplicates             [ANALYSIS] Code duplication detection - duplication percentage [env: MADU_DUPLICATES=]
      --emoji                  [ANALYSIS] Content analysis - emoji count and statistics [env: MADU_EMOJI=]
      --churn                  [ANALYSIS] Git commit frequency analysis - number of commits in time period [env: MADU_CHURN=]
      --hotspots               [ANALYSIS] Risk assessment - complexity × churn score for refactoring priority [env: MADU_HOTSPOTS=]
      --blame                  [ANALYSIS] Git attribution - show primary author name per file [env: MADU_BLAME=]
      --age                    [ANALYSIS] File staleness analysis - days since last modification [env: MADU_AGE=]
      --ownership              [ANALYSIS] Code ownership analysis - primary author commit percentage [env: MADU_OWNERSHIP=]
      --isolation              [ANALYSIS] Commit isolation analysis - single-file commit percentage [env: MADU_ISOLATION=]
      --rhythm                 [ANALYSIS] Development rhythm analysis - commit frequency variation score [env: MADU_RHYTHM=]
      --days <DAYS>            [DISPLAY] Time scope - number of days for git analysis window [env: MADU_DAYS=] [default: 90]
      --author <AUTHOR>        [FILTER] Content filter - filter results by git author name [env: MADU_AUTHOR=]
      --top <TOP>              [FILTER] Limit - show only top N results [env: MADU_TOP=]
      --skip <SKIP>            [FILTER] Limit - skip first N results [env: MADU_SKIP=]
      --min-value <MIN_VALUE>  [FILTER] Threshold - minimum value filter [env: MADU_MIN_VALUE=]
      --threshold <THRESHOLD>  [FILTER] Threshold - percentage-based filter (1-100%) [env: MADU_THRESHOLD=]
  -s, --summary                [MODIFIER] Aggregation - group results by file extension [env: MADU_SUMMARY=]
      --dirs                   [MODIFIER] Aggregation - group results by directory [env: MADU_DIRS=]
      --depth <DEPTH>          [MODIFIER] Aggregation - limit directory depth for --dirs [env: MADU_DEPTH=]
      --watch <WATCH>          [DISPLAY] Interactive - real-time monitoring mode, refresh every N seconds [env: MADU_WATCH=]
      --no-color               [DISPLAY] Formatting - disable colored output for scripts/CI [env: MADU_NO_COLOR=true]
      --max-lines <MAX_LINES>  [DISPLAY] Color scaling - custom threshold for color scaling reference [env: MADU_MAX_LINES=]
  -h, --help                   Print help
  -V, --version                Print version
```

## Examples

### Basic Analysis

### Count lines in all files

```bash
$ ./target/release/madu src
```

```
     441 src/analysis.rs
     223 src/args.rs
      29 src/display.rs
     175 src/file_utils.rs
     261 src/git.rs
     370 src/main.rs
     504 src/watch.rs
    2003 total
```

Shows line count for each file. Numbers are color-coded: red = large files, green = small files.

### Summary by file extension

```bash
$ ./target/release/madu --summary src
```

```
    2003 rs (7 files)
    2003 total
```

Groups results by file extension. Useful for understanding codebase composition.

### Top 5 largest files

```bash
$ ./target/release/madu --top 5 src
```

```
     504 src/watch.rs
     441 src/analysis.rs
     370 src/main.rs
     261 src/git.rs
     223 src/args.rs
    1799 total
```

Limits output to top N results. Essential for focusing on the most significant files.

### Files with more than 100 lines

```bash
$ ./target/release/madu --min-value 100 src
```

```
     504 src/watch.rs
     441 src/analysis.rs
     370 src/main.rs
     261 src/git.rs
     223 src/args.rs
     175 src/file_utils.rs
    1974 total
```

Filters out small files. Helps identify substantial code files that need attention.

### Code Analysis Features

#### Basic Code Metrics

### Cyclomatic complexity analysis

```bash
$ ./target/release/madu --complexity --top 5 src
```

```
    3424 src/watch.rs
    2780 src/main.rs
    2152 src/analysis.rs
     827 src/git.rs
     785 src/file_utils.rs
    9968 total complexity
```

> [!IMPORTANT]
> Measures code complexity using control flow analysis. Higher values indicate more complex, harder-to-test code that may need refactoring.

### Code density calculation

```bash
$ ./target/release/madu --density --threshold 75 src
```

```
      41 src/git.rs
      36 src/watch.rs
      77 total density score
```

> [!IMPORTANT]
> Calculates code density based on operators, keywords, and nesting. Higher scores indicate dense, potentially hard-to-read code.

### File size measurement

```bash
$ ./target/release/madu --size --min-value 1024 src
```

```
   17435 src/watch.rs (17.0K)
   12273 src/main.rs (12.0K)
   11625 src/analysis.rs (11.4K)
    8627 src/git.rs (8.4K)
    5512 src/args.rs (5.4K)
    4011 src/file_utils.rs (3.9K)
   59483 total bytes
```

> [!IMPORTANT]
> Shows file sizes with human-readable units (K, M, G). Useful for identifying bloated files over 1KB.

### Character counting (non-whitespace)

```bash
$ ./target/release/madu --chars --top 10 src
```

```
   10786 src/watch.rs
    7739 src/analysis.rs
    7633 src/main.rs
    4967 src/git.rs
    3707 src/args.rs
    2661 src/file_utils.rs
     574 src/display.rs
   38067 total chars
```

> [!IMPORTANT]
> Counts actual code characters, excluding whitespace. More accurate than line count for measuring code volume.

### Nesting depth analysis

```bash
$ ./target/release/madu --indent --threshold 80 src
```

```
       8 src/git.rs (8↓)
       6 src/analysis.rs (6↓)
       6 src/watch.rs (6↓)
      20 max indent depth
```

> [!IMPORTANT]
> Measures deepest nesting level. High values may indicate overly complex functions needing refactoring.

#### Advanced Code Analysis

### Code duplication detection

```bash
$ ./target/release/madu --duplicates --min-value 10 src
```

```
      68 src/main.rs (68%)
      45 src/watch.rs (45%)
     113 avg duplication %
```

> [!IMPORTANT]
> Detects duplicate code sections. Higher percentages indicate repeated code that could be refactored into functions.

### Content analysis with emoji detection

```bash
$ ./target/release/madu --emoji --include '*.md' --include '*.txt'
```

```
       0 total emojis
```

> [!IMPORTANT]
> Analyzes content for emoji usage. Useful for documentation and communication files.

### Git History Analysis

#### Change Frequency & Activity

### Recent commit activity (last 30 days)

```bash
$ ./target/release/madu --churn --days 30 --top 10 src
```

```
       1 src/main.rs
       1 total changes
```

> [!IMPORTANT]
> Shows commit frequency in recent timeframe. High churn files change often and may need architectural review.

### File staleness analysis

```bash
$ ./target/release/madu --age --threshold 90 src
```

```
       0 avg age (days)
```

> [!IMPORTANT]
> Days since last modification. Helps identify stale code and recent activity patterns. Shows only oldest 10% of files.

### Development rhythm analysis

```bash
$ ./target/release/madu --rhythm --min-value 5 src
```

```
       0 avg rhythm score
```

> [!IMPORTANT]
> Commit frequency variation score. High values indicate irregular development patterns that may need attention.

#### Code Ownership & Attribution

### Primary author ownership analysis

```bash
$ ./target/release/madu --ownership --threshold 70 src
```

```
     100 src/main.rs (100%)
     100 avg ownership %
```

> [!IMPORTANT]
> Percentage of commits by primary author. Shows files with >70% single-author ownership (potential knowledge silos).

### Git attribution by author

```bash
$ ./target/release/madu --blame --author 'john' src
```

```
       0 total
```

> [!IMPORTANT]
> Shows primary author name per file, filtered by specific author. Useful for code review assignments.

### Commit isolation patterns

```bash
$ ./target/release/madu --isolation --min-value 50 src
```

```
       0 avg isolation %
```

> [!IMPORTANT]
> Percentage of single-file commits. High isolation suggests focused, atomic changes. Shows files with >50% isolated commits.

#### Risk Assessment

### Hotspot analysis (complexity × churn)

```bash
$ ./target/release/madu --hotspots --top 5 src
```

```
    2780 src/main.rs
    2780 total hotspot score
```

> [!IMPORTANT]
> Combines complexity and change frequency. High scores identify files that are both complex and frequently modified - prime refactoring candidates.

### Directory Analysis

### Aggregate by directory

```bash
$ ./target/release/madu --dirs src
```

```
    2003 src
    2003 total dirs
```

Rolls up metrics by directory. Useful for understanding module sizes and identifying large subsystems.

### Limit directory depth

```bash
$ ./target/release/madu --dirs --depth 1 src
```

```
    2003 src
    2003 total dirs
```

Controls aggregation depth. Prevents deeply nested structures from cluttering results.

### Advanced Filtering & Output Control

#### File Pattern Filtering

### Include specific file types

```bash
$ ./target/release/madu --include '*.rs' --include '*.js' --complexity src
```

```
    3424 src/watch.rs
    2780 src/main.rs
    2152 src/analysis.rs
     827 src/git.rs
     785 src/file_utils.rs
      92 src/args.rs
      50 src/display.rs
   10110 total complexity
```

> [!IMPORTANT]
> Include only specific file patterns. Essential for language-specific analysis.

### Exclude noise files

```bash
$ ./target/release/madu --exclude '*.lock' --exclude '*.toml' --exclude '*.json' --size src
```

```
   17435 src/watch.rs (17.0K)
   12273 src/main.rs (12.0K)
   11625 src/analysis.rs (11.4K)
    8627 src/git.rs (8.4K)
    5512 src/args.rs (5.4K)
    4011 src/file_utils.rs (3.9K)
     814 src/display.rs (814B)
   60297 total bytes
```

> [!IMPORTANT]
> Filters out noise files using glob patterns. Essential for focusing on actual source code.

### Auto-exclude common noise files

```bash
$ ./target/release/madu --no-noise --complexity --top 10 src
```

```
    3424 src/watch.rs
    2780 src/main.rs
    2152 src/analysis.rs
     827 src/git.rs
     785 src/file_utils.rs
      92 src/args.rs
      50 src/display.rs
   10110 total complexity
```

> [!IMPORTANT]
> Automatically excludes configs, locks, generated files. Quick way to focus on source code.

#### Result Limiting & Thresholds

### Threshold filtering (top 50%)

```bash
$ ./target/release/madu --threshold 50 --complexity src
```

```
    3424 src/watch.rs
    2780 src/main.rs
    2152 src/analysis.rs
    8356 total complexity
```

> [!IMPORTANT]
> Shows only files above percentage threshold of maximum value. Filters out small/unimportant files automatically.

### Minimum value filtering

```bash
$ ./target/release/madu --min-value 200 --chars src
```

```
   10786 src/watch.rs
    7739 src/analysis.rs
    7633 src/main.rs
    4967 src/git.rs
    3707 src/args.rs
    2661 src/file_utils.rs
     574 src/display.rs
   38067 total chars
```

> [!IMPORTANT]
> Shows only files above absolute threshold. Useful for finding substantial files.

### Top N results with skip

```bash
$ ./target/release/madu --top 10 --skip 3 --size src
```

```
    8627 src/git.rs (8.4K)
    5512 src/args.rs (5.4K)
    4011 src/file_utils.rs (3.9K)
     814 src/display.rs (814B)
   18964 total bytes
```

> [!IMPORTANT]
> Shows top 10 results after skipping first 3. Useful when top files are known outliers.

#### Content-Based Filtering

### Filter by git author

```bash
$ ./target/release/madu --author 'jane' --churn --days 7 src
```

```
       0 total changes
```

> [!IMPORTANT]
> Filter results by git author name. Shows recent changes by specific team member.

### Real-Time Monitoring & Watch Mode

#### Development Workflow Integration

**Monitor complexity changes**

```bash
$ ./target/release/madu --watch 10 --complexity --top 5 src
```

> [!IMPORTANT]
> Real-time monitoring refreshing every 10 seconds. Watch complexity metrics during active development. Press Ctrl+C to stop.

**Track file size growth**

```bash
$ ./target/release/madu --watch 30 --size --threshold 80 --include '*.rs' src
```

> [!IMPORTANT]
> Monitor file size changes every 30 seconds. Useful for catching file bloat during development.

**Watch git activity**

```bash
$ ./target/release/madu --watch 60 --churn --days 1 --top 10 src
```

> [!IMPORTANT]
> Track recent commit activity every minute. Ideal for team leads monitoring daily development patterns.

> [!NOTE]
> Watch mode examples above show the command structure. In practice, watch mode runs continuously and updates the display in real-time.

