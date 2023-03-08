use std::fs::{self, File};
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::path::PathBuf;
use itertools::Itertools;

use md5;
use clap::Parser;
use nom::{
    bytes::complete::{is_not, tag, take_till}, IResult,
    combinator::map_res,
};

#[derive(Parser)]
#[command(name="Sql Generator")]
#[command(author="Jagroop Singh")]
#[command(version="1.0")]
#[command(about="Generates sql file with insert statements from wordlists", long_about=None)]
struct Cli{
    #[arg(short='t',long)]
    table: String,

    #[arg(short='v',long)]
    values: Vec<String>,

    #[arg(short='w',long)]
    wordlist: Vec<String>,

    #[arg(short='d',long)]
    dbms: Option<String>,

    #[arg(short='o',long)]
    output: Option<String>,
}

#[derive(Debug, Clone)]
enum Constraint{
    NotNull,
    UNIQUE,
    PrimaryKey,
    CHECK(String),
    Default(String),
    Empty,
}

#[derive(Debug, Clone)]
enum Transformations{
    Empty,
    MD5,
    MD4,
    SHA1,
    SHA256,
    SHA512,
}

#[derive(Debug, Clone)]
struct Value{
    name: String,
    data_type: String,
    constraint: Constraint,
}

#[derive(Debug)]
struct Wordlist{
    value: Value,
    wordlist: String,
    transformation: Transformations,
}

fn parse_values<'a>(input: &'a str, values: &mut Vec<Value>)->IResult<&'a str,Value>{
    let (input, name) = nom::bytes::complete::is_not(":")(input)?;
    let (input, _ ) = tag(":")(input)?;
    let (input,data_type) =  nom::bytes::complete::is_not(":")(input)?;
    let constraint  = match input.to_lowercase().as_str(){
            ":not null" => Constraint::NotNull,
            ":unique" => Constraint::UNIQUE,
            ":primary key" => Constraint::PrimaryKey,
            x if x.contains(":chec") => Constraint::CHECK(x.to_string()),
            ":" | "" => Constraint::Empty,
            _ => panic!("Constraint Not Allowed"),
    };
    
    values.push(
Value{
            name:name.to_string(),
            data_type:data_type.to_string(),
            constraint:constraint.clone(),
        }
    );
    Ok(("",Value{
            name:name.to_string(),
            data_type:data_type.to_string(),
            constraint:constraint.clone(),
        }))
}

fn parse_wordlist<'a>(input: &'a str, values: &Vec<Value>, wordlists: &mut Vec<Wordlist>) -> IResult<&'a str,Wordlist>{
    let (input, value) = nom::bytes::complete::is_not(":")(input)?;
    let mut i = 0;
    let value:Value = loop{
        if i>=values.len(){
            panic!("invalid value in wordlist");
        }
       if value == values[i].name{
            break values[i].clone();
        }
        i+=1;
    };

    let (input, _ ) = tag(":")(input)?;
    let (input,wordlist) =  nom::bytes::complete::is_not(":")(input)?;
    let input = match input.to_lowercase().as_str(){
            ":md5" => Transformations::MD5,
            ":md4" => Transformations::MD4,
            ":sha1" => Transformations::SHA1,
            ":sha256" => Transformations::SHA256,
            ":sha512" => Transformations::SHA512,
            ":" | "" => Transformations::Empty,
            _ =>  panic!("Transformation not allowed"),
        };
    wordlists.push(Wordlist{
            value:value.clone(),
            wordlist:wordlist.to_string(),
            transformation:input.clone(),
        });
    Ok(("",Wordlist{
            value:value.clone(),
            wordlist:wordlist.to_string(),
            transformation:input.clone(),
        }))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cli = Cli::parse();
    // println!("\n\ntable: {:?}, values: {:?}, wordlist: {:?}, output: {:?}", cli.table, cli.values, cli.wordlist, cli.output);

    // Check Conditions
    if cli.wordlist.len() != cli.values.len(){
        panic!("Need a wordlist for each value and vice versa");
    }

    let mut values: Vec<Value> = Vec::new();

    for x in 0..cli.values.len(){
        parse_values(&cli.values[x], &mut values);
    }
    let mut wordlists: Vec<Wordlist> = Vec::new();
    for x in cli.wordlist.iter(){
        parse_wordlist(x, &values, &mut wordlists);
    };
    let mut output = File::create("results.txt")?;

    let mut files: Vec<_> = wordlists.iter()
    .map(|wordlist| BufReader::new(File::open(wordlist.wordlist.as_str()).unwrap()).lines()).collect();
    println!("{:?}", files);


    // Todo! 
    // 1. Write create table query
    // 2. Implement reading from table name and writing to 
    // optional output file 
    // 3. Implement writing query in different dbms language

    // Write insert statements
    let mut pre_arg = String::new();
    pre_arg.push_str("insert into users values(");
    // Insert into fields
    for i in 0..values.len(){
        pre_arg.push_str(&("'".to_owned() + values[i].name.as_str() + "'"));
        if i != values.len() -1{
            pre_arg.push_str(",");
        }
    }
    pre_arg.push_str(") values(");

    while let Some(line) = files[0].next(){
        write!(&mut output, "{}",pre_arg,);
        write!(&mut output, "'{}'", line.unwrap());
        for k in 1..files.len(){
            write!(&mut output, "'{}'", files[k].next().unwrap().unwrap());
            if k != files.len()-1{
            write!(&mut output, ",");
            }
        }
        write!(&mut output, ");\n");
    }

    Ok(())
}

fn encrypt_md5<'i>(input: &'i str) -> String{
    format!("{:x}", md5::compute(input))
}
