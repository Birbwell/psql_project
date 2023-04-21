use inquire::{Password, Select, Text};
use postgres::{types::Type, Client, NoTls};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut client = loop {
        let Ok(user) = Text::new("Enter your psql username:").prompt() else {
            return Ok(())
        };
        let Ok(pass) = ({
            let mut p = Password::new("Enter your psql password:");
            p.enable_confirmation = false;
            p.prompt()
        }) else {
            return Ok(())
        };
        let cred = format!("postgres://{}:{}@codd.mines.edu:5433/csci403", user, pass);

        match Client::connect(&cred, NoTls) {
            Ok(c) => break c,
            Err(e) => {
                let t = e.as_db_error().unwrap().message();
                println!("Error: {}", t);
            }
        };
    };

    client.execute("set search_path to s23_group36", &[])?;

    loop {
        let Ok(choice) = Select::new("Main Menu", vec![
            "Get Top-10 Chains",
            "Search For Specific Correlation",
            "Custom Query",
            "Quit"
        ]).prompt() else {
            break
        };

        match choice {
            "Get Top-10 Chains" => fetch_top_ten_restaurants(&mut client),
            "Search For Specific Correlation" => fetch_specific_correlation(&mut client),
            "Custom Query" => custom_query(&mut client),
            "Quit" => break,
            _ => {
                println!("Invalid Input");
            }
        };
    }

    client.close()?;
    Ok(())
}

fn fetch_top_ten_restaurants(client: &mut Client) {
    let crimes = vec![
        "All Crime",
        "Murder",
        "Rape",
        "Robbery",
        "Aggravated Assault",
        "Burglary",
        "Larceny",
        "Motor Vehicle Theft",
        "Arson",
    ];
    let Ok(selected_c) = Select::new("What type of crime?", crimes).prompt() else {
        return
    };

    let selected_crime = match selected_c {
        "All Crime" => "crime_total",
        "Murder" => "murder",
        "Rape" => "rape",
        "Robbery" => "robbery",
        "Aggravated Assault" => "assault",
        "Burglary" => "burglary",
        "Larceny" => "larceny",
        "Motor Vehicle Theft" => "motor_theft",
        "Arson" => "arson",
        _ => "crime_total",
    };

    let Ok(b) = Select::new("Show Best/Worst?", vec!["Best", "Worst"]).prompt() else {
        return
    };

    let best = match b {
        "Best" => "ASC",
        "Worst" => "DESC",
        _ => "ASC",
    };

    println!("Querying. This may take a bit...\n");
    let query = format!(
        "SELECT *
    FROM (
        SELECT name,
        (
            SELECT CORR(
                COALESCE((
                SELECT count
                FROM chain_count AS count
                WHERE count.chain_id = chain_total_count.chain_id
                AND count.county_id = county.id
                ), 0),
                CAST(crime_1.{} AS DOUBLE PRECISION) / crime_1.population
            ) AS c
            FROM county
            JOIN crime_1 ON county.id = crime_1.county_id
        ) AS corr
        FROM chain_total_count
        JOIN chain_name ON chain_total_count.chain_id = chain_name.id
        WHERE chain_total_count.total_count > 10
        ORDER BY corr {}
    ) AS c
    FETCH FIRST 10 ROWS ONLY",
        selected_crime, best
    );

    let Ok(r) = client.query(&query, &[]) else {
        println!("There was an issue running your query");
        return
    };

    println!("Position: Chain | Correlation to Crime ({})\n", selected_c);
    let mut idx = 1;
    for row in r {
        let chain: String = row.get(0);
        let corr: f64 = row.get(1);
        println!("{}: {} | {}", idx, chain, corr);
        idx += 1;
    }
    println!();
}

