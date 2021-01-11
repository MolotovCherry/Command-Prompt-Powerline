use std::{collections::HashMap, env, ffi::OsString};
use std::path::Path;
use std::process::Command;
use std::str;
use std::io;
use std::io::Write;
use Iterator;
use itertools::Itertools;

use clap::{Arg, App};

use winreg::enums::*;
use winreg::RegKey;

use subprocess::{Popen, PopenConfig, Redirection};

use winapi::HANDLE;
use winapi::wincon::CONSOLE_SCREEN_BUFFER_INFO;
use winapi::wincon::COORD;
use winapi::wincon::SMALL_RECT;
use winapi::WORD;
use winapi::DWORD;

static mut CONSOLE_HANDLE: Option<HANDLE> = None;


fn main() {
    let matches = App::new("Powerline CMD")
        .version("1.0")
        .author("Cherryleafroad <13651622+cherryleafroad@users.noreply.github.com>")
        .about("Run Powerline in Commnd Prompt!")
        .arg(Arg::new("command")
            .short('c')
            .takes_value(true)
            .value_name("CMD")
            .about("Runs a command and exits")
            .conflicts_with("kommand"))
        .arg(Arg::new("kommand")
            .short('k')
            .takes_value(true)
            .value_name("CMD")
            .about("Run command and drop to shell")
            .conflicts_with("command"))
        .get_matches();

    // are we running on Windows Terminal?
    // this should be the first check as to not write to print
    if let Err(_) = env::var("WT_SESSION") {
        // if not, then re-launch in WT
        Command::new("wt").args(&[env::current_exe().unwrap().as_os_str()]).spawn().expect("Windows Terminal not installed");
        return
    }

    if matches.is_present("command") {
        run_cmd(matches.value_of("command").unwrap(), env::vars().collect());
        return
    } else if matches.is_present("kommand") {
        run_cmd(matches.value_of("kommand").unwrap(), env::vars().collect());
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
        // Powerline prompt
        let powerline_go = Command::new("powerline-go")
            .args(&[
                "-shell", "bare", "-colorize-hostname", "-error", &*exit_code, "-newline"
            ]).output().expect("Could not run powerline-go. Is it in your PATH?");
        print!("{}", unsafe {str::from_utf8_unchecked(&powerline_go.stdout)});
        // print! does not flush stdout
        io::stdout().flush().expect("Could not flush stdout");

        // grab user cmd input
        // newline is added, so it needs to be trimmed
        let mut _cmd = String::new();
        io::stdin().read_line(&mut _cmd).unwrap();
        let cmd = _cmd.trim();


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
                continue
            }
            "" => continue,
            _ => ()
        }
        // Process CD command
        let cd =  cmd.get(0..3).unwrap_or("");
        // space ensures it's not like cds or something, but also just match only cd
        if cd.to_lowercase() == "cd " || cmd.to_lowercase() == "cd" {
            // ~ is not handled in regular CMD, so I can ignore it
            let path_s = cmd.get(3..).unwrap_or("").trim_start();
            let path = Path::new(path_s);
            // empty cd passes through, non empty gets evaluated
            if path_s == "" {
                // print current dir
                println!("{}\n", env::current_dir().unwrap().to_str().unwrap());
                continue
            } else {
                match env::set_current_dir(path) {
                    Ok(_) => {
                        println!("");
                        continue
                    },
                    Err(_) => {
                        println!("The system cannot find the path specified.\n");
                        continue
                    }
                }
            }
        }

        exit_code = run_cmd(cmd, env::vars().collect());
    }
}

fn run_cmd(cmd: &str, old_vars: HashMap<String, String>) -> String {
    let env: Vec<(OsString, OsString)> = env::vars_os().collect();

    let mut p = Popen::create(&["cmd", "/k", cmd],
    PopenConfig {
                stdout: Redirection::Pipe, stderr: Redirection::Merge,
                stdin: Redirection::Pipe, env: Some(env), ..Default::default()
            }
    ).unwrap();

    // echo errorlevel and env variables on new input line in order to avoid messing up original command
    let output = p.communicate(Some("echo %errorlevel% && set & exit\n")).unwrap().0.unwrap();

    let mut msg = String::from("");
    let mut errorcode = String::from("0");
    let mut env_follows = false;
    let mut errorcode_follows = false;
    for line in output.lines() {
        if line.ends_with("echo %errorlevel% && set & exit") {
            env_follows = true;
            errorcode_follows = true;
        } else if env_follows {
            // process first line as error code
            if errorcode_follows {
                errorcode = String::from(line).trim().to_string();
                errorcode_follows = false;
                continue
            }

            let (k, v) = line.splitn(2, "=").collect_tuple().unwrap();
            // new key or changed value for existing key
            if !old_vars.contains_key(k) || old_vars.get(k).unwrap() != v {
                env::set_var(k, v);
            }
        } else {
            msg.push_str(&*format!("{}", line.trim()))
        } 
    }

    if msg.eq("") {
        print!("\n");
    }
    else {
        print!("{}\n\n", msg);
    }
    
    io::stdout().flush().expect("Could not flush stdout");
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
            let handle = kernel32::GetStdHandle(winapi::STD_OUTPUT_HANDLE);
            CONSOLE_HANDLE = Some(handle);
            return handle;
        }
    }
}

fn get_buffer_info() -> winapi::CONSOLE_SCREEN_BUFFER_INFO {
    let handle = get_output_handle();
    if handle == winapi::INVALID_HANDLE_VALUE {
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
        kernel32::GetConsoleScreenBufferInfo(handle, &mut buffer);
    }
    buffer
}

fn clear() {
    let handle = get_output_handle();
    if handle == winapi::INVALID_HANDLE_VALUE {
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
        kernel32::FillConsoleOutputCharacterW(
            handle,
            32 as winapi::WCHAR,
            console_size,
            coord_screen,
            &mut amount_chart_written,
        );
    }
    set_cursor_possition(0, 0);
}

fn set_cursor_possition(y: i16, x: i16) {
    let handle = get_output_handle();
    if handle == winapi::INVALID_HANDLE_VALUE {
        panic!("NoConsole")
    }
    unsafe {
        kernel32::SetConsoleCursorPosition(handle, COORD { X: x, Y: y });
    }
}
