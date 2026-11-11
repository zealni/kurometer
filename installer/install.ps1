param([switch]$Uninstall,[switch]$RemoveSettings)
$ErrorActionPreference = 'Stop'
$destination = Join-Path $env:LOCALAPPDATA 'Programs\TaskbarSystemMonitor'
$exe = Join-Path $destination 'Kurmon.exe'
$shortcutPath = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Kurmon.lnk'
$shell = New-Object -ComObject WScript.Shell
if ($Uninstall) {
    $running = Get-Process 'Kurmon' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }
    if ($running) { throw 'Exit Kurmon from its tray menu before uninstalling.' }
    Remove-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name TaskbarSystemMonitor -ErrorAction SilentlyContinue
    if (Test-Path $destination -PathType Container) {
        foreach ($path in @($exe, (Join-Path $destination 'taskbar-system-monitor.exe'), (Join-Path $destination 'install.ps1'))) {
            if (Test-Path -LiteralPath $path -PathType Leaf) { Remove-Item -LiteralPath $path -Force }
        }
        if (!(Get-ChildItem -LiteralPath $destination -Force)) { [System.IO.Directory]::Delete($destination) }
    }
    if ((Test-Path $shortcutPath) -and $shell.CreateShortcut($shortcutPath).TargetPath -eq $exe) { Remove-Item $shortcutPath }
    if ($RemoveSettings) {
        $settingsDir = Join-Path $env:LOCALAPPDATA 'TaskbarSystemMonitor'
        if (Test-Path $settingsDir -PathType Container) {
            foreach ($name in @('settings.ini', 'settings.tmp')) {
                $path = Join-Path $settingsDir $name
                if (Test-Path -LiteralPath $path -PathType Leaf) { Remove-Item -LiteralPath $path -Force }
            }
            if (!(Get-ChildItem -LiteralPath $settingsDir -Force)) {
                [System.IO.Directory]::Delete($settingsDir)
            }
        }
    }
    Write-Output 'Uninstalled. Settings retained unless -RemoveSettings was specified.'
    exit
}
$source = Join-Path $PSScriptRoot 'Kurmon.exe'
if (!(Test-Path $source)) { $source = Join-Path $PSScriptRoot '..\target\release\Kurmon.exe' }
if (!(Test-Path $source)) { throw 'Executable not found. Extract the complete package or run cargo build --release --bin Kurmon first.' }
if (Test-Path $shortcutPath) {
    $shortcutTarget = $shell.CreateShortcut($shortcutPath).TargetPath
    $oldExe = Join-Path $destination 'taskbar-system-monitor.exe'
    if ($shortcutTarget -ne $exe -and $shortcutTarget -ne $oldExe) {
        throw "Cannot replace an existing shortcut that points elsewhere: $shortcutPath"
    }
}
New-Item $destination -ItemType Directory -Force | Out-Null
Copy-Item $source $exe -Force
if (Test-Path (Join-Path $destination 'taskbar-system-monitor.exe')) { Remove-Item (Join-Path $destination 'taskbar-system-monitor.exe') -Force }
Copy-Item $PSCommandPath (Join-Path $destination 'install.ps1') -Force
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $exe
$shortcut.Save()
Write-Output "Installed to $exe. Existing preferences preserved; sign-in startup is not enabled by installation."
