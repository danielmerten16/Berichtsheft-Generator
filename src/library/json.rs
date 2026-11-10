#![allow(non_snake_case)]

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================
// JSON DATENTYPEN
// ============================================================

#[derive(Debug, Clone, PartialEq)]

// Definiert die verschiedenen JSON-Typen
pub enum JsonValue {
    // Definiert den Dateipfad
    Object(BTreeMap<String, JsonValue>),
    // Definiert ein JSON-Array
    Array(Vec<JsonValue>),
    // Definiert einen JSON-String
    String(String),
    // Definiert eine JSON-Nummer
    Number(f64),
    // Definiert einen JSON-Boolean
    Bool(bool),
    // Definiert einen JSON-Null-Wert
    Null,
}

impl JsonValue {
    pub fn object() -> Self {
        JsonValue::Object(BTreeMap::new())
    }

    pub fn array() -> Self {
        JsonValue::Array(Vec::new())
    }

    pub fn string(value: impl Into<String>) -> Self {
        JsonValue::String(value.into())
    }

    pub fn number(value: f64) -> Self {
        JsonValue::Number(value)
    }

    pub fn boolean(value: bool) -> Self {
        JsonValue::Bool(value)
    }

    pub fn null() -> Self {
        JsonValue::Null
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(value) => Some(*value),
            _ => None,
        }
    }
}

// ============================================================
// JSON PARSER
// ============================================================

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos..)?.chars().next()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while matches!(
            self.peek(),
            Some(' ' | '\n' | '\r' | '\t')
        ) {
            self.next();
        }
    }

    fn error(&self, message: &str) -> String {
        format!(
            "JSON-Fehler bei Position {}: {}",
            self.pos,
            message
        )
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        self.skip_whitespace();

        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(self.error(&format!(
                "'{}' erwartet",
                expected
            ))),
        }
    }

    fn consume_literal(&mut self, literal: &str) -> Result<(), String> {
        if self.input[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(self.error(&format!(
                "'{}' erwartet",
                literal
            )))
        }
    }

    fn parse(mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        let value = self.parse_value()?;

        self.skip_whitespace();

        if self.pos != self.input.len() {
            return Err(self.error(
                "Unerwartete Zeichen nach dem JSON-Wert"
            ));
        }

        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => {
                Ok(JsonValue::String(self.parse_string()?))
            }
            Some('t') => {
                self.consume_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some('f') => {
                self.consume_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some('n') => {
                self.consume_literal("null")?;
                Ok(JsonValue::Null)
            }
            Some('-' | '0'..='9') => self.parse_number(),
            Some(_) => Err(self.error("Ungültiger JSON-Wert")),
            None => Err(self.error("Unerwartetes Dateiende")),
        }
    }

    // --------------------------------------------------------
    // OBJECT
    // --------------------------------------------------------

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect('{')?;

        let mut object = BTreeMap::new();

        self.skip_whitespace();

        if self.peek() == Some('}') {
            self.next();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_whitespace();

            if self.peek() != Some('"') {
                return Err(self.error(
                    "Objektschlüssel muss ein String sein"
                ));
            }

            let key = self.parse_string()?;

            self.skip_whitespace();
            self.expect(':')?;

            let value = self.parse_value()?;

            object.insert(key, value);

            self.skip_whitespace();

            match self.next() {
                Some(',') => continue,
                Some('}') => break,
                _ => {
                    return Err(self.error(
                        "',' oder '}' erwartet"
                    ));
                }
            }
        }

        Ok(JsonValue::Object(object))
    }

    // --------------------------------------------------------
    // ARRAY
    // --------------------------------------------------------

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect('[')?;

        let mut array = Vec::new();

        self.skip_whitespace();

        if self.peek() == Some(']') {
            self.next();
            return Ok(JsonValue::Array(array));
        }

        loop {
            array.push(self.parse_value()?);

            self.skip_whitespace();

            match self.next() {
                Some(',') => continue,
                Some(']') => break,
                _ => {
                    return Err(self.error(
                        "',' oder ']' erwartet"
                    ));
                }
            }
        }

        Ok(JsonValue::Array(array))
    }

    // --------------------------------------------------------
    // STRING MIT ESCAPE- UND UNICODE-UNTERSTÜTZUNG
    // --------------------------------------------------------

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect('"')?;

        let mut result = String::new();

        loop {
            match self.next() {
                Some('"') => return Ok(result),

                Some('\\') => {
                    match self.next() {
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some('/') => result.push('/'),
                        Some('b') => result.push('\u{0008}'),
                        Some('f') => result.push('\u{000C}'),
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),

                        Some('u') => {
                            let first = self.parse_hex4()?;

                            let codepoint = if (0xD800..=0xDBFF)
                                .contains(&first)
                            {
                                if self.next() != Some('\\')
                                    || self.next() != Some('u')
                                {
                                    return Err(self.error(
                                        "Ungültiges Unicode-Surrogat"
                                    ));
                                }

                                let second = self.parse_hex4()?;

                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err(self.error(
                                        "Ungültiges Unicode-Surrogat"
                                    ));
                                }

                                0x10000
                                    + (((first - 0xD800) as u32) << 10)
                                    + (second - 0xDC00) as u32
                            } else if (0xDC00..=0xDFFF).contains(&first) {
                                return Err(self.error(
                                    "Unerwartetes Unicode-Surrogat"
                                ));
                            } else {
                                first as u32
                            };

                            let ch = char::from_u32(codepoint)
                                .ok_or_else(|| {
                                    self.error("Ungültiges Unicode-Zeichen")
                                })?;

                            result.push(ch);
                        }

                        Some(_) => {
                            return Err(self.error(
                                "Ungültige Escape-Sequenz"
                            ));
                        }

                        None => {
                            return Err(self.error(
                                "Unvollständige Escape-Sequenz"
                            ));
                        }
                    }
                }

                Some(ch) if ch < '\u{0020}' => {
                    return Err(self.error(
                        "Steuerzeichen in einem String"
                    ));
                }

                Some(ch) => result.push(ch),

                None => {
                    return Err(self.error(
                        "String wurde nicht geschlossen"
                    ));
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, String> {
        let mut value: u16 = 0;

        for _ in 0..4 {
            let ch = self.next()
                .ok_or_else(|| self.error(
                    "Unvollständige Unicode-Sequenz"
                ))?;

            let digit = ch.to_digit(16)
                .ok_or_else(|| self.error(
                    "Ungültige Unicode-Sequenz"
                ))?;

            value = value * 16 + digit as u16;
        }

        Ok(value)
    }

    // --------------------------------------------------------
    // ZAHLEN
    // --------------------------------------------------------

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;

        while matches!(
            self.peek(),
            Some('-' | '+' | '.' | '0'..='9' | 'e' | 'E')
        ) {
            self.next();
        }

        let text = &self.input[start..self.pos];

        if !valid_json_number(text) {
            return Err(self.error("Ungültige JSON-Zahl"));
        }

        let number: f64 = text.parse()
            .map_err(|_| self.error("Zahl konnte nicht gelesen werden"))?;

        if !number.is_finite() {
            return Err(self.error(
                "Zahl liegt außerhalb des unterstützten Bereichs"
            ));
        }

        Ok(JsonValue::Number(number))
    }
}

