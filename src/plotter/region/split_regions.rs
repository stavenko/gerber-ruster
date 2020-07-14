extern crate nalgebra as na;
use std::collections::HashMap;
use std::cmp::Ordering;
use na::*;
use std::f32::EPSILON;
use super::region_impl::Region;
use crate::parser::Polarity;
use super::super:: {
  Path,
  AlgebraicPathElement,
  CircularDirection,
  tree::{ Forest, Tree }
};
use super::super::{ 
  StrokePathElement,
   //Path,
  //Algebraic,
  //AlgebraicPathElement,
  //CircularDirection,
};
type Vec2 = Vector2<f32>;

fn is_end_or_start_of_segment(element: &Box<dyn StrokePathElement>, point: &Vec2) ->bool {
  let point = Point2::new(point.x, point.y);
  let d1 = na::distance(&element.get_end_point().into(), &point);
  let d2 = na::distance(&element.get_start_point().into(), &point);
  // // println!("ddd {}  {}", d1, d2);


  d1 <= f32::EPSILON || d2 <= f32::EPSILON
}

fn is_end_or_start_of_some_segment(element: &Box<dyn StrokePathElement>, element_other: &Box<dyn StrokePathElement>, point: &Vec2) -> bool {
  is_end_or_start_of_segment(element, point) || is_end_or_start_of_segment(element_other, point)
}


fn find_intersection_and_elements(path: &Path) -> Option<(Vec<usize>, Vec2)> {
  let mut result = None;

  'outer: for i in 0..path.elements.len() {
    if let Some(element) = path.elements.get(i) {
      for j in i+2..path.elements.len() {
        if let Some(possible_candidate) = path.elements.get(j) {
          println!(" inter {:?}", element.get_intersector().intersects(possible_candidate.get_intersector()));
          if let Some(intersection) = element.get_intersector().intersects(possible_candidate.get_intersector()).pop() {
            let end_start = (
              is_end_or_start_of_segment(&element, &intersection),
              is_end_or_start_of_segment(&possible_candidate, &intersection),
            );
            println!("end_start {:?}", end_start);
            let indexes = match end_start {
              (false, false) => vec!(i, j),
              (false, true) => vec!(i),
              (true, false) => vec!(j),
              (true, true) => Vec::new()
            };
            
            if !indexes.is_empty() {
              result = Some((indexes, intersection));
              break 'outer;
            }
          }
        }
      }
    }
  }
  result
}

enum MyResult {
  New(Path),
  Old(Path)
}

fn split_primitives_by_intersections(path: Path) -> MyResult {
  let intersection = find_intersection_and_elements(&path);
  println!("found intersection {:?}", intersection);

  if let Some((indexes, split_point)) = intersection {
    let mut elements: Vec<Box<dyn StrokePathElement>> = Vec::new();

    for (i, element) in path.elements.into_iter().enumerate() {
      if indexes.contains(&i) {
        let mut vec = element.split_by(&split_point);
        elements.append(&mut vec);
      } else {
        elements.push(element);
      }
    }

    MyResult::New(Path::stroke(elements))
  } else {
    MyResult::Old(path)
  }
}

fn split_all_primitives_by_intersections(path: Path) -> Path {
  use MyResult::*;
  let mut updated_path = path;
  loop {
    match split_primitives_by_intersections(updated_path) {
      New(new_path) => updated_path = new_path,
      Old(old_path) => {
        updated_path = old_path;
        break;
      }
    }
  }
  updated_path
}

fn is_fully_locked(path: &Path) -> bool {
  let mut map: HashMap<String, usize> = HashMap::new();
  let from = |el: &dyn StrokePathElement| {
    let v = el.get_start_point();
    format!("{:?}", (v.x, v.y))
  };
  let to = |el: &dyn StrokePathElement| {
    let v = el.get_start_point();
    format!("{:?}", (v.x, v.y))
  };
  for el in path.elements.iter() {
    for tup in vec![from(el.as_ref()), to(el.as_ref())] {
      match map.get_mut(&tup) {
        Some(mut_ref) => {*mut_ref+=1;},
        None => {map.insert(tup, 1);}
      }
    }
  }

  let mut result = true;
  for (k, v) in map {
    if v != 2 {
      result = false;
      // println!("some spot appeared not twice {}, {}",k, v);
      break;
    }
  }

  result
}

