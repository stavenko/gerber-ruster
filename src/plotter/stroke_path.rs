extern crate nalgebra as na;
use na::{ Rotation2, Vector2 };
use std::fmt::Debug;

use super::{ 
  Arc,
  Line,
  CircularDirection,
  Algebraic, 
  RectDir, 
  AlgebraicPathElement, 
  Intersects, 
  PathType, 
  PathElement, 
  Path
};

type Vec2 = Vector2<f32>;

mod line_impl {
  use super::StrokePathElement;
  use super::super::{  PathType, PathElement };
  use super::{ CircularDirection, Arc, Line, Rotation2, Vec2, RectDir };


  impl StrokePathElement for Line {
    fn create_forward_with(&self, forward_start_point: Vec2, forward_end_point: Vec2) -> Box<dyn StrokePathElement> {
      let copy = self.clone();
      Box::new(Line {
        from: forward_start_point,
        to: forward_end_point,
        ..copy
      })
    }
    fn create_backward_with(&self, forward_start_point: Vec2, forward_end_point: Vec2) -> Box<dyn StrokePathElement> {
      Box::new(Line {
        from: forward_start_point,
        to: forward_end_point,
        direction: - self.direction,
        normal: - self.normal
      })
    }

  }
}

mod arc_impl {
  use super::StrokePathElement;
  use super::super::{ Arc, PathType };
  use super::Vec2;

  impl StrokePathElement for Arc {

    fn create_forward_with(&self, forward_start_point: Vec2, forward_end_point: Vec2) -> Box<dyn StrokePathElement> {

      Box::new(
        Arc::new_with_fixed_center(
          forward_end_point, 
          forward_start_point, 
          self.center, 
          self.direction.clone()
          ))
    }
    fn create_backward_with(&self, forward_start_point: Vec2, forward_end_point: Vec2) -> Box<dyn StrokePathElement> {
      Box::new(
        Arc::new_with_fixed_center(
          forward_end_point, 
          forward_start_point, 
          self.center, 
          self.direction.clone().reverse()
          ))
    }

  }

}

pub trait StrokePathElement: Algebraic<AlgebraicPathElement> + PathElement + Intersects + Debug  {
  fn get_forward_start_point(&self, path_type: &PathType) -> Vec2 {
    self.get_start_point() + match path_type {
      PathType::Circle(radius) => {
         *radius * self.get_normal_in_start_point()
      },
      PathType::Rect(w,h) => {
        let wh = Vec2::new(*w, *h);
        let rect_dir = RectDir::from_dir(self.get_direction_in_start_point(), self.get_normal_in_start_point());
        println!("forward start {:?}", rect_dir);

        rect_dir.start(&wh)
      },
      _ => unreachable!()
    }
  }

  fn get_forward_end_point(&self, path_type: &PathType) -> Vec2 {
    self.get_end_point() + match path_type {
      PathType::Circle(radius) => {
        *radius * self.get_normal_in_end_poing()
      },
      PathType::Rect(w,h) => {
        let wh = Vec2::new(*w, *h);
        let rect_dir = RectDir::from_dir(self.get_direction_in_end_point(), self.get_normal_in_end_poing());
        println!("forward end {:?}", rect_dir);
        rect_dir.end(&wh)
      },
      _ => unreachable!()
    }
  }

  fn get_backward_start_point(&self, path_type: &PathType) -> Vec2 {
    self.get_end_point() + match path_type {
      PathType::Circle(radius) => {
        *radius * -self.get_normal_in_end_poing()
      },
      PathType::Rect(w,h) => {
        let wh = Vec2::new(*w, *h);
        let rect_dir = RectDir::from_dir(-self.get_direction_in_end_point(), -self.get_normal_in_end_poing());
        rect_dir.start(&wh)
      },
      _ => unreachable!()
    }
  }

