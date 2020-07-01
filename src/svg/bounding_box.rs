extern crate nalgebra as na;
use na::{ Vector2 };
use crate::plotter::{ Line, Arc };
use crate::plotter::{PathElement};
use std::f32::consts::PI;

type Vec2 = Vector2<f32>;

pub trait BoundingBoxTrait {
  fn get_bounding_box(&self) -> bounding_box_struct::BoundingBox;
}

impl BoundingBoxTrait for Line {
  fn get_bounding_box(&self) -> bounding_box_struct::BoundingBox {
      let bb = BoundingBox::default();
      let bb = bb.add(self.get_start_point());
      let bb = bb.add(self.get_end_point());
      bb
  }
}
impl BoundingBoxTrait for Arc {
  fn get_bounding_box(&self) -> bounding_box_struct::BoundingBox {
    let mut bb = BoundingBox::default();
    for a in &[0.0, 0.25, 0.5, 0.75] {
      let a = 2.0 * PI * a ;
      let radius = (self.to - self.center).magnitude();
      let (y, x) = a.sin_cos();
      let center = self.center;
      let point = radius * Vec2::new(x,y) + center;
      if self.is_between(point) {
        bb = bb.add(point);
      }
    }
    bb
  }
}

pub use bounding_box_struct::BoundingBox;

mod bounding_box_struct {
  extern crate nalgebra as na;
  use na::Vector2;
  use std::ops::{ Add, AddAssign };

  pub type Vec2 = Vector2<f32>;


  #[derive(Debug, PartialEq)]
  pub struct BoundingBox {
    pub min: Vector2<f32>,
    pub max: Vector2<f32>,
  }


  impl BoundingBox {
    pub fn new(min: Vec2, max: Vec2) -> Self {
      BoundingBox {min, max}
    }
    pub fn from(v: Vec2) -> Self {
      BoundingBox {min: v, max: v}
    }

    pub fn add(mut self, p: Vec2) -> Self {
      self.min = min(self.min, p);
      self.max = max(self.max, p);

      self
    }
  }

  impl Default for BoundingBox {
    fn default() -> Self {
      BoundingBox {
        min: Vec2::new(f32::INFINITY,f32::INFINITY),
        max: Vec2::new(-f32::INFINITY,-f32::INFINITY)
      }
    }
  }
  pub fn min(v1: Vec2, v2: Vec2) -> Vec2 {
    Vec2::new( v1[0].min(v2[0]), v1[1].min(v2[1])) 
  }

  pub fn max(v1: Vec2, v2: Vec2) -> Vec2 {
    Vec2::new( v1[0].max(v2[0]), v1[1].max(v2[1])) 
  }

  impl Add for BoundingBox {
    type Output = Self;
    fn add(self, other: Self) -> Self {
      BoundingBox {
        min: min(self.min, other.min),
        max: max(self.max, other.max)
      }
    }
  }

  impl AddAssign for BoundingBox {
    fn add_assign(&mut self, other: Self) {
      self.min = min(other.min, self.min);
      self.max = max(other.max, self.max);
    }
  }

  #[test] 
  fn test_min() {
    let a = Vec2::new(0f32, 1f32);
    let b = Vec2::new(1f32, 0f32);

    assert_eq!(min(a, b), Vec2::new(0f32,0f32));
  }

  #[test] 
  fn test_min1() {
    let a = Vec2::new(f32::INFINITY, f32::INFINITY);
    let b = Vec2::new(-4.0, -3.5);

    assert_eq!(min(a, b), Vec2::new(-4.0, -3.5));
  }
}

