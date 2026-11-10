mod config;

use config::json::{Config, JsonValue};

fn main() -> Result<(), String> {
    let mut json = Config::open("data.json")?;

    json.set(
        "user.name",
        JsonValue::string("Daniel"),
    )?;

    json.set(
        "user.settings.theme",
        JsonValue::string("dark"),
    )?;

    if let Some(name) = json.get_string("user.name") {
        println!("Benutzer: {}", name);
    }

    json.print()?;

    Ok(())
}