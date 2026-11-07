param([switch]$Uninstall,[switch]$RemoveSettings)
$ErrorActionPreference = 'Stop'
$destination = Join-Path $env:LOCALAPPDATA 'Programs\TaskbarSystemMonitor'
$exe = Join-Path $destination 'taskbar-system-monitor.exe'
$shortcutPath = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Kurmon.lnk'
$shell = New-Object -ComObject WScript.Shell
if ($Uninstall) {
    $running = Get-Process 'taskbar-system-monitor' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }
    if ($running) { throw 'Exit Kurmon from its tray menu before uninstalling.' }
    Remove-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name TaskbarSystemMonitor -ErrorAction SilentlyContinue
    if (Test-Path $destination) { Remove-Item $destination -Recurse -Force }
    if ((Test-Path $shortcutPath) -and $shell.CreateShortcut($shortcutPath).TargetPath -eq $exe) { Remove-Item $shortcutPath }
    if ($RemoveSettings) { Remove-Item (Join-Path $env:LOCALAPPDATA 'TaskbarSystemMonitor') -Recurse -Force -ErrorAction SilentlyContinue }
    Write-Output 'Uninstalled. Settings retained unless -RemoveSettings was specified.'
    exit
}
$source = Join-Path $PSScriptRoot 'taskbar-system-monitor.exe'
if (!(Test-Path $source)) { $source = Join-Path $PSScriptRoot '..\target\release\taskbar-system-monitor.exe' }
if (!(Test-Path $source)) { throw 'Executable not found. Extract the complete package or run cargo build --release --bin taskbar-system-monitor first.' }
if (Get-Process 'taskbar-system-monitor' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }) { throw 'Exit the installed monitor before updating.' }
if ((Test-Path $shortcutPath) -and $shell.CreateShortcut($shortcutPath).TargetPath -ne $exe) {
    throw "Cannot replace an existing shortcut that points elsewhere: $shortcutPath"
}
New-Item $destination -ItemType Directory -Force | Out-Null
Copy-Item $source $exe -Force
Copy-Item $PSCommandPath (Join-Path $destination 'install.ps1') -Force
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $exe
$shortcut.Save()
Write-Output "Installed to $exe. Existing preferences preserved; sign-in startup is not enabled by installation."
