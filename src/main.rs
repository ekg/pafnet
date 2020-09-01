#![allow(clippy::too_many_arguments)]

use std::fs::File;
use std::io::{self, prelude::*, BufReader};

extern crate clap;
use clap::{App, Arg};

use boomphf::*;

use dedup_by::dedup_by;

use std::collections::HashMap;

fn for_each_line_in_file(paf_filename: &str, mut callback: impl FnMut(&str)) {
    let file = File::open(paf_filename).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
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
        for_each_line_in_file(paf_filename, |l: &str| {
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
        for_each_line_in_file(&self.paf_filename, |l: &str| {
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
        for_each_line_in_file(&self.paf_filename, |l: &str| {
            let q = l.split('\t').nth(0).unwrap().into();
            let t = l.split('\t').nth(5).unwrap().into();
            println!("{} {}", self.get_id(q), self.get_id(t));
        });
    }
    fn to_edgelist(self: &PafFile) {
        for_each_line_in_file(&self.paf_filename, |l: &str| {
            let q = l.split('\t').nth(0).unwrap().into();
            let t = l.split('\t').nth(5).unwrap().into();
            println!("{} {}", self.get_id(q), self.get_id(t));
        });
    }
    fn to_nodelist(self: &PafFile) {
        for name in &self.names {
            print!("{} {}", self.get_id(&name), &name);
        }
    }
    fn to_gexf(self: &PafFile, mut get_color: impl FnMut(&str) -> (u16, u16, u16)) {
        println!(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        println!(r#"<gexf xmlns="http://www.gexf.net/1.2draft" xmlns:viz="http://www.gexf.net/1.1draft/viz" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.gexf.net/1.2draft http://www.gexf.net/1.2draft/gexf.xsd" version="1.2">"#);
        println!("<graph>");
        println!("<nodes>");
        for name in &self.names {
            print!(r#"<node id="{}" label="{}">"#, self.get_id(&name), &name);
            let (r, g, b) = get_color(&name);
            print!(r#"<viz:color r="{}" g="{}" b="{}" a="1.0"/>"#, r, g, b);
            println!(r#"</node>"#);
        }
        println!("</nodes>");
        println!("<edges>");
        let mut i = 1;
        for_each_line_in_file(&self.paf_filename, |l: &str| {
            let q = l.split('\t').nth(0).unwrap().into();
            let t = l.split('\t').nth(5).unwrap().into();
            println!(r#"<edge id="{}" source="{}" target="{}" />"#, i, self.get_id(q), self.get_id(t));
            i += 1;
        });
        println!("</edges>");
        println!("</graph>");
        println!("</gexf>");
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
        .arg(Arg::with_name("gexf")
             .short("g")
             .long("gexf")
             .help("Write GEXF format representing the pairs of sequences aligned in the PAF."))
        .arg(Arg::with_name("net")
             .short("n")
             .long("net")
             .help("Write Pajeck Net format representing the pairs of sequences aligned in the PAF."))
        .arg(Arg::with_name("edgelist")
             .short("e")
             .long("edgelist")
             .help("Write edge list representing the pairs of sequences aligned in the PAF."))
        .arg(Arg::with_name("nodelist")
             .short("l")
             .long("nodelist")
             .help("Write the id to sequence name mapping."))
        .arg(Arg::with_name("colors")
             .short("c")
             .long("colors")
             .takes_value(true)
             .help("Take colors for nodes from the "))
        .arg(Arg::with_name("prefix-char")
             .short("p")
             .long("prefix-char")
             .takes_value(true)
             .help("Split each sequence name on this character, taking the prefix as a group identifier for coloring"))
        .arg(Arg::with_name("rewrite-paf")
             .short("r")
             .long("rewrite-paf")
             .help("Rewrite the input PAF using the internal IDs for query and target sequences."))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    let paf = PafFile::new(filename);

    let prefix_char = if matches.is_present("prefix-char") {
        matches.value_of("prefix-char").unwrap()
    } else {
        ""
    };

    let mut colors = HashMap::new();
    if matches.is_present("colors") {
        let color_file = matches.value_of("colors").unwrap();
        for_each_line_in_file(color_file, |l: &str| {
            let mut name = "".to_string();
            let mut r = 0;
            let mut g = 0;
            let mut b = 0;
            for (i, f) in l.split_ascii_whitespace().enumerate() {
                match i {
                    0 => name = f.to_string(),
                    1 => r = f.parse::<u16>().unwrap(),
                    2 => g = f.parse::<u16>().unwrap(),
                    3 => b = f.parse::<u16>().unwrap(),
                    _ => {}
                }
            }
            colors.insert(name, (r, g, b));
        });
    }

    if matches.is_present("net") {
        paf.to_pajek_net();
    } else if matches.is_present("rewrite-paf") {
        paf.rewrite_with_ids();
    } else if matches.is_present("edgelist") {
        paf.to_edgelist();
    } else if matches.is_present("nodelist") {
        paf.to_nodelist();
    } else if matches.is_present("gexf") {
        paf.to_gexf(|name: &str| {
            if colors.len() > 0 {
                let q = if prefix_char == "" {
                    name
                } else {
                    name.split(prefix_char).nth(0).unwrap()
                };
                match colors.get(q) {
                    Some(color) => *color,
                    None => (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            }
        });
    }

    Ok(())
}
