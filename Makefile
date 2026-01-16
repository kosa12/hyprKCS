# hyprKCS Makefile

APP_NAME = hyprkcs
PREFIX ?= /usr/local
BIN_DIR = $(PREFIX)/bin
SHARE_DIR = $(PREFIX)/share
APP_DIR = $(SHARE_DIR)/applications
ICON_DIR = $(SHARE_DIR)/icons/hicolor/scalable/apps

.PHONY: all build install uninstall clean check fmt

all: build

build:
	cargo build --release

check:
	cargo check

fmt:
	cargo fmt

clean:
	cargo clean

install: build
	install -Dm755 target/release/hyprKCS $(DESTDIR)$(BIN_DIR)/$(APP_NAME)
	install -Dm644 hyprkcs.desktop $(DESTDIR)$(APP_DIR)/$(APP_NAME).desktop
	install -Dm644 assets/icon.svg $(DESTDIR)$(ICON_DIR)/$(APP_NAME).svg

install-user: build
	# Install to ~/.local/bin and ~/.local/share/applications
	@make install PREFIX=$(HOME)/.local

uninstall:
	rm -f $(DESTDIR)$(BIN_DIR)/$(APP_NAME)
	rm -f $(DESTDIR)$(APP_DIR)/$(APP_NAME).desktop
	rm -f $(DESTDIR)$(ICON_DIR)/$(APP_NAME).svg