  fn get_backward_end_point(&self, path_type: &PathType) -> Vec2 {
    self.get_start_point() + match path_type {
      PathType::Circle(radius) => {
        *radius * -self.get_normal_in_start_point()
      },

      PathType::Rect(w,h) => {
        let wh = Vec2::new(*w, *h);
        let rect_dir = RectDir::from_dir(-self.get_direction_in_start_point(), -self.get_normal_in_start_point());
        rect_dir.end(&wh)
      },
      _ => unreachable!()
    }
  }


  fn create_ending_cap(&self, path_type: &PathType) -> Vec<Box<dyn StrokePathElement>> {
    match path_type {
      PathType::Circle(_) => {
        let mut result: Vec<Box<dyn StrokePathElement>> = Vec::new();
        let from = self.get_forward_end_point(path_type);
        let to = self.get_backward_start_point(path_type);
        let center = self.get_end_point();
        result.push(Box::new(Arc::new_with_fixed_center(
            to,
            from,
            center,
            CircularDirection::CW
            )));
        result
      },
      PathType::Rect(w, h) => {
        let wh = Vec2::new(*w, *h);
        let mut result: Vec<Box<dyn StrokePathElement>> = Vec::new();
        let back_start_rect_dir = RectDir::from_dir(-self.get_direction_in_end_point(), -self.get_normal_in_end_poing());
        let forw_end_rect_dir = RectDir::from_dir(self.get_direction_in_end_point(), self.get_normal_in_end_poing());
        if forw_end_rect_dir.is_ortho() {
          let from = forw_end_rect_dir.end(&wh) + self.get_end_point();
          let to = back_start_rect_dir.start(&wh) + self.get_end_point();
          result.push(Box::new(Line::new(to, from)));
          result
        } else {
          let from = forw_end_rect_dir.end(&wh) + self.get_end_point();
          let middle = back_start_rect_dir.opposite(&wh) + self.get_end_point();
          let to = back_start_rect_dir.start(&wh) + self.get_end_point();
          result.push(Box::new(Line::new(middle, from)));
          result.push(Box::new(Line::new(to, middle)));
          result
        }
      },
      _ => unreachable!()
    }
  }

  fn create_starting_cap(&self, path_type: &PathType) -> Vec<Box<dyn StrokePathElement>> {
    match path_type {
      PathType::Circle(_) => {
        let mut result: Vec<Box<dyn StrokePathElement>> = Vec::new();
        let from = self.get_backward_end_point(path_type);
        let to = self.get_forward_start_point(path_type);
        let center = self.get_start_point();
        result.push(Box::new(Arc::new_with_fixed_center(
            to,
            from,
            center,
            CircularDirection::CW
            )));
        result
      },
      PathType::Rect(w, h) => {
        let wh = Vec2::new(*w, *h);
        let mut result: Vec<Box<dyn StrokePathElement>> = Vec::new();
        let back_start_rect_dir = RectDir::from_dir(-self.get_direction_in_start_point(), -self.get_normal_in_start_point());
        let forw_end_rect_dir = RectDir::from_dir(self.get_direction_in_start_point(), self.get_normal_in_start_point());
        if back_start_rect_dir.is_ortho() {
          let from = back_start_rect_dir.end(&wh) + self.get_start_point();
          let to = forw_end_rect_dir.start(&wh) + self.get_start_point();
          result.push(Box::new(Line::new(to, from)));
          result
        } else {
          let from = back_start_rect_dir.end(&wh) + self.get_start_point();
          let middle = forw_end_rect_dir.opposite(&wh) + self.get_start_point();
          let to = forw_end_rect_dir.start(&wh) + self.get_start_point();
          result.push(Box::new(Line::new(middle, from)));
          result.push(Box::new(Line::new(to, middle)));
          result
        }
      },
      _ => unreachable!()
    }
  }

  fn create_forward_with(&self, forward_start_point: Vec2, forward_end_point: Vec2) -> Box<dyn StrokePathElement>;
  fn create_backward_with(&self, forward_start_point: Vec2, forward_end_point: Vec2) -> Box<dyn StrokePathElement>;

