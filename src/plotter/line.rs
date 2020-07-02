extern crate nalgebra as na;
use std::f32::consts::PI;
use super::path_element::*;
use super::intersector::{ Ray, IntersectorEnum, Intersects };
use na::{Rotation2};


#[derive(Debug, Clone, PartialEq)]
pub struct Line {
  pub to: Vec2,
  pub from: Vec2,
  pub(in super) normal: Vec2,
  pub(in super) direction: Vec2
}

impl Line {
  pub fn new(to: Vec2, from: Vec2) -> Self{
    let direction = {
      let d = (to - from).normalize();
      if d.x.is_nan() || d.y.is_nan() {
        Vec2::new(1.0, 0.0)
      } else {
        d
      }
    };
    let normal = Rotation2::new(PI / 2.0) * direction;
    Line {
      to, from, normal, direction
    }
  }
}


impl PathElement for Line {
  fn get_start_point(&self) ->Vec2 {
    self.from
  }
  fn get_end_point(&self) -> Vec2 {
    self.to
  }
  fn get_normal_in_start_point(&self) -> Vec2 {
    self.normal
  }
  fn get_normal_in_end_poing(&self) -> Vec2 {
    self.normal
  }
  fn get_direction_in_start_point(&self) -> Vec2 {
    self.direction
  }
  fn get_direction_in_end_point(&self) -> Vec2 {
    self.direction
  }
}

impl Intersects for Line {
  fn get_intersector(&self) -> IntersectorEnum {
    IntersectorEnum::Ray(Ray {
      origin: self.from,
      dir: self.to - self.from
    })
  }
}
