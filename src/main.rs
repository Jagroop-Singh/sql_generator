use clap::Parser;
use sql_generator::parser::{parse_values, parse_wordlist, Value, Wordlist};
use sql_generator::query::{create_insert_query, create_query};
use std::fs::File;
use std::io::{BufRead, BufReader};

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

    #[arg(short = 'o', long, default_value = "results.txt")]
    output: String,
}

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

    let mut output = File::create(&cli.output)?;
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
