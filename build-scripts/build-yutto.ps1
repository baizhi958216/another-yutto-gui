# PowerShell script to build yutto executable from local source
# Encoding: UTF-8

Write-Host "Building yutto executable from local source..." -ForegroundColor Cyan
Write-Host ""

# ============================================
# 阶段 1: 创建并验证虚拟环境
# ============================================
Write-Host "[1/5] Creating virtual environment..." -ForegroundColor Yellow

# 如果 venv 不存在，创建它
if (-not (Test-Path "venv")) {
    Write-Host "Creating new virtual environment..."
    python -m venv venv
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to create virtual environment" -ForegroundColor Red
        exit 1
    }
    Write-Host "Virtual environment created." -ForegroundColor Green
} else {
    Write-Host "Virtual environment already exists." -ForegroundColor Green
}

# 验证虚拟环境是否可用
Write-Host "Verifying virtual environment..."
if (-not (Test-Path "venv\Scripts\python.exe")) {
    Write-Host "ERROR: Virtual environment is invalid - python.exe not found" -ForegroundColor Red
    Write-Host "Please delete the venv folder and try again." -ForegroundColor Red
    exit 1
}

if (-not (Test-Path "venv\Scripts\pip.exe")) {
    Write-Host "ERROR: Virtual environment is invalid - pip.exe not found" -ForegroundColor Red
    Write-Host "Please delete the venv folder and try again." -ForegroundColor Red
    exit 1
}

Write-Host "Virtual environment is ready." -ForegroundColor Green
Write-Host ""

# ============================================
# 阶段 2: 安装构建工具和 PyInstaller
# ============================================
Write-Host "[2/5] Installing build tools..." -ForegroundColor Yellow
& venv\Scripts\python.exe -m pip install --upgrade pip
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to upgrade pip" -ForegroundColor Red
    exit 1
}

& venv\Scripts\pip.exe install --upgrade setuptools wheel pyinstaller
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to install build tools" -ForegroundColor Red
    exit 1
}
Write-Host "Build tools installed." -ForegroundColor Green
Write-Host ""

# ============================================
# 阶段 3: 从本地源码安装 yutto
# ============================================
Write-Host "[3/5] Installing yutto from local source (..\yutto-main)..." -ForegroundColor Yellow
if (-not (Test-Path "..\yutto-main")) {
    Write-Host "ERROR: ..\yutto-main directory not found!" -ForegroundColor Red
    Write-Host "Please ensure yutto-main source code is in the parent directory." -ForegroundColor Red
    exit 1
}

& venv\Scripts\pip.exe install ..\yutto-main
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to install yutto from local source" -ForegroundColor Red
    exit 1
}
Write-Host "Yutto installed from local source." -ForegroundColor Green
Write-Host ""

# ============================================
# 阶段 4: 构建可执行文件
# ============================================
Write-Host "[4/5] Building executable with PyInstaller..." -ForegroundColor Yellow
& venv\Scripts\pyinstaller.exe yutto.spec --clean
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: PyInstaller build failed" -ForegroundColor Red
    exit 1
}
Write-Host "Build completed." -ForegroundColor Green
Write-Host ""

# ============================================
# 阶段 5: 复制到 binaries 目录
# ============================================
Write-Host "[5/5] Copying to binaries directory..." -ForegroundColor Yellow
$TARGET_NAME = "yutto-x86_64-pc-windows-msvc.exe"
if (-not (Test-Path "..\binaries")) {
    New-Item -ItemType Directory -Path "..\binaries" | Out-Null
}
Copy-Item -Path "dist\yutto.exe" -Destination "..\binaries\$TARGET_NAME" -Force
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to copy executable" -ForegroundColor Red
    exit 1
}
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Build complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Executable: dist\yutto.exe"
Get-Item "dist\yutto.exe" | Format-Table Mode, LastWriteTime, Length, Name -AutoSize
Write-Host ""
Write-Host "Copied to: ..\binaries\$TARGET_NAME"
Write-Host ""
Write-Host "Testing executable..." -ForegroundColor Yellow
& "..\binaries\$TARGET_NAME" --version
