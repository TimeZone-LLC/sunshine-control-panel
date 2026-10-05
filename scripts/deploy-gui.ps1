# deploy-gui.ps1 — 编译并部署 GUI 到 Sunshine 安装目录
# 用法: .\scripts\deploy-gui.ps1 [-Release] [-NoBuild]
param(
    [switch]$Release,
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"

# 定位 Sunshine 安装目录
$sunshineDir = $null
$candidates = @(
    "C:\Program Files\Sunshine",
    "C:\Program Files (x86)\Sunshine",
    "${env:ProgramFiles}\Sunshine"
)
foreach ($dir in $candidates) {
    if (Test-Path "$dir\sunshine.exe") {
        $sunshineDir = $dir
        break
    }
}

if (-not $sunshineDir) {
    # 尝试从注册表读取
    try {
        $regPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Sunshine_is1"
        if (Test-Path $regPath) {
            $sunshineDir = (Get-ItemProperty $regPath).InstallLocation.TrimEnd('\')
        }
    } catch {}
}

if (-not $sunshineDir) {
    Write-Host "Sunshine installation directory was not found." -ForegroundColor Red
    exit 1
}

$guiDir = Join-Path $sunshineDir "assets\gui"
Write-Host "Sunshine directory: $sunshineDir" -ForegroundColor DarkGray
Write-Host "GUI destination: $guiDir" -ForegroundColor DarkGray

# 编译
$projectRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
# 如果是从 sunshine-control-panel/scripts 运行，projectRoot 就是 sunshine-control-panel
if (-not (Test-Path "$projectRoot\src-tauri\Cargo.toml")) {
    $projectRoot = Split-Path -Parent $PSScriptRoot
}
if (-not (Test-Path "$projectRoot\src-tauri\Cargo.toml")) {
    # 直接定位到 sunshine-control-panel
    $projectRoot = $PSScriptRoot | Split-Path -Parent
}

if (-not $NoBuild) {
    Write-Host "`nBuilding the Sunshine GUI..." -ForegroundColor Gray
    & (Join-Path $projectRoot 'scripts\build-native-plugins.ps1') -Configuration $(if ($Release) { 'Release' } else { 'Debug' })
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Native tool plugin build failed." -ForegroundColor Red
        exit 1
    }
    Push-Location "$projectRoot\src-tauri"

    if ($Release) {
        cargo build --release
    } else {
        cargo build
    }

    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Host "GUI build failed." -ForegroundColor Red
        exit 1
    }
    Pop-Location
}

# 定位编译产物
$profile = if ($Release) { "release" } else { "debug" }
$exePaths = @(
    "$projectRoot\src-tauri\target\$profile\sunshine-gui.exe",
    "$projectRoot\src-tauri\target\x86_64-pc-windows-msvc\$profile\sunshine-gui.exe"
)

$exeSrc = $null
foreach ($p in $exePaths) {
    if (Test-Path $p) {
        $exeSrc = $p
        break
    }
}

if (-not $exeSrc) {
    Write-Host "Build output sunshine-gui.exe was not found." -ForegroundColor Red
    exit 1
}

Write-Host "Build output: $exeSrc" -ForegroundColor DarkGray
$exeSize = [math]::Round((Get-Item $exeSrc).Length / 1MB, 1)
Write-Host "   Size: ${exeSize} MB"

$pluginSrc = Join-Path (Split-Path $exeSrc -Parent) 'alkaidlab-plugin-stylus.dll'
if (-not (Test-Path -LiteralPath $pluginSrc -PathType Leaf)) {
    Write-Host "Native tool plugin alkaidlab-plugin-stylus.dll was not found." -ForegroundColor Red
    exit 1
}

# 部署
if (-not (Test-Path $guiDir)) {
    New-Item -ItemType Directory -Path $guiDir -Force | Out-Null
}

# 先停止正在运行的 GUI
$running = Get-Process -Name "sunshine-gui" -ErrorAction SilentlyContinue
if ($running) {
    Write-Host "Stopping the running sunshine-gui process..." -ForegroundColor Gray
    $running | Stop-Process -Force
    Start-Sleep -Seconds 1
}

Write-Host "Copying files to $guiDir ..." -ForegroundColor Gray
Copy-Item $exeSrc "$guiDir\sunshine-gui.exe" -Force
Copy-Item -LiteralPath $pluginSrc -Destination "$guiDir\alkaidlab-plugin-stylus.dll" -Force

# WebView2Loader.dll (如果存在)
$loaderPath = Split-Path $exeSrc -Parent | Join-Path -ChildPath "WebView2Loader.dll"
if (Test-Path $loaderPath) {
    Copy-Item $loaderPath "$guiDir\WebView2Loader.dll" -Force
    Write-Host "   + WebView2Loader.dll"
}

Write-Host "`nGUI deployed to $guiDir"
Write-Host "   sunshine-gui.exe ($exeSize MB)"
