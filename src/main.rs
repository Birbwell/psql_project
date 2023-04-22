use inquire::{Password, Select, Text};
use postgres::{types::Type, Client, NoTls, Error};
use cli_table::*;

type Result<T> = std::result::Result<T, Error>;

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

    client.close()
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

    let Ok(b) = Select::new("Show Most Highest/Lowest Correlation?", vec!["Highest", "Lowest"]).prompt() else {
        return
    };

    let best = match b {
        "Highest" => "FIRST",
        "Lowest" => "LAST",
        _ => "FIRST",
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
        ORDER BY corr DESC
    ) AS c",
        selected_crime,
    );

    let Ok(r) = client.query(&query, &[]) else {
        println!("There was an issue running your query\n");
        return
    };

    let mut table_body = vec![];
    let mut idx = 1;
    for row in &r {
        let chain: String = row.get(0);
        let corr: f64 = row.get(1);
        match best {
            "FIRST" if idx <= 10 => {
                table_body.push(vec![idx.cell(), chain.cell(), corr.cell()]);
                idx += 1;
                if idx > 10 {
                    break
                }
            },
            "LAST" if idx >= r.len() - 10 => {
                table_body.push(vec![idx.cell(), chain.cell(), corr.cell()]);
                idx += 1;
            },
            _ => {
                idx += 1;
            }
        }
    }
    let table = table_body
        .table()
        .title(vec![
            "Rank".cell().bold(true),
            "Chain".cell().bold(true),
            format!("Correlation to Crime ({})", selected_c).cell().bold(true),
        ])
        .bold(true);

    let display = table.display().unwrap();
    println!("{}", display);
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

    let Ok(mut spec) = Text::new("What chain are you looking for:").prompt() else {
        return
    };

    spec = spec.trim().to_string();

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
        ORDER BY corr DESC
    ) AS c",
        selected_crime,
    );

    println!("Querying. This may take a bit...\n");

    let Ok(r) = client.query(&query, &[]) else {
        println!("There was an issue running your query");
        return
    };

    if r.len() == 0 {
        println!("There are no chains by that given name");
        return
    }

    let mut table_body = vec![];
    let mut rank = 1;
    for row in r {
        let chain: String = row.get(0);
        let corr: f64 = row.get(1);
        if chain.to_ascii_lowercase() == spec.to_ascii_lowercase() {
            table_body.push(vec![rank.cell(), chain.cell(), corr.cell()]);
        }
        rank += 1;
    }

    let table = table_body
        .table()
        .title(vec![
            "Rank".cell().bold(true),
            "Chain".cell().bold(true),
            format!("Correlation to Crime ({})", selected_c).cell().bold(true),
        ])
        .bold(true);

    let table_display = table.display().unwrap();

    println!("{}", table_display);
}

fn custom_query(client: &mut Client) {
    let Ok(query) = Text::new("Enter your query:").prompt() else {
        return
    };

    println!("Querying...\n");

    let Ok(r) = client.query(&query, &[]) else {
        eprintln!("There was an issue executing your query");
        eprintln!("Note: This application is only for querying. Statements that create/modify tables, or change the search path are not allowed.\n");
        return
    };

    if r.len() == 0 {
        eprintln!("There was an issue executing your query");
        eprintln!("Note: This application is only for querying. Statements that create/modify tables, or change the search pathare not allowed.\n");
        return;
    }

    let col = r[0].columns();
    let col_names = col.iter().map(|f| f.name()).collect::<Vec<&str>>();
    let mut table_body = vec![];
    for row in &r {
        let mut line = vec![];
        let mut i = 0;
        for c in col {
            match *c.type_() {
                Type::VARCHAR | Type::TEXT | Type::NAME => {
                    line.push(row.get::<usize, String>(i).cell());
                }
                Type::INT2 => {
                    line.push(format!("{}", row.get::<usize, i16>(i)).cell());
                }
                Type::INT4 => {
                    line.push(format!("{}", row.get::<usize, i32>(i)).cell());
                }
                Type::INT8 => {
                    line.push(format!("{}", row.get::<usize, i64>(i)).cell());
                }
                Type::FLOAT4 => {
                    line.push(format!("{}", row.get::<usize, f32>(i)).cell());
                }
                Type::FLOAT8 => {
                    line.push(format!("{}", row.get::<usize, i64>(i)).cell());
                }
                _ => {
                    eprintln!("Type not yet implemented: {}", *c.type_());
                    return
                },
            }
            i += 1;
        }
        table_body.push(line);
    }

    let table = table_body
        .table()
        .title(col_names.iter().map(|f| f.cell().bold(true)).collect::<Vec<CellStruct>>())
        .bold(true);

    let table_display = table.display().unwrap();

    println!("{}", table_display);
}