// Prüft die JSON-Zahlengrammatik.
fn valid_json_number(text: &str) -> bool {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    if i < len && bytes[i] == b'-' {
        i += 1;
    }

    if i >= len {
        return false;
    }

    if bytes[i] == b'0' {
        i += 1;

        if i < len && bytes[i].is_ascii_digit() {
            return false;
        }
    } else if (b'1'..=b'9').contains(&bytes[i]) {
        i += 1;

        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        return false;
    }

    if i < len && bytes[i] == b'.' {
        i += 1;

        let start = i;

        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }

        if i == start {
            return false;
        }
    }

    if i < len && matches!(bytes[i], b'e' | b'E') {
        i += 1;

        if i < len && matches!(bytes[i], b'+' | b'-') {
            i += 1;
        }

        let start = i;

        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }

        if i == start {
            return false;
        }
    }

    i == len
}

// ============================================================
// JSON SERIALISIERUNG
// ============================================================

fn write_indent(output: &mut String, depth: usize) {
    for _ in 0..depth * 4 {
        output.push(' ');
    }
}

fn write_string(value: &str, output: &mut String) {
    output.push('"');

    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0008}' => output.push_str("\\b"),
            '\u{000C}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),

            ch if ch < '\u{0020}' => {
                output.push_str(&format!("\\u{:04x}", ch as u32));
            }

            ch => output.push(ch),
        }
    }

    output.push('"');
}

fn write_value(
    value: &JsonValue,
    output: &mut String,
    depth: usize,
) -> Result<(), String> {
    match value {
        JsonValue::Object(object) => {
            if object.is_empty() {
                output.push_str("{}");
                return Ok(());
            }

            output.push_str("{\n");

            for (index, (key, value)) in object.iter().enumerate() {
                write_indent(output, depth + 1);
                write_string(key, output);
                output.push_str(": ");

                write_value(value, output, depth + 1)?;

                if index + 1 < object.len() {
                    output.push(',');
                }

                output.push('\n');
            }

            write_indent(output, depth);
            output.push('}');
        }

        JsonValue::Array(array) => {
            if array.is_empty() {
                output.push_str("[]");
                return Ok(());
            }

            output.push_str("[\n");

            for (index, value) in array.iter().enumerate() {
                write_indent(output, depth + 1);

                write_value(value, output, depth + 1)?;

                if index + 1 < array.len() {
                    output.push(',');
                }

                output.push('\n');
            }

            write_indent(output, depth);
            output.push(']');
        }

        JsonValue::String(value) => {
            write_string(value, output);
        }

        JsonValue::Number(value) => {
            if !value.is_finite() {
                return Err(
                    "JSON unterstützt keine unendlichen Zahlen oder NaN"
                        .to_string()
                );
            }

            output.push_str(&value.to_string());
        }

        JsonValue::Bool(value) => {
            output.push_str(if *value { "true" } else { "false" });
        }

        JsonValue::Null => {
            output.push_str("null");
        }
    }

    Ok(())
}

