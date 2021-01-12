# Command Prompt Powerline
Powerline Support for good old Command Prompt! Requires Windows Terminal

![Screenshot](https://github.com/cherryleafroad/Command-Prompt-Powerline/blob/main/readme-files/screenshot.png?raw=true)

## Installation

 1. Install [powerline-go](https://github.com/justjanne/powerline-go) and place it in your `PATH`
 2. Download [precompiled files](https://github.com/cherryleafroad/Command-Prompt-Powerline/tree/main/prompt) and place them in the same folder somewhere
 3. Download [Cascadia Code font](https://github.com/microsoft/cascadia-code/) and install all the fonts
 4. Edit the Windows Terminal's Command Prompt profile in `settings.json` to reflect the following (there's also an example [settings.json](https://github.com/cherryleafroad/Command-Prompt-Powerline/blob/main/prompt/settings.json)):
 
 ```
 "commandline": "C:/Path/To/powerline-cmd.exe"
 "fontFace": "Cascadia Code PL"
 ```

### Bugs
- Can't accept batch commands (if you do, it will likely crash. Use it only for regular commands for the time being)

### Limitations
- Can't accept multiline input (this is a CMD limitation)
