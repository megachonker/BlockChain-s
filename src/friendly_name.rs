use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead};
use std::net::SocketAddr;

use tracing::debug;

pub fn get_friendly_name(addr: SocketAddr) -> io::Result<String> {
    // Step 1 & 2: Convert SocketAddr to string and hash it using DefaultHasher
    let addr_str = addr.to_string();
    let mut hasher = DefaultHasher::new();
    addr_str.hash(&mut hasher);
    let hash_result = hasher.finish();
    
    // Step 3 & 4: Open the file and get the number of lines
    let file = File::open("./name.list")?;
    let reader = io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    
    if lines.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "The file is empty"));
    }
    
    // Step 5 & 6: Calculate the line index using modulo operation and return the string at that index
    let max_line_index = lines.len() as u64;
    let line_index = (hash_result % max_line_index) as usize;
    debug!("{}",lines[line_index].clone());
    Ok(lines[line_index].clone())
}

//temporary 
pub fn get_fake_id(friendly_name:&String) -> u64{
    let mut hasher = DefaultHasher::new();
    friendly_name.hash(&mut hasher);
    hasher.finish()
}


//des scénario de test avec 2 node par ex --> oui mais il pouvoir les arreter et le temps de clalcul d'un bloc est alea
//possible de lancer les calcule de block avec une seed par exemple est de simplifier le nombre d'itération
#[cfg(test)]
mod tests {
    use crate::friendly_name::get_friendly_name;


    #[test]
    fn consistance_test() {
        assert_eq!(get_friendly_name("127.0.0.2:8080".parse().unwrap()).unwrap(),"Ivy");
        assert_eq!(get_friendly_name("127.0.0.1:8080".parse().unwrap()).unwrap(),"Zoe");
    }

}
