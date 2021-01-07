@echo off

REM Fix Titlebar
TITLE Command Prompt

REM Print Default message
setlocal enableextensions
setlocal enabledelayedexpansion
for /f "tokens=*" %%a in ('VER') do ( 
    set VERSION=%%a 
) 
echo %VERSION% 
echo (c) 2020 Microsoft Corporation. All rights reserved.
echo.

REM Set up vars
set TERM=xterm-256color
REM change it to your path! Spaces are fine
REM Leave the trailing \
set ENVPATH=D:\Path\To\env_saver\
set ENVBIN=environment_saver.exe
set FINEXE="%ENVPATH%%ENVBIN%"
set PIPECODE=%random%
set CERRCODE=0

REM Kill server if it's already running
REM Don't kill it tho cause then we can do multiple CMD windows
REM taskkill /f /IM "%ENVBIN%"

REM start server in background
start /MIN /HIGH /B "" %FINEXE% --server -i %PIPECODE% > nul 2>&1


REM Infinite loop to always replay powerline shell cmd
:infiniloop

if "!CERRCODE!"=="" (
    REM Execution hit an error ("was unexpected at this time."), so fix display
    set CERRCODE=1
)

REM powerline-go -shell bare -colorize-hostname -error %errorlevel% -newline
powerline-go -shell bare -colorize-hostname -error %CERRCODE% -newline

REM Clear CMD in case we re-enter
set CMD=
REM remove variable so it won't be duplicated in env (and saved over)
set CERRCODE=
REM input the old vars first. This allows us to lower the amount of processing later
REM Using start to allow simultaneous background processing
start /min /b "" cmd /c "set | %FINEXE% --client -i %PIPECODE% -o"

REM this sets errorlevel to 1 - don't know why, but we need to reset it, 
REM not important anymore since I use my own code instead of errorlevel
set /p CMD=

REM filter out some local commands
if "!CMD!"=="exit" (
    exit
)

if /I "!CMD:~0,2!"=="cd" (
    !CMD!
    set CERRCODE=!errorlevel!
) else if /I "!CMD!"=="" (
    REM Nothing, can't execute no command
    set CERRCODE=0
    goto infiniloop
) else (
    cmd /V:ON /c "!CMD! & set | !FINEXE! --client -i !PIPECODE! -e ^!errorlevel^! -s"
    REM get back subshell env and set env vars in this one
    for /f "delims=" %%F in ('%FINEXE% --client -i %PIPECODE% -r') do (
        REM set env variables
        set "%%F"
    )
)

goto infiniloop