// ============================================================
// ÖFFENTLICHE PARSER-FUNKTIONEN
// ============================================================

pub fn parse(input: &str) -> Result<JsonValue, String> {
    Parser::new(input).parse()
}

pub fn stringify(value: &JsonValue) -> Result<String, String> {
    let mut output = String::new();

    write_value(value, &mut output, 0)?;

    output.push('\n');

    Ok(output)
}

// ============================================================
// PFADVERWALTUNG
// ============================================================

// Liest einen Wert über einen Pfad.
//
// Beispiel:
// user.settings.theme
// plugins.0

fn get_path<'a>(
    current: &'a JsonValue,
    path: &[&str],
) -> Option<&'a JsonValue> {
    if path.is_empty() {
        return Some(current);
    }

    match current {
        JsonValue::Object(object) => {
            let next = object.get(path[0])?;
            get_path(next, &path[1..])
        }

        JsonValue::Array(array) => {
            let index: usize = path[0].parse().ok()?;
            let next = array.get(index)?;

            get_path(next, &path[1..])
        }

        _ => None,
    }
}

// Setzt einen Wert über einen Pfad.
//
// Fehlende Zwischenobjekte werden automatisch angelegt.
// Arrays werden nicht automatisch über ihre Länge hinaus erweitert.

fn set_path(
    current: &mut JsonValue,
    path: &[&str],
    value: JsonValue,
) -> Result<(), String> {
    if path.is_empty() {
        *current = value;
        return Ok(());
    }

    if matches!(current, JsonValue::Null) {
        *current = JsonValue::Object(BTreeMap::new());
    }

    match current {
        JsonValue::Object(object) => {
            let key = path[0].to_string();

            if path.len() == 1 {
                object.insert(key, value);
                return Ok(());
            }

            let next = object
                .entry(key)
                .or_insert_with(|| JsonValue::Object(BTreeMap::new()));

            set_path(next, &path[1..], value)
        }

        JsonValue::Array(array) => {
            let index: usize = path[0]
                .parse()
                .map_err(|_| {
                    format!("Ungültiger Array-Index: {}", path[0])
                })?;

            if index > array.len() {
                return Err(format!(
                    "Array-Index {} liegt außerhalb des Arrays",
                    index
                ));
            }

            if index == array.len() {
                if path.len() != 1 {
                    return Err(
                        "Ein neues Array-Element kann nicht direkt \
                         als Zwischenobjekt verwendet werden".to_string()
                    );
                }

                array.push(value);
                return Ok(());
            }

            if path.len() == 1 {
                array[index] = value;
                return Ok(());
            }

            set_path(&mut array[index], &path[1..], value)
        }

        _ => Err(format!(
            "Pfad kann bei '{}' nicht fortgesetzt werden",
            path[0]
        )),
    }
}

// Entfernt einen Wert über einen Pfad.
//
// Bei Arrays wird das Element entfernt und die folgenden Indizes
// verschieben sich um eins.

fn remove_path(
    current: &mut JsonValue,
    path: &[&str],
) -> Result<JsonValue, String> {
    if path.is_empty() {
        return Err(
            "Der Wurzelwert kann nicht über remove entfernt werden"
                .to_string()
        );
    }

    match current {
        JsonValue::Object(object) => {
            if path.len() == 1 {
                return object.remove(path[0]).ok_or_else(|| {
                    format!("Schlüssel '{}' existiert nicht", path[0])
                });
            }

            let next = object.get_mut(path[0]).ok_or_else(|| {
                format!("Schlüssel '{}' existiert nicht", path[0])
            })?;

            remove_path(next, &path[1..])
        }

        JsonValue::Array(array) => {
            let index: usize = path[0]
                .parse()
                .map_err(|_| {
                    format!("Ungültiger Array-Index: {}", path[0])
                })?;

            if index >= array.len() {
                return Err(format!(
                    "Array-Index {} existiert nicht",
                    index
                ));
            }

            if path.len() == 1 {
                return Ok(array.remove(index));
            }

            remove_path(&mut array[index], &path[1..])
        }

        _ => Err(format!(
            "Pfad kann bei '{}' nicht fortgesetzt werden",
            path[0]
        )),
    }
}