fn split_by_locked_countours(path: Path) -> Vec<Path>{
  let amount_of_elements = path.elements.len();
  match get_first_found_locked_contour(path) {
    (Some(_rest), None) => {
      println!("split unsuccessful, rest path is not locked contour, remove it");
      Vec::new()
    },
    (None, Some(cont)) if cont.elements.len() == amount_of_elements => {
      println!("full split amount:{}", cont.elements.len());
      vec!(cont)
    },
    (Some(rest), Some(countur)) => {
      // Perhaps, both have more locked things
      println!("continue split");
      let mut paths = split_by_locked_countours(rest);
      paths.extend(split_by_locked_countours(countur));
      paths
    },
    _ => unreachable!("WTF!!!")
  }
}

fn format_element(element: &Box<dyn StrokePathElement>) -> String {
  let coords_from =  element.get_start_point();
  let coords_to =  element.get_end_point();
  let coords_from = format!("{}; {}", coords_from.x, coords_from.y);
  let coords_to = format!("{}; {}", coords_to.x, coords_to.y);
  let t_el = match element.algebraic() {
    AlgebraicPathElement::Arc(a) => format!("A ({}, {}){} ", a.center.x, a.center.y, match a.direction {
      CircularDirection::CW => "CW",
      CircularDirection::CCW => "CCW"
    }),
    AlgebraicPathElement::Line(_) => "L".into()
  };
  format!("{} ({}, {})", t_el, coords_from, coords_to)
}

fn find_fist_point_with_two_plus_sources(path: &Path) -> Option<(usize, usize)> {
  let mut map: HashMap<String, usize> = HashMap::new();
  let get_stroke_end = |el: &dyn StrokePathElement| {
    let v = el.get_end_point();
    format!("{:?}", (v.x, v.y))
  };
  let mut found: Option<(usize, usize)>= None;
  for (ix, el) in path.elements.iter().enumerate() {
    let coord_label_hash = get_stroke_end(el.as_ref());
    println!("finding spots {} for el ix {}", coord_label_hash, ix);
    match map.get_mut(&coord_label_hash) {
      Some(mut_ref) => {

        found = Some((*mut_ref, ix));
        break;
      },
      None => {map.insert(coord_label_hash, ix);}
    }
  }
  found
}

fn find_element_with_start<'a>(path: &'a Path, point: &'a Vec2) -> Option<usize> {
  let mut found: Option<usize> = None;
  for (ix, element) in path.elements.iter().enumerate() {
    if (element.get_start_point() - point).magnitude() < EPSILON {
      found = Some(ix);
      break;
    }
  }
  found
}

fn cutout_from_element(mut consumed_path: Path, element_index: usize) -> (Option<Path>, Option<Path>) {

  // println!("cutout from element {}", element_index);

  let mut new_path = Path::stroke(Vec::new());
  let mut next_start_point = consumed_path.elements[element_index].get_end_point();
  // println!("first_start_poing {}", next_start_point);

  while let Some(element_index) = find_element_with_start(&consumed_path, &next_start_point) {
    let el = consumed_path.elements.remove(element_index);
    // println!("cutting lille-by-lil {} ({})", element_index, format_element(&el));
    new_path.elements.push(el);

    if new_path.is_locked() {
      break;
    } else {
      next_start_point = new_path.elements.last().unwrap().get_end_point();
    }
  }

  (
    if consumed_path.is_empty() { None } else {
      Some(consumed_path)
    }, 
    if new_path.is_empty() { None } else {
     Some(new_path)
    }
  )
}


