# lgrep Skill for Roo Code

## Overview
lgrep is a semantic code search tool that runs 100% locally. Use it to find relevant code by meaning, not just keywords.

## Core Skill: Semantic Code Search

### When to Use This Skill
- Need to understand how something is implemented in the codebase
- Looking for examples of patterns (error handling, authentication, etc.)
- Finding where specific functionality exists
- Understanding code relationships before making changes
- Locating similar code for consistency

### Basic Commands

```bash
# Index a project (do this once per project)
lgrep index .

# Search for code semantically
lgrep "authentication middleware"
lgrep "database connection setup"
lgrep "error handling patterns"

# Show actual code content in results (-c flag)
lgrep "jwt token validation" -c

# Get more results (-m flag)
lgrep "api endpoints" -m 20 -c

# Get structured JSON output for parsing
lgrep "test patterns" --json

# Get index statistics
lgrep stats

# Keep index updated while working
lgrep watch . &
```

### Recommended Model for Code
```bash
# Use nomic model for best code understanding
lgrep index . --model nomic
```

## Skill Workflow Pattern

### Pattern 1: Before Implementing a Feature
```bash
# 1. Search for similar implementations
lgrep "user authentication" -c -m 10

# 2. Find related components
lgrep "user model" -c -m 5
lgrep "auth middleware" -c -m 5

# 3. Use findings to inform implementation
```

### Pattern 2: Understanding Existing Code
```bash
# 1. Get overview of a concept
lgrep "payment processing" -c -m 10

# 2. Drill into specifics
lgrep "payment validation logic" -c
lgrep "transaction handling" -c
```

### Pattern 3: Maintaining Consistency
```bash
# 1. Find existing patterns
lgrep "error response format" -c -m 10

# 2. Apply same pattern in new code
```

### Pattern 4: Refactoring Preparation
```bash
# 1. Map dependencies
lgrep "database queries" -c -m 20
lgrep "cache usage" -c -m 10

# 2. Understand impact before refactoring
```

## Quick Reference

| Task | Command |
|------|---------|
| Index project | `lgrep index .` |
| Search with content | `lgrep "query" -c` |
| More results | `lgrep "query" -m 20` |
| JSON output | `lgrep "query" --json` |
| Check stats | `lgrep stats` |
| Update index | `lgrep watch .` |

## Key Benefits
- **Private**: All processing happens locally on the machine
- **Fast**: Sub-second search after indexing
- **Free**: No API costs or limits
- **Smart**: Understands code meaning, not just text matching
- **Offline**: Works without internet

## Example Usage in Roo Context

**User Task:** "Add JWT authentication to the API"

**Roo uses lgrep skill:**
```bash
# Gather context
lgrep "authentication implementation" -c -m 10
lgrep "jwt middleware" -c -m 5
lgrep "token validation" -c -m 5

# Use results to:
# - Understand existing auth patterns
# - Find where to integrate JWT
# - Maintain code consistency
# - Then implement the feature
```

**User Task:** "Fix the database connection timeout issue"

**Roo uses lgrep skill:**
```bash
# Find relevant code
lgrep "database connection" -c -m 15
lgrep "connection timeout" -c -m 10
lgrep "connection pool configuration" -c -m 5

# Use results to:
# - Locate connection setup
# - Find timeout configurations
# - Identify related error handling
# - Then apply fix
```

## Installation Check
```bash
# Verify lgrep is installed
lgrep --version

# If not installed, it's in the project:
cargo install --path .
```

## Notes
- Index persists in `.lgrep/` directory (add to `.gitignore`)
- First-time indexing takes 1-2 minutes for medium projects
- Subsequent searches are instant
- Index auto-updates with `watch` mode or manual `--sync` flag
- Use `nomic` model for best code search results