use std::io;
use std::io::Write;
use std::collections::HashMap;

fn main() {
    let mut data = HashMap::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut command = String::new();

        io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");

        process_command(&mut data, &command);
    }
}

fn process_command(data: &mut HashMap<String, Vec<u8>>, command: &str) {
    let v: Vec<&str> = command.split_whitespace().collect();

    let command = v[0];

    match command {

        "SET" if v.len() == 3 => {
            data.insert(v[1].to_string(), v[2].as_bytes().to_vec());
            println!("OK");
        },


        "GET" if v.len() == 2 => {
            if let Some(value) = data.get(v[1]) {
                println!("\"{}\"", String::from_utf8_lossy(value));
            } else {
                println!("ERROR: KEY NOT FOUND");
            }
        },

        "DELETE" if v.len() == 2 => {
            if let Some(_) = data.remove(v[1]) {
                println!("OK");
            } else {
                println!("ERROR: KEY NOT FOUND");
            }
        },

        "EXISTS" if v.len() == 2 => {
            if data.contains_key(v[1]) {
                println!("1");
            } else {
                println!("0");
            }
        },

        _ => {
            println!("Command not recognized. Try again");
        },
    }
}
