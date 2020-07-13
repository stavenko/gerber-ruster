use std::cmp::Ordering;

use super::{
  Region,
  compare_path,
  super::{ 
    Path,
    StrokePathElement,
  }
};

use crate::parser::Polarity;





pub fn to_stroke_around_path(path: Path) -> Vec<Region> {
  println!("-----------------process path ---------------------");
  let mut forward: Vec<Box<dyn StrokePathElement>> = Vec::new();
  let mut backward: Vec<Box<dyn StrokePathElement>> = Vec::new();
  let is_locked = path.is_locked();
  let Path{tp, elements} = path;

  for ix in 0..elements.len() {
    let prev = if ix == 0 { None } else {elements.get(ix - 1).map(|i| i.as_ref())};
    let current = &elements[ix];
    println!("Elem -> {:?}", current);
    let next = elements.get(ix + 1).map(|i| i.as_ref());
    forward.extend(current.forward(&tp, prev, next, is_locked));
    backward.extend(current.backward(&tp, prev, next, is_locked));
  }
  if is_locked {
    let forward = Path::stroke(forward);
    let backward = Path::stroke(backward);
    match compare_path(&forward, &backward) {
      Ordering::Greater => vec!(Region::new(Polarity::Dark, vec!(forward, backward))),
      Ordering::Less => vec!(Region::new(Polarity::Dark, vec!(backward, forward))),
      Ordering::Equal => vec!(
        Region::new(Polarity::Dark, vec!(forward)),
        Region::new(Polarity::Dark, vec!(backward))
        ),
    }
  } else {
    backward.reverse();
    forward.extend(backward);
    println!("-------------- end process path ---------------------");

    let path = Path::stroke(elements);
    vec!(Region::new(Polarity::Dark, vec!(path)))
  }
}
