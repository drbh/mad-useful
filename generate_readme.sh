#!/bin/bash

# Generate README with examples of all madu commands
README_FILE="README.md"

# Build the binary first
echo "Building madu binary..."
cargo build --release

# Use the binary path
MADU_BIN="./target/release/madu"

# Set environment variable to disable colors for clean README output
export MADU_NO_COLOR=true

# Function to run command and format output
run_command() {
    local description="$1"
    local command="$2"
    # Replace cargo run with binary path in command display
    local display_command="${command/cargo run --/$MADU_BIN}"
    echo "### $description" >> "$README_FILE"
    echo "" >> "$README_FILE"
    echo "\`\`\`bash" >> "$README_FILE"
    echo "$ $display_command" >> "$README_FILE"
    echo "\`\`\`" >> "$README_FILE"
    echo "" >> "$README_FILE"
    echo "\`\`\`" >> "$README_FILE"
    # Replace cargo run with binary path in actual execution
    local exec_command="${command/cargo run --/$MADU_BIN}"
    eval "$exec_command" >> "$README_FILE" 2>&1
    echo "\`\`\`" >> "$README_FILE"
    echo "" >> "$README_FILE"
}

# Create README header
cat > "$README_FILE" << 'EOF'
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

EOF

# Add help output
echo "### Help Output" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "\`\`\`bash" >> "$README_FILE"
echo "$ $MADU_BIN --help" >> "$README_FILE"
echo "\`\`\`" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "\`\`\`" >> "$README_FILE"
$MADU_BIN --help >> "$README_FILE" 2>&1
echo "\`\`\`" >> "$README_FILE"
echo "" >> "$README_FILE"

echo "## Examples" >> "$README_FILE"
echo "" >> "$README_FILE"

echo "### Basic Analysis" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Count lines in all files" "cargo run -- src"
echo "Shows line count for each file. Numbers are color-coded: red = large files, green = small files." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Summary by file extension" "cargo run -- --summary src"
echo "Groups results by file extension. Useful for understanding codebase composition." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Top 5 largest files" "cargo run -- --top 5 src"
echo "Limits output to top N results. Essential for focusing on the most significant files." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Files with more than 100 lines" "cargo run -- --min-value 100 src"
echo "Filters out small files. Helps identify substantial code files that need attention." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "### Code Analysis Features" >> "$README_FILE"
echo "" >> "$README_FILE"

echo "#### Basic Code Metrics" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Cyclomatic complexity analysis" "cargo run -- --complexity --top 5 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Measures code complexity using control flow analysis. Higher values indicate more complex, harder-to-test code that may need refactoring." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Code density calculation" "cargo run -- --density --threshold 75 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Calculates code density based on operators, keywords, and nesting. Higher scores indicate dense, potentially hard-to-read code." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "File size measurement" "cargo run -- --size --min-value 1024 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Shows file sizes with human-readable units (K, M, G). Useful for identifying bloated files over 1KB." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Character counting (non-whitespace)" "cargo run -- --chars --top 10 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Counts actual code characters, excluding whitespace. More accurate than line count for measuring code volume." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Nesting depth analysis" "cargo run -- --indent --threshold 80 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Measures deepest nesting level. High values may indicate overly complex functions needing refactoring." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "#### Advanced Code Analysis" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Code duplication detection" "cargo run -- --duplicates --min-value 10 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Detects duplicate code sections. Higher percentages indicate repeated code that could be refactored into functions." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Content analysis with emoji detection" "cargo run -- --emoji --include '*.md' --include '*.txt'"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Analyzes content for emoji usage. Useful for documentation and communication files." >> "$README_FILE"
echo "" >> "$README_FILE"

