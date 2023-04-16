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
            "Search For Correlation",
            "Custom Query",
            "Quit"
        ]).prompt() else {
            break
        };

        match choice {
            "Search For Correlation" => search_db_for_correlation(&mut client),
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
    let Ok(choice) = Text::new("Enter the chain you're looking for:").prompt() else {
        return
    };

    let query = "with chains as (
      SELECT * from chain_count
      WHERE chain_count.chain IN (SELECT cc.chain FROM chain_count AS cc group BY chain HAVING COUNT(*)>19)
      AND chain_count.chain IN (SELECT cc.chain FROM (SELECT DISTINCT chain_count.chain AS chain, chain_count.count AS count FROM chain_count) AS cc GROUP BY chain HAVING COUNT(*)>1)
    )
    SELECT c, chain
    FROM (SELECT CORR(CAST(crime_1.crime_total AS DOUBLE PRECISION) / crime_1.population, chains.count) as c, chains.chain
    FROM crime_1, chains
    WHERE chains.county_id = crime_1.county_id
    GROUP BY chains.chain
    ORDER BY c desc) as corrs";

    let Ok(r) = client.query(query, &[]) else {
        println!("There was an issue running your query");
        return
    };

    for row in r {
        let c: String = row.get(1);
        if choice == c {
            let corr: f64 = row.get(0);
            println!("{} | {}", choice, corr);
        }
    }
}

fn custom_query(client: &mut Client) {
    let Ok(query) = Text::new("Enter your query:").prompt() else {
        return
    };
    let Ok(r) = client.query(&query, &[]) else {
        println!("There was something wrong with your query");
        return
    };
    for row in r {
        let mut line = String::new();
        for n in 0..row.len() {
            let v: String = row.get(n);
            line = format!("{} {}", line, v);
        }
        println!("{}", line);
    }
}
