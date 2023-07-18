use std::fs::File;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{BufReader, BufRead};
use std::string::String;
use std::vec::Vec;
use arrayvec::ArrayVec;
use rayon::prelude::*;

type Kmer = u64;
type Gid = u32;

const MUTEX_NUMBER: usize = 65536;

pub struct Index {
    // CONSTANTS
    k: u32,                         // kmer size
    f: u32,                         // fingerprint used
    w: u32,                         // fingerprint size
    m: u32,                         // Minhash size
    genome_numbers: Arc<Mutex<u32>>, // Number of genomes
    e: u32,                         // Expected genome size (5000000)
    fingerprint_range: u64,         // 2^w

    maximal_remainder: u32,    // 2^H-1
    lf: u32,                   // log2(F)
    mask_fingerprint: u64,     // 2^(64-lf)-1
    mi: u64,                   // -1
    offset_update_kmer: u64,
    min_score: u32,
    filename: String,
    buckets: Vec<Vec<Gid>>,             // The details of "Buckets" and "Buckets_pos" are not clear in the initial code.
    buckets_pos: Vec<Vec<u16>>,         // For now, they are represented as 2D vectors.
    lock: ArrayVec<[Mutex<u32>; MUTEX_NUMBER]>,
    filenames: Arc<Mutex<Vec<String>>>,
    file_sketches: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

impl Index {
    pub fn new(lf: u32, k: u32, w: u32, e: u32, filename: String) -> Index {
        let f = 1u32 << lf;
        let fingerprint_range = 1u64 << w;
        let offset_update_kmer = 1u64 << (2 * k);
        let mi = !0u64;
        let maximal_remainder = (1 << w) - 1;
        let mask_fingerprint = (1u64 << (64 - lf)) - 1;
        let min_score = 0; // Assuming min_score should be initialized to 0

        let mut lock = ArrayVec::new();
        for _ in 0..MUTEX_NUMBER {
            lock.push(Mutex::new(0));
        }
        let buckets = vec![Vec::<Gid>::new(); fingerprint_range as usize];
        let buckets_pos = vec![Vec::<u16>::new(); fingerprint_range as usize];
        Index {
            k,
            f,
            w,
            m: 0, // This needs to be set to the correct initial value
            genome_numbers: Arc::new(Mutex::new(0)),
            e,
            fingerprint_range,
            maximal_remainder,
            lf,
            mask_fingerprint,
            mi,
            offset_update_kmer,
            min_score,
            filename,
            //buckets: Vec::new(),
            //buckets_pos: Vec::new(),
            buckets: vec![Vec::new(); fingerprint_range as usize], // initialize here
            buckets_pos: vec![Vec::new(); fingerprint_range as usize], // initialize here
            lock,
            filenames: Arc::new(Mutex::new(Vec::new())),
            file_sketches: Arc::new(Mutex::new(HashMap::new())),
        }
    }


    pub fn get_k(&self) -> u32 {
        self.k
    }

    pub fn get_f(&self) -> u32 {
        self.f
    }

    pub fn get_fingerprint_range(&self) -> u64 {
        self.fingerprint_range
    }

    pub fn get_w(&self) -> u32 {
        self.w
    }

    pub fn get_e(&self) -> u32 {
        self.e
    }

    // Destructeur
    fn cleanup(&mut self) {
        // Libérer la mémoire
        self.buckets.clear();
        self.buckets_pos.clear();

    }

    pub fn get_nb_genomes(&self) -> u32 {
        *self.genome_numbers.lock().unwrap()
    }

    pub fn exists_test(&self, name: &str) -> bool {
        Path::new(name).exists()
    }


    // This is a placeholder implementation, you need to fill in the actual logic
    pub fn asm_log2(&self, _x: u64) -> u64 {
        0
    }

    pub fn get_data_type(&self, filename: &str) -> char {
        if filename.find(".fq").is_some() {
            return 'Q';
        }
        if filename.find(".fastq").is_some() {
            return 'Q';
        }
        'A'
    }

