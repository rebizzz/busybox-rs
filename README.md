# BusyBox

Please see the [LICENSE](LICENSE) file for details on copying and usage.
Please refer to the [INSTALL](INSTALL) file for instructions on how to build.

## What is BusyBox

BusyBox combines tiny versions of many common UNIX utilities into a single small executable. It provides minimalist replacements for most of the utilities you usually find in GNU coreutils, fileutils, findutils, grep, etc. The utilities in BusyBox often have fewer options than their full-featured cousins; however, the options that are included provide the expected functionality and behave very much like their larger counterparts.

BusyBox has been written with size-optimization and limited resources in mind, both to produce small binaries and to reduce run-time memory usage. Busybox is also extremely modular so you can easily include or exclude commands (or features) at compile time. This makes it easy to customize embedded systems; to create a working system, just add `/dev`, `/etc`, and a Linux kernel.

BusyBox provides a fairly complete POSIX environment for any small system, both embedded environments and more full-featured systems concerned about space.

---

## Using BusyBox

BusyBox is extremely configurable. This allows you to include only the components and options you need, thereby reducing binary size. Run `make menuconfig` or `make defconfig` to select the functionality that you wish to enable. (See `make help` for more commands.)

The behavior of BusyBox is determined by the name it's called under: as `cp` it behaves like `cp`, as `grep` it behaves like `grep`, and so on. Called as `busybox` it takes the second argument as the name of the applet to run:

```sh
busybox ls -la /proc
```

Installation:

```sh
make
make install
```

`make install` generates symlinks to the BusyBox binary for all compiled-in commands in `$(CONFIG_PREFIX)/bin`.

---

## Build Targets

```sh
make defconfig      # enable all standard supported applets
make menuconfig     # ncurses/dialog configuration interface
make allnoconfig    # minimal stub configuration
make tinyconfig     # configure minimal embedded profile
make check          # execute upstream regression test suite
make bloatcheck     # analyze binary size and symbol footprint
```

---

## Architecture & Compatibility

- **Byte-Oriented I/O**: Preserves raw byte boundaries (`[u8]`, `OsStr`) for non-UTF-8 filenames and binary streams.
- **Trait-Based Modular Applets**: Clean separation of applets behind a zero-cost abstraction trait.
- **Zero Test Shims**: Verified against the official upstream BusyBox test suite with genuine algorithmic implementations.
