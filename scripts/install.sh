#!/usr/bin/env bash
set -euo pipefail

MORN_INSTALL_DIR="${INSTALL_DIR:-$HOME/.morn}"
MORN_VENV_DIR="$MORN_INSTALL_DIR/venv"
MORN_CONFIG_DIR="$HOME/.morn"

detect_os() {
    case "$(uname -s)" in
        Linux)
            if grep -qi microsoft /proc/version 2>/dev/null; then
                echo "wsl2"
            elif command -v apt-get &>/dev/null; then
                echo "ubuntu"
            else
                echo "linux"
            fi
            ;;
        Darwin) echo "macos" ;;
        *) echo "unsupported" ;;
    esac
}

check_python() {
    if command -v python3 &>/dev/null; then
        local py_version
        py_version=$(python3 --version 2>&1 | grep -oP '\d+\.\d+')
        if awk "BEGIN {exit !($py_version >= 3.11)}"; then
            echo "python3"
            return 0
        fi
    fi
    echo ""
}

install_python_ubuntu() {
    echo ">>> Installing Python 3.11+ on Ubuntu..."
    sudo apt-get update -qq
    sudo apt-get install -y -qq python3 python3-pip python3-venv
}

install_python_macos() {
    echo ">>> Installing Python 3.11+ on macOS..."
    if ! command -v brew &>/dev/null; then
        echo ">>> Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    brew install python@3.12
}

setup_morn() {
    echo ">>> Creating venv at $MORN_VENV_DIR..."
    python3 -m venv "$MORN_VENV_DIR"
    "$MORN_VENV_DIR/bin/pip" install --quiet --upgrade pip
    echo ">>> Installing morn..."
    "$MORN_VENV_DIR/bin/pip" install --quiet "$MORN_INSTALL_DIR" 2>/dev/null || {
        local repo_dir
        repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
        "$MORN_VENV_DIR/bin/pip" install --quiet -e "$repo_dir"
    }
    mkdir -p "$MORN_CONFIG_DIR"
    if [ ! -f "$MORN_CONFIG_DIR/config.yaml" ]; then
        cat > "$MORN_CONFIG_DIR/config.yaml" << 'YAMLEOF'
instance: default
mode: hybrid
YAMLEOF
        echo ">>> Initialized ~/.morn/config.yaml"
    fi
    local bin_link="$HOME/.local/bin/morn"
    mkdir -p "$HOME/.local/bin"
    ln -sf "$MORN_VENV_DIR/bin/morn" "$bin_link"
    echo ">>> Linked morn to $bin_link"
    if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
        shell_rc="$HOME/.$(basename "$SHELL")rc"
        if [ "$SHELL" = "/bin/zsh" ] || [ "$SHELL" = "/usr/bin/zsh" ]; then
            shell_rc="$HOME/.zshrc"
        fi
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
        echo ">>> Added ~/.local/bin to PATH in $shell_rc"
    fi
}

main() {
    echo "=== Morn Installation ==="
    local os
    os=$(detect_os)
    echo ">>> Detected OS: $os"
    if [ "$os" = "unsupported" ]; then
        echo "Error: unsupported OS" >&2
        exit 1
    fi
    local py_cmd
    py_cmd=$(check_python)
    if [ -z "$py_cmd" ]; then
        case "$os" in
            ubuntu|wsl2) install_python_ubuntu ;;
            macos) install_python_macos ;;
            linux)
                echo "Please install Python 3.11+ manually and re-run this script."
                exit 1
                ;;
        esac
        py_cmd=$(check_python)
        if [ -z "$py_cmd" ]; then
            echo "Error: failed to install Python 3.11+" >&2
            exit 1
        fi
    fi
    echo ">>> Using $py_cmd at $(command -v "$py_cmd")"
    setup_morn
    echo ">>> Installation complete."
    if command -v morn &>/dev/null; then
        echo ">>> morn --version: $(morn --version)"
    else
        export PATH="$HOME/.local/bin:$PATH"
        echo ">>> Run 'source ~/.$(basename $SHELL)rc' or log out/in, then run 'morn --version'"
    fi
}

main "$@"
