#!/usr/bin/env bash
# build.sh — VidCut build script for MSYS2 MinGW64
# Usage:
#   ./scripts/build.sh              # Debug build
#   ./scripts/build.sh release      # Release build
#   ./scripts/build.sh clean        # Wipe build dir
#   ./scripts/build.sh run          # Build + run
#   ./scripts/build.sh deploy       # Copy Qt DLLs next to exe (first-time setup)
#   ./scripts/build.sh ffmpeg       # Build with FFmpeg enabled

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$ROOT_DIR/build"
EXE="$BUILD_DIR/src/VidCut.exe"

# Locate MSYS2 MinGW64 — try common install locations
if   [ -d "/c/msys64/mingw64/bin" ]; then MINGW64="/c/msys64/mingw64/bin"
elif [ -d "/mingw64/bin"           ]; then MINGW64="/mingw64/bin"
else
    echo "ERROR: Cannot find MSYS2 MinGW64. Install from https://www.msys2.org/"
    exit 1
fi

export PATH="$MINGW64:$PATH"

# --- Parse args ---
MODE="${1:-debug}"
BUILD_TYPE="Debug"
FFMPEG_ENABLED="OFF"

case "$MODE" in
    release) BUILD_TYPE="Release" ;;
    ffmpeg)  FFMPEG_ENABLED="ON"  ;;
    clean)
        echo ">>> Cleaning $BUILD_DIR ..."
        rm -rf "$BUILD_DIR"
        echo ">>> Done."
        exit 0
        ;;
    run | deploy | debug | "") ;;
    *)
        echo "Usage: $0 [debug|release|clean|run|deploy|ffmpeg]"
        exit 1
        ;;
esac

echo "========================================"
echo "  VidCut Build"
echo "  Mode:    $BUILD_TYPE"
echo "  FFmpeg:  $FFMPEG_ENABLED"
echo "  Root:    $ROOT_DIR"
echo "========================================"

# --- Configure (only when no build.ninja yet) ---
if [ ! -f "$BUILD_DIR/build.ninja" ]; then
    echo ""
    echo ">>> Configuring CMake..."
    cmake -B "$BUILD_DIR" -S "$ROOT_DIR" \
        -G Ninja \
        -DCMAKE_BUILD_TYPE="$BUILD_TYPE" \
        -DCMAKE_PREFIX_PATH="$(dirname "$MINGW64")" \
        -DVIDCUT_ENABLE_FFMPEG="$FFMPEG_ENABLED"
    echo ">>> Configure done."
fi

# --- Build ---
echo ""
echo ">>> Building..."
ninja -C "$BUILD_DIR"
echo ">>> Build done."

# --- Sign exe (Device Guard) ---
_sign_exe() {
    THUMBPRINT_FILE="$SCRIPT_DIR/dev-cert-thumbprint.txt"

    # First run: auto-setup cert (triggers UAC)
    if [ ! -f "$THUMBPRINT_FILE" ]; then
        echo ""
        echo ">>> Device Guard requires a signed exe. Setting up dev cert (UAC prompt will appear)..."
        SETUP_PS1="$(cygpath -w "$SCRIPT_DIR/setup-devcert.ps1")"
        powershell.exe -NoProfile -Command \
            "Start-Process powershell -ArgumentList '-NoProfile -ExecutionPolicy Bypass -File \"$SETUP_PS1\"' -Verb RunAs -Wait"
        if [ ! -f "$THUMBPRINT_FILE" ]; then
            echo "ERROR: Cert setup failed or was cancelled. Device Guard will block unsigned exe."
            exit 1
        fi
        echo ">>> Cert setup done."
    fi

    THUMB=$(cat "$THUMBPRINT_FILE" | tr -d '[:space:]')
    EXE_WIN="$(cygpath -w "$EXE")"
    echo ""
    echo ">>> Signing $EXE ..."

    # Use PowerShell Set-AuthenticodeSignature — looks in CurrentUser\My for private key
    powershell.exe -NoProfile -Command "
        \$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { \$_.Thumbprint -eq '$THUMB' }
        if (-not \$cert) {
            Write-Error 'Dev cert not found in CurrentUser\My. Delete dev-cert-thumbprint.txt and re-run build to trigger setup again.'
            exit 1
        }
        \$result = Set-AuthenticodeSignature -FilePath '$EXE_WIN' -Certificate \$cert -HashAlgorithm SHA256
        if (\$result.Status -eq 'Valid') {
            Write-Host '>>> Signed OK.'
        } else {
            Write-Warning \"Signing result: \$(\$result.Status) — \$(\$result.StatusMessage)\"
            exit 1
        }
    "
}

_sign_exe

# --- Deploy: copy Qt + MinGW DLLs next to exe ---
_deploy() {
    EXE_DIR="$(dirname "$EXE")"
    echo ""
    echo ">>> Deploying DLLs to $EXE_DIR ..."

    # windeployqt6 copies Qt DLLs
    WINDEPLOYQT="$MINGW64/windeployqt6.exe"
    if [ -f "$WINDEPLOYQT" ]; then
        "$WINDEPLOYQT" "$EXE" --no-translations --no-system-d3d-compiler
    else
        echo "WARNING: windeployqt6 not found, copying DLLs manually..."
    fi

    # Always copy essential MinGW runtime DLLs
    for dll in \
        Qt6Core.dll Qt6Gui.dll Qt6Widgets.dll Qt6OpenGL.dll \
        Qt6OpenGLWidgets.dll Qt6Multimedia.dll Qt6Network.dll \
        libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll \
        platforms/qwindows.dll
    do
        src="$MINGW64/$dll"
        dst="$EXE_DIR/$dll"
        dst_dir="$(dirname "$dst")"
        [ -d "$dst_dir" ] || mkdir -p "$dst_dir"
        if [ -f "$src" ]; then
            cp -u "$src" "$dst" && echo "  copied: $dll"
        fi
    done

    # Qt platform plugin (mandatory)
    PLATFORM_DIR="$EXE_DIR/platforms"
    mkdir -p "$PLATFORM_DIR"
    QT_PLATFORM="$(dirname "$MINGW64")/share/qt6/plugins/platforms/qwindows.dll"
    [ -f "$QT_PLATFORM" ] && cp -u "$QT_PLATFORM" "$PLATFORM_DIR/" && echo "  copied: platforms/qwindows.dll"

    echo ">>> Deploy done."
}

if [ "$MODE" = "deploy" ]; then
    _deploy
    exit 0
fi

# --- Run ---
if [ "$MODE" = "run" ]; then
    # Auto-deploy DLLs if not present yet
    if [ ! -f "$(dirname "$EXE")/Qt6Widgets.dll" ]; then
        echo ""
        echo ">>> Qt DLLs missing — running deploy first..."
        _deploy
    fi

    echo ""
    echo ">>> Launching VidCut..."
    cmd //c "$(cygpath -w "$EXE")"
else
    echo ""
    echo ">>> Executable: $EXE"
    echo ">>> First run:  ./scripts/build.sh deploy && ./scripts/build.sh run"
    echo ">>> Quick run:  ./scripts/build.sh run   (auto-deploys if needed)"
fi