fn fetch_specific_correlation(client: &mut Client) {
    let crimes = vec![
        "All Crime",
        "Murder",
        "Rape",
        "Robbery",
        "Aggravated Assault",
        "Burglary",
        "Larceny",
        "Motor Vehicle Theft",
        "Arson",
    ];
    let Ok(selected_c) = Select::new("What type of crime?", crimes).prompt() else {
        return
    };

    let selected_crime = match selected_c {
        "All Crime" => "crime_total",
        "Murder" => "murder",
        "Rape" => "rape",
        "Robbery" => "robbery",
        "Aggravated Assault" => "assault",
        "Burglary" => "burglary",
        "Larceny" => "larceny",
        "Motor Vehicle Theft" => "motor_theft",
        "Arson" => "arson",
        _ => "crime_total",
    };

    let query = format!(
        "SELECT *
    FROM (
        SELECT name,
        (
            SELECT CORR(
                COALESCE((
                SELECT count
                FROM chain_count AS count
                WHERE count.chain_id = chain_total_count.chain_id
                AND count.county_id = county.id
                ), 0),
                CAST(crime_1.{} AS DOUBLE PRECISION) / crime_1.population
            ) AS c
            FROM county
            JOIN crime_1 ON county.id = crime_1.county_id
        ) AS corr
        FROM chain_total_count
        JOIN chain_name ON chain_total_count.chain_id = chain_name.id
        WHERE chain_total_count.total_count > 10 AND name = $1
        ORDER BY corr desc
    ) AS c",
        selected_crime
    );

    let Ok(mut spec) = Text::new("What chain are you looking for:").prompt() else {
        return
    };

    spec = capitalize(&spec);

    println!("Querying...\n");

    let Ok(r) = client.query(&query, &[&spec]) else {
        println!("There was an issue running your query");
        return
    };

    if r.len() == 0 {
        println!("There are no chains by that given name");
        return;
    }

    println!("Chain | Correlation to Crime ({})\n", selected_c);
    for row in r {
        let chain: String = row.get(0);
        let corr: f64 = row.get(1);
        println!("{} | {}", chain, corr);
    }
    println!();
}

fn custom_query(client: &mut Client) {
    let Ok(query) = Text::new("Enter your query:").prompt() else {
        eprintln!("There was an issue getting your query");
        return
    };

    println!("Querying...\n");

    let Ok(r) = client.query(&query, &[]) else {
        eprintln!("There was an issue executing your query");
        eprintln!("Note: This application is only for querying. Statements that modify the tables are not allowed");
        return
    };

    if r.len() == 0 {
        eprintln!("There was an issue executing your query");
        eprintln!("Note: This application is only for querying. Statements that modify or create tables are not allowed");
        return;
    }

    let col = r[0].columns();
    let mut line = String::new();
    for c in col {
        line = format!("{} {}", line, c.name());
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
                Type::VARCHAR | Type::TEXT | Type::NAME => {
                    line = format!("{} | {}", line, row.get::<usize, String>(i))
                }
                Type::INT2 => {
                    line = format!("{} | {}", line, row.get::<usize, i16>(i));
                }
                Type::INT4 => {
                    line = format!("{} | {}", line, row.get::<usize, i32>(i));
                }
                Type::INT8 => {
                    line = format!("{} | {}", line, row.get::<usize, i64>(i));
                }
                Type::FLOAT4 => {
                    line = format!("{} | {}", line, row.get::<usize, f32>(i));
                }
                Type::FLOAT8 => {
                    line = format!("{} | {}", line, row.get::<usize, f64>(i));
                }
                Type::NUMERIC => {
                    println!("Sorry, but due to the nature of Numeric types, this query will fail");
                    return;
                }
                _ => println!("Type not yet implemented: {}", *c.type_()),
            }
            i += 1;
        }
        println!("{}", line)
    }
    println!();
}

fn capitalize(s: &str) -> String {
    let v = s
        .split_ascii_whitespace()
        .map(|f| f.chars().collect::<Vec<char>>())
        .collect::<Vec<Vec<char>>>();
    let mut ret = Vec::<String>::new();
    for word in v {
        ret.push(match &word[..] {
            [first, rest @ ..] => {
                let t = first.to_ascii_uppercase();
                let r = rest
                    .iter()
                    .map(|f| f.to_ascii_lowercase())
                    .collect::<String>();
                format!("{}{}", t, r)
            }
            _ => "".to_owned(),
        })
    }
    ret.join(" ")
}
