@echo off
setlocal enabledelayedexpansion

echo Building yutto executable...
echo.

REM 创建虚拟环境
echo [1/4] Creating virtual environment...
if not exist venv (
    python -m venv venv
    if errorlevel 1 (
        echo ERROR: Failed to create virtual environment
        exit /b 1
    )
)
echo Virtual environment ready.
echo.

REM 安装依赖
echo [2/4] Installing dependencies...
venv\Scripts\pip.exe install yutto pyinstaller
if errorlevel 1 (
    echo ERROR: Failed to install dependencies
    exit /b 1
)
echo Dependencies installed.
echo.

REM 构建
echo [3/4] Building executable with PyInstaller...
venv\Scripts\pyinstaller.exe yutto.spec --clean
if errorlevel 1 (
    echo ERROR: PyInstaller build failed
    exit /b 1
)
echo Build completed.
echo.

REM 复制到 binaries 目录（使用 Tauri 期望的命名约定）
echo [4/4] Copying to binaries directory...
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
