# Contributing to BusyBox-RS

Thank you for your interest in contributing to **BusyBox-RS**! We welcome bug reports, documentation improvements, new applets, and code optimizations.

## Core Tenets & Guidelines

When writing code for BusyBox-RS, please keep these rules firmly in mind:

1. **Memory Safety & No Unsound Hacks**:
   - Utilize safe Rust patterns. Avoid unnecessary `unsafe` blocks.
   - Use Rust's ownership and type system to eliminate buffer overflows, use-after-free, dangling pointers, and race conditions.

2. **Unix As Raw Bytes (`[u8]`, `OsStr`, `Path`)**:
   - Do NOT assume all inputs are valid UTF-8. Unix paths, arguments, and standard streams can contain arbitrary non-UTF-8 bytes.
   - Treat streams and arguments as raw bytes where possible. Only convert to strings when specifically formatted for human-readable output.

3. **No Test Shims or Cheating**:
   - Every applet must implement real Unix / POSIX semantics.
   - Do NOT hardcode test inputs or add shims to cheat test suites. Real algorithms only.

4. **Zero-Cost Modularity (Compile-Time Selection)**:
   - Every new applet must be compile-time selectable via a Cargo feature in `Cargo.toml`.
   - Feature flags should match the applet name (e.g. `feature = "grep"`).

5. **Binary Size & Minimal Dependencies**:
   - Avoid bloated dependencies. Any new dependency must be strictly justified in terms of static binary footprint.
   - Standard library and minimal libc bindings are preferred.

## Development Workflow

### Building and Testing

1. Drop into the reproducible Nix shell:
   ```bash
   nix develop
   ```

2. Build release binary:
   ```bash
   cargo build --release
   ```

3. Run verification against upstream BusyBox tests:
   ```bash
   cd ../busybox-upstream/testsuite
   bindir=/path/to/test-bin ./runtest <applet-name>
   ```

4. Format and lint:
   ```bash
   cargo fmt --check
   cargo clippy
   ```

## Pull Requests

1. Fork the repository and create a branch for your feature or bug fix.
2. Ensure your changes compile cleanly without warnings.
3. Verify that all existing applet tests pass.
4. Submit a clear, descriptive Pull Request detailing the changes and test results.
