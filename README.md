

```
 ██╗      ██████╗ ██████╗ ███████╗██████╗
 ██║     ██╔════╝ ██╔══██╗██╔════╝██╔══██╗
 ██║     ██║  ███╗██████╔╝█████╗  ██████╔╝
 ██║     ██║   ██║██╔══██╗██╔══╝  ██╔═══╝
 ███████╗╚██████╔╝██║  ██║███████╗██║
 ╚══════╝ ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═╝
```

# lgrep

**Local semantic grep** - A 100% offline, privacy-preserving semantic code search tool.

Search your codebase using natural language queries. All processing happens locally on your machine using embedded ONNX models - no internet required, no API keys, no data leaves your computer.

## Features

- **🔒 Complete Privacy**: All embeddings and search happen locally
- **⚡ Fast**: Sub-second semantic search after indexing
- **💾 Offline**: Works without internet connection
- **🆓 Free**: No API costs or usage limits
- **🎯 Smart**: Understands code context, not just keywords
- **🔄 Live Updates**: Watch mode keeps index synchronized
- **🔍 Advanced Filtering**: Filter by extension, language, path, or similarity score
- **🎭 Hybrid Search**: Combine semantic + keyword matching for precision
- **📚 Query History**: Track and analyze your search patterns

## Quick Start

```bash
# Build from source
cargo build --release

# Index your project (first time)
lgrep index .

# Basic search
lgrep "where do we handle authentication"
lgrep "database connection setup" -c  # show content
lgrep "error handling" -m 20          # more results

# Advanced features
lgrep "api endpoint" --ext rs,py      # filter by extension
lgrep "auth" -k "jwt|token"           # hybrid search
lgrep history                         # view search history

# Watch for changes (keeps index updated)
lgrep watch .
```

## Installation

```bash
# Clone and build
git clone https://github.com/yourusername/lgrep
cd lgrep
cargo build --release

# Install to cargo bin directory
cargo install --path .
```

### Adding to PATH

After installation, you may need to add Cargo's bin directory to your PATH:

**For zsh (macOS default):**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**For bash:**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Verify installation:
```bash
lgrep --version
```

## Commands

### `lgrep [query]` - Search (default)

```bash
lgrep "authentication middleware"
lgrep "setup database" -c             # show content
lgrep "handle errors" -m 20           # max 20 results
lgrep "api endpoints" --json          # JSON output

# Filter searches
lgrep "error handling" --ext rs,py    # only Rust and Python files
lgrep "database query" --lang rust    # only Rust language
lgrep "config" --path-pattern "src/.*" # only in src/
lgrep "test" --exclude "test.*"       # exclude test files
lgrep "query" --min-score 0.8         # high similarity only

# Hybrid search (semantic + keyword)
lgrep "user auth" -k "jwt|token"      # boost results with jwt/token
```

### `lgrep index <path>` - Build index

```bash
lgrep index .                         # index current directory
lgrep index . --model nomic           # use different model
lgrep index . --force                 # force rebuild
```

### `lgrep watch <path>` - Live updates

```bash
lgrep watch .                         # watch and auto-update
```

### `lgrep stats` - Show statistics

```bash
lgrep stats
```

### `lgrep history` - Query history

```bash
lgrep history                         # show recent searches
lgrep history --top                   # show most frequent
lgrep history --clear                 # clear history
```

### `lgrep models` - List available models

```bash
lgrep models
```

## Advanced Features

### Metadata Filtering

Filter search results by file attributes:

```bash
# By file extension
lgrep "query" --ext rs,py,js

# By programming language
lgrep "query" --lang rust,python

# By path pattern (regex)
lgrep "query" --path-pattern "src/api/.*"

# Exclude paths (regex)
lgrep "query" --exclude "test.*|.*_test\.rs"

# Minimum similarity score
lgrep "query" --min-score 0.75
```

### Hybrid Search

Combine semantic search with keyword matching for better precision:

```bash
# Semantic search + keyword boost
lgrep "authentication" -k "jwt|token|oauth"

# The -k flag accepts regex patterns
# Results matching the pattern get a score boost
```

### Query History

lgrep tracks your searches and provides insights:

```bash
# View recent searches
lgrep history

# View most frequent queries
lgrep history --top

# Limit number of results
lgrep history -n 20

# Clear history
lgrep history --clear
```

History includes:
- Query text
- Number of results
- Timestamp
- Filters used

### Combining Features

```bash
# Advanced filtered hybrid search
lgrep "database connection" \
  --ext rs \
  --lang rust \
  --path-pattern "src/.*" \
  --exclude "test.*" \
  --min-score 0.7 \
  -k "pool|timeout" \
  -c -m 15
```

## Embedding Models

All models run locally via ONNX runtime - no API keys needed!

| Model | Dimensions | Size | Best For |
|-------|------------|------|----------|
| `minilm` (default) | 384 | ~30MB | Quick indexing, general use |
| `bge` | 384 | ~90MB | Better semantic understanding |
| `nomic` | 768 | ~90MB | Code and technical content |
| `multilingual` | 384 | ~470MB | Multi-language codebases |

## System Requirements

### Minimum Requirements
- **RAM**: 2GB (for codebases up to 100MB)
- **Disk**: 500MB free (for model cache + index)
- **OS**: macOS, Linux, or Windows

### Recommended by Codebase Size

| Codebase Size | RAM | Notes |
|---------------|-----|-------|
| < 100MB | 4GB | Comfortable for most projects |
| 100-250MB | 8GB | Good for medium projects |
| 250-500MB | 16GB | Large projects, monorepos |
| 500MB-1GB | 32GB | Very large codebases |
| > 1GB | 64GB+ | Enterprise monorepos |

**Note**: High memory usage only during initial indexing. Searching and incremental updates use minimal RAM (~100-200MB).

### Model Requirements
- **Storage**: 30-470MB per model (cached in `~/.cache/huggingface/`)
- **Initial Download**: One-time download on first use
- **Shared**: Models are reused across all projects

## Environment Variables

```bash
export LGREP_MAX_COUNT=20      # default max results
export LGREP_CONTENT=1         # always show content
export LGREP_MODEL=nomic       # default model
```

## Ignore Files

lgrep respects `.gitignore`, `.ignore`, and `.lgrepignore`.

## How It Works

1. **Chunking**: Files split into ~512 char overlapping chunks
2. **Embedding**: Each chunk → 384-dim vector (local ONNX model)
3. **Indexing**: Vectors stored in HNSW graph for fast search
4. **Search**: Query embedded, nearest neighbors found in ~ms

## License

MIT OR Apache-2.0
