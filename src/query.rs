use crate::constraint::constraint_check;
use crate::parser::{Value, Wordlist};
use crate::transform::transform_input;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Write};

pub fn create_query(
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

pub fn create_insert_query(
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
