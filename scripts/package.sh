#!/usr/bin/env bash
# Build zeroclash release and produce a platform-specific installer.
# Usage: bash scripts/package.sh
#
# macOS: produces dist/ZeroClash-{version}-{arch}.dmg
# Linux: produces dist/zeroclash_{version}_{arch}.deb + dist/ZeroClash-{version}-{arch}.AppImage
#
# Inspired by Zed's script/bundle-linux and script/bundle-mac.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
APP_DIR="$PROJECT_DIR/apps/zeroclash"
DIST_DIR="$PROJECT_DIR/dist"
CACHE_DIR="$PROJECT_DIR/.cache"

# ── Platform detection ──
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64)  ARCH_LABEL="amd64" ;;
    aarch64) ARCH_LABEL="arm64" ;;
    arm64)   ARCH_LABEL="arm64" ;;
    *)       ARCH_LABEL="$ARCH" ;;
esac

TARGET="$(rustc -vV | grep 'host:' | awk '{print $2}')"
BIN_DIR="$PROJECT_DIR/target/$TARGET/release"

# ── Version (cargo metadata approach, inspired by Zed's script/get-crate-version) ──
if [ -n "${APP_VERSION:-}" ]; then
    VERSION="$APP_VERSION"
else
    GIT_TAG="$(git describe --tags --abbrev=0 2>/dev/null || true)"
    if echo "$GIT_TAG" | grep -qE '^v[0-9]+\.[0-9]+\.[0-9]+'; then
        VERSION="${GIT_TAG#v}"
    else
        VERSION="$(cargo metadata --no-deps --format-version=1 2>/dev/null |
            sed -n 's/.*"name":"zeroclash","version":"\([^"]*\)".*/\1/p' |
            head -1)"
        if [ -z "$VERSION" ]; then
            VERSION="$(grep '^version = ' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')"
        fi
        echo "WARN: No git version tag found. Using Cargo.toml version: $VERSION (local/dev build)" >&2
    fi
fi

echo "[package] zeroclash v$VERSION for $TARGET ($OS $ARCH)"

# ── Step 1: Build ──
echo "[package] Building zeroclash --release..."
cargo build --release

# ── Step 2: Verify required files ──
for bin in zeroclash zeroclash-cli mihomo; do
    ext=""
    [ "$OS" = "Windows" ] && ext=".exe"
    if [ ! -f "$BIN_DIR/$bin$ext" ]; then
        echo "WARN: Missing: $BIN_DIR/$bin$ext — package may be incomplete" >&2
    fi
done

mkdir -p "$DIST_DIR"

# ════════════════════════════════════════════════════════════════════
# macOS: .app bundle → .dmg
# ════════════════════════════════════════════════════════════════════
if [ "$OS" = "Darwin" ]; then
    APP_BUNDLE="ZeroClash.app"
    CONTENTS="$APP_BUNDLE/Contents"

    echo "[package] Creating macOS .app bundle..."
    mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"

    cp "$BIN_DIR/zeroclash" "$CONTENTS/MacOS/"
    cp "$BIN_DIR/zeroclash-cli" "$CONTENTS/MacOS/"
    cp "$BIN_DIR/mihomo" "$CONTENTS/MacOS/"
    chmod +x "$CONTENTS/MacOS/"*

    sed "s/REPLACE_VERSION/$VERSION/g" "$APP_DIR/Info.plist" > "$CONTENTS/Info.plist"

    echo "[package] Creating macOS DMG..."
    mkdir -p dmg_staging
    cp -r "$APP_BUNDLE" dmg_staging/
    ln -sf /Applications dmg_staging/Applications

    DMG_NAME="ZeroClash-${VERSION}-${ARCH_LABEL}.dmg"
    hdiutil create \
        -volname "ZeroClash" \
        -srcfolder dmg_staging \
        -ov -format UDZO -fs HFS+ \
        "$DMG_NAME"

    mv "$DMG_NAME" "$DIST_DIR/"
    rm -rf "$APP_BUNDLE" dmg_staging
    echo "[package] Done. DMG: $DIST_DIR/$DMG_NAME"
fi

