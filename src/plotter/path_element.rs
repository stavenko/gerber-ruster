extern crate nalgebra as na;
// use std::fmt::Debug;
// use std::cmp::PartialEq;
use na::{Vector2};
pub type Vec2 = Vector2<f32>;

pub trait PathElement
{
  fn get_start_point(&self) -> Vec2;
  fn get_end_point(&self) -> Vec2;
  fn get_normal_in_start_point(&self) -> Vec2;
  fn get_normal_in_end_poing(&self) -> Vec2;
  fn get_direction_in_start_point(&self) -> Vec2;
  fn get_direction_in_end_point(&self) -> Vec2;
}