fn get_first_found_locked_contour(path: Path) -> (Option<Path>, Option<Path>) {
  match find_fist_point_with_two_plus_sources(&path) {
    Some((first, _)) => cutout_from_element(path, first),
    None => {
      println!("WTF first {}", format_element(path.elements.first().unwrap()));
      println!("WTF last {}", format_element(path.elements.last().unwrap()));
      
      if path.is_locked() {
        println!("PATH IS LOCKED");
        (None, Some(path))
      } else {
        (Some(path), None)
      }
    }
  }
}

fn get_first_found_locked_contour_wrong(path: Path) -> (Path, Option<Path>) {
  let mut indexes: Option<(usize, usize)> = None;

  'out: for first_element_index in 0..path.elements.len() {
    for final_element_index in first_element_index..path.elements.len() {
      let final_element = &path.elements[final_element_index];
      let first_element = &path.elements[first_element_index];
      let distance = final_element.get_end_point() - first_element.get_start_point();
      // println!("{} =>>>> {} ({})", format_element(first_element), format_element(final_element), distance.magnitude());

      if distance.magnitude() < std::f32::EPSILON {
        indexes = Some((first_element_index, final_element_index));
        break 'out;
      }
    }
  }

  if let Some((from, to)) = indexes {
    let mut original_elements: Vec<Box<dyn StrokePathElement>>  = Vec::new();
    let mut selected_elements: Vec<Box<dyn StrokePathElement>>  = Vec::new();
    for (ix, element) in path.elements.into_iter().enumerate() {
      // let coords_from =  element.get_start_point();
      // let coords_to =  element.get_end_point();
      // let coords_from = format!("{}; {}", coords_from.x, coords_from.y);
      // let coords_to = format!("{}; {}", coords_to.x, coords_to.y);
      /*
      let t_el = match element.algebraic() {
        AlgebraicPathElement::Arc(a) => format!("A {}", match a.direction {
          CircularDirection::CW => "CW",
          CircularDirection::CCW => "CCW"
        }),
        AlgebraicPathElement::Line(_) => "L".into()
      };
      */
      // println!("{}: {} ", ix, format_element(&element));

      if ix >= from && ix <= to {
        selected_elements.push(element)
      } else {
        original_elements.push(element)
      }
    }

    // println!("found some {}, {}", original_elements.len(), selected_elements.len());
    (
      Path::stroke(original_elements),
      Some(Path::stroke(selected_elements))
    )
  } else {
    // println!("not found any");
    (path, None)
  }
}

fn remove_unlocked_and_zero_square_conturs(paths: Vec<Path>) -> Vec<Path> {
  paths.into_iter().filter(|path| {
    match path.elements.len() {
      1 => {
        let first = path.elements.first().unwrap();
        (first.get_start_point() - first.get_end_point()).magnitude() <= EPSILON
      },
      2 => {
        let first = path.elements.first().unwrap();
        let last = path.elements.last().unwrap();
        let distance = first.get_start_point() - last.get_end_point();
        if distance.magnitude() <= EPSILON {
          let mut items: Vec<AlgebraicPathElement> = path.elements.iter().map(|el| el.algebraic()).collect();
          let items = (items.pop().unwrap(), items.pop().unwrap());
          use AlgebraicPathElement::*;
          match items {
            (Line(_), Line(_)) =>  {
              // locked contur with both lines cannot be with non-zero square
              false
            },
            _ => {
              // any locked countour with arc (if arc with non-zero radius) has some useful square 
              true
            }
          }
        } else {
          false
        }
      },
      _ => true
    }
  }).collect()
}

