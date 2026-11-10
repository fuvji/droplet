@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\install-msix.ps1" -NoPause %*
if errorlevel 1 (
  echo.
  echo Droplet installation failed.
  pause
  exit /b 1
)
rem One click: install/update, then open the packaged app without hardcoding
rem the package-family hash (it changes when the signing identity changes).
powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "$p=Get-AppxPackage -Name 'DropletMacroSuite' | Select-Object -First 1; if ($p) { Start-Process ('shell:AppsFolder\' + $p.PackageFamilyName + '!Droplet') }"
if errorlevel 1 (
  echo.
  echo Droplet is installed, but the app could not be opened.
  pause
  exit /b 1
)
endlocal
