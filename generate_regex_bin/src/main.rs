use regex_automata::{dfa::{dense, regex::Regex}, Match};
use std::fs::File;
use std::io::Write;


// Generates binary representations of a regex DFA 
// for matching email addresses (in generate_username) and saves them into binary files
fn main() {
    let re = Regex::new(r"This email was meant for (@\w+)").unwrap();
    let (fwd_bytes, _) = re.forward().to_bytes_native_endian();
    let (rev_bytes, _) = re.reverse().to_bytes_native_endian();
    let mut file = File::create("dfa_fwd_bytes.bin").unwrap();
    file.write_all(&fwd_bytes).unwrap();
    let mut file = File::create("dfa_rev_bytes.bin").unwrap();
    file.write_all(&rev_bytes).unwrap();
}