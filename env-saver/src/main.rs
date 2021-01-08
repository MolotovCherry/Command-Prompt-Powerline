use clap::{Arg, App};
use std::{collections::HashMap, env, io::{self, Read}};
use named_pipe_manager::{PipeClient, PipeServer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use itertools::Itertools;


#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
enum Command {
    SaveEnv,
    ReadEnv,
    OldEnv
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
struct EnvironmentData {
    command: Option<Command>,
    data: Option<String>
}

fn main() -> Result<(), io::Error> {
    let matches = App::new("Environment Saver")
        .version("1.0")
        .author("Cherryleaf")
        .about("Communicates environment info between cmd shells")
        .arg(Arg::new("server")
            .long("server")
            .about("Starts the server")
            .required_unless_present_any(&[
                "client",
                "exitcode",
                "saveenv",
                "readenv",
                "oldenv"
            ]))
        .arg(Arg::new("client")
            .long("client")
            .about("Starts the client")
            .required_unless_present("server"))
        .arg(Arg::new("seed")
            .short('i')
            .long("seed")
            .about("Randomized number to use for internal pipe name")
            .takes_value(true)
            .value_name("NUM")
            .required(true))
        .arg(Arg::new("exitcode")
            .short('e')
            .long("exitcode")
            .about("Save the exit code")
            .takes_value(true)
            .value_name("CODE")
            .requires("saveenv")
            .conflicts_with_all(&["readcode", "readenv"]))
        .arg(Arg::new("saveenv")
            .short('s')
            .long("saveenv")
            .about("Save the ENV info through <STDIN>")
            .requires("exitcode")
            .conflicts_with_all(&["readcode", "readenv"]))
        .arg(Arg::new("readenv")
            .short('r')
            .long("readenv")
            .about("Read the ENV info from server through <STDOUT>")
            .conflicts_with_all(&["exitcode", "saveenv", "readcode"]))
        .arg(Arg::new("oldenv")
            .short('o')
            .long("oldenv")
            .about("Save old environment info to send less data")
            .conflicts_with_all(&["readcode", "readenv", "saveenv", "exitcode"]))
        .arg(Arg::new("hide-output")
            .long("hide-output")
            .about("Don't output any normal messages (errors are still shown)"))
        .get_matches();


    // get the exe name for pipe name
    let default = PathBuf::from("environmentsaver");
    let pipe = env::current_exe().unwrap_or(default);
    let mut pipe_name = pipe.file_stem().unwrap().to_str().unwrap().to_lowercase();
    // then append random numbers
    let nums = matches.value_of("seed").unwrap();
    pipe_name = format!("{}{}", pipe_name, nums);
    
    
    if matches.is_present("server") {
        
        let mut server_data = EnvironmentData {
            command: None,
            data: None
        };

        let mut old_env: Option<HashMap<String, String>> = None;

        if !matches.is_present("hide-output") {
            println!("[Server listening on pipe: {}]", pipe_name);
        }
        let mut server = PipeServer::new(pipe_name);

        server.start().unwrap();

        loop {
            server.wait().unwrap();

            let data: EnvironmentData = server.recv().unwrap().unwrap();
            match data.command.unwrap() {
                Command::ReadEnv => {
                    server.send(&server_data).unwrap();
                    server_data.data = None;
                },
                Command::SaveEnv => {
                    let env_vars: String = data.data.unwrap();

                    let v = env_vars.split("\r\n");
                    let mut buf = String::new();
                    let old_owned = old_env.take().unwrap();
                    for line in v {
                        if line.len() > 0 {
                            let (k, v): (&str, &str) = line.splitn(2, "=").collect_tuple().unwrap();

                            // this is a new key , or the value was modified
                            if !old_owned.contains_key(k) || old_owned.get(k).unwrap().ne(v) {
                                buf.push_str(&*format!("$env:{}=\"{}\"\n", k, v.trim_end()));
                            }
                        }
                    }

                    // remove excess newlines
                    let buf = buf.trim_end().to_string();

                    old_env = None;
                    
                    server_data.data = Some(buf);
                },
                Command::OldEnv => {
                    let mut old_vars: HashMap<String, String> = HashMap::new();
                    let env_vars: String = data.data.unwrap();

                    let v = env_vars.split("\r\n");
                    for line in v {
                        if line.len() > 0 {
                            let (k, v) = line.splitn(2, "=").collect_tuple().unwrap();

                            old_vars.insert(String::from(k), String::from(v));
                        }
                    }

                    old_env = Some(old_vars);
                }
            }

            // disconnect and wait for another connection on next loop
            server.disconnect().unwrap();
        }

        
    } else if matches.is_present("client") {
        let mut client_data = EnvironmentData {
            command: None,
            data: None
        };

        let mut client = PipeClient::new(pipe_name);
        client.connect().unwrap();

        if matches.is_present("saveenv") {
            client_data.command = Some(Command::SaveEnv);
            
            let mut buffer = String::new();
            let mut stdin = io::stdin();
            stdin.read_to_string(&mut buffer)?;
            // place exit code at beginning for one data chunk
            let exitcode = format!("cerrcode={}\r\n", matches.value_of("exitcode").unwrap_or("0"));
            buffer.insert_str(0, &*exitcode);
            client_data.data = Some(buffer);
            client.send(&client_data).unwrap();
        } else if matches.is_present("readenv") {
            client_data.command = Some(Command::ReadEnv);
            client.send(&client_data).unwrap();
            let server_data: EnvironmentData = client.recv().unwrap().unwrap();
            // Got some data back!
            println!("{}", server_data.data.unwrap_or(String::new()));
        } else if matches.is_present("oldenv") {
            client_data.command = Some(Command::OldEnv);

            let mut buffer = String::new();
            let mut stdin = io::stdin();
            stdin.read_to_string(&mut buffer)?;

            client_data.data = Some(buffer);
            client.send(&client_data).unwrap();
        }
    }

    Ok(())
}
