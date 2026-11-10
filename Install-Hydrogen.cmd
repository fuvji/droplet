@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\install-msix.ps1" -NoPause %*
if errorlevel 1 (
  echo.
  echo Hydrogen installation failed.
  pause
  exit /b 1
)
rem One click: install/update, then open the packaged app.
powershell.exe -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "Start-Process -FilePath 'shell:AppsFolder\HydrogenMacroSuite_aw2q6j67g7xty!Hydrogen'"
if errorlevel 1 (
  echo.
  echo Hydrogen is installed, but the app could not be opened.
  pause
  exit /b 1
)
endlocal
