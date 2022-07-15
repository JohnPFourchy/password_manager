use::std::collections::HashMap;
use::std::path::Path;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use arboard::Clipboard;
use std::io::Write;
use std::str;
use std::fs::OpenOptions;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

/*
 * Once the program is exited, it removes and recreates the file of saved passwords
 * to elimate any potential discrepencies that may occur if improperly using the 
 * program.
*/
pub fn restore_files(map: &mut HashMap<String, String>) {
    // remove the potentially outdated file
    remove_file("data/saved.txt");

    // create the new upto date file
    create_file("data/saved.txt");

    for (name, pw) in map {
        let strtowrite: String = format!("{} {}", name.to_string(), pw);
        write_to_file(strtowrite, "data/saved.txt");
    }
}

/*
 * Handles input into the program. Reads in lines from
 * stdin and passes them off to a function which handles
 * individual lines.
*/
pub fn input_loop(map: &mut HashMap<String, String>) {
    let mut line = String::new();
    println!("Passman: type quit to exit program");
    while line.trim() != "quit" {
        line.clear();
        print!("   => ");
        // flush the buffer and read the line
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut line).unwrap();

        let splitline = line.trim().split(" ");
        let arguments: Vec<&str> = splitline.collect();
        if line.trim() != "quit" {
            argument_handling(&arguments, map);
        }
    }
}

/*
* Handles the creation and storing of the master password.
* Asks to create a master password if one does not exist,
* or validates that the inputed password matches.
*/
pub fn master_password() {
    let path: bool = Path::new("data/master.txt").exists();

    if path == false {
        create_file("data/master.txt");
        // Ask for user to create master password
        println!("Enter a master password. This is needed to login. Don't forget it!");
        print!("   => ");
        // flush the buffer and read the line
        std::io::stdout().flush().unwrap();
        let password = rpassword::read_password().unwrap();
        // hash and store the pw
        let encoded: String = hash(&password);
        // write to file
        write_to_file(encoded, "data/master.txt");
    } else {
        // read pw from the file, unhash, and verify that input is the same, else don't let in
        let file = File::open("data/master.txt").expect("file can't be opened");
        let reader = BufReader::new(file);

        // this should only run once - safe to put rest of code in for loop
        for line in reader.lines() {
            let unhashedpw = unhash(line.unwrap());

            // get password from user
            println!("Enter your master password to login.");
            print!("   => ");
            std::io::stdout().flush().unwrap();
            let password = rpassword::read_password().unwrap();

            // exits on mismatch to help prevent brute force attacks
            if password != unhashedpw {
                println!("Master password mismatch ... exiting");
                std::process::exit(1);
            }
        }
    }
}

/*
 * Handles saved password loading and saved password file creation.
*/
pub fn saved_data_handling(map: &mut HashMap<String, String>) {
    let path: bool = Path::new("data/saved.txt").exists();
    // change to match later
    if path == false {
        // create the file if it does not exist
        create_file("data/saved.txt");
    } else {
        let file = open_file("data/saved.txt");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // loop through file and add to hashmap
                    let sline = line;
                    let sline = sline.split(" ");
                    let vec: Vec<&str> = sline.collect();
                    map.insert(vec[0].to_string(), vec[1].to_string());
                },
                Err(_e) => {
                    println!("Problem reading internal file. Exiting");
                    std::process::exit(1);
                }
            };
        }
    }
}

/*
* Handles a line of user input, based on how long the line is it is passed
* to various helper functions to handle each line.
*/
fn argument_handling(args: &Vec<&str>, map: &mut HashMap<String, String>) {
    // add command
    if args.len() >= 3 {
        three_arg_handling(args, map);
    // del and cpy commands
    } else if args.len() == 2 {
        two_arg_handling(args, map);
    // lst command
    } else {
        one_arg_handling(args, map);
    }
}

// From here down are logic functions used to handle different user inputs to the program

