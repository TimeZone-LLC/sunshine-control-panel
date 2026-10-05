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
    Push-Location $projectRoot
    npm run build:renderer
    if ($LASTEXITCODE -ne 0) {
        Pop-Location
        Write-Host "Renderer build failed." -ForegroundColor Red
        exit 1
    }
    Pop-Location

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

$exeDest = Join-Path $guiDir 'sunshine-gui.exe'
$pluginDest = Join-Path $guiDir 'alkaidlab-plugin-stylus.dll'
$transactionId = [Guid]::NewGuid().ToString('N')
$exeStage = Join-Path $guiDir ".sunshine-gui.$transactionId.new"
$pluginStage = Join-Path $guiDir ".alkaidlab-plugin-stylus.$transactionId.new"
$exeBackup = Join-Path $guiDir ".sunshine-gui.$transactionId.bak"
$pluginBackup = Join-Path $guiDir ".alkaidlab-plugin-stylus.$transactionId.bak"
$destinationRoot = [IO.Path]::GetFullPath($guiDir).TrimEnd('\') + '\'
$transactionPaths = @($exeDest, $pluginDest, $exeStage, $pluginStage, $exeBackup, $pluginBackup)
foreach ($path in $transactionPaths) {
    if (-not [IO.Path]::GetFullPath($path).StartsWith($destinationRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Deployment path escaped the GUI directory: $path"
    }
}

$hadExe = Test-Path -LiteralPath $exeDest -PathType Leaf
$hadPlugin = Test-Path -LiteralPath $pluginDest -PathType Leaf
$exeBackedUp = $false
$pluginBackedUp = $false
$exeActivated = $false
$pluginActivated = $false
$deploymentCommitted = $false
Write-Host "Copying files to $guiDir ..." -ForegroundColor Gray
try {
    Copy-Item -LiteralPath $exeSrc -Destination $exeStage
    Copy-Item -LiteralPath $pluginSrc -Destination $pluginStage

    if ($hadExe) {
        Move-Item -LiteralPath $exeDest -Destination $exeBackup
        $exeBackedUp = $true
    }
    if ($hadPlugin) {
        Move-Item -LiteralPath $pluginDest -Destination $pluginBackup
        $pluginBackedUp = $true
    }

    Move-Item -LiteralPath $exeStage -Destination $exeDest
    $exeActivated = $true
    Move-Item -LiteralPath $pluginStage -Destination $pluginDest
    $pluginActivated = $true
    $deploymentCommitted = $true
} catch {
    $deploymentError = $_
    try {
        if ($exeActivated -and (Test-Path -LiteralPath $exeDest -PathType Leaf)) {
            Remove-Item -LiteralPath $exeDest -Force
        }
        if ($pluginActivated -and (Test-Path -LiteralPath $pluginDest -PathType Leaf)) {
            Remove-Item -LiteralPath $pluginDest -Force
        }
        if ($exeBackedUp -and (Test-Path -LiteralPath $exeBackup -PathType Leaf)) {
            Move-Item -LiteralPath $exeBackup -Destination $exeDest
        }
        if ($pluginBackedUp -and (Test-Path -LiteralPath $pluginBackup -PathType Leaf)) {
            Move-Item -LiteralPath $pluginBackup -Destination $pluginDest
        }
    } catch {
        Write-Host "Deployment rollback failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    throw $deploymentError
} finally {
    foreach ($temporaryPath in @($exeStage, $pluginStage)) {
        if (Test-Path -LiteralPath $temporaryPath -PathType Leaf) {
            Remove-Item -LiteralPath $temporaryPath -Force
        }
    }
    if ($deploymentCommitted) {
        foreach ($backupPath in @($exeBackup, $pluginBackup)) {
            if (Test-Path -LiteralPath $backupPath -PathType Leaf) {
                Remove-Item -LiteralPath $backupPath -Force
            }
        }
    }
}

# WebView2Loader.dll (如果存在)
$loaderPath = Split-Path $exeSrc -Parent | Join-Path -ChildPath "WebView2Loader.dll"
if (Test-Path $loaderPath) {
    Copy-Item $loaderPath "$guiDir\WebView2Loader.dll" -Force
    Write-Host "   + WebView2Loader.dll"
}

Write-Host "`nGUI deployed to $guiDir"
Write-Host "   sunshine-gui.exe ($exeSize MB)"
