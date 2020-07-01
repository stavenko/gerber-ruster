use crate::plotter::{ Line, Arc, CircularDirection };
use super::bounding_box::BoundingBoxTrait;
use std::f32::consts::PI;

pub trait Serializable: BoundingBoxTrait {
  fn serialize(&self) -> String;
  fn initial(&self) -> String;
}

impl Serializable for Line {
  fn initial(&self) -> String {
    let Line {from, ..} = self;
    format!("M {} {}",  from.x, from.y)
  }
  fn serialize(&self) -> String {
    let Line {to, ..} = self;
    format!("L {} {}",  to.x, to.y)
  }
}

impl Serializable for Arc {
  fn initial(&self) -> String {
    let Arc {from, ..} = self;
    format!("M {} {}",  from.x, from.y)
  }
  fn serialize(&self) -> String {
    use CircularDirection::*;
    let Arc{to, direction, center, angle_length, ..} = self;
    let radius = (to - center).magnitude();
    let is_large = *angle_length > PI;

    format!("A {} {} {} {} {} {} {}", 
            radius, 
            radius, 
            0, 
            if is_large {1}else{0}, 
            match direction{ CCW => 1, _ => 0 }, 
            to.x, 
            to.y
            )
  }
}
