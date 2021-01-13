use std::{collections::HashMap, env, io::Write};
use std::process;
use std::str;
use std::io;
use Iterator;
use itertools::Itertools;

use device_query::{DeviceQuery, DeviceState, Keycode};

use clap::{Arg, App};

use lazy_static::lazy_static;
use winreg::enums::*;
use winreg::RegKey;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Command;
use tokio::fs::File;
use std::process::Stdio;

use rand::Rng;
use regex::{Captures, Regex};

use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::shared::minwindef::{
    WORD, DWORD
};
use winapi::um::winnt::{
    WCHAR, HANDLE
};
use winapi::um::wincon::{
    CONSOLE_SCREEN_BUFFER_INFO,
    COORD,
    SMALL_RECT,
    FillConsoleOutputCharacterW,
    SetConsoleCursorPosition,
    GetConsoleScreenBufferInfo
};
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::STD_OUTPUT_HANDLE;

static mut CONSOLE_HANDLE: Option<HANDLE> = None;


#[tokio::main]
async fn main() {
    let matches = App::new("Powerline CMD")
        .version("1.0")
        .author("Cherryleafroad <13651622+cherryleafroad@users.noreply.github.com>")
        .about("Run Powerline in Commnd Prompt!")
        .arg(Arg::new("command")
            .short('c')
            .takes_value(true)
            .value_name("CMD")
            .about("Runs a command and exits (does not accept batch as command; use interactive console for that)")
            .conflicts_with("kommand"))
        .arg(Arg::new("kommand")
            .short('k')
            .takes_value(true)
            .value_name("CMD")
            .about("Run command and drop to shell (does not accept batch as command; use interactive console for that)")
            .conflicts_with("command"))
        .get_matches();

    // are we running on Windows Terminal?
    // this should be the first check as to not write to print
    if let Err(_) = env::var("WT_SESSION") {
        // if not, then re-launch in WT
        process::Command::new("wt").args(&[env::current_exe().unwrap().as_os_str()]).spawn().expect("Windows Terminal not installed");
        return
    }

    if matches.is_present("command") {
        run_cmd(matches.value_of("command").unwrap(), env::vars().collect(), false).await;
        return
    } else if matches.is_present("kommand") {
        run_cmd(matches.value_of("kommand").unwrap(), env::vars().collect(), false).await;
    } else {
        let version = get_version();
        println!(
            "Microsoft Windows [Version {}]\n\
            (c) {} Microsoft Corporation. All rights reserved.\n",
            version.0, version.1
        );
    }

    // setup env vars
    env::set_var("TERM", "xterm-256color");

    // ignore ctrl+c
    ctrlc::set_handler(|| {
        println!("^C");
    }).expect("Error setting Ctrl-C handler");


    let mut exit_code = String::from("0");
    loop {
        let mut powerline_go = Command::new("powerline-go");
        powerline_go.args(&[
                "-shell", "bare", "-colorize-hostname", "-error", &*exit_code, "-newline"
        ]);

        let child = powerline_go.spawn().expect("failed to spawn command");
        let out = child.wait_with_output().await.expect("child process encountered an error");
        
        print!("{}", unsafe {str::from_utf8_unchecked(&out.stdout)});
        //AsyncWriteExt::flush(&mut out.stdout).await.unwrap();
        // print! does not flush stdout
        //io::stdout().flush().expect("Could not flush stdout");

        // grab user cmd input
        // newline is added, so it needs to be trimmed
        let mut _cmd = String::new();
        io::stdin().read_line(&mut _cmd).unwrap();

        // check if left ctrl is pressed for multiple entry
        let mut multiline = false;
        let mut device_state = DeviceState::new();
        let mut keys: Vec<Keycode> = device_state.get_keys();
        while keys.contains(&Keycode::LShift) {
            print!(">> ");
            io::stdout().flush().expect("Could not flush stdout");
            io::stdin().read_line(&mut _cmd).unwrap();
            device_state = DeviceState::new();
            keys = device_state.get_keys();
            multiline = true;
        }

        let cmd = &*_cmd.replace("\r\n", "");


        // process other commands
        match &*cmd.to_lowercase() {
            // exit #num, not currently supported
            "exit" => {
                println!("");
                break
            },
            "cls" => {
                clear();
                println!("");
                exit_code = String::from("0");
                continue;
            }
            "" => {
                exit_code = String::from("0");
                continue;
            },
            _ => ()
        }

        exit_code = run_cmd(cmd, env::vars().collect(), multiline).await;
    }
}

