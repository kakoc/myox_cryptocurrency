use sha2::{Digest, Sha256};

pub fn gen_hash(parts: Vec<String>) -> String {
    let input: String = parts.into_iter().collect();

    format!("{:x}", Sha256::digest(&input.as_bytes()))
        .chars()
        .map(|v| i64::from_str_radix(&format!("{}", v), 16).unwrap())
        .map(|v| {
            let bin = format!("{:b}", v);
            format!("{}{}", "0".repeat(4 - bin.to_string().len()), bin)
        })
        .collect::<String>()
}

pub fn to_binary(string: &str) -> String {
    Sha256::digest(&string.as_bytes())
        .iter()
        .map(|v| format!("{:b}", v))
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}
