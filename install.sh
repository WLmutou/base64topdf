#!/usr/bin/env bash
#
# Base64 转图片 —— Linux 安装脚本
#
# 支持 Ubuntu / Debian（apt）与 CentOS / RedHat / Fedora（dnf / yum）。
# 自动安装构建依赖、构建 release 版本，并安装可执行文件、桌面入口与图标。
#
# 用法：
#   ./install.sh                 # 安装到 /usr/local（需要时自动调用 sudo）
#   ./install.sh --user          # 安装到 ~/.local（当前用户，无需 root）
#   ./install.sh --prefix /opt   # 自定义安装前缀
#   ./install.sh --no-deps       # 跳过系统依赖安装
#   ./install.sh --uninstall     # 卸载
#
set -euo pipefail

APP_ID="base64topdf"
APP_NAME="Base64 转图片"
ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

MODE="system"
PREFIX=""
INSTALL_DEPS=1
UNINSTALL=0

usage() {
    cat <<'EOF'
用法：./install.sh [选项]

选项：
  --user             安装到当前用户目录（~/.local），无需 root
  --prefix <路径>    自定义安装前缀
                     系统级默认 /usr/local，用户级默认 ~/.local
  --no-deps          跳过系统依赖安装
  --uninstall        卸载已安装的文件
  -h, --help         显示帮助

支持的发行版：
  Ubuntu / Debian 系（apt-get）
  CentOS / RedHat / Fedora 系（dnf / yum / microdnf）
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --user)
            MODE="user"
            shift
            ;;
        --prefix)
            if [[ $# -lt 2 ]]; then
                echo "错误：--prefix 需要一个路径参数" >&2
                exit 1
            fi
            PREFIX="$2"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --no-deps)
            INSTALL_DEPS=0
            shift
            ;;
        --uninstall)
            UNINSTALL=1
            shift
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            echo "错误：未知参数 $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [[ -z "$PREFIX" ]]; then
    if [[ "$MODE" == "user" ]]; then
        PREFIX="$HOME/.local"
    else
        PREFIX="/usr/local"
    fi
fi

