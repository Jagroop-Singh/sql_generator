use crate::parser::{Constraint, Wordlist};
use std::collections::HashSet;

pub fn constraint_check(
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
