extern crate nalgebra as na;
use super::StrokePathElement;
use super::intersector::{ IntersectorEnum, Ray };
use std::cmp::Ordering;
use na::Vector2;
use std::f32::EPSILON;

type Vec2 = Vector2<f32>;

#[derive(Clone, Debug, PartialEq)]
pub enum PathType {
  Rect(f32, f32),
  Circle(f32),
  Stroke
}

#[derive(Debug)]
pub struct Path{ 
  pub(in super) tp: PathType,
  pub elements: Vec<Box<dyn StrokePathElement>>
}

impl Path {
  pub fn new(tp: PathType) -> Path {
    Path {
      tp,
      elements: Vec::new()
    }
  }

  pub fn is_empty(&self) -> bool {
    self.elements.is_empty()
  }

  pub fn is_locked(&self) -> bool {
    let first = self.elements.first().unwrap();
    let last = self.elements.last().unwrap();
    let dist = first.get_start_point() - last.get_end_point();
    dist.magnitude() <= EPSILON
  }

  pub fn stroke(els: Vec<Box<dyn StrokePathElement>>) -> Self {
    Path {
      tp: PathType::Stroke,
      elements: els
    }
  }

  pub fn add(&mut self, element: Box<dyn StrokePathElement>) {
    self.elements.push(element);
  }

  pub fn is_point_inside(&self, point: &Vec2) -> bool {
    /*
    for el in self.elements.iter() {
      println!("path {} {}", el.get_start_point().x, el.get_start_point().y);
    }
    */

    let ray = Ray::new(*point, point.normalize());
    // println!("testing ray {}, {}", ray.origin, ray.dir);
    let ray = IntersectorEnum::Ray(ray);
    let mut intersects: Vec<Vec2> = self.elements.iter()
      .enumerate()
      .map(|(ix, el)| { 
        let r_int = el.get_intersector();
         // println!("WTF!!! {:?}", r_int);
        let result = ray.intersects(el.get_intersector());
        if !result.is_empty() {
          // println!("element {} is around result {}", ix, result[0]);
        }
        result

      })

      .flatten()
      .collect();
     // println!("found intersects {:?}", intersects);
    intersects.sort_unstable_by(|a, b| {
        let c = a - b;
        let magn = c.magnitude();
        // println!("     magnitude {} {} ", magn, magn <= 1e-6 );
        if magn <= 1e-6  {
          Ordering::Equal
         } else {
          Ordering::Less
        }
      });
    intersects.dedup_by(|a, b| {
      let c = *a - *b;
      let l = c.magnitude();
      l <= 1e-6
    });


    let count = intersects.len();

    // println!("is_point_inside me {}>>>> {}; {}  (count: {})", self.elements.len(), point.x, point.y, count);

    count % 2 != 0
    
  }
}


#[cfg(test)]
mod tests {
  use super::*;
  use crate::plotter::{ Line, Vec2 };

  #[test]
  fn is_point_inside () {
    let elements: Vec<Box<dyn StrokePathElement>> = vec!(
      Box::new(Line::new(Vec2::new(4.0, 0.0), Vec2::new(0.0, 0.0))),
      Box::new(Line::new(Vec2::new(4.0, 4.0), Vec2::new(4.0, 0.0))),
      Box::new(Line::new(Vec2::new(0.0, 4.0), Vec2::new(4.0, 4.0))),
      Box::new(Line::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 4.0)))
    );

    let path = Path::stroke(elements);
    assert_eq!(path.is_point_inside(&Vec2::new(1.0, 1.0)), true);
  }
}