case "$PREFIX" in
    /*) ;;
    *) PREFIX="$PWD/$PREFIX" ;;
esac
PREFIX="${PREFIX%/}"

BIN_DIR="$PREFIX/bin"
APPS_DIR="$PREFIX/share/applications"
ICON_DIR="$PREFIX/share/icons/hicolor/scalable/apps"
ICON_THEME_DIR="$PREFIX/share/icons/hicolor"

SUDO=""
if [[ "$MODE" == "system" && $EUID -ne 0 ]]; then
    if command -v sudo >/dev/null 2>&1; then
        SUDO="sudo"
    else
        echo "错误：系统级安装需要 root 权限，请使用 sudo 运行，或加 --user 安装到用户目录。" >&2
        exit 1
    fi
fi

# 安装系统依赖时使用的提权命令（与 --user 无关）
PKG_SUDO=""
if [[ $EUID -ne 0 ]] && command -v sudo >/dev/null 2>&1; then
    PKG_SUDO="sudo"
fi

run_priv() {
    if [[ -n "$SUDO" ]]; then
        "$SUDO" "$@"
    else
        "$@"
    fi
}

run_root() {
    if [[ -n "$PKG_SUDO" ]]; then
        "$PKG_SUDO" "$@"
    else
        "$@"
    fi
}

refresh_caches() {
    if command -v update-desktop-database >/dev/null 2>&1; then
        run_priv update-desktop-database "$APPS_DIR" >/dev/null 2>&1 || true
    fi
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        run_priv gtk-update-icon-cache -f -t "$ICON_THEME_DIR" >/dev/null 2>&1 || true
    fi
}

detect_package_manager() {
    if command -v apt-get >/dev/null 2>&1; then
        echo "apt"
    elif command -v dnf >/dev/null 2>&1; then
        echo "dnf"
    elif command -v yum >/dev/null 2>&1; then
        echo "yum"
    elif command -v microdnf >/dev/null 2>&1; then
        echo "microdnf"
    else
        echo "unknown"
    fi
}

install_dependencies() {
    local pm="$1"

    if [[ $EUID -ne 0 && -z "$PKG_SUDO" ]]; then
        echo "错误：安装系统依赖需要 root 权限，请使用 sudo 运行，或加 --no-deps 跳过。" >&2
        exit 1
    fi

    case "$pm" in
        apt)
            echo "正在通过 apt 安装系统依赖……"
            run_root apt-get update
            run_root env DEBIAN_FRONTEND=noninteractive \
                apt-get install -y --no-install-recommends \
                build-essential pkg-config curl ca-certificates \
                libgtk-3-dev libxkbcommon-dev libwayland-dev libx11-dev
            ;;
        dnf | microdnf)
            echo "正在通过 $pm 安装系统依赖……"
            run_root "$pm" install -y \
                gcc gcc-c++ make pkgconf-pkg-config curl ca-certificates \
                gtk3-devel libxkbcommon-devel libX11-devel wayland-devel
            ;;
        yum)
            echo "正在通过 yum 安装系统依赖……"
            run_root yum install -y \
                gcc gcc-c++ make pkgconfig curl ca-certificates \
                gtk3-devel libxkbcommon-devel libX11-devel wayland-devel
            ;;
        unknown)
            echo "错误：无法识别包管理器，请手动安装 GTK3、X11、Wayland 开发库与 Rust 工具链，" >&2
            echo "      或使用 --no-deps 跳过此步骤。" >&2
            exit 1
            ;;
    esac
}

ensure_fonts() {
    local zh_fonts=""

    if command -v fc-list >/dev/null 2>&1; then
        zh_fonts="$(fc-list :lang=zh 2>/dev/null || true)"
    fi

    if [[ -n "$zh_fonts" ]]; then
        return
    fi

    echo "正在安装中文字体……"
    case "$PM" in
        apt)
            run_root env DEBIAN_FRONTEND=noninteractive \
                apt-get install -y --no-install-recommends fonts-noto-cjk ||
                echo "提示：中文字体安装失败，请手动安装 fonts-noto-cjk。" >&2
            ;;
        dnf | microdnf)
            run_root "$PM" install -y google-noto-sans-cjk-fonts ||
                echo "提示：中文字体安装失败，请手动安装 google-noto-sans-cjk-fonts。" >&2
            ;;
        yum)
            run_root yum install -y google-noto-sans-cjk-fonts ||
                echo "提示：中文字体安装失败，请手动安装 google-noto-sans-cjk-fonts。" >&2
            ;;
        unknown) ;;
    esac
}

ensure_rust() {
    if command -v cargo >/dev/null 2>&1; then
        return
    fi

    echo "未检测到 Rust 工具链，正在通过 rustup 安装……"
    if ! command -v curl >/dev/null 2>&1; then
        echo "错误：缺少 curl，无法安装 rustup。" >&2
        exit 1
    fi

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
        sh -s -- -y --profile minimal

    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
    fi
    export PATH="$HOME/.cargo/bin:$PATH"

    if ! command -v cargo >/dev/null 2>&1; then
        echo "错误：Rust 工具链安装失败，请手动安装后重试。" >&2
        exit 1
    fi
}

if [[ $UNINSTALL -eq 1 ]]; then
    echo "正在卸载 $APP_NAME ……"
    run_priv rm -f \
        "$BIN_DIR/$APP_ID" \
        "$APPS_DIR/$APP_ID.desktop" \
        "$ICON_DIR/$APP_ID.svg"
    refresh_caches
    echo "卸载完成。"
    exit 0
fi

PM="$(detect_package_manager)"

if [[ $INSTALL_DEPS -eq 1 ]]; then
    if command -v pkg-config >/dev/null 2>&1 &&
        pkg-config --exists gtk+-3.0 &&
        command -v curl >/dev/null 2>&1; then
        echo "已检测到 GTK3 开发库，跳过系统依赖安装。"
    else
        install_dependencies "$PM"
    fi

    ensure_fonts
fi

ensure_rust

echo "正在构建 release 版本（首次构建可能需要几分钟）……"
cd "$ROOT_DIR"

if [[ -f Cargo.lock ]]; then
    cargo build --release --locked
else
    cargo build --release
fi

if [[ ! -f "$ROOT_DIR/target/release/$APP_ID" ]]; then
    echo "错误：构建失败，未找到 target/release/$APP_ID" >&2
    exit 1
fi

if [[ ! -f "$ROOT_DIR/assets/$APP_ID.svg" ]]; then
    echo "错误：缺少图标文件 assets/$APP_ID.svg" >&2
    exit 1
fi

echo "正在安装到 $PREFIX ……"
run_priv install -Dm755 "$ROOT_DIR/target/release/$APP_ID" "$BIN_DIR/$APP_ID"
run_priv install -Dm644 "$ROOT_DIR/assets/$APP_ID.svg" "$ICON_DIR/$APP_ID.svg"

DESKTOP_FILE="$(mktemp)"
trap 'rm -f "$DESKTOP_FILE"' EXIT

cat >"$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Version=1.0
Name=$APP_NAME
Name[en]=Base64 to Image
GenericName=Base64 Image Converter
Comment=将 Base64 字符串解码为 PNG / JPG / JPEG 图片
Exec="$BIN_DIR/$APP_ID"
TryExec=$BIN_DIR/$APP_ID
Icon=$APP_ID
Terminal=false
Categories=Graphics;
Keywords=base64;image;png;jpg;jpeg;convert;
StartupNotify=true
StartupWMClass=$APP_ID
EOF

run_priv install -Dm644 "$DESKTOP_FILE" "$APPS_DIR/$APP_ID.desktop"
refresh_caches

echo
echo "安装完成："
echo "  可执行文件：$BIN_DIR/$APP_ID"
echo "  桌面入口：  $APPS_DIR/$APP_ID.desktop"
echo "  图标：      $ICON_DIR/$APP_ID.svg"
echo
echo "启动命令：$APP_ID"

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo
    echo "提示：$BIN_DIR 不在当前 PATH 中，可将其加入 PATH："
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
fi
