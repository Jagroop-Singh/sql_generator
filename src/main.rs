use clap::Parser;
use md4::Md4;
use md5;
use nom::{
    bytes::complete::{is_not, tag},
    sequence::terminated,
    IResult,
};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[derive(Parser)]
#[command(name = "Sql Generator")]
#[command(author = "Jagroop Singh")]
#[command(version = "1.0")]
#[command(about="Generates sql file with insert statements from wordlists", long_about=None)]
struct Cli {
    #[arg(short = 't', long)]
    table: String,

    #[arg(short = 'v', long)]
    values: Vec<String>,

    #[arg(short = 'w', long)]
    wordlist: Vec<String>,

    #[arg(short = 'd', long)]
    dbms: Option<String>,

    #[arg(short = 'o', long)]
    output: Option<String>,
}

/*
 *  Implement NotNull with check for non-empty value
 *  Implement UNIQUE with Hashmap
 *  Implement Primary Key with table level constraint and use UNIQUE check
 */
#[derive(Debug, Clone)]
enum Constraint {
    NotNull,
    UNIQUE,
    PrimaryKey,
    // CHECK(String),
    Empty,
}

#[derive(Debug, Clone)]
enum Transformations {
    Empty,
    MD5,
    MD4,
    SHA1,
    SHA256,
    SHA512,
}

#[derive(Debug, Clone)]
struct Value {
    name: String,
    data_type: String,
    constraint: Constraint,
}

#[derive(Debug)]
struct Wordlist {
    value: Value,
    wordlist: String,
    transformation: Transformations,
}

fn parse_values<'a>(input: &'a str) -> IResult<&'a str, Value> {
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

fn parse_wordlist<'a>(input: &'a str, values: &Vec<Value>) -> IResult<&'a str, Wordlist> {
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

// Todo!
// 1. Implement constraints in queries
// 2. Check if values meet constraints
// 3. Implement writing query in different dbms language

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if cli.wordlist.len() != cli.values.len() {
        panic!("Need a wordlist for each value and vice versa");
    }

    let mut values: Vec<Value> = Vec::new();
    for x in 0..cli.values.len() {
        values.push(parse_values(&cli.values[x]).unwrap().1);
    }

    let mut wordlists: Vec<Wordlist> = Vec::new();
    for x in cli.wordlist.iter() {
        wordlists.push(parse_wordlist(x, &values).unwrap().1);
    }

    let mut output = match &cli.output {
        Some(k) => File::create(k)?,
        None => File::create("results.txt")?,
    };

    let mut files: Vec<_> = wordlists
        .iter()
        .map(|wordlist| BufReader::new(File::open(wordlist.wordlist.as_str()).unwrap()).lines())
        .collect();

    // Create Table Query
    create_query(&cli.table, &values, &mut output)?;
    // Write insert statements
    create_insert_query(&values, &wordlists, &mut files, &mut output)?;
    Ok(())
}

fn create_query(
    table: &str,
    values: &Vec<Value>,
    output: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut s = String::new();
    s.push_str("create table ");
    s.push_str(table);
    s.push_str("(\n");
    for i in 0..values.len() {
        s.push_str("    ");
        s.push_str(&values[i].name);
        s.push_str("    ");
        s.push_str(&values[i].data_type);
        s.push_str(",\n");
    }
    s.push_str(");\n\n");
    write!(output, "{}", s)?;
    Ok(())
}

fn create_insert_query(
    values: &Vec<Value>,
    wordlists: &Vec<Wordlist>,
    files: &mut Vec<std::io::Lines<BufReader<File>>>,
    output: &mut File,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pre_arg = String::new();
    pre_arg.push_str("insert into users values(");
    // Insert into fields
    for i in 0..values.len() {
        pre_arg.push_str(&("'".to_owned() + values[i].name.as_str() + "'"));
        if i != values.len() - 1 {
            pre_arg.push_str(",");
        }
    }
    pre_arg.push_str(") values(");
    let mut h: HashSet<String> = HashSet::new();
    let mut p: HashSet<String> = HashSet::new();

    'outer: while let Some(line) = files[0].next() {
        let mut l = String::new();
        l.push_str(&pre_arg);
        let line = line.unwrap();
        if !constraint_check(&line, &wordlists[0], &mut h, &mut p) {
            continue 'outer;
        }
        l.push_str("'");
        l.push_str(&transform_input(&line, &wordlists[0]));
        l.push_str("',");
        for k in 1..files.len() {
            let s: &str = &files[k].next().unwrap().unwrap();
            if !constraint_check(&s, &wordlists[k], &mut h, &mut p) {
                println!("Duplicate value: {}", s);
                continue 'outer;
            }
            l.push_str("'");
            l.push_str(&transform_input(s, &wordlists[k]));
            l.push_str("'");
            if k != files.len() - 1 {
                l.push_str(",");
            }
        }
        l.push_str(");\n");
        write!(output, "{}", l)?;
    }

    Ok(())
}

fn transform_input<'a>(s: &'a str, w: &Wordlist) -> String {
    match w.transformation {
        Transformations::MD4 => encrypt_md4(s),
        Transformations::MD5 => encrypt_md5(s),
        Transformations::SHA1 => encrypt_sha1(s),
        Transformations::SHA256 => encrypt_sha256(s),
        Transformations::SHA512 => encrypt_sha512(s),
        Transformations::Empty => s.to_string(),
    }
}

fn constraint_check(
    s: &str,
    w: &Wordlist,
    h: &mut HashSet<String>,
    p: &mut HashSet<String>,
) -> bool {
    match w.value.constraint {
        Constraint::NotNull => not_null(s),
        Constraint::UNIQUE => unique(s, h),
        Constraint::PrimaryKey => primary_key(s, p),
        Constraint::Empty => true,
    }
}

fn not_null(s: &str) -> bool {
    !s.is_empty()
}

fn unique(s: &str, h: &mut HashSet<String>) -> bool {
    if !h.contains(s) {
        h.insert(String::from(s));
        true
    } else {
        false
    }
}

fn primary_key(s: &str, p: &mut HashSet<String>) -> bool {
    if !p.contains(s) {
        p.insert(String::from(s));
        true
    } else {
        false
    }
}

fn encrypt_md4(input: &str) -> String {
    let mut hasher = Md4::new();
    hasher.update(input);
    format!("{:X}", hasher.finalize())
}
fn encrypt_md5(input: &str) -> String {
    format!("{:X}", md5::compute(input))
}

fn encrypt_sha1(input: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(input);
    format!("{:X}", hasher.finalize())
}

fn encrypt_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:X}", hasher.finalize())
}

fn encrypt_sha512(input: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(input);
    format!("{:X}", hasher.finalize())
}
