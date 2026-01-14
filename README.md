

```  _                           
 | |   __ _  _ __  ___  _ __  
 | |  / _` || '__|/ _ \| '_ \ 
 | | | (_| || |  |  __/| |_) |
 |_|  \__, ||_|   \___|| .__/ 
      |___/            |_|    
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

## Quick Start

```bash
# Build from source
cargo build --release

# Index your project (first time)
lgrep index .

# Search!
lgrep "where do we handle authentication"
lgrep "database connection setup" -c  # show content
lgrep "error handling" -m 20          # more results

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

### `lgrep models` - List available models

```bash
lgrep models
```

## Embedding Models

All models run locally via ONNX runtime - no API keys needed!

| Model | Dimensions | Size | Best For |
|-------|------------|------|----------|
| `minilm` (default) | 384 | ~30MB | Quick indexing, general use |
| `bge` | 384 | ~90MB | Better semantic understanding |
| `nomic` | 768 | ~90MB | Code and technical content |
| `multilingual` | 384 | ~470MB | Multi-language codebases |

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
