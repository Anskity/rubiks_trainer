use rand::{prelude::IndexedRandom, rng};
use std::{fs::{self, ReadDir}, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Movement {
    R,
    U,
    F,
    L,
    B,
    X,
    Y,

    RP,
    UP,
    FP,
    LP,
    BP,
    XP,
    YP,

    R2,
    U2,
    F2,
    L2,
    B2,
    X2,
    Y2,
}
impl Movement {
    pub fn inv(&self) -> Movement {
        match self {
            Movement::R => Movement::RP,
            Movement::U => Movement::UP,
            Movement::F => Movement::FP,
            Movement::L => Movement::LP,
            Movement::B => Movement::BP,
            Movement::X => Movement::XP,
            Movement::Y => Movement::YP,

            Movement::RP => Movement::R,
            Movement::UP => Movement::U,
            Movement::FP => Movement::F,
            Movement::LP => Movement::L,
            Movement::BP => Movement::B,
            Movement::XP => Movement::X,
            Movement::YP => Movement::Y,

            Movement::R2 => Movement::R2,
            Movement::U2 => Movement::U2,
            Movement::F2 => Movement::F2,
            Movement::L2 => Movement::L2,
            Movement::B2 => Movement::B2,
            Movement::X2 => Movement::X2,
            Movement::Y2 => Movement::Y2,
        }
    }

    pub fn from_text(text: &str) -> Option<Movement> {
        match text {
            "R" => Some(Movement::R),
            "U" => Some(Movement::U),
            "F" => Some(Movement::F),
            "L" => Some(Movement::L),
            "B" => Some(Movement::B),
            "x" => Some(Movement::X),
            "y" => Some(Movement::Y),

            "R'" => Some(Movement::RP),
            "U'" => Some(Movement::UP),
            "F'" => Some(Movement::FP),
            "L'" => Some(Movement::LP),
            "B'" => Some(Movement::BP),
            "x'" => Some(Movement::XP),
            "y'" => Some(Movement::YP),

            "R2" => Some(Movement::R2),
            "U2" => Some(Movement::U2),
            "F2" => Some(Movement::F2),
            "L2" => Some(Movement::L2),
            "B2" => Some(Movement::B2),
            "x2" => Some(Movement::X2),
            "y2" => Some(Movement::Y2),

            "R2'" => Some(Movement::R2),
            "U2'" => Some(Movement::U2),
            "F2'" => Some(Movement::F2),
            "L2'" => Some(Movement::L2),
            "B2'" => Some(Movement::B2),
            "x2'" => Some(Movement::X2),
            "y2'" => Some(Movement::Y2),
            _ => None,
        }
    }

    pub fn as_text(&self) -> &'static str {
        match self {
            Movement::R => "R",
            Movement::U => "U",
            Movement::F => "F",
            Movement::L => "L",
            Movement::B => "B",
            Movement::X => "x",
            Movement::Y => "y",

            Movement::RP => "R'",
            Movement::UP => "U'",
            Movement::FP => "F'",
            Movement::LP => "L'",
            Movement::BP => "B'",
            Movement::XP => "x'",
            Movement::YP => "y'",

            Movement::R2 => "R2",
            Movement::U2 => "U2",
            Movement::F2 => "F2",
            Movement::L2 => "L2",
            Movement::B2 => "B2",
            Movement::X2 => "x2",
            Movement::Y2 => "y2",
        }
    }
}

#[derive(Debug)]
enum RubiksError {
    IOError(std::io::Error),
    InvalidMovement(String),
}

#[derive(Debug, Clone)]
pub struct AlgSet {
    pub name: String,
    pub algs: Vec<Vec<Movement>>,
    pub enabled: bool,
}

impl AlgSet {
    pub fn parse_scramble(text: &str) -> Result<Vec<Movement>, RubiksError> {
        let mut scramble: Vec<Movement> = Vec::new();

        let mut text = text.to_string();

        // TODO: Add proper parenthesis support
        text.retain(|c| c != '(' && c != ')');

        for tk in text.split(' ').filter(|tk| tk.len() > 0) {
            match Movement::from_text(tk) {
                Some(movement) => scramble.push(movement),
                None => {
                    return Err(RubiksError::InvalidMovement(tk.to_string()));
                }
            }
        }

        Ok(scramble)
    }

    pub fn load_from<P: Into<PathBuf>>(path: P) -> Result<AlgSet, RubiksError> {
        let path = path.into();
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| RubiksError::IOError(e))?;
        let mut scrambles: Vec<Vec<Movement>> = Vec::new();

        for line in text.lines() {
            let line = line.split('#').nth(0).unwrap();
            let line: String = line.chars().map(|c| match c {'’' => '\'', c => c}).collect();
            let mut is_whitespace = true;
            for chr in line.chars() {
                if chr != ' ' {
                    is_whitespace = false;
                }
            }
            if is_whitespace {
                continue;
            }
            let scramble = AlgSet::parse_scramble(&line)?;
            scrambles.push(scramble);
        }

        Ok(AlgSet {
            name: name,
            algs: scrambles,
            enabled: true,
        })
    }
}

fn handle_rubiks_error(err: RubiksError) -> ! {
    match err {
        RubiksError::IOError(err) => {
            eprintln!("IO Error: {:?}", err);
            std::process::exit(1);
        }
        RubiksError::InvalidMovement(movement) => {
            eprintln!("Invalid movement: {}", movement);
            std::process::exit(1);
        }
    }
}

#[derive(Debug)]
pub struct AlgDB {
    pub entries: Vec<AlgEntry>,
}

impl AlgDB {
    fn parse_entry(path: PathBuf) -> AlgEntry {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        if path.is_dir() {
            let paths = fs::read_dir(path).unwrap();
            let mut entries: Vec<AlgEntry> = Vec::new();
            for path in paths {
                let path = path.unwrap().path();
                let entry = AlgDB::parse_entry(path);
                entries.push(entry);
            }
            AlgEntry::Group(name, entries)
        } else {
            let alg_set = AlgSet::load_from(path).unwrap();

            AlgEntry::Algs(name, alg_set)
        }
    }
    
    pub fn load(path: PathBuf) -> AlgDB {
        let mut entries = Vec::new();
        let paths: ReadDir = fs::read_dir(path).unwrap();
        for path in paths {
            let path: PathBuf = path.unwrap().path();
            let alg_entry = AlgDB::parse_entry(path);
            entries.push(alg_entry);
        }
        AlgDB { entries }
    }

    fn add_entries<'a>(vec: &mut Vec<&'a [Movement]>, entries: &'a [AlgEntry]) {
        for entry in entries {
            match entry {
                AlgEntry::Algs(_, alg_set) => {
                    for algs in alg_set.algs.iter() {
                        vec.push(&algs);
                    }
                }
                AlgEntry::Group(_, entries) => {
                    AlgDB::add_entries(vec, entries);
                }
            }
        }
    }
    pub fn get_rand<'a>(&'a self) -> &'a [Movement] {
        let mut possibilities: Vec<&[Movement]> = Vec::new();
        AlgDB::add_entries(&mut possibilities, &self.entries);
        
        possibilities.choose(&mut rng()).unwrap()
    }
}

#[derive(Debug)]
pub enum AlgEntry {
    Group(String, Vec<AlgEntry>),
    Algs(String, AlgSet),
}

