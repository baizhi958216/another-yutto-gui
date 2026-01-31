#!/bin/bash
set -e

echo "Building yutto executable..."
echo ""

# 创建虚拟环境
echo "[1/4] Creating virtual environment..."
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi
echo "Virtual environment ready."
echo ""

# 安装依赖
echo "[2/4] Installing dependencies..."
venv/bin/pip install yutto pyinstaller
echo "Dependencies installed."
echo ""

# 构建
echo "[3/4] Building executable with PyInstaller..."
venv/bin/pyinstaller yutto.spec --clean
echo "Build completed."
echo ""

# 检测平台并复制到对应目录（使用 Tauri 期望的命名约定）
echo "[4/4] Copying to binaries directory..."
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
