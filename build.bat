@echo off
setlocal
set "ROOT=%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
  if exist "%USERPROFILE%\.cargo\bin\cargo.exe" (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
  ) else (
    echo cargo was not found. Install Rust from https://rustup.rs/ first.
    exit /b 1
  )
)

where cl >nul 2>nul
if errorlevel 1 (
  set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
  if exist "%VSWHERE%" (
    for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSDIR=%%i"
  )
  if defined VSDIR if exist "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" (
    call "%VSDIR%\VC\Auxiliary\Build\vcvars64.bat" >nul
  )
)

if not exist "%ROOT%.tmp" mkdir "%ROOT%.tmp"
set "TEMP=%ROOT%.tmp"
set "TMP=%ROOT%.tmp"
cd /d "%ROOT%"
cargo %*
exit /b %ERRORLEVEL%