  fn has_forward_transition(&self, path_type: &PathType, next: &dyn StrokePathElement) -> bool {
    match path_type {
      PathType::Rect(_,_) => false,
      PathType::Circle(_) => {
        let my_dir = self.get_direction_in_end_point();
        let other_dir = next.get_direction_in_start_point();
        let angle = Rotation2::rotation_between(&my_dir, &other_dir).angle();
        angle < 0.0
      },
      _ => unreachable!()
    }
  }

  fn has_backward_transition(&self, path_type: &PathType, prev: &dyn StrokePathElement) -> bool {
    match path_type {
      PathType::Rect(_,_) => false,
      PathType::Circle(_) => {
        let my_dir = self.get_direction_in_start_point();
        let other_dir = prev.get_direction_in_end_point();
        let angle = Rotation2::rotation_between(&other_dir, &my_dir).angle();
        angle > 0.0
      },
      _ => unreachable!()
    }
  }

  fn create_forward_transition(&self, path_type: &PathType, next: &dyn StrokePathElement) 
    -> Box<dyn StrokePathElement> {
    match path_type {
      PathType::Circle(_) => {
        let center = self.get_end_point();
        let from = self.get_forward_end_point(&path_type);
        let to = next.get_forward_start_point(&path_type);

        Box::new(Arc::new_with_fixed_center(to, from, center, CircularDirection::CW))
      },
      PathType::Rect(_w, _h) => {
        unreachable!("Cannot make transition for rect path")
      },
      _ => unreachable!()
    }
  }
  fn create_backward_transition(&self, path_type: &PathType, prev: &dyn StrokePathElement) 
    -> Box<dyn StrokePathElement> {
    match path_type {
      PathType::Circle(_) => {
        let center = self.get_start_point();
        let from = self.get_backward_end_point(&path_type);
        let to = prev.get_backward_start_point(&path_type);

        println!("make arc {:?}", Arc::new_with_fixed_center(to, from, center, CircularDirection::CW));


        Box::new(Arc::new_with_fixed_center(to, from, center, CircularDirection::CW))
      },
      PathType::Rect(_w, _h) => {
        unreachable!("Cannot make transition for rect path")
      },
      _ => unreachable!()
    }
  }

  fn has_backward_transition_with_next(&self, path_type: &PathType, next: &dyn StrokePathElement) -> bool {
    match path_type {
      PathType::Rect(_,_) => false,
      PathType::Circle(_) => {
        let my_dir = self.get_direction_in_end_point();
        let other_dir = next.get_direction_in_start_point();
        let angle = Rotation2::rotation_between(&other_dir, &my_dir).angle();
        angle < 0.0
      },
      _ => unreachable!()
    }
  }

  fn has_forward_transition_with_prev(&self, path_type: &PathType, prev: &dyn StrokePathElement) -> bool {
    match path_type {
      PathType::Rect(_,_) => false,
      PathType::Circle(_) => {
        let my_dir = self.get_direction_in_start_point();
        let other_dir = prev.get_direction_in_end_point();
        let angle = Rotation2::rotation_between(&other_dir, &my_dir).angle();
        angle < 0.0
      },
      _ => unreachable!()
    }
  }

