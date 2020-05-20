extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::collections::HashMap;
use ansi_term::Style;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SMFParser;

#[derive(Debug)]
pub enum MaterialVariableType {
    NONE,
    FLOAT (f32),
    DOUBLE (f64),
    INTEGER (i32),
    STRING (String),

    ARRAY2 (i32, i32),
    ARRAY3 (i32, i32, i32),
    ARRAY4 (i32, i32, i32, i32),

    ARRAY2F (f32, f32),
    ARRAY3F (f32, f32, f32),
    ARRAY4F (f32, f32, f32, f32),

    ARRAY2D (f64, f64),
    ARRAY3D (f64, f64, f64),
    ARRAY4D (f64, f64, f64, f64),
}

pub enum MaterialSourceOrDestination {
    TYPE (MaterialVariableType),
    VARIABLE (String),
    ARRAYREF (String, u32),
}

pub struct MaterialProxyParameter {
    pub name: String,
    pub value: MaterialVariableType,
}

pub struct MaterialProxy {
    pub name: String,
    pub parameters: Vec<MaterialProxyParameter>,
}

pub struct MaterialFile {
    pub shader: String,
    pub variables: HashMap<String, MaterialVariableType>,
}

fn treat_identblockstart(pair: &mut pest::iterators::Pairs<'_, Rule>, material: &mut MaterialFile) -> Result<(), &'static str> {
    match pair.nth(0) {
        Some(ident_pair) => {
            match ident_pair.as_rule() {
                Rule::ident => {
                    material.shader = ident_pair.as_str().to_owned();
                },
                _ => return Err("Expected identifier inside of 'identblockstart'")
            }
        },
        None => return Err("Empty identblockstart")
    };
    Ok(())
}

fn treat_vardec(pair: &mut pest::iterators::Pairs<'_, Rule>, material: &mut MaterialFile) -> Result<(), &'static str> {
    let varname = match pair.nth(0) {
        Some(variable) => {
            match variable.into_inner().nth(0) {
                Some(ident) => {
                    ident.as_str().to_owned()
                },
                None => return Err("Invalid identifier in vardec")
            }
        },
        None => return Err("Expected 2 elements in vardec")
    };

    let val = match pair.nth(0) {
        Some(value) => {
            match value.into_inner().nth(0) {
                Some(data) => data,
                None => return Err("Invalid value in vardec")
            }
        },
        None => return Err("Expected 2 elements in vardec")
    };

    let type_: MaterialVariableType;
    match val.as_rule() {
        Rule::string => {
            let string = match val.into_inner().nth(0) {
                Some(data) => data.as_str().to_owned(),
                None => return Err("Invalid string")
            };
            type_ = MaterialVariableType::STRING(string);
        },
        Rule::number => {
            let number = match val.into_inner().nth(0) {
                Some(data) => data,
                None => return Err("Invalid number")
            };
            match number.as_rule() {
                Rule::float => {
                    let string = number.as_str().to_owned();
                    let number = match f32::from_parsed_number_string(&string) {
                        Ok(n) => n,
                        Err(e) => return Err(e)
                    };
                    type_ = MaterialVariableType::FLOAT(number);
                },
                Rule::double => {
                    let string = number.as_str().to_owned();
                    let number = match f64::from_parsed_number_string(&string) {
                        Ok(n) => n,
                        Err(e) => return Err(e)
                    };
                    type_ = MaterialVariableType::DOUBLE(number);
                },
                Rule::non_int | Rule::signed_non_int => {
                    let string = number.as_str().to_owned();
                    let number = match string.parse::<f64>() {
                        Ok(n) => n,
                        Err(_) => return Err("non_int considered as a double but resulted in parsing error")
                    };
                    type_ = MaterialVariableType::DOUBLE(number);
                },
                Rule::integer | Rule::signed_integer => {
                    let string = number.as_str().to_owned();
                    let number = match string.parse::<i32>() {
                        Ok(n) => n,
                        Err(_) => return Err("Invalid integer")
                    };
                    type_ = MaterialVariableType::INTEGER(number);
                },
                _ => return Err("Invalid number")
            }
        },
        //todo: Arrays are currently unsupported
        Rule::array => return Err("Arrays are currently unsupported"),
        _ => return Err("Invalid value type in vardec")
    }
    material.variables.insert(varname,   type_ );
    Ok(())
}

trait FromParsedNumberString {
    fn from_parsed_number_string(string: &String) -> Result<Self, &'static str> where Self: Sized;
}

impl FromParsedNumberString for f32 {
    //todo: use string.strip_suffix("f") when it will be stabilized
    fn from_parsed_number_string(string: &String) -> Result<Self, &'static str> {
        let size = string.len();
        let string = string[..size - 1].to_owned();
        match string.parse::<f32>() {
            Ok(res) => Ok(res),
            Err(_) => Err("Error while trying to parse a float string")
        }
    }
}

impl FromParsedNumberString for f64 {
    //todo: use string.strip_suffix("d") when it will be stabilized
    fn from_parsed_number_string(string: &String) -> Result<Self, &'static str> {
        let size = string.len();
        let string = string[..size - 1].to_owned();
        match string.parse::<f64>() {
            Ok(res) => Ok(res),
            Err(_) => Err("Error while trying to parse a double string")
        }
    }
}

fn parse_material_file(data: &String) -> Result<MaterialFile, &'static str> {
    let pairs = match SMFParser::parse(Rule::material, data) {
        Ok(mut p) => {
            match p.next() {
                Some(item) => item,
                None => return Err("Invalid Material File")
            }
        }
        Err(_) => return Err("Invalid Material File")
    };

    let mut material = MaterialFile {
        shader: String::new(),
        variables: HashMap::new(),
    };

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::identblockstart => {
                match treat_identblockstart(&mut pair.into_inner(), &mut material) {
                    Err(e) => return Err(e),
                    _ => {}
                }
            },
            Rule::vardec => {
                match treat_vardec(&mut pair.into_inner(), &mut material) {
                    Err(e) => return Err(e),
                    _ => {}
                }
            },
            _ => println!("Unsupported rule: {:?}", pair.as_rule()),
        }
    }
    // Sanity check
    if material.shader.is_empty() {
        return Err("No shader specified")
    }
    Ok(material)
}

fn print_material_information(material: &MaterialFile) {
    println!("{}", Style::new().bold().paint("===============================\nINFORMATION ABOUT THE MATERIAL\n==============================="));
    println!("{} {}", Style::new().bold().paint("SHADER:"), material.shader);
    println!("{}", Style::new().bold().paint("VARIABLES:"));

    for value in &material.variables {
        println!("\t{}: {:?}", Style::new().italic().paint(value.0), value.1);
    }
}

fn main() {
    #[cfg(target_os = "windows")] //stupid windows stuff
    ansi_term::enable_ansi_support();

    let buf = include_str!("UnlitGeneric.smf");

    match parse_material_file(&buf.to_owned()) {
        Ok(material) => print_material_information(&material),
        Err(e) => eprintln!("ERROR: {}", e)
    }
}