// ============================================================
// JSON MANAGER
// ============================================================

pub struct Config {
    path: PathBuf,
    data: JsonValue,
}

impl Config {
    // --------------------------------------------------------
    // DATEI ÖFFNEN ODER ERSTELLEN
    // --------------------------------------------------------

    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent)
                        .map_err(|e| e.to_string())?;
                }
            }

            fs::write(&path, "{}\n")
                .map_err(|e| e.to_string())?;
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| {
                format!("Datei konnte nicht gelesen werden: {}", e)
            })?;

        let data = parse(&content)
            .map_err(|e| {
                format!("JSON konnte nicht geladen werden: {}", e)
            })?;

        Ok(Self {
            path,
            data,
        })
    }

    // --------------------------------------------------------
    // DATEI NEU LADEN
    // --------------------------------------------------------

    pub fn reload(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.path)
            .map_err(|e| e.to_string())?;

        self.data = parse(&content)?;

        Ok(())
    }

    // --------------------------------------------------------
    // DATEI SPEICHERN
    // --------------------------------------------------------

    pub fn save(&self) -> Result<(), String> {
        let content = stringify(&self.data)?;

        fs::write(&self.path, content)
            .map_err(|e| {
                format!("JSON konnte nicht gespeichert werden: {}", e)
            })
    }

    // --------------------------------------------------------
    // GESAMTEN INHALT AUSGEBEN
    // --------------------------------------------------------

    pub fn print(&self) -> Result<(), String> {
        println!("{}", stringify(&self.data)?);
        Ok(())
    }

    // --------------------------------------------------------
    // GESAMTEN INHALT ALS STRING
    // --------------------------------------------------------

    pub fn to_string(&self) -> Result<String, String> {
        stringify(&self.data)
    }

    // --------------------------------------------------------
    // WERT AUSLESEN
    // --------------------------------------------------------

    pub fn get(&self, path: &str) -> Option<&JsonValue> {
        if path.is_empty() {
            return Some(&self.data);
        }

        let parts: Vec<&str> = path.split('.').collect();

        get_path(&self.data, &parts)
    }

    // --------------------------------------------------------
    // STRING AUSLESEN
    // --------------------------------------------------------

    pub fn get_string(&self, path: &str) -> Option<&str> {
        self.get(path)?.as_str()
    }

    // --------------------------------------------------------
    // ZAHL AUSLESEN
    // --------------------------------------------------------

    pub fn get_number(&self, path: &str) -> Option<f64> {
        self.get(path)?.as_number()
    }

    // --------------------------------------------------------
    // BOOLEAN AUSLESEN
    // --------------------------------------------------------

    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path)?.as_bool()
    }

    // --------------------------------------------------------
    // WERT HINZUFÜGEN ODER BEARBEITEN
    // --------------------------------------------------------

    pub fn set(
        &mut self,
        path: &str,
        value: JsonValue,
    ) -> Result<(), String> {
        if path.is_empty() {
            return Err(
                "Der Pfad darf nicht leer sein".to_string()
            );
        }

        let parts: Vec<&str> = path.split('.').collect();

        if parts.iter().any(|part| part.is_empty()) {
            return Err(
                "Der Pfad enthält einen leeren Abschnitt".to_string()
            );
        }

        set_path(&mut self.data, &parts, value)?;

        self.save()
    }

    // --------------------------------------------------------
    // WERT ENTFERNEN
    // --------------------------------------------------------

    pub fn remove(&mut self, path: &str) -> Result<JsonValue, String> {
        if path.is_empty() {
            return Err(
                "Der Pfad darf nicht leer sein".to_string()
            );
        }

        let parts: Vec<&str> = path.split('.').collect();

        let removed = remove_path(&mut self.data, &parts)?;

        self.save()?;

        Ok(removed)
    }

    // --------------------------------------------------------
    // GESAMTE JSON-DATEI LÖSCHEN
    // --------------------------------------------------------

    pub fn delete_file(self) -> Result<(), String> {
        fs::remove_file(&self.path)
            .map_err(|e| {
                format!("Datei konnte nicht gelöscht werden: {}", e)
            })
    }

    // --------------------------------------------------------
    // ZUGRIFF AUF DIE INTERNEN DATEN
    // --------------------------------------------------------

    pub fn data(&self) -> &JsonValue {
        &self.data
    }

    // --------------------------------------------------------
    // DIREKTE BEARBEITUNG DER DATEN
    // --------------------------------------------------------

    pub fn data_mut(&mut self) -> &mut JsonValue {
        &mut self.data
    }

    // Änderungen an data_mut() müssen anschließend mit save()
    // gespeichert werden.
}