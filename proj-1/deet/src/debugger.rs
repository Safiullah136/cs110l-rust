use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use crate::inferior::Status;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::FileHistory;
use crate::dwarf_data::{DwarfData, Error as DwarfError};

pub struct Debugger<'a> {
    target: String,
    history_path: String,
    readline: Editor<(), FileHistory>,
    inferior: Option<Inferior>,
    debug_data: DwarfData<'a>,
    breakpoints: HashMap<usize, u8>
}

impl Debugger<'_> {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData

        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<(), FileHistory>::new().expect("Create Editor Failed");
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data
        }
    }

    fn parse_address(addr: &str) -> Option<usize> {
        let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
            &addr[2..]
        } else {
            &addr
        };
        usize::from_str_radix(addr_without_0x, 16).ok()
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if let Some(inferior) = &mut self.inferior {
                        inferior.kill();
                        self.inferior = None;
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args, &self.breakpoints) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        self.inferior_cont();
                    } else {
                        println!("Error starting subprocess");
                    }
                },
                DebuggerCommand::Cont => {
                    self.inferior_cont();
                }
                DebuggerCommand::Quit => {
                    if let Some(inferior) = &mut self.inferior {
                        inferior.kill();
                        self.inferior = None;
                    }
                    return;
                },
                DebuggerCommand::Backtrace => {
                    if self.inferior.is_some() {
                        self.inferior.as_ref().unwrap().print_backtrace(&self.debug_data).unwrap();
                    }
                },
                DebuggerCommand::Breakpoint(location) => {
                    let breakpoint_addr;
                    if location.starts_with("*") {
                        if let Some(address) = self.parse_address(&location[1..]) {
                            breakpoint_addr = address;
                        } else {
                            println!("Invalid address!");
                            continue;
                        }
                    } else if let Some(line) = usize::from_str_radix(&location, 10).ok() {
                        if let Some(addr) = self.debug_data.get_addr_for_line(None, line) {
                            breakpoint_addr = addr;
                        } else {
                            println!("Invalid line number!");
                            continue;
                        }
                    } else if let Some(addr) = self.debug_data.get_addr_for_function(None, &location) {
                        breakpoint_addr = addr;
                    } else {
                        println!("Usage: b|break|breakpoint *address|line|func");
                        continue;
                    }

                    if self.inferior.is_some() {
                        if let Some(val) = self.inferior.as_mut().unwrap().write_byte(breakpoint_addr, 0xcc).ok() {
                            self.breakpoints.insert(breakpoint_addr, val);
                            println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), breakpoint_addr);
                        } else {
                            println!("Invalid breakpoint address {:#x}", breakpoint_addr);
                        }
                    } else {
                        println!("Set breakpoint {} at {:#x}", self.breakpoints.len(), breakpoint_addr);
                        self.breakpoints.insert(breakpoint_addr, 0);
                    }
                }
            }
        }
    }

    fn inferior_cont(&mut self) {
        if let Some(inf) = &mut self.inferior {
            match inf.cont() {
                Ok(status) => {
                    match status {
                        Status::Exited(exit_code) => {
                            println!("Process exited with exit code: {}", exit_code);
                            self.inferior = None;
                        },
                        Status::Stopped(signal, ip) => {
                            println!("Process stopped by signal {} at 0x{:X}", signal, ip);
                            let line = self.debug_data.get_line_from_addr(rip);
                            let func = self.debug_data.get_function_from_addr(rip);
                            if line.is_some() && func.is_some() {
                                println!("Stopped at {} ({})", func.unwrap(), line.unwrap());
                            } 
                        },
                        Status::Signaled(signal) => {
                            println!("Process got a signal, {}", signal)
                        },
                    };
                },
                Err(err) => {
                    println!("Failed to wake up inferior or execute. Got Error: {}", err);
                }
            }
        } else {
            println!("Error continuing subprocess");
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
