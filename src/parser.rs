use nom::{
    bytes::complete::{is_not, tag},
    sequence::terminated,
    IResult,
};

/*
 *  Implement NotNull with check for non-empty value
 *  Implement UNIQUE with Hashmap
 *  Implement Primary Key with table level constraint and use UNIQUE check
 */
#[derive(Debug, Clone)]
pub enum Constraint {
    NotNull,
    UNIQUE,
    PrimaryKey,
    // CHECK(String),
    Empty,
}

#[derive(Debug, Clone)]
pub enum Transformations {
    Empty,
    MD5,
    MD4,
    SHA1,
    SHA256,
    SHA512,
}

#[derive(Debug, Clone)]
pub struct Value {
    pub name: String,
    pub data_type: String,
    pub constraint: Constraint,
}

#[derive(Debug)]
pub struct Wordlist {
    pub value: Value,
    pub wordlist: String,
    pub transformation: Transformations,
}
pub fn parse_values<'a>(input: &'a str) -> IResult<&'a str, Value> {
    let (input, name) = terminated(is_not(":"), tag(":"))(input)?;
    let (input, data_type) = terminated(is_not(":"), tag(":"))(input)?;
    let constraint = match input.to_lowercase().as_str() {
        "not null" => Constraint::NotNull,
        "unique" => Constraint::UNIQUE,
        "primary key" => Constraint::PrimaryKey,
        // x if x.contains(":chec") => Constraint::CHECK(x.to_string()),
        "" => Constraint::Empty,
        _ => panic!("Constraint Not Allowed"),
    };

    Ok((
        "",
        Value {
            name: name.to_string(),
            data_type: data_type.to_string(),
            constraint: constraint.clone(),
        },
    ))
}

pub fn parse_wordlist<'a>(input: &'a str, values: &Vec<Value>) -> IResult<&'a str, Wordlist> {
    let (input, value) = is_not(":")(input)?;
    let mut i = 0;
    let value: Value = loop {
        if i >= values.len() {
            panic!("invalid value in wordlist");
        }
        if value == values[i].name {
            break values[i].clone();
        }
        i += 1;
    };

    let (input, _) = tag(":")(input)?;
    let (input, wordlist) = is_not(":")(input)?;
    let input = match input.to_lowercase().as_str() {
        ":md5" => Transformations::MD5,
        ":md4" => Transformations::MD4,
        ":sha1" => Transformations::SHA1,
        ":sha256" => Transformations::SHA256,
        ":sha512" => Transformations::SHA512,
        ":" | "" => Transformations::Empty,
        _ => panic!("Transformation not allowed"),
    };

    Ok((
        "",
        Wordlist {
            value: value.clone(),
            wordlist: wordlist.to_string(),
            transformation: input.clone(),
        },
    ))
}
