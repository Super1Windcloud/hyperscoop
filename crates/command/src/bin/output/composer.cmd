@rem A:\Scoop\apps\composer\current\composer.ps1
@echo off
where /q pwsh.exe
if %errorlevel% equ 0 (
    pwsh -noprofile -ex unrestricted -file "A:\Scoop\apps\composer\current\composer.ps1" %*
) else (
    powershell -noprofile -ex unrestricted -file "A:\Scoop\apps\composer\current\composer.ps1" %*
)