# ════════════════════════════════════════════════════════════════════
# Linux: .deb + AppImage
# ════════════════════════════════════════════════════════════════════
if [ "$OS" = "Linux" ]; then
    # ── .deb via cargo-deb ──
    echo "[package] Building .deb..."
    if ! cargo deb --help &>/dev/null 2>&1; then
        echo "[package] Installing cargo-deb..."
        cargo install cargo-deb
    fi

    # cargo-deb expects binaries at target/release/ relative to crate
    mkdir -p "$PROJECT_DIR/target/release"
    cp "$BIN_DIR/zeroclash" "$PROJECT_DIR/target/release/"
    cp "$BIN_DIR/zeroclash-cli" "$PROJECT_DIR/target/release/"
    cp "$BIN_DIR/mihomo" "$PROJECT_DIR/target/release/"

    cargo deb --no-build --target "$TARGET" -p zeroclash

    DEB_DIR="$PROJECT_DIR/target/$TARGET/debian"
    if ls "$DEB_DIR"/*.deb 1>/dev/null 2>&1; then
        mv "$DEB_DIR"/*.deb "$DIST_DIR/"
        echo "[package] .deb package(s) in $DIST_DIR/"
    else
        echo "WARN: No .deb files found in $DEB_DIR/" >&2
    fi

    # ── AppImage ──
    echo "[package] Building AppImage..."
    mkdir -p AppDir/usr/bin AppDir/usr/lib AppDir/usr/share/icons/hicolor/256x256/apps

    cp "$BIN_DIR/zeroclash" AppDir/usr/bin/
    cp "$BIN_DIR/zeroclash-cli" AppDir/usr/bin/
    cp "$BIN_DIR/mihomo" AppDir/usr/bin/
    chmod +x AppDir/usr/bin/*

    # Bundle shared libraries (inspired by Zed's find_libs / ldd approach)
    echo "[package] Bundling shared libraries..."
    if command -v ldd &>/dev/null; then
        for bin in AppDir/usr/bin/*; do
            ldd "$bin" 2>/dev/null | cut -d' ' -f3 | grep -vE '\<lib(c|m|dl|pthread|gcc_s)\.so' | sort -u | while read -r lib; do
                if [ -n "$lib" ] && [ -f "$lib" ]; then
                    cp -n "$lib" AppDir/usr/lib/ 2>/dev/null || true
                fi
            done
        done
    fi

    cp "$APP_DIR/zeroclash.desktop" AppDir/
    cp "$APP_DIR/AppRun" AppDir/
    chmod +x AppDir/AppRun

    # Minimal 1x1 PNG as placeholder icon
    printf '\x89PNG\x0d\x0a\x1a\x0a\x00\x00\x00\x0dIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\x0aIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\x0d\x0a\x2d\xb4\x00\x00\x00\x00IEND\xaeB\x60\x82' > AppDir/zeroclash.png
    cp AppDir/zeroclash.png AppDir/usr/share/icons/hicolor/256x256/apps/

    # Download appimagetool (cached)
    mkdir -p "$CACHE_DIR"
    case "$ARCH" in
        x86_64)  APPIMAGE_TOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/13/appimagetool-x86_64.AppImage" ;;
        aarch64) APPIMAGE_TOOL_URL="https://github.com/AppImage/AppImageKit/releases/download/13/appimagetool-aarch64.AppImage" ;;
        *)       echo "WARN: Unknown arch $ARCH for AppImage, skipping" >&2; rm -rf AppDir; continue ;;
    esac

    APPIMAGE_TOOL="$CACHE_DIR/appimagetool-${ARCH}.AppImage"
    if [ ! -f "$APPIMAGE_TOOL" ]; then
        echo "[package] Downloading appimagetool for $ARCH..."
        wget -q "$APPIMAGE_TOOL_URL" -O "$APPIMAGE_TOOL"
        chmod +x "$APPIMAGE_TOOL"
    fi

    APPIMAGE_NAME="ZeroClash-${VERSION}-${ARCH_LABEL}.AppImage"
    "$APPIMAGE_TOOL" --no-appstream AppDir "$APPIMAGE_NAME"
    mv "$APPIMAGE_NAME" "$DIST_DIR/"
    rm -rf AppDir
    echo "[package] Done. AppImage: $DIST_DIR/$APPIMAGE_NAME"
fi
