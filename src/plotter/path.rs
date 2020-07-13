extern crate nalgebra as na;
use super::StrokePathElement;
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
    false
  }
}

