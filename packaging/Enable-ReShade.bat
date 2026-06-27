@echo off
setlocal
title NightZoom FPS Limiter - Enable ReShade in FiveM

rem ============================================================================
rem  NightZoom FPS Limiter - Enable ReShade in FiveM
rem
rem  WHAT THIS DOES (and only this):
rem    FiveM blocks ReShade 5+ until you acknowledge it once. This adds a single
rem    line to your CitizenFX.ini, under [Addons]:
rem        ReShade5=ID:<your-pc-id> acknowledged that ReShade 5.x has a bug ...
rem    <your-pc-id> is just a hash of your PC name - exactly what FiveM expects
rem    (see FiveM's ReShadeFixups.cpp). Nothing is downloaded, no FiveM launch is
rem    needed, and every other setting in CitizenFX.ini is left untouched: it
rem    uses the same Windows INI API FiveM itself reads the file with.
rem
rem  Open source - read the PowerShell below this header to see exactly what runs.
rem  https://github.com/Nipeno/nightzoom-fps-limiter
rem ============================================================================

rem Hand our own path + folder to PowerShell via env vars so a path containing a
rem quote/apostrophe (e.g. C:\Users\O'Brien\...) can never break the script.
set "NZ_SELF=%~f0"
set "NZ_DIR=%~dp0"

powershell -NoProfile -ExecutionPolicy Bypass -Command "$m=[char]35+':PS:'+[char]35; $t=[IO.File]::ReadAllText($env:NZ_SELF); $i=$t.LastIndexOf($m); Invoke-Expression $t.Substring($i+$m.Length)"

echo.
pause
exit /b
#:PS:#
# ---------------------------------------------------------------------------
#  PowerShell body - this is the real logic. Plain text, read it freely.
# ---------------------------------------------------------------------------
$ErrorActionPreference = 'Stop'
$batDir = $env:NZ_DIR

Write-Host ''
Write-Host '  NightZoom FPS Limiter - Enable ReShade in FiveM' -ForegroundColor Cyan
Write-Host '  ------------------------------------------------'

# 1) Compute this PC's ReShade5 acknowledgement ID.
#    FiveM uses Joaat(lowercase(COMPUTERNAME)), ASCII only. Verified: PC -> 46750aa6.
#    (Non-ASCII PC names are rare; if the ID looks wrong, use the manual steps in
#     INSTALL.html instead.)
$name = $env:COMPUTERNAME
if ([string]::IsNullOrEmpty($name)) { $name = 'a' }

[long]$h = 0
foreach ($b in [System.Text.Encoding]::ASCII.GetBytes($name)) {
    [long]$c = $b
    if ($c -ge 65 -and $c -le 90) { $c += 32 }            # A-Z -> a-z (FiveM ToLower)
    $h = ($h + $c) -band 0xFFFFFFFFL
    $h = ($h + (($h -shl 10) -band 0xFFFFFFFFL)) -band 0xFFFFFFFFL
    $h = ($h -bxor ($h -shr 6)) -band 0xFFFFFFFFL
}
$h = ($h + (($h -shl 3)  -band 0xFFFFFFFFL)) -band 0xFFFFFFFFL
$h = ($h -bxor ($h -shr 11)) -band 0xFFFFFFFFL
$h = ($h + (($h -shl 15) -band 0xFFFFFFFFL)) -band 0xFFFFFFFFL
$id    = '{0:x8}' -f $h
$value = "ID:$id acknowledged that ReShade 5.x has a bug that will lead to game crashes"

Write-Host ("  PC name   : {0}" -f $name)
Write-Host ("  ReShade ID: {0}" -f $id)

# 2) Find CitizenFX.ini (it lives in the FiveM.app folder).
function Test-FiveMApp([string]$dir) {
    if ([string]::IsNullOrEmpty($dir)) { return $false }
    return (Test-Path (Join-Path $dir 'FiveM.exe')) -or
           (Test-Path (Join-Path $dir 'CitizenFX.ini')) -or
           (Test-Path (Join-Path $dir 'plugins'))
}

$candidates = @()
try { if ($batDir) { $candidates += (Resolve-Path (Join-Path $batDir '..')).Path } } catch {}  # ships in FiveM.app\plugins
$candidates += (Join-Path $env:LOCALAPPDATA 'FiveM\FiveM.app')                                  # default install
try {                                                                                            # fivem:// handler
    $cmd = (Get-ItemProperty 'Registry::HKEY_CLASSES_ROOT\fivem\shell\open\command' -ErrorAction Stop).'(default)'
    if ($cmd -match '([A-Za-z]:\\[^"]+FiveM\.exe)') { $candidates += (Split-Path $matches[1] -Parent) }
} catch {}

$ini = $null
foreach ($dir in $candidates) { if (Test-FiveMApp $dir) { $ini = (Join-Path $dir 'CitizenFX.ini'); break } }

if (-not $ini) {
    Write-Host ''
    Write-Host '  Could not find your FiveM folder automatically.' -ForegroundColor Yellow
    Write-Host '  Put this file in your FiveM plugins folder and run it again:'
    Write-Host '    %localappdata%\FiveM\FiveM.app\plugins'
    Write-Host '  ...or follow the manual steps in INSTALL.html.'
    exit 1
}

# 3) Surgical INI edit via the same Win32 API FiveM uses. Only the ReShade5 key
#    under [Addons] is touched; the file is created if missing; nothing else moves.
Add-Type -Namespace NZ -Name Ini -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("kernel32", CharSet=System.Runtime.InteropServices.CharSet.Unicode)]
public static extern bool WritePrivateProfileString(string section, string key, string val, string filePath);
[System.Runtime.InteropServices.DllImport("kernel32", CharSet=System.Runtime.InteropServices.CharSet.Unicode)]
public static extern int GetPrivateProfileString(string section, string key, string def, System.Text.StringBuilder ret, int size, string filePath);
'@

$sb = New-Object System.Text.StringBuilder 512
[void][NZ.Ini]::GetPrivateProfileString('Addons', 'ReShade5', '', $sb, 512, $ini)
$current = $sb.ToString()

Write-Host ''
Write-Host ("  File: {0}" -f $ini)

if ($current -eq $value) {
    Write-Host '  Already enabled for this PC - nothing to change.' -ForegroundColor Green
    Write-Host '  Start FiveM and press Home (or Insert with NVE) to open the menu.'
    exit 0
}

Write-Host '  Adding this line under [Addons]:'
Write-Host ("    ReShade5={0}" -f $value) -ForegroundColor Gray
$ok = [NZ.Ini]::WritePrivateProfileString('Addons', 'ReShade5', $value, $ini)
if ($ok) {
    Write-Host '  Done! ReShade is now allowed in FiveM.' -ForegroundColor Green
    Write-Host '  Start FiveM, press Home (or Insert with NVE), then tick "Limit to 60 FPS".'
} else {
    Write-Host ("  Failed to write {0}" -f $ini) -ForegroundColor Red
    Write-Host '  Close FiveM if it is open, or right-click this file and "Run as administrator".'
    exit 1
}
