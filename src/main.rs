use inquire::{Password, Select, Text};
use postgres::{Client, NoTls};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {

    let mut client = loop {
        let Ok(user) = Text::new("Enter the username:").prompt() else {
            return Ok(())
        };
        let Ok(pass) = ({
            let mut p = Password::new("Enter the password:");
            p.enable_confirmation = false;
            p.prompt()
        }) else {
            return Ok(())
        };
        let cred = format!("postgres://{}:{}@codd.mines.edu:5433/csci403", user, pass);

        match Client::connect(cred.as_str(), NoTls) {
            Ok(c) => break c,
            Err(e) => {
                println!("There was an error: {:?}\nPlease try again.", e);
            }
        };
    };

    client.execute("set search_path to s23_group36", &[])?;

    loop {
        let Ok(choice) = Select::new("Main Menu", vec![
            "Search",
            "Quit"
        ]).prompt() else {
            break
        };

        match choice {
            "Search" => search_db(&mut client),
            "Quit" => break,
            _ => {
                println!("Invalid Input");
            }
        };
    }

    Ok(())
}

fn search_db(client: &mut Client) {
    let choice = Text::new("Enter the chain you wish to search for:").prompt().unwrap();
    for row in client.query("select distinct county, distinct state from chain where chain = $1", &[&choice.as_str()]).unwrap() {
        let county: String = row.get(0);
        let state: String = row.get(1);
        println!("State: {}, County: {}", state, county);
    }
}
