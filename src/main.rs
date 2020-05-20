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
                    ident.as_str().to_owned();
                },
                None => return Err("Invalid identifier in vardec")
            }
        },
        None => return Err("Expected 2 elements in vardec")
    };

    let val = match pair.nth(1) {
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
            let string = val.into_inner().nth(0).unwrap().as_str().to_owned();
            type_ = MaterialVariableType::STRING(string);
        },
        Rule::number => {
            let number = val.into_inner().nth(0).unwrap();
            match number.as_rule() {
                Rule::float => {
                    //Nightly only at type of writing: let string = number.as_str().to_owned().strip_suffix("f");
                    let string = number.as_str().to_owned();
                    let size = string.len();
                    let string = string[..size - 1].to_owned();
                    type_ = MaterialVariableType::FLOAT(string.parse::<f32>().expect("Invalid float!"));
                },
                Rule::double => {
                    //Nightly only at type of writing: let string = number.as_str().to_owned().strip_suffix("d");
                    let string = number.as_str().to_owned();
                    let size = string.len();
                    let string = string[..size - 1].to_owned();
                    type_ = MaterialVariableType::DOUBLE(string.parse::<f64>().expect("Invalid double!"));
                },
                Rule::non_int | Rule::signed_non_int => {
                    let string = number.as_str().to_owned();
                    type_ = MaterialVariableType::DOUBLE(string.parse::<f64>().expect("non_int considered as a float but resulted in parsing error"));
                },
                Rule::integer | Rule::signed_integer => {
                    let string = number.as_str().to_owned();
                    type_ = MaterialVariableType::INTEGER(string.parse::<i32>().expect("Invalid integer!"));
                },
                _ => panic!("Invalid number in variable declaration")
            }
        },
        //todo: Arrays are currently unsupported
        Rule::array => return Err("Arrays are currently unsupported"),
        _ => return Err("Invalid value type in vardec")
    }
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

                treat_vardec(&mut pair.into_inner(), &mut material);

                let varname = pair.clone()
                            .into_inner().nth(0).unwrap()
                            .into_inner().nth(0).unwrap()
                            .as_str().to_owned();
                let val = pair.clone()
                            .into_inner().nth(1).unwrap()
                            .into_inner().nth(0).unwrap();
                let type_: MaterialVariableType;
                match val.as_rule() {
                    Rule::string => {
                        let string = val.into_inner().nth(0).unwrap().as_str().to_owned();
                        type_ = MaterialVariableType::STRING(string);
                    },
                    Rule::number => {
                        let number = val.into_inner().nth(0).unwrap();
                        match number.as_rule() {
                            Rule::float => {
                                //Nightly only at type of writing: let string = number.as_str().to_owned().strip_suffix("f");
                                let string = number.as_str().to_owned();
                                let size = string.len();
                                let string = string[..size - 1].to_owned();
                                type_ = MaterialVariableType::FLOAT(string.parse::<f32>().expect("Invalid float!"));
                            },
                            Rule::double => {
                                //Nightly only at type of writing: let string = number.as_str().to_owned().strip_suffix("d");
                                let string = number.as_str().to_owned();
                                let size = string.len();
                                let string = string[..size - 1].to_owned();
                                type_ = MaterialVariableType::DOUBLE(string.parse::<f64>().expect("Invalid double!"));
                            },
                            Rule::non_int | Rule::signed_non_int => {
                                let string = number.as_str().to_owned();
                                type_ = MaterialVariableType::DOUBLE(string.parse::<f64>().expect("non_int considered as a float but resulted in parsing error"));
                            },
                            Rule::integer | Rule::signed_integer => {
                                let string = number.as_str().to_owned();
                                type_ = MaterialVariableType::INTEGER(string.parse::<i32>().expect("Invalid integer!"));
                            },
                            _ => panic!("Invalid number in variable declaration")
                        }
                    },
                    Rule::array => {
                        let mut type_to_interpret_to = MaterialVariableType::NONE;
                        let array = val.into_inner().nth(0).unwrap();
                        for number in array.clone().into_inner() {
                            match number.into_inner().nth(0).unwrap().as_rule() {
                                Rule::integer => {
                                    match type_to_interpret_to {
                                        MaterialVariableType::NONE => {
                                            type_to_interpret_to = MaterialVariableType::INTEGER(0);
                                        }
                                        _ => {},
                                    }
                                },
                                Rule::float => {
                                    match type_to_interpret_to {
                                        MaterialVariableType::NONE | MaterialVariableType::INTEGER(_) => {
                                            type_to_interpret_to = MaterialVariableType::FLOAT(0.0);
                                        },
                                        _ => {},
                                    }
                                }
                                Rule::double | Rule::non_int | Rule::signed_non_int => {
                                    match type_to_interpret_to {
                                        MaterialVariableType::NONE | MaterialVariableType::INTEGER(_) | MaterialVariableType::FLOAT(_) => {
                                            type_to_interpret_to = MaterialVariableType::DOUBLE(0.0);
                                        },
                                        _ => {},
                                    }
                                },
                                _ => panic!("Invalid number")
                            }
                        }
                        match array.as_rule() {
                            Rule::array2 => {
                                let s0 = array.clone().into_inner().nth(0).unwrap().as_str();
                                let s1 = array.into_inner().nth(1).unwrap().as_str();
                                match type_to_interpret_to {
                                    MaterialVariableType::INTEGER(_) => {
                                        let n0 = s0.parse::<i32>().unwrap();
                                        let n1 = s1.parse::<i32>().unwrap();
                                        type_ = MaterialVariableType::ARRAY2(n0, n1);
                                    },
                                    MaterialVariableType::FLOAT(_) => {
                                        let n0 = s0.parse::<f32>().unwrap();
                                        let n1 = s1.parse::<f32>().unwrap();
                                        type_ = MaterialVariableType::ARRAY2F(n0, n1);
                                    },
                                    MaterialVariableType::DOUBLE(_) => {
                                        let n0 = s0.parse::<f64>().unwrap();
                                        let n1 = s1.parse::<f64>().unwrap();
                                        type_ = MaterialVariableType::ARRAY2D(n0, n1);
                                    },
                                    _ => panic!("Invalid array type"),
                                }
                            },
                            Rule::array3 => {
                                let s0 = array.clone().into_inner().nth(0).unwrap().as_str();
                                let s1 = array.clone().into_inner().nth(1).unwrap().as_str();
                                let s2 = array.into_inner().nth(2).unwrap().as_str();
                                match type_to_interpret_to {
                                    MaterialVariableType::INTEGER(_) => {
                                        let n0 = s0.parse::<i32>().unwrap();
                                        let n1 = s1.parse::<i32>().unwrap();
                                        let n2 = s2.parse::<i32>().unwrap();
                                        type_ = MaterialVariableType::ARRAY3(n0, n1, n2);
                                    },
                                    MaterialVariableType::FLOAT(_) => {
                                        let n0 = s0.parse::<f32>().unwrap();
                                        let n1 = s1.parse::<f32>().unwrap();
                                        let n2 = s2.parse::<f32>().unwrap();
                                        type_ = MaterialVariableType::ARRAY3F(n0, n1, n2);
                                    },
                                    MaterialVariableType::DOUBLE(_) => {
                                        let n0 = s0.parse::<f64>().unwrap();
                                        let n1 = s1.parse::<f64>().unwrap();
                                        let n2 = s2.parse::<f64>().unwrap();
                                        type_ = MaterialVariableType::ARRAY3D(n0, n1, n2);
                                    },
                                    _ => panic!("Invalid array type"),
                                }
                            },
                            Rule::array4 => {
                                let s0 = array.clone().into_inner().nth(0).unwrap().as_str();
                                let s1 = array.clone().into_inner().nth(1).unwrap().as_str();
                                let s2 = array.clone().into_inner().nth(2).unwrap().as_str();
                                let s3 = array.into_inner().nth(3).unwrap().as_str();
                                match type_to_interpret_to {
                                    MaterialVariableType::INTEGER(_) => {
                                        let n0 = s0.parse::<i32>().unwrap();
                                        let n1 = s1.parse::<i32>().unwrap();
                                        let n2 = s2.parse::<i32>().unwrap();
                                        let n3 = s3.parse::<i32>().unwrap();
                                        type_ = MaterialVariableType::ARRAY4(n0, n1, n2, n3);
                                    },
                                    MaterialVariableType::FLOAT(_) => {
                                        let n0 = s0.parse::<f32>().unwrap();
                                        let n1 = s1.parse::<f32>().unwrap();
                                        let n2 = s2.parse::<f32>().unwrap();
                                        let n3 = s3.parse::<f32>().unwrap();
                                        type_ = MaterialVariableType::ARRAY4F(n0, n1, n2, n3);
                                    },
                                    MaterialVariableType::DOUBLE(_) => {
                                        let n0 = s0.parse::<f64>().unwrap();
                                        let n1 = s1.parse::<f64>().unwrap();
                                        let n2 = s2.parse::<f64>().unwrap();
                                        let n3 = s3.parse::<f64>().unwrap();
                                        type_ = MaterialVariableType::ARRAY4D(n0, n1, n2, n3);
                                    },
                                    _ => panic!("Invalid array type"),
                                }
                            },
                            _ => panic!("Invalid array")
                        }

                    },
                    _ => panic!("Invalid value in variable declaration")
                }
                material.variables.insert(varname,   type_ );
            },
            Rule::proxyblock => {

            },
            _ => println!("Unsupported rule: {:?}", pair.as_rule()),
        }
    }

    //todo: sanity checking

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