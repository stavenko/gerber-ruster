extern crate nalgebra as na;
use std::f32::consts::PI;
use super::path_element::*;
use std::cmp::Ordering;
use super::circular_direction::*;

use super::intersector::{ Arc as SimpleArc, IntersectorEnum, Intersects };
use na::{ Vector2, Rotation2 };

type Vec2 = Vector2<f32>;
#[derive(Debug, PartialEq, Clone)]
pub struct Arc {
  pub to: Vec2,
  pub from: Vec2,
  pub is_initially_single: bool,
  pub direction: CircularDirection,
  pub center: Vec2,
  pub angle_start: f32,
  pub angle_end: f32,
  pub angle_length: f32,
  pub normal_in_start_point: Vec2,
  pub direction_in_start_point: Vec2,
  pub normal_in_end_point: Vec2,
  pub direction_in_end_point: Vec2,
}

fn cmp(a: &f32, b: &f32) -> Ordering {
  if a > b {
    Ordering::Greater
  }else {
    Ordering::Less
  }
}

impl Arc {
  fn arc_len(dir: &CircularDirection, center: &Vec2, from: &Vec2, to: &Vec2) -> f32 {
    use CircularDirection::*;
    let basis_x = to - center;
    let basis_y = Rotation2::new(PI) * basis_x;
    let x = from.dot(&basis_x);
    let y = from.dot(&basis_y);
    let angle = y.atan2(x) * match dir {CW => -1.0, _ => 1.0};

    if angle < 0.0 {
      PI * 2.0 + angle
    }else {
      angle
    }
  }

  fn kross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - b.x * a.y
  }

  pub fn is_between(&self, v: Vec2) ->bool {
    Arc::kross(v - self.from, self.to - self.from) <= 0.0
  }


  pub fn new_with_fixed_center(to: Vec2, from: Vec2, center: Vec2, direction: CircularDirection)->Self {
    use CircularDirection::*;
    let normal_in_start_point = (from - center).normalize();
    let normal_in_end_point = (to - center).normalize();
    let direction_in_start_point = match &direction {
      CW => Rotation2::new(-PI/2.0) * normal_in_start_point,
      CCW => Rotation2::new(PI/2.0) * normal_in_start_point
    };
    let direction_in_end_point = match &direction {
      CW => Rotation2::new(-PI/2.0) * normal_in_end_point,
      CCW => Rotation2::new(PI/2.0) * normal_in_end_point
    };

    let angle_start = normal_in_start_point.y.atan2(normal_in_start_point.x);
    let angle_end = normal_in_end_point.y.atan2(normal_in_end_point.x);

    let angle_length = Rotation2::rotation_between(
      &normal_in_start_point,
      &normal_in_end_point
    ).angle();

    let angle_length = match direction {
      CCW => {
        if angle_length < 0.0 {
          2.0 * PI + angle_length
        } else {
          angle_length
        }
      },
      CW => {
        if angle_length < 0.0 {
          angle_length.abs()
        } else {
          2.0 * PI - angle_length
        }
      }
    };

    Arc {
      to, 
      from,
      center,
      is_initially_single: false,
      direction,
      direction_in_start_point,
      direction_in_end_point,
      normal_in_start_point,
      normal_in_end_point,
      angle_length,
      angle_start,
      angle_end
    }
  }

  pub fn new(
    to: Vec2, 
    from: Vec2, 
    i: Option<f32>,
    j: Option<f32>,
    is_initially_single: bool,
    direction: CircularDirection
  ) -> Self{
    let center = if is_initially_single {
      let cx = i.unwrap_or(0.0);
      let cy = j.unwrap_or(0.0);
      vec!(
        from + Vec2::new(cx, cy),
        from + Vec2::new(cx, -cy),
        from + Vec2::new(-cx, cy),
        from + Vec2::new(-cx, -cy)
        ).into_iter().min_by(|c1, c2| {

        let arc_len1 = Arc::arc_len(&direction, &c1, &from, &to);
        let arc_len2 = Arc::arc_len(&direction, &c2, &from, &to);
        cmp(&arc_len1, &arc_len2)
      }).unwrap()
    } else {
      let cx = i.map(|i| i + from.x).unwrap_or(from.x);
      let cy = j.map(|i| i + from.y).unwrap_or(from.y);
      Vec2::new(cx, cy)
    };

    Arc::new_with_fixed_center(to, from, center, direction )

  }
}

impl PathElement for Arc {
  fn get_start_point(&self) ->Vec2 {
    self.from
  }
  fn get_end_point(&self) -> Vec2 {
    self.to
  }
  fn get_normal_in_start_point(&self) -> Vec2 {
    match self.direction {
      CircularDirection::CW => self.normal_in_start_point,
      CircularDirection::CCW => -self.normal_in_start_point
    }
  }
  fn get_normal_in_end_poing(&self) -> Vec2 {
    match self.direction {
      CircularDirection::CW => self.normal_in_end_point,
      CircularDirection::CCW => -self.normal_in_end_point
    }
  }
  fn get_direction_in_start_point(&self) -> Vec2 {
    self.direction_in_start_point
  }
  fn get_direction_in_end_point(&self) -> Vec2 {
    self.direction_in_end_point
  }
}


impl Intersects for Arc {
  fn get_intersector(&self) -> IntersectorEnum {
    IntersectorEnum::Arc(SimpleArc{
      from: self.from.clone(),
      to: self.to.clone(),
      direction: self.direction.clone(),
      center: self.center.clone()
    })
  }
}