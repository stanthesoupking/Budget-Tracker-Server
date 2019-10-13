use std::fs::File;
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use toml;

const CONFIG_PATH: &str = "./config.toml";

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub binding: String,
    pub ssl_key_path: String,
    pub ssl_cert_path: String,
    pub secret: String
}

impl Config {
    fn generate_new_config() -> Config {
        println!(" === Initial Configuration ===");

        let mut binding = String::new();
        let mut ssl_key_path = String::new();
        let mut ssl_cert_path = String::new();
        let mut secret = String::new();

        // Get HTTPS binding address
        {
            println!("HTTPS Server Binding Address (e.g. localhost:443): ");
            let buffer = std::io::stdin().read_line(&mut binding).unwrap();
            binding.pop(); // Remove trailing \n
        }

        // Get SSL key path
        {
            println!("SSL Private Key Path (e.g. key.pem): ");
            let buffer = std::io::stdin().read_line(&mut ssl_key_path).unwrap();
            ssl_key_path.pop(); // Remove trailing \n

        }

        // Get SSL certificate path
        {
            println!("SSL Certificate Path (e.g. cert.pem): ");
            let buffer = std::io::stdin().read_line(&mut ssl_cert_path).unwrap();
            ssl_cert_path.pop(); // Remove trailing \n
        }

        // Get server secret
        {
            println!("Server Secret: ");
            let buffer = std::io::stdin().read_line(&mut secret).unwrap();
            secret.pop(); // Remove trailing \n
        }

        Config {
            binding,
            ssl_key_path,
            ssl_cert_path,
            secret
        }
    }

    pub fn load() -> Config {
        match File::open(CONFIG_PATH) {
            Ok(mut f) => {
                let mut file_contents = String::new();
                f.read_to_string(&mut file_contents).unwrap();

                let c: Config = match toml::from_str(file_contents.as_str()) {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Error: config file is corrupt.");
                        println!("Please delete '{}' and try again.", CONFIG_PATH);
                        panic!("CORRUPT CONFIG FILE");
                    }
                };

                c
            },
            Err(_) => { 
                // Generate new config
                let mut f = File::create(CONFIG_PATH).unwrap();

                let c = Config::generate_new_config();

                // Write to file
                f.write_all(toml::to_string(&c).unwrap().as_bytes()).unwrap();

                c
            }
        }
    }
}