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

## FAQ
#### How do I enter batch / multiline?
Just hold down shift and press enter and you'll enter multiline mode where you can enter batch as well

## Known issues
- Can't handle programs which require stdin
