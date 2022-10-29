use std::fs::File;
use std::io;
use std::io::prelude::*;

use blake2::{Blake2b512, Digest};
use clap::Parser;

/// Print or check BLAKE2 (512-bit) checksums.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // There's no difference between --binary and --text on GNU systems, so I'm not
    // sure how to implement and test this.
    /// read in binary mode
    #[arg(short, long)]
    binary: bool,

    /// read BLAKE2 sums from the FILEs and check them
    #[arg(short, long)]
    check: bool,

    files: Vec<String>,

    /// don't fail or report status for missing files
    #[arg(long)]
    ignore_missing: bool,

    /// digest length in bits; must not exceed the maximum for the blake2 algorithm and must be a multiple of 8
    #[arg(short, long, default_value_t = 128)]
    length: i32,

    /// don't print OK for each successfully verified file
    #[arg(long)]
    quiet: bool,

    /// don't output anything, status code shows success
    #[arg(long)]
    status: bool,

    /// exit non-zero for improperly formatted checksum lines
    #[arg(long)]
    strict: bool,

    /// create a BSD-style checksum
    #[arg(long)]
    tag: bool,

    /// read in text mode (default)
    #[arg(short, long)]
    text: bool,

    /// warn about improperly formatted files
    #[arg(short, long)]
    warn: bool,

    /// end each output line with NUL, not newline, and disable file name escaping
    #[arg(short, long)]
    zero: bool,
}

struct B2Hash {
    filename: String,
    hash: String,
}

fn main() {
    let args = Args::parse();
    let checksums = run(&args);

    for checksum in checksums {
        if args.check {
        } else {
            if args.length == 0 {
                // print the hex hash
                println!("{} {}", checksum.hash, checksum.filename);
            } else if args.length % 8 == 0 {
                // length must be a multiple of 8
                let slice = &checksum.hash[..args.length as usize];
                println!("{} {}", slice, checksum.filename);
            } else {
                println!("length ({}) is not a multiple of 8", args.length)
            }
        }
    }
}

fn run(args: &Args) -> Vec<B2Hash> {
    let mut retval = Vec::new();

    // TODO: This return type doesn't make sense b/c we could be dealing with
    // stdin or a vector of files to be read in.

    if !args.files.is_empty() {
        for filename in &args.files {
            let file = match File::open(&filename) {
                Err(why) => panic!("couldn't open: {}", why),
                Ok(file) => file,
            };

            let mut hasher = Blake2b512::new();
            let mut reader = io::BufReader::new(file);
            let mut buf = String::new();
            while reader.read_line(&mut buf).unwrap() > 0 {
                // Update the hasher with the next line in the file
                hasher.update(&buf);

                // clear the buffer for the next read
                buf.clear();
            }
            let res = hasher.finalize();

            retval.push(B2Hash {
                hash: format!("{:x}", res),
                filename: filename.to_string(),
            });
        }
    } else {
        // read from stdin
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let buf = line.unwrap();

            // Get the result of the b2sum hash
            let res = b2sum(buf);

            retval.push(B2Hash {
                hash: res.to_string(),
                filename: "-".to_string(),
            });
        }
    }

    retval
}

fn b2sum(buf: String) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(buf);

    let res = hasher.finalize();

    format!("{:x}", res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b2sum_hello() {
        let hash = b2sum(String::from("hello"));
        assert_eq!(
            &hash,
            "e4cfa39a3d37be31c59609e807970799caa68a19bfaa15135f165085e01d41a65ba1e1b146aeb6bd0092b49eac214c103ccfa3a365954bbbe52f74a2b3620c94"
        );
    }

    #[test]
    fn test_b2sum_hello_world() {
        let hash = b2sum(String::from("hello, world"));
        assert_eq!(
            &hash,
            "7355dd5276c21cfe0c593b5063b96af3f96a454b33216f58314f44c3ade92e9cd6cec4210a0836246780e9baf927cc50b9a3d7073e8f9bd12780fddbcb930c6d"
        );
    }
}
