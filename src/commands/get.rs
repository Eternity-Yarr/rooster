// Copyright 2014-2017 The Rooster Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::super::getopts;
use super::super::password;
use super::super::clipboard::{copy_to_clipboard, paste_keys};
use std::io::Write;
use std::io::stdin;
use std::ops::Deref;

pub fn callback_help() {
    println!("Usage:");
    println!("    rooster get -h");
    println!("    rooster get <query>");
    println!("");
    println!("Example if you want to get your Google password:");
    println!("    rooster get google");
    println!("    rooster get ggl # fuzzy matching works too");
    println!("");
    println!("If multiple passwords match your search, you will be asked to choose.")
}

pub fn check_args(matches: &getopts::Matches) -> Result<(), i32> {
    if matches.free.len() < 2 {
        println_err!("Woops, seems like the app name is missing here. For help, try:");
        println_err!("    rooster get -h");
        return Err(1);
    }

    Ok(())
}

pub fn callback_exec(matches: &getopts::Matches,
                     store: &mut password::v2::PasswordStore)
                     -> Result<(), i32> {
    check_args(matches)?;

    let show = matches.opt_present("show");

    let query = matches.free[1].clone();

    let passwords = store.search_passwords(query.as_str());

    if passwords.len() == 0 {
        println_stderr!("I can't find any passwords for \"{}\"", query);
        return Ok(());
    }

    if let Some(password) = passwords
           .iter()
           .find(|p| p.name.to_lowercase() == query.to_lowercase()) {
        if show {
            println_ok!("Alright! Here is your password for {}: {}",
                        password.name,
                        password.password.deref());
        } else {
            if copy_to_clipboard(password.password.deref()).is_err() {
                println_ok!("Hmm, I tried to copy your new password to your clipboard, but \
                             something went wrong. You can see it with `rooster get '{}' --show`",
                            password.name);
            } else {
                println_ok!("Alright! You can paste your {} password anywhere with {}.",
                            password.name,
                            paste_keys());
            }
        }

        return Ok(());
    }

    let longest_app_name = passwords
        .iter()
        .fold(0, |acc, p| if p.name.len() > acc {
            p.name.len()
        } else {
            acc
        });

    let i_width = ((passwords.len() as f64).log10() + 1 as f64).floor() as usize;

    println_stderr!("");
    for (i, p) in passwords.iter().enumerate() {
        println!("{:i_width$} {:app_name_width$} {:30}",
                 i + 1,
                 p.name,
                 p.username,
                 i_width = i_width,
                 app_name_width = longest_app_name);
    }
    println_stderr!("");

    if show {
        println_stderr!("Which password would you like to see? ");
    } else {
        println_stderr!("Which password would you like me to copy to your clipboard? ");
    }


    // Read the index from the command line and convert to a number
    let mut line = String::new();
    let mut index: usize;
    loop {
        line.clear();
        match stdin().read_line(&mut line) {
            Ok(_) => {}
            Err(err) => {
                println_err!("I couldn't read that (reason: {}).", err);
                println_err!("Would you mind trying that out again please?");
            }
        }

        index = match line.trim().parse() {
            Ok(index) => index,
            Err(err) => {
                println_err!("This isn't a valid number (reason: {}). Please give me a number \
                              between 1 and {}:",
                             err,
                             passwords.len());
                continue;
            }
        };

        if index == 0 || index > passwords.len() {
            println_err!("Sorry, I need a number between 1 and {}. Let's try this again:",
                         passwords.len());
            continue;
        }

        break;
    }

    let app_name = passwords[index - 1].name.as_str();
    // This whould never fail, since we've just checked that this password exists
    let password = store.get_password(app_name).unwrap();
    if show {
        println_ok!("Alright! Here is your password for {}: {}",
                    password.name,
                    password.password.deref());
    } else {
        if copy_to_clipboard(password.password.deref()).is_err() {
            println_ok!("Hmm, I tried to copy your new password to your clipboard, but \
                         something went wrong. You can see it with `rooster get '{}' --show`",
                        password.name);
        } else {
            println_ok!("Alright! You can paste your {} password anywhere with {}.",
                        password.name,
                        paste_keys());
        }
    }

    Ok(())
}
