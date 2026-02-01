#!/bin/bash
set -e

echo "Building yutto executable from local source..."
echo ""

# 创建虚拟环境
echo "[1/5] Creating virtual environment..."
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi
echo "Virtual environment ready."
echo ""

# 安装构建工具和 PyInstaller
echo "[2/5] Installing build tools..."
venv/bin/pip install --upgrade pip setuptools wheel pyinstaller
echo "Build tools installed."
echo ""

# 从本地源码安装 yutto
echo "[3/5] Installing yutto from local source (../yutto-main)..."
if [ ! -d "../yutto-main" ]; then
    echo "Error: ../yutto-main directory not found!"
    echo "Please ensure yutto-main source code is in the parent directory."
    exit 1
fi

# 安装 yutto 及其依赖（使用可编辑模式以便调试，或直接安装）
venv/bin/pip install ../yutto-main
echo "Yutto installed from local source."
echo ""

# 构建
echo "[4/5] Building executable with PyInstaller..."
venv/bin/pyinstaller yutto.spec --clean
echo "Build completed."
echo ""

# 检测平台并复制到对应目录（使用 Tauri 期望的命名约定）
echo "[5/5] Copying to binaries directory..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    ARCH=$(uname -m)
    if [ "$ARCH" = "arm64" ]; then
        TARGET_NAME="yutto-aarch64-apple-darwin"
    else
        TARGET_NAME="yutto-x86_64-apple-darwin"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    TARGET_NAME="yutto-x86_64-unknown-linux-gnu"
else
    echo "Unknown platform: $OSTYPE"
    exit 1
fi

cp dist/yutto "../binaries/$TARGET_NAME"
chmod +x "../binaries/$TARGET_NAME"
echo ""

echo "========================================"
echo "Build complete!"
echo "========================================"
echo "Executable: dist/yutto"
ls -lh dist/yutto
echo ""
echo "Copied to: ../binaries/$TARGET_NAME"
echo ""
echo "Testing executable..."
"../binaries/$TARGET_NAME" --version
