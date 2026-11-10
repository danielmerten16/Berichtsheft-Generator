mod config;

use config::json::Config;

fn main() -> Result<(), String> {
    let json = Config::open("data.json")?;

    let benutzer = json
        .get_string("user.name")
        .ok_or("Benutzername nicht gefunden")?;

    println!("{}", benutzer);

    Ok(())
}