# Command Prompt Powerline
Powerline Support for good old Command Prompt! Requires Windows Terminal

![Screenshot](https://github.com/cherryleafroad/Command-Prompt-Powerline/blob/main/readme-files/screenshot.png?raw=true)

## Installation

 1. Install [powerline-go](https://github.com/justjanne/powerline-go) and place it in your `PATH`
 2. Download [precompiled files and scripts](https://github.com/cherryleafroad/Command-Prompt-Powerline/tree/main/prompt) and place them in the same folder somewhere
 3. Download [Cascadia Code font](https://github.com/microsoft/cascadia-code/) and install all the fonts
 4. Edit the Windows Terminal's Command Prompt profile in `settings.json` to reflect the following (there's also an example [settings.json](https://github.com/cherryleafroad/Command-Prompt-Powerline/blob/main/prompt/settings.json)):
 
 ```
 "commandline": "Powershell.exe -File C:/Path/To/prompt.ps1"
 "fontFace": "Cascadia Code PL"
 ```

### FAQ

#### Why is this a Powershell script? I thought this was for Command Prompt?
It IS for the command prompt. The Powershell script is only the engine for it, that's all. Any commands you enter are executed in the normal command prompt like they should be, don't worry. ;)

### Limitations
- Can't accept multiline input
