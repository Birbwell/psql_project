use inquire::{Password, Select, Text};
use postgres::{Client, NoTls, types::Type};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut client = loop {
        let Ok(user) = Text::new("Enter your psql username:").prompt() else {
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
                let t = e.as_db_error().unwrap().message();
                println!("{}", t);
            }
        };
    };

    client.execute("set search_path to s23_group36", &[])?;

    loop {
        let Ok(choice) = Select::new("Main Menu", vec![
            "Search For Correlation",
            "Search By Crime",
            "Custom Query",
            "Quit"
        ]).prompt() else {
            break
        };

        match choice {
            "Search For Correlation" => search_db_for_correlation(&mut client),
            "Search By Crime" => search_by_crime(&mut client),
            "Custom Query" => custom_query(&mut client),
            "Quit" => break,
            _ => {
                println!("Invalid Input");
            }
        };
    }

    Ok(())
}

fn search_db_for_correlation(client: &mut Client) {
    println!("Under Construction");
    return;
    
    let Ok(choice) = Text::new("Enter the chain you're looking for:").prompt() else {
        return
    };

    let query = "";

    let Ok(r) = client.query(query, &[]) else {
        println!("There was an issue running your query");
        return
    };

    for row in r {
        let c: String = row.get(1);
        if choice == c {
            let corr: f64 = row.get(0);
            println!("{} | {}", choice, corr);
            return
        }
    }

    println!("There were no results for that query");
}

fn custom_query(client: &mut Client) {
    let Ok(query) = Text::new("Enter your query:").prompt() else {
        eprintln!("There was an issue getting your query");
        return
    };

    let Ok(r) = client.query(&query, &[]) else {
        eprintln!("There was an issue executing your query");
        return
    };

    let col = r[0].columns();
    let mut line = String::new();
    for c in col {
        line = format!("{} {}", line, capitalize(c.name()));
    }
    line.push('\n');
    for _ in 0..line.len() {
        line.push('-');
    }
    println!("{}", line);
    for row in &r {
        let mut line = String::new();
        let mut i = 0;
        for c in col {
            match *c.type_() {
                Type::VARCHAR | Type::TEXT => {
                    line = format!("{} {}", line, row.get::<usize, String>(i))
                },
                Type::INT2 => {
                    line = format!("{} {}", line, row.get::<usize, i16>(i));
                },
                Type::INT4 => {
                    line = format!("{} {}", line, row.get::<usize, i32>(i));
                },
                Type::INT8 => {
                    line = format!("{} {}", line, row.get::<usize, i64>(i));
                },
                Type::FLOAT4 => {
                    line = format!("{} {}", line, row.get::<usize, f32>(i));
                },
                Type::FLOAT8 => {
                    line = format!("{} {}", line, row.get::<usize, f64>(i));
                },
                _ => println!("Not Yet Implemented! {}", *c.type_())
            }
            i += 1;
        }
        println!("{}\n", line)
    }
}

fn capitalize(s: &str) -> String {
    let mut v = s.chars().collect::<Vec<char>>();
    v[0] = v[0].to_ascii_uppercase();
    v.into_iter().collect::<String>()
}

fn search_by_crime(client: &mut Client) {
    let crimes = vec![
        "Larceny",
        "Burglary",
        "Murder",
        "Rape",
        "Arson"
    ];
    let Ok(selected_crime) = Select::new("What type of crime?", crimes).prompt() else {
        eprintln!("Something went wrong");
        return
    };

    println!("Under Construction")
}
