use std::fs::{self, File};
use std::io::Write;
use std::io::BufReader;
use std::io::BufRead;
use std::path::PathBuf;
use md5;
use clap::Parser;
use nom::{
    bytes::complete::{is_not, tag, take_till},
    IResult,
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

    #[arg(short='o',long)]
    output: Option<String>,
}

#[derive(Debug)]
enum Constraint{
    NotNull,
    Unique,
    PrimaryKey,
    Check(Checks),
    Default(String),
    Empty,
}

#[derive(Debug)]
enum Checks{
    GreaterThan,
    LessThan,
    EqualTo,
    InBetween,
}

#[derive(Debug)]
enum Transformations{
    Empty,
    Transform(String),
    // MD5,
    // MD4,
    // SHA1,
    // SHA256,
    // SHA512,
}

#[derive(Debug)]
struct Value{
    name: String,
    data_type: String,
    constraint: Constraint,
}

struct Values{
    values: Vec<Value>,
}

#[derive(Debug)]
struct Wordlist{
    value: String,
    wordlist: String,
    transformation: Transformations,
}

fn parse_values(input: &str)->IResult<&str,Value>{
    let (input, name) = nom::bytes::complete::is_not(":")(input)?;
    let (input, _ ) = tag(":")(input)?;
    let (input,data_type) =  nom::bytes::complete::is_not(":")(input)?;
    if input == ""{
    Ok(("",Value{
            name:name.to_string(),
            data_type:data_type.to_string(),
            constraint:Constraint::Empty
        }))
    }
    else{
    let (input,_) = tag(":")(input)?;
    Ok(("",Value{
            name:name.to_string(),
            data_type:data_type.to_string(),
            constraint:Constraint::Default(input.to_string())
        }))
    }
}

fn parse_wordlist(input: &str) -> IResult<&str,Wordlist>{
    let (input, value) = nom::bytes::complete::is_not(":")(input)?;
    let (input, _ ) = tag(":")(input)?;
    let (input,wordlist) =  nom::bytes::complete::is_not(":")(input)?;
    if input == ""{
    Ok(("",Wordlist{
            value:value.to_string(),
            wordlist:wordlist.to_string(),
            transformation:Transformations::Empty
        }))
    }
    else{
    let (input,_) = tag(":")(input)?;
    Ok(("",Wordlist{
            value:value.to_string(),
            wordlist:wordlist.to_string(),
            transformation:Transformations::Transform(input.to_string())
        }))
    }
}

// fn parse_wordlist(input: &str)->IResult<&str,&str,()>{
//
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let cli = Cli::parse();
    println!("\n\ntable: {:?}, values: {:?}, wordlist: {:?}, output: {:?}", cli.table, cli.values, cli.wordlist, cli.output);

    println!("wordlist: {:?}", parse_wordlist(&cli.wordlist[0]));
    for x in cli.values.iter(){
        println!("value: {:?}", parse_values(x));
    };
    // TODO! 
    // Create State Machine for sql table creation

    // println!("wodlists: {:?}", cli.wordlist);
    // let f = File::open("usernames.txt")?;
    // let path = "results.txt";
    // let mut output = File::create(path)?;
    // let bufr = BufReader::new(f);
    // let k:u32 = bufr.lines().map(|l|{
    //     let line = l.unwrap();
    //     write!(&mut output, "insert into users(username,password) values('{}','{}');\n",line,encrypt_md5(&line));
    //     1
    // }).sum();
    // 
    Ok(())
}

fn encrypt_md5<'i>(input: &'i str) -> String{
    format!("{:x}", md5::compute(input))
}
