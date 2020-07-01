extern crate nalgebra as na;
use std::cmp::Ordering;
use na::{ Vector2 };

type Vec2 = Vector2<f32>;

fn cmp(a: &f32, b: &f32) -> Ordering {
  if a > b {
    Ordering::Greater
  }else {
    Ordering::Less
  }
}

#[derive(Clone, Debug)]
pub enum RectDir {
  LeftBottom,
  RightBottom,
  LeftTop,
  RightTop,
  Left,
  Right,
  Top,
  Bottom
}


impl RectDir {
  pub fn vec(&self, wh: &Vec2) -> Vec2 {
    use RectDir::*;
    let hw = wh.x / 2.0;
    let hh = wh.y / 2.0;
    match self {
      LeftBottom => Vec2::new(-hw, -hh),
      RightBottom => Vec2::new(hw, -hh),
      LeftTop => Vec2::new(-hw, hh),
      RightTop => Vec2::new(hw, hh),
      Left => Vec2::new(-1.0, 0.0),
      Right => Vec2::new(1.0, 0.0),
      Top => Vec2::new(0.0, 1.0),
      Bottom => Vec2::new(0.0, -1.0)
    }
  }

  pub fn from_dir(dir: Vec2, normal: Vec2) -> Self {
    use RectDir::*;
    let wh = Vec2::new(1.0, 1.0);
    let top = Top.vec(&wh);
    let right = Right.vec(&wh);
    if normal.dot(&top).abs() < f32::EPSILON {
      if dir.dot(&top) > 0.0 {
        Top
      } else {
        Bottom
      }
    } else if normal.dot(&right).abs() < f32::EPSILON {
      if dir.dot(&right) > 0.0 {
        Right
      } else {
        Left
      }
    } else {
      let (dir,_) = {
        vec![LeftTop, LeftBottom, RightTop, RightBottom].into_iter()
        .map(|i| (i.clone(), i.vec(&wh).dot(&dir)))
        .max_by(|(_, i), (_, u)| cmp(i,u)).unwrap()
      };
      dir
    }
  }

  pub fn start(&self, wh: &Vec2) -> Vec2 {
    use RectDir::*;
    match self {
      Left => RightBottom.vec(wh),
      Right => LeftTop.vec(wh),
      Top => LeftBottom.vec(wh),
      Bottom => RightTop.vec(wh),

      LeftBottom => RightBottom.vec(wh),
      RightBottom => RightTop.vec(wh),
      RightTop => LeftTop.vec(wh),
      LeftTop => LeftBottom.vec(wh),
    }
  }

  pub fn is_ortho(&self) -> bool {
    use RectDir::*;
    match self {
      Left => true,
      Right => true,
      Top => true,
      Bottom => true,
      _ => false
    }
  }

  pub fn opposite(&self, wh: &Vec2) -> Vec2 {
    use RectDir::*;
    match self {
      Left => Right.vec(wh),
      Right => Left.vec(wh),
      Top => Bottom.vec(wh),
      Bottom => Top.vec(wh),

      LeftBottom => RightTop.vec(wh),
      RightBottom => LeftTop.vec(wh),
      RightTop => LeftBottom.vec(wh),
      LeftTop => RightBottom.vec(wh),
    }
  }

  pub fn end(&self, wh: &Vec2) -> Vec2 {
    use RectDir::*;
    match self {
      Left => LeftBottom.vec(wh),
      Right => RightTop.vec(wh),
      Top => LeftTop.vec(wh),
      Bottom => RightBottom.vec(wh),

      LeftBottom => RightBottom.vec(wh),
      RightBottom => RightTop.vec(wh),
      RightTop => LeftTop.vec(wh),
      LeftTop => LeftBottom.vec(wh),
    }
  }

}
