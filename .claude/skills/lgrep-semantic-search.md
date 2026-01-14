# lgrep: Semantic Code Search

## Description
Use lgrep to perform semantic code search across the project. lgrep understands code meaning, not just keywords, making it ideal for finding relevant implementations, patterns, and examples. All processing happens locally with no external API calls.

## When to Use
- Finding implementations of specific functionality
- Discovering code patterns (error handling, authentication, etc.)
- Understanding how features are implemented before making changes
- Locating similar code to maintain consistency
- Mapping dependencies before refactoring
- Finding examples of specific APIs or frameworks being used

## Setup
Ensure lgrep is installed and the project is indexed:

```bash
# Install if needed (from project root)
cargo install --path .

# Index the project once (use nomic model for best code understanding)
lgrep index . --model nomic

# Verify installation
lgrep stats
```

## Basic Usage

### Search for Code
```bash
# Basic semantic search
lgrep "authentication middleware"

# Show code content in results
lgrep "database connection setup" -c

# Get more results
lgrep "error handling patterns" -c -m 20

# Get JSON output for parsing
lgrep "api endpoints" --json
```

### Keep Index Updated
```bash
# Auto-update index while working (run in background)
lgrep watch . &

# Or sync manually before search
lgrep "query" --sync
```

## Common Patterns

### Before Implementing a Feature
Always search for similar implementations to understand patterns and maintain consistency:

```bash
# Find existing implementations
lgrep "user authentication flow" -c -m 10

# Find related components
lgrep "user model definition" -c
lgrep "password validation" -c -m 5

# Find middleware patterns
lgrep "request middleware" -c -m 5
```

### Understanding Dependencies
Before refactoring, map out relationships:

```bash
# Find all database usage
lgrep "database query" -c -m 20

# Find caching patterns
lgrep "cache implementation" -c -m 10

# Find configuration usage
lgrep "config loading" -c -m 10
```

### Finding Patterns to Follow
Maintain code consistency by finding existing patterns:

```bash
# Error handling patterns
lgrep "error response format" -c -m 10

# Logging patterns
lgrep "logging implementation" -c -m 10

# Test setup patterns
lgrep "test setup" -c -m 10
```

### Debugging
Locate relevant code when investigating issues:

```bash
# Find timeout configurations
lgrep "connection timeout" -c -m 10

# Find retry logic
lgrep "retry mechanism" -c -m 5

# Find error handling
lgrep "error handling in payments" -c -m 10
```

## Tips

### Model Selection
- Use `nomic` model (default setup above) for best code understanding
- Model is specified during indexing: `lgrep index . --model nomic`

### Search Quality
- Be specific but natural: "jwt token validation" vs "validate jwt"
- Use domain terms: "middleware", "handler", "controller", etc.
- Ask questions: "how is authentication implemented"
- Describe purpose: "code that handles user login"

### Result Management
- Start with fewer results (`-m 10`) for focused search
- Increase for broader context (`-m 20` or `-m 30`)
- Always use `-c` flag to see actual code content
- Use `--json` for structured output you need to parse

### Performance
- Index once per project (persists in `.lgrep/` directory)
- Use `watch` mode during active development
- Use `--sync` flag only when needed (not every search)
- Search is instant after indexing (sub-second)

## Example Workflow

### Task: "Add JWT authentication to the API"

1. **Understand existing auth patterns:**
```bash
lgrep "authentication implementation" -c -m 10
```

2. **Find middleware patterns:**
```bash
lgrep "middleware setup" -c -m 5
```

3. **Find JWT-specific code (if any):**
```bash
lgrep "jwt token" -c -m 5
```

4. **Check API route protection:**
```bash
lgrep "protected routes" -c -m 5
```

5. **Use findings to inform implementation that matches project patterns**

### Task: "Fix database connection timeout"

1. **Locate connection setup:**
```bash
lgrep "database connection" -c -m 15
```

2. **Find timeout configurations:**
```bash
lgrep "connection timeout" -c -m 10
lgrep "pool configuration" -c -m 5
```

3. **Check retry logic:**
```bash
lgrep "connection retry" -c -m 5
```

4. **Apply fix based on current implementation patterns**

## Command Reference

| Command | Purpose |
|---------|---------|
| `lgrep index .` | Index current directory |
| `lgrep index . --model nomic` | Index with best model for code |
| `lgrep "query"` | Basic search |
| `lgrep "query" -c` | Search with code content |
| `lgrep "query" -m 20` | Search with more results |
| `lgrep "query" --json` | Get JSON output |
| `lgrep stats` | Show index statistics |
| `lgrep watch .` | Auto-update index |
| `lgrep --help` | Full command help |

## Key Benefits
- **Semantic Understanding**: Finds code by meaning, not just text matching
- **Privacy**: 100% local processing, no external API calls
- **Fast**: Sub-second search after initial indexing
- **Free**: No API costs or usage limits
- **Offline**: Works without internet connection
- **Git-Aware**: Respects `.gitignore` automatically

## Notes
- Index is stored in `.lgrep/` directory (add to `.gitignore`)
- First-time indexing takes 1-2 minutes for typical projects
- Index updates are incremental and fast
- All models run locally via ONNX (no cloud dependencies)