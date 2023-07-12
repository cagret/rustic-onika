mod index;

use std::process::exit;
use std::fs::File;
use std::io::{self, BufRead};
use structopt::StructOpt;
use index::Index; 

#[derive(Debug, StructOpt)]
#[structopt(name = "rustic-onika", about = "Description du programme")]
struct Options {
    #[structopt(
        short = "I",
        long = "index",
        help = "Input file of files to Index."
    )]
    index: Option<std::path::PathBuf>,

    #[structopt(
        short = "Q",
        long = "query",
        help = "Input file of file to Query."
    )]
    query: Option<std::path::PathBuf>,

   
    #[structopt(
        short = "K",
        long = "kmer",
        help = "Kmer size (31)"
    )]
    kmer: Option<i32>,

    #[structopt(
        short = "S",
        long = "sketch",
        help = "Set sketch size to 2^S (15)"
    )]
    sketch: Option<i32>,

    #[structopt(
        short = "W",
        long = "word",
        help = "Fingerprint size (12). Modify with caution, larger fingerprints enable queries with less false positive but increase EXPONENTIALLY the overhead as the index count S*2^W cells."
    )]
    word: Option<i32>,

    #[structopt(
        short = "E",
        long = "EGS",
        help = "Expected genome size (5,000,000)"
    )]
    egs: Option<i32>,


    #[structopt(
        long = "dist",
        help = "Generate a full, asymmetric distance matrix."
    )]
    dist: bool,


    #[structopt(
        short = "h",
        long = "help",
        help = "Print usage and exit."
    )]
    help: bool,
}

fn main() {
    let opts: Options = Options::from_args();
    let mut monindex = Index::new(10, 31, 8, 5000000, String::from("example.txt"));

     if let Some(list_file) = opts.index {
        if let Ok(file) = File::open(&list_file) {
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                if let Ok(genome_file) = line {
                    monindex.get_filename(&genome_file);
                }
            }
        } else {
            eprintln!("Unable to open the file '{}'", list_file.display());
            exit(1);
        }
         monindex.get_filename(list_file.to_str().unwrap());
     }

    if opts.dist {
        monindex.print_matrix();
    }
  
    
    if opts.help {
        Options::clap().print_help().unwrap();
        println!();
        exit(0);
    }

    println!("+-------------------------------------------------------------------+");
    println!("|                            Informations                           |");
    println!("+-----------------------------------+-------------------------------+");
println!("| k-mer size                        |{:>30} |", monindex.get_k());
println!("| S                                 |{:>30} |", monindex.get_f());
println!("| Number of fingerprints            |{:>30} |", monindex.get_fingerprint_range());
println!("| W                                 |{:>30} |", monindex.get_w());
println!("| E                                 |{:>30} |", monindex.get_e());
println!("| Number of indexed genomes         |{:>30} |", monindex.get_nb_genomes());
    println!("+-------------------------------------------------------------------+");
    println!("|                                  Done                             |");
    println!("+-----------------------------------+-------------------------------+");
}

