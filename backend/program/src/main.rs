#![no_main]
sp1_zkvm::entrypoint!(main);
use base64::prelude::*;
use regex::Regex;
use regex_automata::{dfa::{Automaton, dense, regex::Regex as AutomataRegex}, Match, util::{lazy::Lazy, wire::AlignAs},};
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Sign, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
struct DKIM {
    public_key: String,
    signature: String,
    headers: String,
    body: String,
    body_hash: String,
    signing_domain: String,
    selector: String,
    algo: String,
    format: String,
    modulus_length: u32, // unused
}

static FWD_ALIGNED: &AlignAs<[u8], u32> = &AlignAs {
    _align: [],
    #[cfg(target_endian = "little")]
    bytes: *include_bytes!("../../../generate_regex_bin/dfa_fwd_bytes.bin"),
};
static REV_ALIGNED: &AlignAs<[u8], u32> = &AlignAs {
    _align: [],
    #[cfg(target_endian = "little")]
    bytes: *include_bytes!("../../../generate_regex_bin/dfa_rev_bytes.bin"),
};

// The seriallized DFA regex for "This email was meant for (@\w+)".
static DFA_FWD_BYTES: &[u8] = &FWD_ALIGNED.bytes;
static DFA_REV_BYTES: &[u8] = &REV_ALIGNED.bytes;

pub fn main() {
    let dkim = sp1_zkvm::io::read::<DKIM>();
    let crypto_address = sp1_zkvm::io::read::<String>();
    let fwd: dense::DFA<&[u32]> = dense::DFA::from_bytes(&DFA_FWD_BYTES).expect("Failed to convert bytes to DFA").0;
    let rev: dense::DFA<&[u32]> = dense::DFA::from_bytes(&DFA_REV_BYTES).expect("Failed to convert bytes to DFA").0;
    let re = AutomataRegex::builder().build_from_dfas(fwd, rev);

    let body_verified = verify_body(&dkim);
    let signature_verified = verify_signature(&dkim);
    let from_address_verified = verify_from_address(&dkim);
    let is_pw_reset_email = verify_pw_reset_email(&dkim);
    let twitter_username = get_twitter_username(&dkim, &re); 
    let twitter_proved = body_verified
        && signature_verified
        && from_address_verified
        && is_pw_reset_email
        && twitter_username.len() > 0;

    sp1_zkvm::io::commit(&twitter_username);
    sp1_zkvm::io::commit(&crypto_address);
    sp1_zkvm::io::commit(&twitter_proved);
}

fn verify_body(dkim: &DKIM) -> bool {
    // get sha256 hash of body
    let mut hasher = Sha256::new();
    hasher.update(dkim.body.as_bytes());
    let hash = hasher.finalize();

    // encode hash to base64
    let base64_hash = BASE64_STANDARD.encode(&hash);

    // compare computed body hash with signed body hash & print if fails
    if base64_hash != dkim.body_hash {
        println!("Body Invalid");
        return false;
    } else {
        return true;
    }
}

fn verify_signature(dkim: &DKIM) -> bool {
    // signature scheme: rsa-sha256
    // 1. get sha256 hash of header
    let mut hasher = Sha256::new();
    hasher.update(dkim.headers.as_bytes());
    let hash = hasher.finalize();

    // 2. decode the public key from PEM format
    let public_key = RsaPublicKey::from_public_key_pem(&dkim.public_key)
        .expect("error decoding public key into PEM format");

    // 3. decode the signature from base64 into binary
    let signature = BASE64_STANDARD
        .decode(&dkim.signature)
        .expect("error decoding signature into binary");

    // 4. verify the signature
    // RSASSA-PKCS1-V1_5 magic padding bytes
    // https://crypto.stackexchange.com/questions/86385/initial-value-for-rsa-and-sha-256-signature-encoding
    let prefix: Box<[u8]> = Box::new([
        0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01,
        0x05, 0x00, 0x04, 0x20,
    ]);
    // SHA-256 produces hash output of 32 bytes
    let hash_len = Some(32);
    let padding = Pkcs1v15Sign {
        hash_len: hash_len,
        prefix: prefix,
    };
    let result = public_key.verify(padding, &hash, &signature);

    // Print if signature is invalid
    if !result.is_ok() {
        println!("Signature Invalid");
        return false;
    } else {
        return true;
    }
}

fn verify_from_address(dkim: &DKIM) -> bool {
    // Get email address after from: in header
    let re = Regex::new(r"\r\nfrom:.*?<(.+?)>").unwrap();

    let mut match_count = 0;
    let mut email = String::new();

    for captures in re.captures_iter(dkim.headers.as_str()) {
        match_count += 1;
        if match_count > 1 {
            println!("Only one 'from' address is supported.");
            return false;
        }
        email = captures[1].to_string();
    }

    if match_count == 0 {
        println!("No match found for the 'from' field.");
        return false;
    }

    if !email.ends_with("@x.com") {
        println!("Email address is not from x.com.");
        return false;
    }

    true
}

fn verify_pw_reset_email(dkim: &DKIM) -> bool {
    // Verify the subject in the headers
    // "This email was meant for @username" in the body is already verified in the get_twitter_username fn.
    // These two methods are sufficient for verifying that the email is a password reset email.
    let subject_re = Regex::new(r"\r\nsubject:Password reset request").unwrap();
    if !subject_re.is_match(&dkim.headers) {
        println!("Email is not password reset email");
        return false;
    }

    true
}

fn get_twitter_username(dkim: &DKIM, re: &AutomataRegex<dense::DFA<&[u32]>>) -> String {
    let matches: Vec<Match> = re.find_iter(&dkim.body).collect();
    if matches.len() > 0 {
        let start_index = matches[0].start();
        let end_index = matches[0].end();
        let substring = &dkim.body[start_index..end_index].to_string();
        let start_username = substring.find("@").unwrap();
        let username = &substring[start_username..];
        return username.to_string();
    }
    else {
        println!("No twitter username found");
        return String::new(); // blank string if no/invalid twitter username
    }
}
