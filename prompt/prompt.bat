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

REM powerline-go -shell bare -colorize-hostname -error %errorlevel% -newline
powerline-go -shell bare -colorize-hostname -error %CERRCODE% -newline

REM input the old vars first. This allows us to lower the amount of processing later
set | %FINEXE% --client -i %PIPECODE% -o

REM this sets errorlevel to 1 - don't know why, but we need to reset it, 
REM not important anymore since I use my own code instead of errorlevel
set /p CMD=

REM filter out some local commands
if "!CMD!"=="exit" (
    exit
)

if /I "!CMD:~0,2!"=="cd" (
    !CMD!
) else if /I "!CMD!"=="" (
    REM Nothing, can't execute no command
    goto infiniloop
) else (
    REM remove variable so it won't be duplicated
    set "CERRCODE="
    cmd /V:ON /c "!CMD! & set | !FINEXE! --client -i !PIPECODE! -e ^!errorlevel^! -s"
    REM get back subshell env and set env vars in this one
    set /a count = 1
    for /f "delims=" %%F in ('%FINEXE% --client -i %PIPECODE% -r') do (
        if "!count!"=="1" (
            REM set error code
            set "CERRCODE=%%F"
        ) else (
            REM set env variables
            set "%%F"
        )
        set /a count += 1
    )
)

REM Clear CMD in case we re-enter
set CMD=

goto infiniloop