/*
 * Handles user inputs that utilize the add command
*/
fn three_arg_handling(args: &Vec<&str>, map: &mut HashMap<String, String>) {
    let command: &str = &args[0];
    let username: &str = &args[1];
    let password: &str = &args[2..].join(" ");
    
    // add command
    if command == "add" {
        if map.contains_key(username) {
            println!("The name {} already exists! Please delete and recreate it if you wish to change the password.", username);
        } else {
            // encode password
            let encoded: String = hash(password);
            // write to file
            let strtowrite: String = format!("{} {}", username.to_string(), encoded);
            write_to_file(strtowrite, "data/saved.txt");
            // add to hashmap
            map.insert(username.to_string(), encoded.to_string());
        }
    } else {
        println!("Incorrect command: Did you mean \"add {} {}\"?", username, password);
    }
}

/*
 * Handles user inputs that utilize the copy or delete commands
*/
fn two_arg_handling(args: &Vec<&str>, map: &mut HashMap<String, String>) {
    let command: &str = &args[0];
    let username: &str = &args[1];

    if command == "del" {
        map.remove(&username.to_string());
        println!("Removed {}.", username);
    } else if command == "cpy" {
        if map.contains_key(username) {
            let string: String = map.get(username).unwrap().to_string();
            // set text to clipboard
            let mut clipboard = Clipboard::new().unwrap();
            let decoded: String = unhash(string);
            clipboard.set_text(decoded.into()).unwrap();
        } else {
            println!("{} does not exist! Type \"lst\" to see options.", username);
        }
    } else {
        println!("Incorrect command: Did you mean \"del {}\" or \"cpy {}\"?", username, username);
    }
}

/* 
* Handles user input that utilizes the list command
*/
fn one_arg_handling(args: &Vec<&str>, map: &mut HashMap<String, String>) {
    let command: &str = &args[0];
        
    if command == "lst" {
        for (name, _pw) in map {
            println!("{}", name);
        }
    } else {
        println!("Incorrect command: Did you mean \"lst\"?");
    }
}

// From here down are helper functions that handle common operations and their errors all in one place

/* 
* Creates a file along with handling errors it could throw
*/
fn create_file(file_path: &str) {
    let f = File::create(file_path);

    let _f = match f {
        Ok(file) => file,
        Err(_e) => {
            println!("Problem creating an internal file. Exiting the program.");
            std::process::exit(1);
        }
    };
}

/*
 * Handles writing to a file
*/
fn write_to_file(str_to_write: String, file_path: &str) {
    // Should not fail as file always exists before this is called
    let mut f = OpenOptions::new().write(true).append(true).open(file_path).unwrap();
    writeln!(f, "{}", str_to_write.trim()).unwrap();
}

/*
 * Handles opening a file along with any errors it could throw
*/
fn open_file(file_path: &str) -> File {
    let f = File::open(file_path);

    let f = match f {
        Ok(file) => file,
        Err(_e) => {
            println!("Problem opening an internal file. Exiting the program.");
            std::process::exit(1);
        }
    };

    f
}

/*
 * Handles removing from a file along with any errors it could throw
*/
fn remove_file(file_path: &str) {
    let f = fs::remove_file(file_path);

    match f {
        Ok(_) => (),
        Err(_e) => {
            println!("Problem removing an internal file. Exiting.");
            std::process::exit(1);
        }
    };
}

/* 
* Hashes a password entered by a user with a 256 bit key
*/
fn hash(pass: &str) -> String {
    let mc = new_magic_crypt!("magickey", 256);
    let base64 = mc.encrypt_str_to_base64(pass);
    base64
}

/*
 * Unhashes a hashed password
*/
fn unhash(encoded: String) -> String {
    let mc = new_magic_crypt!("magickey", 256);
    let decoded: String = mc.decrypt_base64_to_string(&encoded).unwrap();
    decoded
}