  fn forward(&self, path_type: &PathType, prev: Option<&dyn StrokePathElement>, next: Option<&dyn StrokePathElement>) -> Vec<Box<dyn StrokePathElement>> {
    let mut result: Vec<Box<dyn StrokePathElement>> = Vec::new();
    let mut forward_start_point = self.get_forward_start_point(&path_type);
    let mut forward_end_point = self.get_forward_end_point(&path_type);
    let mut needs_start_cap = false;
    let mut needs_end_cap = false;
    let mut to_for_transition: Option<&dyn StrokePathElement> = None;
    match prev {
      None => {
        needs_start_cap = true;
      },
      Some(element) => {
        if !self.has_forward_transition_with_prev(&path_type, element) {
          let line_one = self.create_forward_with(
            self.get_forward_start_point(&path_type), 
            self.get_forward_end_point(&path_type)
            );

          let line_two = element.create_forward_with(
            element.get_forward_start_point(&path_type),
            element.get_forward_end_point(&path_type)
            );

          forward_start_point = line_two.get_intersector().intersects(line_one.get_intersector());
        }
      }
    }
    match next {
      None => {
        needs_end_cap = true;
      },
      Some(element) => {
        if self.has_forward_transition(&path_type, element) {
          to_for_transition.replace(element);
        }else {
          let line_one = self.create_forward_with(
            self.get_forward_start_point(&path_type), 
            self.get_forward_end_point(&path_type)
            );

          let line_two = element.create_forward_with(
            element.get_forward_start_point(&path_type),
            element.get_forward_end_point(&path_type)
            );

          forward_end_point = line_one.get_intersector().intersects(line_two.get_intersector());
        }
      }
    }

    if needs_start_cap {
      for p in self.create_starting_cap(&path_type).into_iter() {
        result.push(p);
      }
    }

    result.push(self.create_forward_with(forward_start_point, forward_end_point));

    if let Some(next_element) = to_for_transition {
      result.push(self.create_forward_transition(&path_type, next_element));
    }

    if needs_end_cap {
      result.extend(self.create_ending_cap(&path_type));
    }

    result
  }

  fn backward(&self, path_type: &PathType, prev: Option<&dyn StrokePathElement>, next: Option<&dyn StrokePathElement>) -> Vec<Box<dyn StrokePathElement>> {
    let mut result: Vec<Box<dyn StrokePathElement>> = Vec::new();
    let mut backward_start_point = self.get_backward_start_point(&path_type);
    let mut backward_end_point = self.get_backward_end_point(&path_type);
    let mut to_for_transition: Option<&dyn StrokePathElement> = None;
    match prev {
      None => (),
      Some(element) => {
        if self.has_backward_transition(&path_type, element) {
          to_for_transition.replace(element);
        }else {
          let line_one = self.create_backward_with(
            self.get_backward_start_point(&path_type), 
            self.get_backward_end_point(&path_type)
          );
          let line_two = element.create_backward_with(
            element.get_backward_start_point(&path_type), 
            element.get_backward_end_point(&path_type)
          );

          println!("lets check arcs with prev {:?}, {:?}", line_one, line_two);

          backward_end_point = line_one.get_intersector().intersects(line_two.get_intersector());
        }
      }
    }

    match next {
      None => (),
      Some(element) => {
        if !self.has_backward_transition_with_next(&path_type, element) {
          let line_one = self.create_backward_with(
            self.get_backward_start_point(&path_type), 
            self.get_backward_end_point(&path_type)
          );
          let line_two = element.create_backward_with(
            element.get_backward_start_point(&path_type), 
            element.get_backward_end_point(&path_type)
          );
          println!("lets check arcs with next \n\n{:?}, \n\n{:?}, \n\n{:?}", line_one, line_two, element);

          backward_start_point = line_two.get_intersector().intersects(line_one.get_intersector());
        }
      }
    }


    if let Some(prev_node) = to_for_transition {
      println!("------------make transition-----------------");

      result.push(self.create_backward_transition(&path_type, prev_node));
    };

    result.push(self.create_backward_with(backward_start_point, backward_end_point));

    result
  }
}



pub fn to_stroke_around_path(path: Path) -> Path {

  let mut forward: Vec<Box<dyn StrokePathElement>> = Vec::new();
  let mut backward: Vec<Box<dyn StrokePathElement>> = Vec::new();
  let Path{tp, elements} = path;

  for ix in 0..elements.len() {
    let prev = if ix == 0 { None } else {elements.get(ix - 1).map(|i| i.as_ref())};
    let current = &elements[ix];
    let next = elements.get(ix + 1).map(|i| i.as_ref());
    forward.extend(current.forward(&tp, prev, next));
    backward.extend(current.backward(&tp, prev, next));
  }
  for item in forward.iter() {
    println!("forward {:?}", item);
    println!("--------------------------------------------------------");
  }
  for item in backward.iter() {
    println!("backward {:?}", item);
    println!("--------------------------------------------------------");
  }
  backward.reverse();
  forward.extend(backward);


  Path {
    tp,
    elements: forward
  }
}
