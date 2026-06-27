@echo off
setlocal
title NightZoom FPS Limiter - Enable ReShade in FiveM
rem  Auto-allows ReShade in FiveM by writing the [Addons] ReShade5 line to
rem  CitizenFX.ini. The required ID is Joaat(lowercase(COMPUTERNAME)) - exactly
rem  what FiveM expects (see ReShadeFixups.cpp). No FiveM launch needed.
rem  Open source: https://github.com/Nipeno/nightzoom-fps-limiter
powershell -NoProfile -ExecutionPolicy Bypass -Command "$batdir='%~dp0'; $t=[IO.File]::ReadAllText('%~f0'); $i=$t.LastIndexOf([char]35+':PS:'+[char]35); iex $t.Substring($i+6)"
echo.
pause
exit /b
#:PS:#
$ErrorActionPreference = 'Stop'
Write-Host ''
Write-Host '  NightZoom FPS Limiter - Enable ReShade in FiveM' -ForegroundColor Cyan
Write-Host '  -----------------------------------'

# 1. Compute the ReShade5 acknowledgement ID for this PC.
#    FiveM uses Joaat(lowercase(COMPUTERNAME)); ASCII A-Z only. Verified: "PC" -> 46750aa6.
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
$id = '{0:x8}' -f $h
$value = "ID:$id acknowledged that ReShade 5.x has a bug that will lead to game crashes"

Write-Host ("  PC name   : {0}" -f $name)
Write-Host ("  ReShade ID: {0}" -f $id)

# 2. Find CitizenFX.ini (it lives in the FiveM.app folder).
function Test-FiveMApp([string]$dir) {
    if ([string]::IsNullOrEmpty($dir)) { return $false }
    return (Test-Path (Join-Path $dir 'FiveM.exe')) -or
           (Test-Path (Join-Path $dir 'CitizenFX.ini')) -or
           (Test-Path (Join-Path $dir 'plugins'))
}

$candidates = @()
try { $candidates += (Resolve-Path (Join-Path $batdir '..')).Path } catch {}   # script ships in FiveM.app\plugins
$candidates += (Join-Path $env:LOCALAPPDATA 'FiveM\FiveM.app')                 # default install
try {                                                                          # fivem:// protocol handler
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
    exit 1
}

# 3. Read the current value, and only write if it isn't already correct.
#    Same Win32 API FiveM uses; Write creates the file/section if missing and
#    leaves every other setting untouched.
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
if ($current -eq $value) {
    Write-Host ("  Already enabled for this PC - nothing to do.") -ForegroundColor Green
    Write-Host ("  File: {0}" -f $ini)
    Write-Host '  Just start FiveM and press Home (or Insert with NVE) to open the menu.'
    exit 0
}

$ok = [NZ.Ini]::WritePrivateProfileString('Addons', 'ReShade5', $value, $ini)
if ($ok) {
    Write-Host ("  Done! Updated: {0}" -f $ini) -ForegroundColor Green
    Write-Host '  ReShade is now allowed. Start FiveM, press Home (or Insert with'
    Write-Host '  NVE) to open the menu, then tick "Limit to 60 FPS".'
} else {
    Write-Host ("  Failed to write {0}" -f $ini) -ForegroundColor Red
    Write-Host '  Try right-clicking this file and "Run as administrator".'
    exit 1
}
