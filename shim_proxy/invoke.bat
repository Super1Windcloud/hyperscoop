@echo off

if not defined SCOOP set SCOOP=%USERPROFILE%\scoop

for %%x in ("%SCOOP%\shims\*.exe") do (
  echo Replacing %%x by new shim.
  del "%%~x"
  copy "%~dp0"\bin\shim.exe "%%~x"
)

if not defined SCOOP_GLOBAL set SCOOP_GLOBAL=%ProgramData%\scoop

for %%x in ("%SCOOP_GLOBAL%\shims\*.exe") do (
  echo Replacing %%x by new shim.
  del "%%~x"
  copy "%~dp0"\bin\shim.exe "%%~x"
)
