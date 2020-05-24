extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use std::collections::HashMap;
use ansi_term::Style;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct SMFParser;

#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug)]pub enum MaterialVariableReference { // Can be a value or reference a variable
    TYPE (MaterialVariableType),
    VARIABLE(String),
    ARRAYREF (String, u32),
}
#[derive(Debug)]
pub struct MaterialProxy {
    pub name: String,
    pub parameters: HashMap<String, MaterialVariableReference>,
}
#[derive(Debug)]
pub struct MaterialFile {
    pub shader: String,
    pub variables: HashMap<String, MaterialVariableType>,
    pub setup_proxies: Vec<MaterialProxy>,
    pub render_proxies: Vec<MaterialProxy>,
}

fn var_to_string(pair: &mut pest::iterators::Pairs<'_, Rule>) -> Result<String, &'static str> {
    let p = pair.nth(0);
    match p {
        Some(data) => {
            match data.as_rule() {
                Rule::ident => {
                    return Ok(data.as_str().to_owned())
                },
                _ => return Err("Expected 'ident' in 'variable'")
            }
        },
        None => return Err("Empty variable")
    }
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

fn treat_arrayref(arrayref: &mut pest::iterators::Pairs<'_, Rule>) -> Result<MaterialVariableReference, &'static str> {

    let mut name = String::new();
    let mut index = 0;
    match arrayref.nth(0) {
        Some(data) => {
            match data.as_rule() {
                Rule::ident => {
                    name = data.as_str().to_owned();
                }
                _ => return Err("Expected 'ident' in 'arrayref'")
            }
        },
        None => return Err("Empty arrayref")
    }

    match arrayref.nth(0) {
        Some(data) => {
            match data.as_rule() {
                Rule::integer => {
                    let string = data.as_str().to_owned();
                    index = match string.parse::<u32>() {
                        Ok(data) => data,
                        Err(_) => return Err("Integer parsing error")
                    };
                }
                _ => return Err("Expected 'ident' in 'arrayref'")
            }
        },
        None => return Err("Empty arrayref")
    }

    return Ok(MaterialVariableReference::ARRAYREF(name, index))
}

fn treat_proxy(proxy: pest::iterators::Pair<'_, Rule>) -> Result<MaterialProxy, &'static str> {
    let mut name = String::new();
    let mut parameters = HashMap::new();

    let proxy = proxy.into_inner();

    for element in proxy {
        match element.as_rule() {
            Rule::identblockstart => {
                match element.into_inner().nth(0) {
                    Some(data) => {
                        match data.as_rule() {
                            Rule::ident => {
                                name = data.as_str().to_owned();
                            },
                            _ => return Err("Expected 'ident' in 'identblockstart'")
                        }
                    },
                    None => return Err("Invalid identblockstart")
                }
                
            },
            Rule::proxyparam => {
                let param_name: String;
                //todo: remove ugly .clone()
                match element.clone().into_inner().nth(0) {
                    Some(data) => {
                        match data.as_rule() {
                            Rule::ident => {
                                param_name = data.as_str().to_owned();
                            },
                            _ => return Err("Expected ident in 'proxyparam'")
                        }
                    },
                    None => return Err("Expected 2 elements in 'proxyparam'")
                };
            
                match element.into_inner().nth(1) {
                    Some(data) => {
                        match data.as_rule() {
                            Rule::srcdest => {
                                match data.into_inner().nth(0) {
                                    Some(srcdst) => {
                                        match srcdst.as_rule() {
                                            Rule::variable => {
                                                let vts = var_to_string(&mut srcdst.into_inner());
                                                match vts {
                                                    Ok(varname) => {
                                                        parameters.insert(param_name, MaterialVariableReference::VARIABLE(varname));
                                                    },
                                                    Err(e) => return Err(e)
                                                }
                                                
                                            },
                                            Rule::arrayref => {
                                                let matvarref = match treat_arrayref(&mut srcdst.into_inner()) {
                                                    Ok(data) => data,
                                                    Err(e) => return Err(e)
                                                };

                                                parameters.insert(param_name, matvarref);
                                            },
                                            Rule::value => {
                                                let srcdst = match srcdst.into_inner().nth(0) {
                                                    Some(data) => data,
                                                    None => return Err("Empty value")
                                                };
                                                let type_ = match treat_value(srcdst, false) {
                                                    Ok(data) => data,
                                                    Err(e) => return Err(e)
                                                };
                                                parameters.insert(param_name, MaterialVariableReference::TYPE(type_));
                                            },
                                            _ => return Err("Invalid srcdest")
                                        }
                                    },
                                    None => return Err("Empty srcdest")
                                }
                            },
                            _ => return Err("Expected srcdest in 'proxyparam'")
                        }
                    },
                    None => return Err("Expected 2 elements in 'proxyparam'")
                }
                
            },
            _ => return Err("Invalid proxy")
        }
    }
    
    Ok(MaterialProxy {
        name,
        parameters
    })
}

fn treat_proxyblock(pair: &mut pest::iterators::Pairs<'_, Rule>, proxy_vec: &mut Vec<MaterialProxy>) -> Result<(), &'static str> {
    for element in pair {
        match element.as_rule() {
            Rule::proxy => {
                match treat_proxy(element) {
                    Ok(proxy) => proxy_vec.push(proxy),
                    Err(e) => return Err(e)
                }
            },
            _ => return Err("Expected proxy")
        }
    }
    Ok(())
}

fn comp_types(type1: &MaterialVariableType, type2: &MaterialVariableType) -> bool {
    std::mem::discriminant(type1) == std::mem::discriminant(type2)
}

//this code is a mess!
fn treat_arraydec(array: pest::iterators::Pairs<'_, Rule>) -> Result<MaterialVariableType, &'static str> {
    let mut type_ = MaterialVariableType::NONE;
    let mut vec = Vec::with_capacity(4);
    for element in array {
        match element.as_rule() {
            Rule::number => {
                match element.into_inner().nth(0) {
                    Some(data) => {
                        match data.as_rule() {
                            //todo: For now non_int = double, that is not desirable as it could also represent a float
                            Rule::non_int => {
                                let string = data.as_str().to_owned();
                                let n = match string.parse::<f64>() {
                                    Ok(data) => data,
                                    Err(_) => return Err("Double parsing error in vector declaration")
                                };
                                let type2 = MaterialVariableType::DOUBLE(n);
                                if type_ != MaterialVariableType::NONE && !comp_types(&type_, &type2) {
                                    return Err("Ambiguous number type in vector declaration")
                                }
                                type_ = type2;
                                vec.push(type_.clone());
                            },
                            Rule::float => {
                                let string = data.as_str().to_owned();
                                let n = match f32::from_parsed_number_string(&string) {
                                    Ok(data) => data,
                                    Err(e) => return Err(e)
                                };
                                let type2 = MaterialVariableType::FLOAT(n);
                                if type_ != MaterialVariableType::NONE && !comp_types(&type_, &type2) {
                                    return Err("Ambiguous number type in vector declaration")
                                }
                                type_ = type2;
                                vec.push(type_.clone());
                            },
                            Rule::double => {
                                let string = data.as_str().to_owned();
                                let n = match f64::from_parsed_number_string(&string) {
                                    Ok(data) => data,
                                    Err(e) => return Err(e)
                                };
                                let type2 = MaterialVariableType::DOUBLE(n);
                                if type_ != MaterialVariableType::NONE && !comp_types(&type_, &type2) {
                                    return Err("Ambiguous number type in vector declaration")
                                }
                                type_ = type2;
                                vec.push(type_.clone());
                            },
                            Rule::integer => {
                                let string = data.as_str().to_owned();
                                let n = match string.parse::<i32>() {
                                    Ok(data) => data,
                                    Err(_) => return Err("Integer parsing error in vector declaration")
                                };
                                let type2 = MaterialVariableType::INTEGER(n);
                                if type_ != MaterialVariableType::NONE && !comp_types(&type_, &type2) {
                                    return Err("Ambiguous number type in vector declaration")
                                }
                                type_ = type2;
                                vec.push(type_.clone());
                            },
                            _ => return Err("Invalid number")
                        }
                    },
                    None => return Err("Empty number")
                }
            },
            _ => return Err("Only numbers are allowed in a vector declaration")
        }
    }

    match vec.len() {
        2 => {
            match type_ {
                MaterialVariableType::INTEGER(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY2(n1, n2))
                },
                MaterialVariableType::FLOAT(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY2F(n1, n2))
                },
                MaterialVariableType::DOUBLE(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY2D(n1, n2))
                }
                _ => return Err("????")
            }
        },
        3 => {
            match type_ {
                MaterialVariableType::INTEGER(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    let n3 = match vec[2] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY3(n1, n2, n3))
                },
                MaterialVariableType::FLOAT(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    let n3 = match vec[2] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY3F(n1, n2, n3))
                },
                MaterialVariableType::DOUBLE(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    let n3 = match vec[2] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY3D(n1, n2, n3))
                }
                _ => return Err("????")
            }
        },
        4 => {
            match type_ {
                MaterialVariableType::INTEGER(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    let n3 = match vec[2] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    let n4 = match vec[3] {
                        MaterialVariableType::INTEGER(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY4(n1, n2, n3, n4))
                },
                MaterialVariableType::FLOAT(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    let n3 = match vec[2] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    let n4 = match vec[3] {
                        MaterialVariableType::FLOAT(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY4F(n1, n2, n3, n4))
                },
                MaterialVariableType::DOUBLE(_) => {
                    let n1 = match vec[0] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    let n2 = match vec[1] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    let n3 = match vec[2] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    let n4 = match vec[3] {
                        MaterialVariableType::DOUBLE(data) => data,
                        _ => return Err("???")
                    };
                    return Ok(MaterialVariableType::ARRAY4D(n1, n2, n3, n4))
                },
                _ => return Err("????")
            }
        },
        _ => return Err("Invalid vector size"),
    }
}

fn treat_value(val: pest::iterators::Pair<'_, Rule>, support_rvalues: bool) -> Result<MaterialVariableType, &'static str> {
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
        Rule::array => {
            if support_rvalues {
                type_ = match treat_arraydec(val.into_inner()) {
                    Ok(data) => data,
                    Err(e) => return Err(e)
                }
            } else {
                return Err("Cannot use vector declaration as proxy parameter")
            }
        }
        _ => return Err("Invalid value type in value")
    }
    return Ok(type_)
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

    let type_ = match treat_value(val, true) {
        Ok(data) => data,
        Err(e) => return Err(e)
    };
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
        setup_proxies: Vec::with_capacity(2),
        render_proxies: Vec::with_capacity(5),
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
            Rule::setupproxyblock => {
                match treat_proxyblock(&mut pair.into_inner(), &mut material.setup_proxies) {
                    Ok(_) => {},
                    Err(e) => return Err(e)
                }
            },
            Rule::renderproxyblock => {
                match treat_proxyblock(&mut pair.into_inner(), &mut material.render_proxies) {
                    Ok(_) => {},
                    Err(e) => return Err(e)
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

    println!("{}", Style::new().bold().paint("PROXIES:"));
    println!("{}", Style::new().bold().paint("\tSETUP"));
    for value in &material.setup_proxies {
        println!("\t  {}:", Style::new().italic().paint(&value.name));
        for param in &value.parameters {
            println!("\t    {}: {:?}", Style::new().italic().paint(param.0), param.1);
        }
    }
    println!("{}", Style::new().bold().paint("\tRENDER"));
    for value in &material.render_proxies {
        println!("\t  {}:", Style::new().italic().paint(&value.name));
        for param in &value.parameters {
            println!("\t    {}: {:?}", Style::new().italic().paint(param.0), param.1);
        }
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