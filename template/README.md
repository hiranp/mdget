# {{project-name}}

{{long-description}}

## Installation

### From crates.io

```bash
cargo install {{project-name}}
```

### From source

```bash
git clone https://github.com/{{gh-username}}/{{project-name}}.git
cd {{project-name}}
cargo install --path .
```

## Usage

```bash
{{application}} --help
```

## Development

Install `just` (optional but recommended):

```bash
cargo install just
```

Run common tasks:

```bash
just check
just run -- --help
just release
just deny
```

Equivalent raw Cargo commands:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## License

Licensed under MIT.

## Releases

Pushing a git tag like `v0.1.0` triggers the release workflow and builds
cross-platform artifacts for Linux, macOS, and Windows.

## Dependency Policy

`cargo-deny` policy is configured in `deny.toml` and can be run locally:

```bash
cargo install cargo-deny --locked
just deny
```
