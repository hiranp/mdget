# External Integrations

**Mapped:** 2026-05-21  
**Project:** mdget

## Summary

Currently a **template-based CLI project** with no external integrations implemented. The codebase is scaffolding for future HTTP client functionality (the "agent-first curl alternative" described in Cargo.toml).

## External Services

**None currently integrated.**

## Databases

**None currently integrated.**

## Authentication Providers

**None currently integrated.**

## APIs & Webhooks

**None currently integrated.**

Expected future integrations based on "curl alternative" positioning:
- HTTP/HTTPS endpoints (target servers)
- Potentially LLM APIs for "agent-first" functionality
- Certificate validation / TLS infrastructure

## File System Integration

### XDG Base Directory Specification
- **Config directory:** `~/.config/mdget/` (or `$XDG_CONFIG_HOME/mdget/`)
  - Expected file: `config.toml`
  - System-level fallback: `/etc/mdget/config.toml`

- **Cache directory:** `~/.cache/mdget/` (or `$XDG_CACHE_HOME/mdget/`)
  - Used for: rolling log files when file logging enabled

- **Environment override:** `MDGET_HOME` — custom config directory path

### Log Files
- Location: Determined by config (`log.file.path`)
- Default: XDG cache directory
- Filename: `rolling.log` (daily rotation)
- Format: Plain text, no ANSI codes

## Build-time Generation

### Shell Completions
- Generated at compile time by `build.rs`
- Output directory: `completions/` (git-ignored)
- Generated for: bash, zsh, fish, PowerShell
- Distribution: Packaged with binary releases

## Future Integration Points

Based on project description ("agent-first curl alternative"), likely future needs:

### HTTP Client Infrastructure
- HTTP/HTTPS request handling
- TLS certificate validation
- Proxy support
- Cookie management
- Header manipulation

### AI/Agent Integration (Speculative)
- LLM API integration (OpenAI, Anthropic, etc.) for "agent-first" functionality
- Potential: request/response interpretation, context extraction, automation hints

### Data Formats
- JSON parsing/formatting
- XML/HTML parsing
- Multipart form data
- URL encoding

### Authentication (Speculative)
- Bearer token handling
- Basic auth
- OAuth flows
- API key management

## Integration Patterns

### Configuration Layering
Current pattern for all configurable behavior:
1. Compile-time defaults
2. System config (`/etc/mdget/config.toml`)
3. User config (`~/.config/mdget/config.toml`)
4. Environment variables (`MDGET_*`)
5. Command-line flags (highest priority)

This pattern should extend to future HTTP client settings, auth, proxies, etc.

### Error Handling
All external interactions should use `miette::Result<T>` for:
- Network errors → rich diagnostic context
- File I/O errors → path-aware error messages
- Parse errors → snippet highlighting

---
*Integrations mapped: 2026-05-21*
*Status: Template project — primary HTTP client functionality not yet implemented*
