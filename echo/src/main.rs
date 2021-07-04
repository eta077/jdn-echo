mod lib;

use std::io::Write;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use jdn_cli::manager::CliManager;
use jdn_cli::{CliError, CliHandler};

use crate::lib::EchoClient;

fn main() {
    let echo_handler = Arc::new(EchoCliHandler::new());
    let mut cli_manager = CliManager::new();
    cli_manager.add_handler(echo_handler);
    cli_manager.start();
}

const SET_ADDRESS_COMMAND: &str = "set-address";
const IS_RUNNING_COMMAND: &str = "is-running";
const IS_CONNECTED_COMMAND: &str = "is-connected";
const SEND_MESSAGE_COMMAND: &str = "send-message";
const START_COMMAND: &str = "start";
const STOP_COMMAND: &str = "stop";
const COMMANDS: [&str; 6] = [
    SET_ADDRESS_COMMAND,
    IS_RUNNING_COMMAND,
    IS_CONNECTED_COMMAND,
    SEND_MESSAGE_COMMAND,
    START_COMMAND,
    STOP_COMMAND,
];

struct EchoCliHandler {
    client: Mutex<EchoClient>,
}

impl EchoCliHandler {
    pub fn new() -> Self {
        EchoCliHandler {
            client: Mutex::new(EchoClient::new(
                SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            )),
        }
    }
}

impl CliHandler for EchoCliHandler {
    fn get_commands(&self) -> std::collections::HashSet<&'static str> {
        COMMANDS.iter().cloned().collect()
    }

    fn handle_command(
        &self,
        command: &str,
        args: Vec<String>,
        writer: &mut dyn Write,
    ) -> Result<(), CliError> {
        match command {
            SET_ADDRESS_COMMAND => {
                if let Some(address) = args.get(0) {
                    self.client.lock().unwrap().set_address(
                        SocketAddr::from_str(address)
                            .map_err(|e| CliError::ArgumentParseFailure(e.to_string()))?,
                    );
                } else {
                    return Err(CliError::InvalidNumberOfArguments {
                        min: 1,
                        max: None,
                        given: 0,
                    });
                }
            }
            IS_RUNNING_COMMAND => {
                writeln!(writer, "{}", self.client.lock().unwrap().is_running()).map_err(|_| {
                    CliError::ExecutionError(String::from("Unable to write output"))
                })?;
            }
            IS_CONNECTED_COMMAND => {
                writeln!(writer, "{}", self.client.lock().unwrap().is_connected()).map_err(
                    |_| CliError::ExecutionError(String::from("Unable to write output")),
                )?;
            }
            SEND_MESSAGE_COMMAND => {
                if let Some(message) = args.get(0) {
                    self.client.lock().unwrap().send_message(message);
                } else {
                    return Err(CliError::InvalidNumberOfArguments {
                        min: 1,
                        max: None,
                        given: 0,
                    });
                }
            }
            START_COMMAND => {
                self.client.lock().unwrap().start();
            }
            STOP_COMMAND => {
                self.client.lock().unwrap().stop();
            }
            _ => {
                return Err(CliError::ExecutionError(format!(
                    "Unknown command: {}",
                    command
                )));
            }
        }
        Ok(())
    }
}
