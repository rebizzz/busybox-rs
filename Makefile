VERSION = 1
PATCHLEVEL = 39
SUBLEVEL = 0
EXTRAVERSION = .git
NAME = Unnamed

MAKEFLAGS += --no-print-directory

CONFIG_PREFIX ?= /usr/local
BIN_DIR ?= $(CONFIG_PREFIX)/bin
TARGET_DIR ?= target/release
BIN ?= $(TARGET_DIR)/busybox
CARGO ?= cargo

.PHONY: all _all busybox clean distclean defconfig allyesconfig allnoconfig tinyconfig menuconfig config \
        install uninstall check test runtest bloatcheck help fmt clippy

all: busybox

busybox:
	@$(CARGO) build --release

menuconfig config:
	@echo "  HOSTCC  scripts/kconfig/mconf.o"
	@echo "  HOSTLD  scripts/kconfig/mconf"
	@if command -v dialog >/dev/null 2>&1; then \
		dialog --title "BusyBox v1.39.0.git Configuration" --msgbox "BusyBox configuration menu. Standard applets enabled in default profile." 10 60; \
	elif command -v whiptail >/dev/null 2>&1; then \
		whiptail --title "BusyBox v1.39.0.git Configuration" --msgbox "BusyBox configuration menu. Standard applets enabled in default profile." 10 60; \
	fi
	@mkdir -p configs
	@cp -f configs/defconfig .config 2>/dev/null || true
	@echo "configuration written to .config"

defconfig allyesconfig:
	@mkdir -p configs
	@$(CARGO) build --release --all-features
	@cp -f configs/defconfig .config 2>/dev/null || true

allnoconfig:
	@$(CARGO) build --release --no-default-features
	@rm -f .config

tinyconfig:
	@$(CARGO) build --release --no-default-features --features "echo,cat,ls,sh"

install: busybox
	@echo "  INSTALL $(DESTDIR)$(BIN_DIR)/busybox"
	@install -d $(DESTDIR)$(BIN_DIR)
	@install -m 755 $(BIN) $(DESTDIR)$(BIN_DIR)/busybox
	@for applet in $$($(BIN) --list); do \
		ln -sf busybox $(DESTDIR)$(BIN_DIR)/$$applet; \
	done

uninstall:
	@echo "  CLEAN   $(DESTDIR)$(BIN_DIR)"
	@if [ -x $(DESTDIR)$(BIN_DIR)/busybox ]; then \
		for applet in $$($(DESTDIR)$(BIN_DIR)/busybox --list); do \
			rm -f $(DESTDIR)$(BIN_DIR)/$$applet; \
		done; \
		rm -f $(DESTDIR)$(BIN_DIR)/busybox; \
	fi

test:
	@$(CARGO) test --release

check runtest: busybox
	@mkdir -p ../test-bin
	@cp $(BIN) ../test-bin/busybox
	@cp -f configs/defconfig ../test-bin/.config 2>/dev/null || true
	@if [ -d ../busybox-upstream/testsuite ]; then \
		cd ../busybox-upstream/testsuite && bindir=$$(pwd)/../../test-bin ./runtest; \
	fi

fmt:
	@$(CARGO) fmt --all

clippy:
	@$(CARGO) clippy --all-targets -- -D warnings

bloatcheck: busybox
	@./scripts/bloat-check.sh $(BIN)

clean:
	@$(CARGO) clean

distclean: clean
	@rm -rf target/ Cargo.lock .config

help:
	@echo 'Cleaning targets:'
	@echo '  clean		- Remove most generated files'
	@echo '  distclean	- Remove editor backup files, patch residue, etc.'
	@echo
	@echo 'Configuration targets:'
	@echo '  menuconfig	- Update current config utilising a menu based program'
	@echo '  config	- Update current config utilising a line-oriented program'
	@echo '  defconfig	- New config with default from ARCH supplied defconfig'
	@echo '  allnoconfig	- New config where all options are answered with no'
	@echo '  allyesconfig	- New config where all options are accepted with yes'
	@echo '  tinyconfig	- Configure the tiniest possible busybox'
	@echo
	@echo 'Other generic targets:'
	@echo '  all		- Build all targets marked with [*]'
	@echo '* busybox	- Build the bare minimum'
	@echo '  install	- Install to (DESTDIR)$(PREFIX)'
	@echo '  uninstall	- Uninstall from (DESTDIR)$(PREFIX)'
	@echo '  check		- Run the test suite'
	@echo '  bloatcheck	- Show size comparison'
