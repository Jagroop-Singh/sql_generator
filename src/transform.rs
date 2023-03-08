use crate::parser::{Transformations, Wordlist};
use md4::Md4;
use md5;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};

pub fn transform_input<'a>(s: &'a str, w: &Wordlist) -> String {
    match w.transformation {
        Transformations::MD4 => encrypt_md4(s),
        Transformations::MD5 => encrypt_md5(s),
        Transformations::SHA1 => encrypt_sha1(s),
        Transformations::SHA256 => encrypt_sha256(s),
        Transformations::SHA512 => encrypt_sha512(s),
        Transformations::Empty => s.to_string(),
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
