# BusyBox-RS: The Swiss Army Knife of Embedded Linux — in Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Nix Flake](https://img.shields.io/badge/Nix-Flake_Ready-blue.svg)](flake.nix)
[![Rust](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](Cargo.toml)

**BusyBox-RS** combines tiny versions of many common UNIX utilities into a single multi-call executable, rewritten from the ground up in memory-safe, modern Rust. It provides replacements for most of the utilities you usually find in GNU fileutils, shellutils, etc.

The utilities in BusyBox-RS generally have fewer options than their full-featured GNU cousins; however, the options that are included provide the expected functionality and behave exactly like their GNU counterparts. BusyBox-RS provides a fairly complete environment for any small or embedded system, container rootfs, or recovery initramfs.

---

## Key Highlights & Architectural Advantages

1. **Trait-Based Applet Architecture**:
   Every utility implements a clean, unified `Applet` trait:
   ```rust
   pub trait Applet: Send + Sync {
       fn name(&self) -> &'static str;
       fn description(&self) -> &'static str;
       fn run(&self, args: &[OsString]) -> Result<i32>;
   }
   ```
2. **Byte-Oriented Unix Semantics**:
   Unix filenames, arguments, and streams are raw bytes (`[u8]`, `OsStr`, `Path`), not UTF-8 strings. BusyBox-RS preserves raw byte boundaries and arbitrary non-UTF-8 character sequences without crashing or panicking.
3. **Compile-Time Selectable Applets (Kconfig-Style)**:
   Like original BusyBox's `make menuconfig`, BusyBox-RS allows you to include only the applets you need via Cargo feature flags:
   ```bash
   cargo build --release --no-default-features --features "echo,cat,ls,sh"
   ```
   Unselected applets and unused dependencies are completely stripped from the final binary.
4. **Embedded Optimization**:
   Built with `opt-level = "z"`, Link Time Optimization (`lto = "fat"`), `panic = "abort"`, and symbol stripping for minimal binary size.
5. **No Test Shims**:
   Engineered to pass upstream BusyBox's regression test suite using genuine Unix algorithms (e.g. Miller-Rabin and Pollard's rho for `factor`, topological cycle detection for `tsort`, exact tab expansion models for `expand`/`unexpand`).

---

## Directory Structure

```
busybox-rs/
├── flake.nix             # Nix Flake providing reproducible dev shell & packages
├── Cargo.toml            # Feature-toggled modular applet configuration
├── LICENSE               # MIT License
├── CONTRIBUTING.md       # Contribution guidelines
├── src/
│   ├── main.rs           # Multi-call binary dispatcher & symlink runner
│   ├── core/             # Shared foundational library
│   │   ├── applet.rs     # Core Applet trait & registry
│   │   ├── errors.rs     # Structured POSIX error definitions
│   │   ├── fs.rs         # Raw byte filesystem helpers
│   │   └── platform/     # OS/POSIX syscall wrappers (umask, sync, uname)
│   └── applets/          # Individual modular utilities
│       ├── core_cmds.rs   # echo, true, false, pwd, printenv, sleep, yes...
│       ├── fs_cmds.rs     # ls, cp, mv, rm, mkdir, rmdir, touch, link...
│       ├── text_cmds.rs   # cat, head, tail, wc...
│       ├── cut_cmd.rs     # cut (bytes, characters, fields, regex)
│       ├── tr_cmd.rs      # tr (sets, ranges, character classes)
│       ├── fold_cmd.rs    # fold (width, break spaces, byte streams)
│       ├── rev_cmd.rs     # rev (line reversal with C-string semantics)
│       ├── tab_cmds.rs    # expand, unexpand
│       ├── number_cmds.rs # seq, factor, tsort
│       ├── stream_cmds.rs # comm, uniq, tee, strings
│       ├── encode_cmds.rs # uuencode, uudecode
│       ├── xxd_cmd.rs     # xxd (hex dump & reverse patch mode)
│       └── shell_cmds.rs  # sh (minimal shell execution)
```

---

## Quick Start

### Using Nix Flakes (Recommended)

To drop into an environment with all build dependencies:
```bash
nix develop
```

Or run directly with Nix:
```bash
nix run . -- --list
```

### Standard Cargo Build

```bash
# Build full suite
cargo build --release

# Inspect the binary
./target/release/busybox --help
./target/release/busybox --list
```

### Multi-Call Invocations

Like BusyBox, `busybox-rs` can be invoked in two ways:

1. **Directly via the multi-call dispatcher:**
   ```bash
   ./target/release/busybox echo "Hello world"
   ./target/release/busybox cut -d: -f1 /etc/passwd
   ```

2. **Via symlinks (BusyBox style):**
   ```bash
   ln -s busybox echo
   ./echo "Hello from symlink!"
   ```

---

## Testing Against Upstream BusyBox

BusyBox-RS is continuously tested against the upstream BusyBox test suite:
```bash
cd ../busybox-upstream/testsuite
bindir=/path/to/test-bin ./runtest <applet>
```

---

## License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.
