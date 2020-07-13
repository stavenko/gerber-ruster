use super::super::{ PathType, Path };
use crate::parser::Polarity;
use super::{ split_region_paths, to_stroke_around_path };

pub struct Region {
  pub starting_polirity: Polarity,
  pub paths: Vec<Path>
}

impl Region {
  pub fn from_raw_region(path: Path) -> Vec<Self> {
    println!("read reg");
    match path.tp {
      PathType::Stroke => split_region_paths(path),
      _ => to_stroke_around_path(path)
    }
  }

  pub fn new(starting_polirity: Polarity, paths: Vec<Path>) -> Self {
    Region {
      starting_polirity, 
      paths
    }
  }
}

