mod config;
mod pdf;

use pdf::reader::PdfReader;

use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        println!("========================================");
        println!("       PDF zu JSON Konverter");
        println!("========================================");
        println!();
        println!("Verwendung:");
        println!("  main.exe datei.pdf");
        println!("  main.exe -f datei.pdf -o ergebnis.json");
        println!();
        println!("Du kannst eine PDF-Datei auch auf die EXE-Datei ziehen.");

        let mut input = String::new();

        print!("\nDrücke Enter zum Schließen...");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut input);

        return;
    }

    let (pdf_path, json_path) = match parse_args(&args) {
        Ok(values) => values,
        Err(error) => {
            eprintln!("[Fehler]: {}", error);
            return;
        }
    };

    match PdfReader::open(&pdf_path) {
        Ok(pdf) => {
            match pdf.version() {
                Ok(version) => {
                    println!("[Info]: PDF-Version: {}", version);
                }
                Err(error) => {
                    eprintln!("[Warnung]: {}", error);
                }
            }

            println!("[Info]: Dateigröße: {} Bytes", pdf.size());
            println!("[Info]: Quelldatei: {}", pdf.path);
            println!("[Info]: Ausgabedatei: {}", json_path);

            println!(
                "[Hinweis]: Die PDF-Textanalyse und JSON-Erstellung \
                 sind noch nicht implementiert."
            );
        }
        Err(error) => {
            eprintln!("[Fehler]: {}", error);
        }
    }

    let mut input = String::new();

    print!("\nDrücke Enter zum Schließen...");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut input);
}

fn parse_args(args: &[String]) -> Result<(String, String), String> {
    if args.iter().any(|arg| arg.starts_with('-')) {
        let mut file = None;
        let mut output = String::from("ergebnis.json");

        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-f" | "--file" => {
                    i += 1;

                    if i >= args.len() {
                        return Err("Nach -f fehlt der PDF-Dateipfad.".to_string());
                    }

                    file = Some(args[i].clone());
                }

                "-o" | "--output" => {
                    i += 1;

                    if i >= args.len() {
                        return Err("Nach -o fehlt der JSON-Dateipfad.".to_string());
                    }

                    output = args[i].clone();
                }

                other => {
                    return Err(format!("Unbekannter Parameter: {}", other));
                }
            }

            i += 1;
        }

        let file = file.ok_or("Der Parameter -f ist erforderlich.")?;

        Ok((file, output))
    } else {
        let file = args.join(" ");

        let path = Path::new(&file);

        let output = path.with_extension("json");

        Ok((
            file,
            output.to_string_lossy().into_owned(),
        ))
    }
}