# Git-based analysis (if in git repo)
if git rev-parse --git-dir > /dev/null 2>&1; then
    echo "### Git History Analysis" >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    echo "#### Change Frequency & Activity" >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "Recent commit activity (last 30 days)" "cargo run -- --churn --days 30 --top 10 src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Shows commit frequency in recent timeframe. High churn files change often and may need architectural review." >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "File staleness analysis" "cargo run -- --age --threshold 90 src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Days since last modification. Helps identify stale code and recent activity patterns. Shows only oldest 10% of files." >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "Development rhythm analysis" "cargo run -- --rhythm --min-value 5 src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Commit frequency variation score. High values indicate irregular development patterns that may need attention." >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    echo "#### Code Ownership & Attribution" >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "Primary author ownership analysis" "cargo run -- --ownership --threshold 70 src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Percentage of commits by primary author. Shows files with >70% single-author ownership (potential knowledge silos)." >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "Git attribution by author" "cargo run -- --blame --author 'john' src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Shows primary author name per file, filtered by specific author. Useful for code review assignments." >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "Commit isolation patterns" "cargo run -- --isolation --min-value 50 src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Percentage of single-file commits. High isolation suggests focused, atomic changes. Shows files with >50% isolated commits." >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    echo "#### Risk Assessment" >> "$README_FILE"
    echo "" >> "$README_FILE"
    
    run_command "Hotspot analysis (complexity × churn)" "cargo run -- --hotspots --top 5 src"
    echo "> [!IMPORTANT]" >> "$README_FILE"
    echo "> Combines complexity and change frequency. High scores identify files that are both complex and frequently modified - prime refactoring candidates." >> "$README_FILE"
    echo "" >> "$README_FILE"
fi

echo "### Directory Analysis" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Aggregate by directory" "cargo run -- --dirs src"
echo "Rolls up metrics by directory. Useful for understanding module sizes and identifying large subsystems." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Limit directory depth" "cargo run -- --dirs --depth 1 src"
echo "Controls aggregation depth. Prevents deeply nested structures from cluttering results." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "### Advanced Filtering & Output Control" >> "$README_FILE"
echo "" >> "$README_FILE"

echo "#### File Pattern Filtering" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Include specific file types" "cargo run -- --include '*.rs' --include '*.js' --complexity src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Include only specific file patterns. Essential for language-specific analysis." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Exclude noise files" "cargo run -- --exclude '*.lock' --exclude '*.toml' --exclude '*.json' --size src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Filters out noise files using glob patterns. Essential for focusing on actual source code." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Auto-exclude common noise files" "cargo run -- --no-noise --complexity --top 10 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Automatically excludes configs, locks, generated files. Quick way to focus on source code." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "#### Result Limiting & Thresholds" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Threshold filtering (top 50%)" "cargo run -- --threshold 50 --complexity src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Shows only files above percentage threshold of maximum value. Filters out small/unimportant files automatically." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Minimum value filtering" "cargo run -- --min-value 200 --chars src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Shows only files above absolute threshold. Useful for finding substantial files." >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Top N results with skip" "cargo run -- --top 10 --skip 3 --size src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Shows top 10 results after skipping first 3. Useful when top files are known outliers." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "#### Content-Based Filtering" >> "$README_FILE"
echo "" >> "$README_FILE"

run_command "Filter by git author" "cargo run -- --author 'jane' --churn --days 7 src"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Filter results by git author name. Shows recent changes by specific team member." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "### Real-Time Monitoring & Watch Mode" >> "$README_FILE"
echo "" >> "$README_FILE"

echo "#### Development Workflow Integration" >> "$README_FILE"
echo "" >> "$README_FILE"

echo "**Monitor complexity changes**" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "\`\`\`bash" >> "$README_FILE"
echo "$ $MADU_BIN --watch 10 --complexity --top 5 src" >> "$README_FILE"
echo "\`\`\`" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Real-time monitoring refreshing every 10 seconds. Watch complexity metrics during active development. Press Ctrl+C to stop." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "**Track file size growth**" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "\`\`\`bash" >> "$README_FILE"
echo "$ $MADU_BIN --watch 30 --size --threshold 80 --include '*.rs' src" >> "$README_FILE"
echo "\`\`\`" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Monitor file size changes every 30 seconds. Useful for catching file bloat during development." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "**Watch git activity**" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "\`\`\`bash" >> "$README_FILE"
echo "$ $MADU_BIN --watch 60 --churn --days 1 --top 10 src" >> "$README_FILE"
echo "\`\`\`" >> "$README_FILE"
echo "" >> "$README_FILE"
echo "> [!IMPORTANT]" >> "$README_FILE"
echo "> Track recent commit activity every minute. Ideal for team leads monitoring daily development patterns." >> "$README_FILE"
echo "" >> "$README_FILE"

echo "> [!NOTE]" >> "$README_FILE"
echo "> Watch mode examples above show the command structure. In practice, watch mode runs continuously and updates the display in real-time." >> "$README_FILE"
echo "" >> "$README_FILE"

EOF

echo "README.md generated successfully!"