fn is_element_has_point_within_path(element: &dyn StrokePathElement, path: &Path) -> bool {
  let is_start_on_path = path.elements.iter().filter(|el| {
    // println!("check if point {} {} in path", element.get_start_point().x, element.get_end_point().y);
    el.has_point(&element.get_start_point())
  }).count() != 0;
  let is_end_point_on_path = path.elements.iter().filter(|el| {
    el.has_point(&element.get_end_point())
  }).count() != 0;
  // println!("is start-end-on : {}   {},",is_start_on_path, is_end_point_on_path);

  if is_start_on_path && is_end_point_on_path {
    //print!("el start {}, {} ", element.get_start_point().x, element.get_start_point().y);
    //print!("el end {}, {} \n", element.get_end_point().x, element.get_end_point().y);
    // println!("el center {}, {}", element.get_central_point().x, element.get_central_point().y);
    path.is_point_inside(&element.get_central_point())
  } else if is_start_on_path {
    path.is_point_inside(&element.get_end_point())
  } else {

    let s = element.get_start_point();
    let ipi = path.is_point_inside(&element.get_start_point());
    // println!("is points inside: {} {}  {},",s.x, s.y, ipi);
    ipi
  }
  
}

pub fn compare_path(path1: &Path, path2: &Path) -> Ordering {
  //println!("-----------------------*******************-------------------------");
  for el in path1.elements.iter() {
    //println!("path1 > {}", format_element(&el))
  }
  for el in path2.elements.iter() {
    //println!("path2 > {}", format_element(&el))
  }
  //println!("compare_path1  with {} els with path2 with {} elements", 
           //path1.elements.len(), path2.elements.len());
  let is_path1_within_path2 = path1.elements.iter()
    .filter(|el| is_element_has_point_within_path(el.as_ref(), path2))
    .count() > 0;
  ////println!("======      is path1 within path2  {} ========", is_path1_within_path2);
  let is_path2_within_path1 = path2.elements.iter()
    .filter(|el| is_element_has_point_within_path(el.as_ref(), path1))
    .count() > 0;
  //println!("======      is path2 within path1  {} ========", is_path2_within_path1);
    


  if is_path1_within_path2 {
    //println!("path 1 within path 2");
    Ordering::Less
  } else if is_path2_within_path1 {
    //println!("path 2 within path 1");
    Ordering::Greater
  } else {
    //println!("Eq");
    Ordering::Equal
  }
}

fn attach_leafs(forest: &mut Forest<Path>, mut paths: Vec<Path>) -> Vec<Path> {
  for node in forest.iter_mut() {
    if node.is_leaf() {
      // println!("node is leaf");
      let (my, rest): (Vec<_>, Vec<_>) = paths.into_iter()
        .partition(|path| {
          // println!(" {:?}", compare_path(&node.data, &path));
          compare_path(&node.data, &path) == Ordering::Greater
        });
      if !my.is_empty() {
        let local_forest: Forest<Path> = my
          .into_iter()
          .map(|path| Box::new(Tree::new(path)))
          .collect();
        node.extend(local_forest);
      }
      paths = rest;
    } else {
      paths = attach_leafs(&mut node.forest_mut(), paths);
    }
  };
  paths
}


pub fn compose_regions(mut paths: Vec<Path>) -> Vec<Region> {
  println!("paths len: {}", paths.len());

  if paths.len() == 1 {
    vec!(Region::new(Polarity::Dark, Tree::new(paths.pop().unwrap())))
  } else {
    let (some_top_node_ix, some_top_node) = paths.iter().enumerate()
      .max_by(|(_,p), (_,y)| compare_path(&p, &y)).unwrap();
    let mut equal_nodes = paths.iter()
      .enumerate()
      .filter(|(_, n)| compare_path(&some_top_node, n) == Ordering::Equal)
      .map(|(ix, _)| ix)
      .collect::<Vec<_>>();
    equal_nodes.push(some_top_node_ix);
    let (top_nodes, other_nodes): (Vec<_>, Vec<_>) = paths.into_iter()
      .enumerate()
      .partition(|(ix, _path)| equal_nodes.contains(ix));
    let mut forest: Forest<Path> = Vec::new();
    for top_node in top_nodes.into_iter().map(|(_, el)| el) {
      forest.push(Box::new(Tree::new(top_node)))
    }

    let mut other_nodes: Vec<Path> = other_nodes.into_iter().map(|(_, el)| el).collect();
    let mut ___handle = other_nodes.len();


    while !other_nodes.is_empty() {
      other_nodes = attach_leafs(&mut forest, other_nodes);
      if other_nodes.len() == ___handle {
        panic!("adfasdf");
      }
    }

    forest.into_iter().map(|tr| {
      Region::new(Polarity::Dark, *tr)
    }).collect::<Vec<Region>>()
  }

}

