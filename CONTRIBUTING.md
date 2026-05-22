# Contributing to mdget

Thank you for your interest in contributing to **mdget**! Contributions from the community help make this tool better for everyone.

By contributing to this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

---

## 📖 Table of Contents

- [How Can I Contribute?](#how-can-i-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Enhancements](#suggesting-enhancements)
  - [Submitting Pull Requests](#submitting-pull-requests)
- [Development Setup](#development-setup)
- [Coding Standards & Style Guide](#coding-standards--style-guide)
- [Pull Request Checklist](#pull-request-checklist)
- [Questions or Security Reporting](#questions-or-security-reporting)

---

## How Can I Contribute?

### Reporting Bugs

If you find a bug, please open an issue on GitHub. Before creating a new issue, search existing ones to make sure it hasn't been reported yet. When filing a bug report, please include:
- A clear description of the issue.
- Steps to reproduce the bug (including the exact command and target URL).
- The expected behavior vs. actual behavior.
- Your OS, Rust version, and `mdget` version.

### Suggesting Enhancements

We are always looking for ways to improve `mdget` (e.g., adding support for more content types, caching optimizations, or filtering features). Open an issue with the tag `enhancement` and describe:
- The problem you want to solve.
- The proposed solution or feature design.
- Why this enhancement would be useful to other users/agents.

### Submitting Pull Requests

1. Fork the repository and create your branch from `main` (e.g., `git checkout -b feature/my-cool-feature`).
2. Make your changes, keeping them as concise and focused as possible.
3. Add or update unit/integration tests covering your changes.
4. Ensure all tests and style checks pass (see [Development Setup](#development-setup)).
5. Push your branch to GitHub and open a Pull Request.

---

## Development Setup

### Prerequisites

Ensure you have the following installed on your system:
- **Rust Toolchain**: Rust 1.90+ (Edition 2024).
- **Cargo**: Usually bundled with Rust.
- **Just** (Optional): A handy command runner to build/check the project easily.

### Local Workflow

Clone the repository and build the project:

```bash
git clone https://github.com/hiranp/mdget.git
cd mdget
cargo build
```

Run unit and integration tests:

```bash
cargo test
```

Run the local binary with debug output enabled:

```bash
RUST_LOG=debug cargo run -- fetch https://example.com
```

Format code and check for lints:

```bash
# Format code
cargo fmt --all -- --check

# Check clippy lints
cargo clippy --all-targets --all-features -- -D warnings
```

If you have `just` installed, you can run all local CI checks with:

```bash
just check
```

---

## Coding Standards & Style Guide

- **Idiomatic Rust**: Write clean, modern Rust adhering to common idioms and compiler recommendations.
- **Rust Edition**: We use the Rust 2024 edition. Make sure your toolchain supports it.
- **Formatting**: All code must conform to `rustfmt`. Run `cargo fmt --all` before committing.
- **Linting**: Keep code warning-free. Always run `cargo clippy --all-targets -- -D warnings`.
- **Commit Messages**: We recommend using descriptive commit messages, ideally following [Conventional Commits](https://www.conventionalcommits.org/):
  - `feat: add support for PDF content extraction`
  - `fix: resolve quotes escaping in meta tag parsing`
  - `docs: update setup instructions in README`
  - `test: add integration test for redirect chains`

---

## Pull Request Checklist

Before submitting your PR, please verify the following:
- [ ] Code is formatted with `cargo fmt`.
- [ ] Code passes all `cargo clippy` lints without warnings.
- [ ] All tests in `cargo test` pass successfully.
- [ ] You have added tests covering the new functionality or bug fix.
- [ ] Documentation (README or inline docs/docstrings) has been updated if applicable.
- [ ] The PR title and description clearly explain what changes were made and why.

---

## Questions or Security Reporting

If you have questions about the codebase or wish to report a security vulnerability, please reach out to the project maintainers:
- **Contact Email**: `hiranp@yahoo.com`
- **GitHub Issues**: For non-sensitive bugs and feature requests.
