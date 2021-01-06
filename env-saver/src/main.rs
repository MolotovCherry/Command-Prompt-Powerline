use clap::{Arg, App};
use std::{env, io::{self, Read}};
use named_pipe_manager::{PipeClient, PipeServer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;


#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
enum Command {
    SaveEnv,
    ReadEnv
}

#[derive(Debug)]
#[derive(Serialize)]
#[derive(Deserialize)]
struct EnvironmentData {
    command: Option<Command>,
    exit_code: Option<usize>,
    env_vars: Option<String>
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
                "readenv"
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
            exit_code: None,
            env_vars: None
        };
        println!("[Server listening on pipe: {}]", pipe_name);
        let mut server = PipeServer::new(pipe_name);

        server.start().unwrap();

        loop {
            server.wait().unwrap();

            let data: EnvironmentData = server.read().unwrap().unwrap();
            match data.command.unwrap() {
                Command::ReadEnv => {
                    server.write(&server_data).unwrap();
                    server_data.env_vars = None;
                    server_data.exit_code = None;
                },
                Command::SaveEnv => {
                    server_data.env_vars = Some(data.env_vars.unwrap());
                    server_data.exit_code = Some(data.exit_code.unwrap());
                }
            }

            // disconnect and wait for another connection on next loop
            server.disconnect().unwrap();
        }

        
    } else if matches.is_present("client") {
        let mut client_data = EnvironmentData {
            command: None,
            exit_code: None,
            env_vars: None
        };

        let mut client = PipeClient::new(pipe_name);
        client.connect().unwrap();

        if matches.is_present("saveenv") {
            client_data.command = Some(Command::SaveEnv);
            
            let mut buffer = String::new();
            let mut stdin = io::stdin();
            stdin.read_to_string(&mut buffer)?;
            client_data.env_vars = Some(buffer);
            let exitcode = matches.value_of("exitcode").unwrap().parse::<usize>().unwrap();
            client_data.exit_code = Some(exitcode);
            client.write(&client_data).unwrap();
        } else if matches.is_present("readenv") {
            client_data.command = Some(Command::ReadEnv);
            client.write(&client_data).unwrap();
            let server_data: EnvironmentData = client.read().unwrap().unwrap();
            // Got some data back!
            // if this fails do a silent fail (cause ctrl+c in terminal)

            println!("{}\n{}", server_data.exit_code.unwrap_or(0), server_data.env_vars.unwrap_or("".to_string()));
        }
    }

    Ok(())
}
