# Command Prompt Powerline
Powerline Support for good old Command Prompt! Requires Windows Terminal

![Screenshot](https://github.com/cherryleafroad/Command-Prompt-Powerline/blob/main/readme-files/screenshot.png?raw=true)

## Installation

 1. Install [powerline-go](https://github.com/justjanne/powerline-go) and place it in your `PATH`
 2. Download [precompiled files and scripts](https://github.com/cherryleafroad/Command-Prompt-Powerline/tree/main/prompt) and place them in a folder
 3. Download [Cascadia Code font](https://github.com/microsoft/cascadia-code/) and install all the fonts
 4. Edit the Windows Terminal's Command Prompt profile in `settings.json` to reflect the following (there's also an example [settings.json](https://github.com/cherryleafroad/Command-Prompt-Powerline/blob/main/prompt/settings.json)):
 
 ```
 "commandline": "cmd.exe /c D:/Path/To/prompt.bat"
 "fontFace": "Cascadia Code PL"
 ```
 5. Edit the `ENVPATH` in `prompt.bat` to the folder where the binary/script is located:
 
 `set ENVPATH=C:\Path\To\prompt-powerline\`

### Known Issues
- Ctrl+C will crash the shell
