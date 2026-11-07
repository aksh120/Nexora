# Contributing to NEXORA

Thank you for your interest in contributing to **NEXORA**! We are building the fastest, leanest, streaming-first desktop browser for Windows, and we welcome contributions from developers who care deeply about performance, native systems programming, and user privacy.

---

## 📜 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Design Principles](#design-principles)
- [Prerequisites & Environment Setup](#prerequisites--environment-setup)
- [Development Workflow](#development-workflow)
- [Coding Standards & Guidelines](#coding-standards--guidelines)
- [Testing & Quality Assurance](#testing--quality-assurance)
- [Submitting a Pull Request](#submitting-a-pull-request)
- [Community & Questions](#community--questions)

---

## Code of Conduct

All contributors and maintainers are expected to follow our [Code of Conduct](CODE_OF_CONDUCT.md). Please read it before participating.

---

## Design Principles

Before writing code or proposing architectural changes, understand the core philosophy that governs every line in NEXORA:

1. **Maximum Performance, Minimum Overhead**: Every feature must justify its CPU cycles and memory footprint. If a feature adds measurable idle CPU usage or unbounded RAM growth, it will not be merged.
2. **Streaming-First Discipline**: Hardware video decoding (Direct3D 11, DXVA2) and seamless DRM media playback take priority over non-essential browser abstractions.
3. **Zero Telemetry**: No background telemetry, no pings, no cloud sync backdoors, no keystroke analytics. 100% local-first.
4. **Native Win32 Integration**: Prefer native OS-provided components (e.g. `winsqlite3.dll`, Direct3D 11, DWM composition, Win32 double-buffering) over bundling third-party runtime bloat.
5. **No Extensions in Core**: To maintain strict security boundaries and sub-700 KB binary footprint, WebExtension runtime layers are excluded from core.

---

## Prerequisites & Environment Setup

### 1. Hardware Requirements
- **OS**: Windows 10 (Build 1809+ / x64) or Windows 11 (x64)
- **CPU**: x86_64 architecture (2+ cores recommended)
- **RAM**: 4 GB minimum (8+ GB recommended)

### 2. Software Requirements
- **Rust Toolchain**: Rust 1.80+ (`x86_64-pc-windows-gnu` or `x86_64-pc-windows-msvc`)
- **Microsoft WebView2 Evergreen Runtime**: Pre-installed on Windows 11 and modern Windows 10.
- **Git** and **PowerShell 5.1+**

### 3. Clone and Verify
```powershell
# Clone the repository
git clone https://github.com/Nexora/Nexora.git
cd Nexora

# Run the test suite to verify your environment
$env:CARGO_INCREMENTAL = "0"
cargo test
```

---

## Development Workflow

### Building
```powershell
# Debug build (fast iteration)
cargo build

# Optimized release build (LTO, strip, panic abort)
cargo build --release
```

### Running
```powershell
# Launch debug build
cargo run

# Launch release binary
.\target\release\Nexora.exe
```

### Benchmarking
Run the automated benchmark suite against your local build:
```powershell
powershell -ExecutionPolicy Bypass -File tests/benchmark.ps1 -TargetBrowser Simulated -Scenario All
```

---

## Coding Standards & Guidelines

- **Formatting**: Format all Rust code using `cargo fmt` before committing.
- **Linter**: Clean linting with `cargo clippy --all-targets -- -D warnings`.
- **Error Handling**: Use the unified error handling architecture in `src/core/error.rs` (`NexoraError`, `Result<T, NexoraError>`). Do not use `unwrap()` or `expect()` in production runtime paths.
- **Resource Management**: COM pointers and Win32 handles must implement deterministic cleanup (`Drop` or RAII wrappers).
- **Documentation**: Provide clear Rustdoc comments (`///`) on all public APIs, modules, and structs. Maintain documentation integrity in `/dev`.

---

## Testing & Quality Assurance

Every bug fix or feature must include passing unit tests.

- Run all unit tests:
  ```powershell
  cargo test
  ```
- Run tests with full backtrace:
  ```powershell
  $env:RUST_BACKTRACE = "1"
  cargo test -- --nocapture
  ```
- All tests must pass with **zero failures** before pull request review.

---

## Submitting a Pull Request

1. **Fork** the repository and create your branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. **Follow Conventional Commits**:
   - `feat(module): description`
   - `fix(module): description`
   - `perf(module): description`
   - `docs(module): description`
   - `test(module): description`
3. **Verify Everything Passes**:
   - `cargo fmt --check`
   - `cargo test`
   - `cargo build --release`
4. **Open a PR** against `main` using our [Pull Request Template](.github/pull_request_template.md).

---

## Community & Questions

- **Bug Reports**: Open an issue using the [Bug Report Template](.github/ISSUE_TEMPLATE/bug_report.md).
- **Feature Proposals**: Open a discussion using the [Feature Request Template](.github/ISSUE_TEMPLATE/feature_request.md).
- **Architecture Discussions**: Read [`dev/Architecture.md`](dev/Architecture.md) and [`dev/Decisions.md`](dev/Decisions.md).