#[allow(non_upper_case_globals)]
async fn run_cmd(cmd_str: &str, old_vars: HashMap<String, String>, multiline: bool) -> String {
    let mut cmd = Command::new("cmd");
    if multiline {
        let mut rng = rand::thread_rng();
        let n2: u16 = rng.gen();

        let mut file_path = env::temp_dir();
        file_path.push(format!("powerline-cmd-tmp-{}.bat", n2));
        cmd.args(&["/k", file_path.to_str().unwrap()]);

        // match %f variables to convert into %%f (batch file requirement)
        // compile regex only once cause they are expensive
        // (?<!%)%[A-Za-z0-9_(){}\[\]\$\*\+-\\\/#',;\.@!\?]++(?!%) is the best version, but won't work
        lazy_static! {
            // first match all vars
            static ref all_vars: Regex = Regex::new(r"(%?%([A-Za-z\-_]+)%?%?)").unwrap();
        }

        let contents = format!("@echo off\n\n\
            {}\n\
        ", all_vars.replace_all(cmd_str, |caps: &Captures| {
            if caps[1].ends_with("%") {
                // regular var - return whole match
                String::from(&caps[1])
            } else {
                // a %%x or %x type - return basename + %%
                format!("%%{}", &caps[2])
            }
        }));

        let mut file = File::create(file_path).await.expect("Could not create tmp file");
        file.write_all(contents.as_bytes()).await.unwrap();
    } else {
        cmd.args(&["/k", cmd_str]);
    }

    cmd.env("PROMPT", "<EOF>Exit>>\n");

    cmd.stdout(Stdio::piped());
    cmd.stdin(Stdio::piped());

    let mut child = cmd.spawn()
        .expect("failed to spawn command");

    let stdout = child.stdout.take()
        .expect("child did not have a handle to stdout");
    let stdin = child.stdin.take().expect("child did not have stdin handle");

    let mut reader = BufReader::new(stdout).lines();
    let mut writer = BufWriter::new(stdin);

    tokio::spawn(async {
        // nothing? heihei
    });

    // async process of incoming read data
    let mut marker = false;
    let mut errorcode = String::from("");
    let mut check_next_exit_code = false;
    let mut check_cd = false;
    while let Some(line) = reader.next_line().await.unwrap() {
        // found end of output, so write a new command to input
        if line.ends_with("<EOF>Exit>>") {
            // echo errorlevel and env variables on new input line in order to avoid messing up original command
            writer.write_all(b"echo %errorlevel% & echo %CD% & set & exit\n").await.unwrap();
            writer.flush().await.unwrap();
            marker = true;
        // we found a marker, this is the env section
        } else if marker {
            if line != "" && !line.starts_with("PROMPT=") {
                // this is one of the lines since we \n'd it
                if line.ends_with("echo %errorlevel% & echo %CD% & set & exit") {
                    check_next_exit_code = true;
                    continue;
                } else if check_next_exit_code {
                    // it was the error code
                    errorcode = line.trim().to_string();
                    check_next_exit_code = false;
                    check_cd = true;
                    continue;
                } else if check_cd {
                    check_cd = false;
                    env::set_current_dir(line).unwrap_or(());
                    continue;
                }

                if let Some((k, v)) = line.splitn(2, "=").collect_tuple() {
                    // new key or changed value for existing key
                    if !old_vars.contains_key(k) || old_vars.get(k).unwrap() != v {
                        env::set_var(k, v);
                    }
                } else {
                    // tuple unpacking failed
                    println!("Did you enter batch? Batch requires multiline input~~\n");
                    errorcode = String::from("1");
                    child.kill().await.unwrap();
                    break;
                }
            }
        // print ordinary output
        } else {
            println!("{}", line);
        }
    }


    if errorcode == "" {
        // cmd returned early because of syntax error, didn't process errorcode
        errorcode = String::from("1");
        println!("");
    }
    //_exit_code
    errorcode
}

fn get_version() -> (String, String) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let cur_ver = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion").expect("Failed to find system version");
    let major: u32 = cur_ver.get_value("CurrentMajorVersionNumber").expect("Failed to find system version");
    let minor: u32 = cur_ver.get_value("CurrentMinorVersionNumber").expect("Failed to find system version");
    let build: String = cur_ver.get_value("CurrentBuildNumber").expect("Failed to find system version");
    let ubr: u32 = cur_ver.get_value("UBR").expect("Failed to find system version");
    let version = format!("{}.{}.{}.{}", major, minor, build, ubr);

    let year: String = match &*build {
        "18363" => "2019".to_string(),
        "19041" => "2020".to_string(),
        "19042" => "2020".to_string(),
        _ => panic!("Your Windows installation is EOL")
    };

    (version, year)
}

fn get_output_handle() -> HANDLE {
    unsafe {
        if let Some(handle) = CONSOLE_HANDLE {
            return handle;
        } else {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            CONSOLE_HANDLE = Some(handle);
            return handle;
        }
    }
}

fn get_buffer_info() -> CONSOLE_SCREEN_BUFFER_INFO {
    let handle = get_output_handle();
    if handle == INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }
    let mut buffer = CONSOLE_SCREEN_BUFFER_INFO {
        dwSize: COORD { X: 0, Y: 0 },
        dwCursorPosition: COORD { X: 0, Y: 0 },
        wAttributes: 0 as WORD,
        srWindow: SMALL_RECT {
            Left: 0,
            Top: 0,
            Right: 0,
            Bottom: 0,
        },
        dwMaximumWindowSize: COORD { X: 0, Y: 0 },
    };
    unsafe {
        GetConsoleScreenBufferInfo(handle, &mut buffer);
    }
    buffer
}

fn clear() {
    let handle = get_output_handle();
    if handle == INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }

    let screen_buffer = get_buffer_info();
    let console_size: DWORD = screen_buffer.dwSize.X as u32 * screen_buffer.dwSize.Y as u32;
    
    let coord_screen = COORD { X: 0, Y: 0 };

    // clear screen with /n -> otherwise we'll have issue with leftover color blocks
    for _ in 0..screen_buffer.dwSize.Y {
        println!("");
    }

    let mut amount_chart_written: DWORD = 0;
    unsafe {
        FillConsoleOutputCharacterW(
            handle,
            32 as WCHAR,
            console_size,
            coord_screen,
            &mut amount_chart_written,
        );
    }
    set_cursor_position(0, 0);
}

fn set_cursor_position(y: i16, x: i16) {
    let handle = get_output_handle();
    if handle == INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }
    unsafe {
        SetConsoleCursorPosition(handle, COORD { X: x, Y: y });
    }
}
