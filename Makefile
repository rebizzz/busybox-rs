# ==========================================================================
# BusyBox-RS Build system (Safe Rust Reimplementation)
# ==========================================================================

CONFIG_PREFIX ?= /usr/local
BIN_DIR ?= $(CONFIG_PREFIX)/bin
TARGET_DIR ?= target/release
BIN ?= $(TARGET_DIR)/busybox
CARGO ?= cargo
NIX ?= nix

.PHONY: all busybox clean distclean defconfig allyesconfig allnoconfig tinyconfig \
        install uninstall check test runtest bloatcheck help fmt clippy

all: busybox

busybox:
	@$(CARGO) build --release

# --------------------------------------------------------------------------
# Configuration targets (Kconfig / Cargo features)
# --------------------------------------------------------------------------

defconfig allyesconfig:
	@$(CARGO) build --release --all-features

allnoconfig:
	@$(CARGO) build --release --no-default-features

tinyconfig:
	@$(CARGO) build --release --no-default-features --features "echo,cat,ls,sh"

# --------------------------------------------------------------------------
# Installation targets
# --------------------------------------------------------------------------

install: busybox
	@echo "Installing BusyBox-RS to $(BIN_DIR)..."
	@install -d $(DESTDIR)$(BIN_DIR)
	@install -m 755 $(BIN) $(DESTDIR)$(BIN_DIR)/busybox
	@for applet in $$($(BIN) --list); do \
		ln -sf busybox $(DESTDIR)$(BIN_DIR)/$$applet; \
	done
	@echo "Installed $$($(BIN) --list | wc -w) applets."

uninstall:
	@echo "Uninstalling BusyBox-RS from $(BIN_DIR)..."
	@if [ -x $(DESTDIR)$(BIN_DIR)/busybox ]; then \
		for applet in $$($(DESTDIR)$(BIN_DIR)/busybox --list); do \
			rm -f $(DESTDIR)$(BIN_DIR)/$$applet; \
		done; \
		rm -f $(DESTDIR)$(BIN_DIR)/busybox; \
	fi

# --------------------------------------------------------------------------
# Development & Testing targets
# --------------------------------------------------------------------------

test:
	@$(CARGO) test --release

check runtest: busybox
	@mkdir -p ../test-bin
	@cp $(BIN) ../test-bin/busybox
	@if [ -d ../busybox-upstream/testsuite ]; then \
		echo "Running upstream test suite verification..."; \
		cd ../busybox-upstream/testsuite && bindir=$$(pwd)/../../test-bin ./runtest; \
	else \
		echo "Upstream testsuite not found at ../busybox-upstream/testsuite"; \
	fi

fmt:
	@$(CARGO) fmt --all

clippy:
	@$(CARGO) clippy --all-targets -- -D warnings

bloatcheck: busybox
	@echo "=== BusyBox-RS Binary Footprint ==="
	@size $(BIN) 2>/dev/null || true
	@ls -lh $(BIN)

# --------------------------------------------------------------------------
# Cleaning targets
# --------------------------------------------------------------------------

clean:
	@$(CARGO) clean

distclean: clean
	@rm -rf target/ Cargo.lock .config

# --------------------------------------------------------------------------
# Help
# --------------------------------------------------------------------------

help:
	@echo 'Cleaning:'
	@echo '  clean			- delete temporary files created by build'
	@echo '  distclean		- delete all non-source files'
	@echo
	@echo 'Build:'
	@echo '  all			- compile the swiss-army executable'
	@echo '  busybox		- compile the release executable'
	@echo
	@echo 'Configuration:'
	@echo '  defconfig		- enable all standard supported applets'
	@echo '  allyesconfig		- enable all feature flags'
	@echo '  allnoconfig		- disable all applets (bare dispatcher)'
	@echo '  tinyconfig		- build minimal embedded profile'
	@echo
	@echo 'Installation:'
	@echo '  install		- install busybox and symlink farm into CONFIG_PREFIX'
	@echo '  uninstall		- remove installed binary and symlinks'
	@echo
	@echo 'Development:'
	@echo '  check / runtest	- run test suite verification against upstream tests'
	@echo '  test			- run Rust unit and integration tests'
	@echo '  fmt			- format all Rust sources'
	@echo '  clippy		- run clippy linter'
	@echo '  bloatcheck		- inspect static executable size and sections'
	@echo