    pub fn get_filename(&mut self, filestr: &str) {
        if cfg!(debug_assertions) {
            dbg!(filestr);
        }
        let file = File::open(filestr).expect("Unable to open the file");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(filename) = line {
                if cfg!(debug_assertions) {
                    dbg!(&filename);
                }
                if filename.len() > 2 && self.exists_test(&filename) {

                    let id = {

                        let mut genome_numbers = self.genome_numbers.lock().unwrap();

                        let id = *genome_numbers;

                        *genome_numbers += 1;

                        id

                    };
                    println!("Adding file: '{}'", filename);
                    self.insert_file(&filename, id);
                    println!("File: '{}' added", filename);
                }
            }
        }
    }


    pub fn insert_sketch(&mut self, sketch: &[u64], genome_id: u32) {
        let mutex_number = self.lock.len() as u64;
        sketch.iter().enumerate().for_each(|(i, &val)| {
            if cfg!(debug_assertions) {
                dbg!(val);
            }
            // Check if val is within the size of buckets and buckets_pos
            if val < self.fingerprint_range && (val as usize) < self.buckets.len() && (val as usize) < self.buckets_pos.len() {
                let lock_index = (val + (i as u64) * self.fingerprint_range) % mutex_number;
                // Check if lock_index is within the size of self.lock
                if (lock_index as usize) < self.lock.len() {
                    let lock = &self.lock[lock_index as usize];
                    if cfg!(debug_assertions) {
                        dbg!(lock_index);
                    }
                    let guard = lock.lock().unwrap();
                    if cfg!(debug_assertions) {
                        dbg!(&self.buckets.len(), &val);
                    }
                    self.buckets[val as usize].push(genome_id);
                    self.buckets_pos[val as usize].push(i as u16);
                    drop(guard);
                }
            }
        });
    }



