#![allow(clippy::too_many_arguments)]

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

extern crate clap;
use clap::{App, Arg};

use boomphf::*;

use dedup_by::dedup_by;

fn for_each_line_in_paf(paf_filename: &str, mut callback: impl FnMut(&str)) {
    let file = File::open(paf_filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        //let l = line.unwrap();
        callback(&line.unwrap());
    }
}

struct PafFile {
    paf_filename: String,
    names: Vec<String>,
    seq_name_mphf: Mphf<String>,
}

impl PafFile {
    fn new(paf_filename: &str) -> Self {
        let mut names: Vec<String> = Vec::new();
        for_each_line_in_paf(paf_filename, |l: &str| {
            names.push(l.split('\t').nth(0).unwrap().into());
            names.push(l.split('\t').nth(5).unwrap().into());
        });
        names.sort();
        dedup_by(&mut names, |a, b| { a == b });
        let seq_name_mphf = Mphf::new(1.7, &names);
        PafFile {
            paf_filename: paf_filename.to_string(),
            names,
            seq_name_mphf,
        }
    }
    fn get_id(self: &PafFile, name: &str) -> u64 {
        self.seq_name_mphf.hash(&name.to_string()) + 1
    }
    fn rewrite_with_ids(self: &PafFile) {
        for_each_line_in_paf(&self.paf_filename, |l: &str| {
            for (i, f) in l.split("\t").enumerate() {
                if i == 0 {
                    print!("{}", self.get_id(f));
                } else if i == 5 {
                    print!("\t{}", self.get_id(f));
                } else {
                    print!("\t{}", f);
                }
            }
            println!();
        });
    }
    fn to_pajek_net(self: &PafFile) {
        println!("*Vertices {}", self.names.len());
        for name in &self.names {
            println!("{} \"{}\"", self.get_id(&name), &name);
        }
        println!("*arcs");
        for_each_line_in_paf(&self.paf_filename, |l: &str| {
            let v: Vec<&str> = l.split("\t").collect();
            println!("{} {}", self.get_id(v[0]), self.get_id(v[5]));
        });
    }
}

fn main() -> io::Result<()> {
    let matches = App::new("pafnet")
        .version("0.1.0")
        .author("Erik Garrison <erik.garrison@gmail.com>")
        .about("Project PAF into network formats")
        .arg(Arg::with_name("INPUT")
             .required(true)
             .takes_value(true)
             .index(1)
             .help("input PAF file"))
        .arg(Arg::with_name("net")
             .short("n")
             .long("net")
             .help("Write Pajeck Net format representing the pairs of sequences aligned in the PAF."))
        .arg(Arg::with_name("rewrite-paf")
             .short("r")
             .long("rewrite-paf")
             .help("Rewrite the input PAF using the internal IDs for query and target sequences."))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    let paf = PafFile::new(filename);

    if matches.is_present("net") {
        paf.to_pajek_net();
    } else if matches.is_present("rewrite-paf") {
        paf.rewrite_with_ids();
    }

    Ok(())
}