pub fn split_region_paths(path: Path) -> Vec<Region> {
  let splitted_path = split_all_primitives_by_intersections(path);
  let contours = split_by_locked_countours(splitted_path);
  for c in &contours {
    println!("\ncontour {:?}\n", c);
  }
  let contours = remove_unlocked_and_zero_square_conturs(contours);

  let regs = compose_regions(contours);
  //println!("Regs are good");

  regs

}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::plotter::{ Arc, Line };
  use crate::plotter::{ StrokePathElement, CircularDirection };
  use std::f32::EPSILON;

  #[test]
  fn find_intersection_and_elements_test() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(5.0, 0.0), Vec2::new(-5.0, 10.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(5.0, 0.0))),
      Box::new(Line::new(Vec2::new(5.0, 5.0), Vec2::new(0.0, 0.0))),
      );
    let path = Path::stroke(elements);


    if let Some((u, ve)) = find_intersection_and_elements(&path) {
      println!("some staff found {:?}, {:?}", u, ve);
      assert_eq!(u[0], 0);
      assert_eq!(u[1], 2);
      assert_eq!((ve - Vec2::new(2.5, 2.5)).magnitude() < EPSILON, true);
    } else {
      panic!("Cannot find intersection");
    }
  }

  #[test]
  fn split_all_primitives_by_intersections() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(5.0, 0.0), Vec2::new(-5.0, 10.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(5.0, 0.0))),
      Box::new(Line::new(Vec2::new(5.0, 5.0), Vec2::new(0.0, 0.0))),
      );
    let path = Path::stroke(elements);
    let new_path = super::split_all_primitives_by_intersections(path);
    assert_eq!(new_path.elements.len(), 5);
  }

  #[test]
  fn get_first_found_locked_contour() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(5.0, 0.0), Vec2::new(-5.0, 10.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(5.0, 0.0))),
      Box::new(Line::new(Vec2::new(5.0, 5.0), Vec2::new(0.0, 0.0))),
      );
    let path = Path::stroke(elements);
    let path = super::split_all_primitives_by_intersections(path);
    let (rest, new_path) = super::get_first_found_locked_contour(path);
    if let Some(path) = new_path {
      assert_eq!(path.elements.len(), 3);
      assert_eq!(rest.unwrap().elements.len(), 2);
    } else {
      panic!("Path is not parsed {:?}", rest);
    }
  }

  #[test]
  fn get_first_found_locked_contour_full_arc() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(0.0, -5.0), Vec2::new(0.0, -6.0))),
      Box::new(Arc::new_with_fixed_center(
          Vec2::new(0.0, -5.0),
          Vec2::new(0.0, -5.0),
          Vec2::new(0.0, 0.0),
          CircularDirection::CW
          ))
    );
    let path = Path::stroke(elements);
    let path = super::split_all_primitives_by_intersections(path);
    let (rest, new_path) = super::get_first_found_locked_contour(path);
    if let Some(path) = new_path {
      assert_eq!(path.elements.len(), 1);
      assert_eq!(rest.unwrap().elements.len(), 1);
    } else {
      panic!("Path is not splitted {:?}", rest);
    }

  }
  #[test]
  fn split_by_locked_countours_full_arc() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(0.0, -5.0), Vec2::new(0.0, -6.0))),
      Box::new(Arc::new_with_fixed_center(
          Vec2::new(0.0, -5.0),
          Vec2::new(0.0, -5.0),
          Vec2::new(0.0, 0.0),
          CircularDirection::CW
          ))
    );
    let path = Path::stroke(elements);
    let paths = super::split_by_locked_countours(path);
    assert_eq!(paths.len(), 1);
  }

  #[test]
  fn split_by_locked_countours_triangle() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(5.0, 0.0), Vec2::new(-5.0, 10.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(5.0, 0.0))),
      Box::new(Line::new(Vec2::new(5.0, 5.0), Vec2::new(0.0, 0.0))),
    );
    let path = Path::stroke(elements);
    let path = super::split_all_primitives_by_intersections(path);
    let paths = super::split_by_locked_countours(path);
    assert_eq!(paths.len(), 1);
  }

  #[test]
  fn test_path_clearing_tri() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(5.0, 0.0), Vec2::new(-5.0, 10.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(5.0, 0.0))),
      Box::new(Line::new(Vec2::new(5.0, 5.0), Vec2::new(0.0, 0.0))),
    );

    let path = Path::stroke(elements);
    let mut regions = super::split_region_paths(path);

    assert!(regions.len() == 1);
    let Region{ paths, ..} = regions.remove(0);
    let mut iter = paths.into_iter();
    assert!(iter.next().unwrap().data.elements.len() == 3);
  }

  #[test]
  fn test_path_clearing_tri_tri() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(4.0, 1.0), Vec2::new(0.0, 10.0))),
      Box::new(Line::new(Vec2::new(1.0, 4.0), Vec2::new(4.0, 1.0))),
      Box::new(Line::new(Vec2::new(10.0, 0.0), Vec2::new(1.0, 4.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0))),
      Box::new(Line::new(Vec2::new(0.0, 10.0), Vec2::new(0.0, 0.0))),
    );

    let path = Path::stroke(elements);
    let mut regions = super::split_region_paths(path);

    assert_eq!(regions.len(), 1);
    let Region{ paths, ..} = regions.remove(0);
    // let mut tw = trees::TreeWalk::from(paths);
    //
    let mut iter = paths.into_iter();
    assert!(iter.next().unwrap().data.elements.len() == 4);
    assert!(iter.next().unwrap().data.elements.len() == 3);
  }

  #[test]
  fn test_path_clearing_tri_circle() {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(10.0, 0.0), Vec2::new(0.0, 0.0))),
      Box::new(Line::new(Vec2::new(10.0, 10.0), Vec2::new(10.0, 0.0))),
      Box::new(Line::new(Vec2::new(0.0, 10.0), Vec2::new(10.0, 10.0))),
      Box::new(Line::new(Vec2::new(0.0, 5.0), Vec2::new(0.0, 10.0))),
      Box::new(Line::new(Vec2::new(2.0, 5.0), Vec2::new(0.0, 5.0))),
      Box::new(Arc::new_with_fixed_center(
          Vec2::new(2.0, 5.0),
          Vec2::new(2.0, 5.0),
          Vec2::new(5.0, 5.0),
          CircularDirection::CW
          )),
      Box::new(Line::new(Vec2::new(0.0, 5.0), Vec2::new(2.0, 5.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 5.0))),
    );
    let path = Path::stroke(elements);
    let mut regions = super::split_region_paths(path);

    assert_eq!(regions.len(), 1);
    let Region{ paths, ..} = regions.remove(0);
    let mut iter = paths.into_iter();
    assert!(iter.next().unwrap().data.elements.len() == 5);
    assert!(iter.next().unwrap().data.elements.len() == 1);
  }

  #[test]
  fn for_loop() {
    let mut a = 0;
    'outer: for i in 1..5 {
      for j in 1..7 {
        if i >= 3 && j >= 4 {
          a = i*j;
          break 'outer;
        }
      }

    }

    assert_eq!(a, 12);
  }
}
