@echo off
setlocal enabledelayedexpansion

echo Building yutto executable from local source...
echo.

REM 创建虚拟环境
echo [1/5] Creating virtual environment...
if not exist venv (
    python -m venv venv
    if errorlevel 1 (
        echo ERROR: Failed to create virtual environment
        exit /b 1
    )
)
echo Virtual environment ready.
echo.

REM 安装构建工具和 PyInstaller
echo [2/5] Installing build tools...
REM 使用 python -m pip 来升级 pip 自身
venv\Scripts\python.exe -m pip install --upgrade pip
venv\Scripts\pip.exe install --upgrade setuptools wheel pyinstaller
if errorlevel 1 (
    echo ERROR: Failed to install build tools
    exit /b 1
)
echo Build tools installed.
echo.

REM 从本地源码安装 yutto
echo [3/5] Installing yutto from local source (..\yutto-main)...
if not exist ..\yutto-main (
    echo ERROR: ..\yutto-main directory not found!
    echo Please ensure yutto-main source code is in the parent directory.
    exit /b 1
)

venv\Scripts\pip.exe install ..\yutto-main
if errorlevel 1 (
    echo ERROR: Failed to install yutto from local source
    exit /b 1
)
echo Yutto installed from local source.
echo.

REM 构建
echo [4/5] Building executable with PyInstaller...
venv\Scripts\pyinstaller.exe yutto.spec --clean
if errorlevel 1 (
    echo ERROR: PyInstaller build failed
    exit /b 1
)
echo Build completed.
echo.

REM 复制到 binaries 目录（使用 Tauri 期望的命名约定）
echo [5/5] Copying to binaries directory...
set TARGET_NAME=yutto-x86_64-pc-windows-msvc.exe
if not exist ..\binaries mkdir ..\binaries
copy /Y dist\yutto.exe ..\binaries\%TARGET_NAME%
if errorlevel 1 (
    echo ERROR: Failed to copy executable
    exit /b 1
)
echo.

echo ========================================
echo Build complete!
echo ========================================
echo Executable: dist\yutto.exe
dir dist\yutto.exe
echo.
echo Copied to: ..\binaries\%TARGET_NAME%
echo.
echo Testing executable...
..\binaries\%TARGET_NAME% --version
