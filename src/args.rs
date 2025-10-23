use clap::Parser;

#[derive(Parser)]
#[command(author, version, about = "A fast, parallel code analysis tool for understanding codebase metrics and changes over time", long_about = None)]
pub struct Args {
    // Input/Target
    #[arg(
        default_value = ".",
        env = "MADU_PATH",
        help = "Directory or file to analyze"
    )]
    pub path: String,

    // File Filtering
    #[arg(
        long,
        env = "MADU_INCLUDE",
        value_delimiter = ',',
        help = "[FILTER] File filter - include files by glob pattern"
    )]
    pub include: Vec<String>,

    #[arg(
        long,
        env = "MADU_EXCLUDE",
        value_delimiter = ',',
        help = "[FILTER] File filter - exclude files by glob pattern"
    )]
    pub exclude: Vec<String>,

    #[arg(
        long,
        env = "MADU_NO_NOISE",
        help = "[FILTER] File filter - exclude common noise files (configs, locks, generated)"
    )]
    pub no_noise: bool,

    // Code Analysis
    #[arg(
        long,
        short,
        env = "MADU_COMPLEXITY",
        help = "[ANALYSIS] Calculate cyclomatic complexity - control flow complexity score"
    )]
    pub complexity: bool,

    #[arg(
        long,
        env = "MADU_DENSITY",
        help = "[ANALYSIS] Code density calculation - operator/keyword density score"
    )]
    pub density: bool,

    #[arg(
        long,
        env = "MADU_INDENT",
        help = "[ANALYSIS] Nesting depth analysis - maximum indentation level"
    )]
    pub indent: bool,

    #[arg(
        long,
        env = "MADU_CHARS",
        help = "[ANALYSIS] Character counting - non-whitespace character count"
    )]
    pub chars: bool,

    #[arg(
        long,
        env = "MADU_SIZE",
        help = "[ANALYSIS] File size measurement - bytes with human-readable units"
    )]
    pub size: bool,

    #[arg(
        long,
        env = "MADU_DUPLICATES",
        help = "[ANALYSIS] Code duplication detection - duplication percentage"
    )]
    pub duplicates: bool,

    #[arg(
        long,
        env = "MADU_EMOJI",
        help = "[ANALYSIS] Content analysis - emoji count and statistics"
    )]
    pub emoji: bool,

    // Git Analysis
    #[arg(
        long,
        env = "MADU_CHURN",
        help = "[ANALYSIS] Git commit frequency analysis - number of commits in time period"
    )]
    pub churn: bool,

    #[arg(
        long,
        env = "MADU_HOTSPOTS",
        help = "[ANALYSIS] Risk assessment - complexity × churn score for refactoring priority"
    )]
    pub hotspots: bool,

    #[arg(
        long,
        env = "MADU_BLAME",
        help = "[ANALYSIS] Git attribution - show primary author name per file"
    )]
    pub blame: bool,

    #[arg(
        long,
        env = "MADU_AGE",
        help = "[ANALYSIS] File staleness analysis - days since last modification"
    )]
    pub age: bool,

    #[arg(
        long,
        env = "MADU_OWNERSHIP",
        help = "[ANALYSIS] Code ownership analysis - primary author commit percentage"
    )]
    pub ownership: bool,

    #[arg(
        long,
        env = "MADU_ISOLATION",
        help = "[ANALYSIS] Commit isolation analysis - single-file commit percentage"
    )]
    pub isolation: bool,

    #[arg(
        long,
        env = "MADU_RHYTHM",
        help = "[ANALYSIS] Development rhythm analysis - commit frequency variation score"
    )]
    pub rhythm: bool,

    #[arg(
        long,
        default_value = "90",
        env = "MADU_DAYS",
        help = "[DISPLAY] Time scope - number of days for git analysis window"
    )]
    pub days: u32,

    #[arg(
        long,
        env = "MADU_AUTHOR",
        help = "[FILTER] Content filter - filter results by git author name"
    )]
    pub author: Option<String>,

    // Result Filtering & Limits
    #[arg(
        long,
        env = "MADU_TOP",
        help = "[FILTER] Limit - show only top N results"
    )]
    pub top: Option<usize>,

    #[arg(
        long,
        env = "MADU_SKIP",
        help = "[FILTER] Limit - skip first N results"
    )]
    pub skip: Option<usize>,

    #[arg(
        long,
        env = "MADU_MIN_VALUE",
        help = "[FILTER] Threshold - minimum value filter"
    )]
    pub min_value: Option<usize>,

    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=100), env = "MADU_THRESHOLD", help = "[FILTER] Threshold - percentage-based filter (1-100%)")]
    pub threshold: Option<u8>,

    // Aggregation & Grouping
    #[arg(
        long,
        short,
        env = "MADU_SUMMARY",
        help = "[MODIFIER] Aggregation - group results by file extension"
    )]
    pub summary: bool,

    #[arg(
        long,
        env = "MADU_DIRS",
        help = "[MODIFIER] Aggregation - group results by directory"
    )]
    pub dirs: bool,

    #[arg(
        long,
        env = "MADU_DEPTH",
        help = "[MODIFIER] Aggregation - limit directory depth for --dirs"
    )]
    pub depth: Option<usize>,

    // Display & Output
    #[arg(
        long,
        env = "MADU_WATCH",
        help = "[DISPLAY] Interactive - real-time monitoring mode, refresh every N seconds"
    )]
    pub watch: Option<u64>,

    #[arg(
        long,
        env = "MADU_NO_COLOR",
        help = "[DISPLAY] Formatting - disable colored output for scripts/CI"
    )]
    pub no_color: bool,

    #[arg(
        long,
        env = "MADU_MAX_LINES",
        help = "[DISPLAY] Color scaling - custom threshold for color scaling reference"
    )]
    pub max_lines: Option<usize>,
}