    pub fn insert_file(&mut self, filestr: &str, identifier: u32) {
        if cfg!(debug_assertions) {
            dbg!(filestr, identifier);
        }
        //let r#type = self.get_data_type(filestr); // Utilise self pour appeler la méthode de la structure
        let file = File::open(filestr).expect("Unable to open the file");
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(ref_value) = line {
                dbg!(&ref_value);
                let mut ref_value = ref_value.to_string();
                //let mut kmer_sketch = Vec::new();
                let mut sketch = vec![u64::MAX; self.f as usize];

                if ref_value.len() > self.k as usize {
                    self.compute_sketch(&mut ref_value, &mut sketch); // Assuming this function is defined elsewhere
                }

                {
                    let file_sketches = Arc::clone(&self.file_sketches);
                    let mut file_sketches = file_sketches.lock().unwrap();
                    dbg!(&ref_value, &sketch);
                    file_sketches.insert(ref_value.clone(), sketch.clone());
                }

                self.insert_sketch(&sketch, identifier); // Assuming this function is defined elsewhere
            }
        }
    }

    pub fn compute_sketch(&self, reference: &str, sketch: &mut Vec<u64>) {
        if sketch.len() != self.f as usize {
            *sketch = vec![u64::MAX; self.f as usize];
        }
        println!("------------------SKETCH--------------------");
        let mut empty_cell = self.f;
        let mut S_kmer = self.str2numstrand(&reference[0..(self.k as usize - 1)]);
        let mut RC_kmer = self.rcb(S_kmer);

        for i in 0..(reference.len() - self.k as usize) {
            self.update_kmer(&mut S_kmer, reference.chars().nth((i + self.k as usize - 1) as usize).unwrap());
            self.update_kmer_RC(&mut RC_kmer, reference.chars().nth((i + self.k as usize - 1) as usize).unwrap());

            let canon = S_kmer.min(RC_kmer);
            let hashed = self.revhash64(canon);
            let bucket_id = self.unrevhash64(canon) >> (64 - self.lf);
            let fp = hashed;

            if sketch[bucket_id as usize] == u64::MAX {
                empty_cell -= 1;
                sketch[bucket_id as usize] = fp;
            } else if sketch[bucket_id as usize] > fp {
                sketch[bucket_id as usize] = fp;
            }
        }

        self.sketch_densification(sketch, empty_cell);

        for i in 0..sketch.len() {
            sketch[i] = self.get_perfect_fingerprint(sketch[i]);
        }

        println!("------------------SKETCH END--------------------");
    }


    pub fn Biogetline(file_path: &str, result: &mut String, header: &mut String) {
        let file = File::open(file_path).expect("Error opening file");
        let mut reader = BufReader::new(file);
        result.clear();

        let first_char = reader
            .fill_buf()
            .expect("Error reading buffer")
            .first()
            .cloned()
            .unwrap_or(0 as u8) as char;

        match first_char {
            'Q' => {
                reader.read_line(header).expect("Error reading line");
                reader.read_line(result).expect("Error reading line");
                reader.read_line(&mut String::new()).expect("Error reading line");
                reader.read_line(&mut String::new()).expect("Error reading line");
            }
            'A' => {
                let mut discard = String::new();
                reader.read_line(&mut discard).expect("Error reading line");
                let mut c = reader
                    .fill_buf()
                    .expect("Error reading buffer")
                    .first()
                    .cloned()
                    .unwrap_or(0 as u8) as char;

                while c != '>' && c != std::char::REPLACEMENT_CHARACTER {
                    reader.read_line(&mut discard).expect("Error reading line");
                    result.push_str(&discard);
                    c = reader
                        .fill_buf()
                        .expect("Error reading buffer")
                        .first()
                        .cloned()
                        .unwrap_or(0 as u8) as char;
                }
            }
            _ => {}
        }

        if result.len() < 31 {
            result.clear();
            header.clear();
        }

    }


    pub fn rcb(&self, mut min: u64) -> u64 {
        let mut res = 0;
        let mut offset = 1;
        offset <<= 2 * self.k - 2;
        for _ in 0..self.k {
            res += (3 - (min % 4)) * offset;
            min >>= 2;
            offset >>= 2;
        }
        res
    }

    pub fn nuc2int(&self, c: char) -> u64 {
        match c {
            'C' => 1,
            'G' => 2,
            'T' => 3,
            _ => 0,
        }
    }

    pub fn nuc2intrc(&self, c: char) -> u64 {
        match c {
            'A' => 3,
            'C' => 2,
            'G' => 1,
            _ => 0,
        }
    }

    pub fn str2numstrand(&self, str: &str) -> u64 {
        let mut res = 0;
        for c in str.chars() {
            res <<= 2;
            res += match c {
                'A' | 'a' => 0,
                'C' | 'c' => 1,
                'G' | 'g' => 2,
                'T' | 't' => 3,
                _ => return 0,
            };
        }
        res
    }

    pub fn revhash64(&self, mut x: u64) -> u64 {
        x = ((x >> 32) ^ x).wrapping_mul(0xD6E8FEB86659FD93);
        x = ((x >> 32) ^ x).wrapping_mul(0xD6E8FEB86659FD93);
        (x >> 32) ^ x
    }

    pub fn unrevhash64(&self, mut x: u64) -> u64 {
        x = ((x >> 32) ^ x).wrapping_mul(0xCFEE444D8B59A89B);
        x = ((x >> 32) ^ x).wrapping_mul(0xCFEE444D8B59A89B);
        (x >> 32) ^ x
    }

    pub fn update_kmer(&self, min: &mut u64, nuc: char) {
        *min <<= 2;
        *min += self.nuc2int(nuc);
        *min %= self.offset_update_kmer;
    }

    pub fn update_kmer_RC(&self, min: &mut u64, nuc: char) {
        *min >>= 2;
        *min += self.nuc2intrc(nuc) << (2 * self.k - 2);
    }

    pub fn hash_family(&self, x: u64, factor: u64) -> u64 {
        self.unrevhash64(x) + factor * self.revhash64(x)
    }
    pub fn get_perfect_fingerprint(&self, hashed: u64) -> u64 {
        let b = hashed;
        let mut frac = (1u64 << 63 - b) as f64 / (1u64 << 63) as f64;
        frac = frac.powf(self.e as f64 / self.f as f64);
        frac = 1.0 - frac;
        (self.fingerprint_range as f64 * frac) as u64
    }


    pub fn sketch_densification(&self, sketch: &mut Vec<u64>, empty_cell: u32) {
        let size = sketch.len();
        let empty_cell = Arc::new(Mutex::new(empty_cell));
        let mi = u64::MAX;

        while *empty_cell.lock().unwrap() != 0 {
            for i in 0..size {
                if sketch[i] != mi {
                    let hash = self.hash_family(sketch[i], mi.into()) % self.f as u64;
                    let mut empty_cell_guard = empty_cell.lock().unwrap();
                    if *empty_cell_guard > 0 && sketch[hash as usize] == mi {
                        let temp = std::mem::replace(&mut sketch[i], 0);
                        let _ = std::mem::replace(&mut sketch[hash as usize], temp);
                        sketch[hash as usize] = temp;
                        *empty_cell_guard -= 1;
                    }
                }
            }
        }
    }

    pub fn query_sketch(&self, sketch: &[u64]) -> Vec<u32> {
        let genome_numbers = self.genome_numbers.lock().unwrap();
        let mut result = vec![0; *genome_numbers as usize];

        for (i, val) in sketch.iter().enumerate() {
            if *val < self.fingerprint_range {
                let bucket = &self.buckets[*val as usize];
                let bucket_pos = &self.buckets_pos[*val as usize];

                for j in 0..bucket.len() {
                    if bucket_pos[j] == i as u16 {
                        result[bucket[j] as usize] += 1;
                    }
                }
            }
        }

        result
    }



    pub fn merge_sketch(&self, sketch1: &mut [i32], sketch2: &[i32]) {
        for i in 0..sketch1.len() {
            sketch1[i] = sketch1[i].min(sketch2[i]);
        }
    }
    pub fn print_matrix(&self) {
        println!("PRINT MATRIX: ");
        let size = self.genome_numbers.lock().unwrap().clone() as usize;
        let mut matrix = vec![vec![0.0; size]; size];

        for i in 0..size {
            println!("i:'{}' new", i);
            let filenames_lock = self.filenames.lock().unwrap();
            if i < filenames_lock.len() {
                let filename_i = &filenames_lock[i];
                let file_sketches = self.file_sketches.lock().unwrap();
                let sketch_i = match file_sketches.get(filename_i) {
                    Some(sketch) => sketch,
                    None => {
                        eprintln!("Could not find sketch for filename '{}'", filename_i);
                        continue;
                    }
                };

                for j in i..size {
                    println!("j:'{}' new", j);
                    if j < filenames_lock.len() {
                        if i == j {
                            matrix[i][j] = 0.0;
                        } else {
                            let filename_j = &filenames_lock[j];
                            let sketches = self.file_sketches.lock().unwrap();
                            let sketch_j = match sketches.get(filename_j) {
                                Some(sketch) => sketch,
                                None => {
                                    eprintln!("Could not find sketch for filename '{}'", filename_j);
                                    continue;
                                }
                            };

                            let intersection: Vec<&u64> = sketch_i
                                .iter()
                                .filter(|sketch| sketch_j.contains(sketch))
                                .collect();

                            let mut union_set: HashSet<u64> = HashSet::new();
                            union_set.extend(sketch_i);
                            union_set.extend(sketch_j);

                            let jaccard_distance =
                                1.0 - (intersection.len() as f64) / (union_set.len() as f64);

                            matrix[i][j] = jaccard_distance;
                            matrix[j][i] = jaccard_distance;
                        }
                    }
                }
            }
        }

        print!("##Names ");
        let filenames_guard = self.filenames.lock().unwrap();
        for filename in &*filenames_guard {
            print!("{}\t", filename);
        }
        println!();

        for i in 0..size {
            let filenames_guard = self.filenames.lock().unwrap();
            if i < filenames_guard.len() {
                print!("{}\t", filenames_guard[i]);
                for j in 0..size {
                    if j < filenames_guard.len() {
                        if i == j {
                            print!("-\t");
                        } else {
                            print!("{:.3}\t", matrix[i][j]);
                        }
                    }
                }
                println!();
            }
        }

    }
}
