use regex_automata::{dfa::{dense::DFA, regex::Regex}, Match};
use std::fs::File;
use std::io::Write;


// Generates binary representations of a regex DFA for matching email addresses (in generate_username) 
// and saves them into binary files.
fn main() {
    let re = Regex::new(r"This email was meant for (@\w+)").unwrap();
    // #[repr(C)]
    // struct Aligned<B: ?Sized> {
    //     _align: [u32; 0],
    //     bytes: B,
    // }
    // let mut buf_fwd = Aligned { _align: [], bytes: [0u8; 4 * (1<<10)] };
    // let mut buf_rev = Aligned { _align: [], bytes: [0u8; 4 * (1<<10)] };
    // let written_fwd = re.forward().write_to_native_endian(&mut buf_fwd.bytes).expect("Failed to write DFA to buffer");
    // let written_rev = re.reverse().write_to_native_endian(&mut buf_rev.bytes).expect("Failed to write DFA to buffer");
    // let mut file = File::create("dfa_fwd_bytes.bin").unwrap();
    // file.write_all(&buf_fwd.bytes[..written_fwd]).unwrap();
    // let mut file = File::create("dfa_rev_bytes.bin").unwrap();
    // file.write_all(&buf_rev.bytes[..written_rev]).unwrap();
    let (fwd_bytes, fwd_pad) = re.forward().to_bytes_little_endian();
    let (rev_bytes, rev_pad) = re.reverse().to_bytes_little_endian();
    // let mut file = File::create("dfa_fwd_bytes.bin").unwrap();
    // file.write_all(&fwd_bytes[fwd_pad..]).unwrap();
    // let mut file = File::create("dfa_rev_bytes.bin").unwrap();
    // file.write_all(&rev_bytes[rev_pad..]).unwrap();
    std::fs::write("dfa_fwd_bytes.bin", &fwd_bytes[fwd_pad..]).unwrap();
    std::fs::write("dfa_rev_bytes.bin", &rev_bytes[rev_pad..]).unwrap();
}
