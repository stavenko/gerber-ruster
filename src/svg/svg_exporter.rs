extern crate nalgebra as na;
use super::bounding_box::*;
// use super::ray::Ray;
use crate::plotter::{PathElement, PathType };
// use PathElement::*;
use std::f32::consts::PI;
// use std::cmp::Ordering;
use na::{  Rotation2 };


/*
#[derive(Debug, PartialEq)]
struct Arc {
  radius: f32,
  to: Vec2,
  from: Vec2,
  is_large: bool,
  direction: CircularDirection
}

#[derive(Debug, Clone, PartialEq)]
enum RectDir {
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
}

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

fn cmp(a: &f32, b: &f32) -> Ordering {
  if a > b {
    Ordering::Greater
  }else {
    Ordering::Less
  }
}

impl Arc {
  fn new(r: f32, to: Vec2, is_large: bool, dir: CircularDirection) -> Self {
    Arc{
      radius: r,
      to,
      is_large,
      direction: dir,
      from: Vec2::new(0.0, 0.0)
    }
  }

  fn new_with_from(r: f32, from: Vec2, to: Vec2, is_large: bool, dir: CircularDirection) -> Self {
    Arc{
      radius: r,
      to,
      is_large,
      direction: dir,
      from
    }
  }

  fn is_between(&self, from: &Vec2, a: f32) -> bool {
    use CircularDirection::*;
    let center = self.get_center(from);
    let from = from - center;
    let to = self.to - center;
    let afrom = from.y.atan2(from.x);
    let afrom = if afrom > 0.0 {afrom} else {2.0 * PI + afrom};
    let anglebw = Rotation2::rotation_between(&to, &from).angle(); 
    let anglebw = if self.is_large { 2.0 * PI - anglebw } else {anglebw};



    match &self.direction {
      CW => {
        let ato = afrom - anglebw;
        let dat = a - ato;
        let daf = a - afrom;


        (a >= ato && a <= afrom ) || dat.abs() <= 1e-6 || daf.abs() <= 1e-6
      }
      CCW => {
        let ato = afrom + anglebw;
        let dat = a - ato;
        let daf = a - afrom;
        a >= afrom && a <= ato || dat.abs() <= 1e-6 || daf.abs() <= 1e-6
      }
    }
  }

  fn bounding_box(&self, from: &Vec2) -> BoundingBox {
    let mut bb = BoundingBox::default();
    for a in vec!(0.0, 0.25, 0.5, 0.75) {
      let a = 2.0 * PI * a ;
      if self.is_between(from, a) {
        let (y, x) = a.sin_cos();
        let center = self.get_center(from);
        let point = self.radius * Vec2::new(x,y) + center;
        if point.y > 0.7 {
          println!("add poing {} {} {} {}", a, point, self.radius, center);
        };
        bb = bb.add(point);
      }
    }
    bb
  }


  fn get_center(&self, from: &Vec2) -> Vec2 {
    let rot90 = Rotation2::new(PI / 2.0);
    let horde = self.to - from;
    let center = (self.to + from) / 2.0;
    let h = (self.radius.powi(2) - (horde.magnitude()/2.0).powi(2)).abs().sqrt();
    let n = rot90 * horde.normalize();
    let candidate1 = center + n * h;
    let candidate2 = center + n * -h;

    let basis_x = from - candidate1;
    let basis_y = rot90 * basis_x;
    let vector = self.to - candidate1;
    let x = vector.dot(&basis_x);
    let y = vector.dot(&basis_y); 

    let angle = y.atan2(x) * match &self.direction {CircularDirection::CW => -1.0, _ => 1.0};
    //println!(" {:?} {:?} {}", candidate1, center + n * -h, angle);

    if self.is_large {
      if angle < 0.0 {
        candidate1
      } else {
        candidate2
      }
    } else {
      if angle < 0.0 {
        candidate2
      } else {
        candidate1
      }
    }
  }
}



*/
#[derive(Debug, PartialEq)]
pub enum SvgCommand {
  MoveTo(Vec2),
  LineTo(Vec2),
  ArcTo(Arc)
}

impl SvgCommand {
  pub fn to(&self) -> Vec2 {
    use SvgCommand::*;
    match self {
      MoveTo(v) => v.clone(),
      LineTo(v) => v.clone(),
      ArcTo(Arc{to,..}) => to.clone()
    }
  }
  pub fn bounding_box(&self, prev: &Vec2) -> BoundingBox {
    use SvgCommand::*;
    match self {
      MoveTo(_) => BoundingBox::default(),
      LineTo(v) => BoundingBox::from(*v),
      ArcTo(arc) => arc.bounding_box(prev)
    }
  }
}

impl SvgCommand{
  pub fn serialize(&self) -> String {
    use SvgCommand::*;
    use CircularDirection::*;
    match self {
      MoveTo(v) => String::from(format!("M {} {}", v.x, v.y)),
      LineTo(v) => String::from(format!("L {} {}", v.x, v.y)),
      ArcTo(Arc{ radius, to, is_large, direction,.. }) => String::from(
        format!("A {} {} {} {} {} {} {}", radius, radius, 0, if *is_large {1}else{0}, match direction{
          CCW => 1,
          _ => 0
        }, to.x, to.y)
      )
    }
  }
}

pub type SvgPath = Vec<SvgCommand>;


pub fn stroke_around_path(path: &Path) -> SvgPath {
  let path_builder = PathBuilder::new(path.tp.clone(), path.elements.clone());
  path_builder.collect_svg()
}

/*
fn get_coords(pe: &PathElement) -> &Vector2<f32> {
  match &pe {
    Linear(v) => v,
    Start(v) => v,
    SingleQuadrant(a) => &a.to,
    MultiQuadrant(a) => &a.to,
  }
}
*/

type PathAccum = (SvgPath, SvgPath);

struct PathBuilder {
  path_type: PathType,
  paths: PathAccum,
  elements: Vec<Box<dyn PathElement>>,
  current_el: usize,
  rot90: Rotation2<f32>,// = Rotation2::new(PI/2.0);
  rot90_minus: Rotation2<f32>,// = Rotation2::new(-PI/2.0);
}

impl PathBuilder {
  fn new (path_type: PathType, elements: Vec<Box<dyn PathElement>>) -> Self {
    PathBuilder {
      current_el: 0,
      elements,
      path_type,
      paths: ((), ()), //(Vec::new(), Vec::new()),
      rot90: Rotation2::new(PI/2.0),
      rot90_minus: Rotation2::new(-PI/2.0)
    }
  }

  /*
  fn forward(&mut self, el: SvgCommand)  {
    self.paths.0.push(el)
  }

  fn backward(&mut self, el: SvgCommand)  {
    self.paths.1.push(el);
  }


  fn prepare_paths(&mut self) {
    let len = self.elements.len();
    for ix in 0..len {
      self.current_el = ix;
      self.process_node();
    }
  }

  fn process_node(&mut self) {
    let current_element = self.elements.get(self.current_el);
    println!("--->{:?}", current_element);
    match current_element {
      Some(Start(_)) => self.process_start_node(),
      Some(Linear(_)) => self.process_linear_node(),
      Some(SingleQuadrant(_)) => self.process_cicular_node(),
      Some(MultiQuadrant(_)) => self.process_cicular_node(),
      None => ()
    }
  }

  fn process_start_node(&mut self) {
    let next_node = self.elements.get(self.current_el +1);
    println!("---=>{:?}", next_node);
    match next_node {
      Some(Linear(_)) => self.start_node_with_linear_next(),
      Some(SingleQuadrant(_)) => self.start_node_with_single_quad_next(),
      Some(MultiQuadrant(_)) => self.start_node_with_multiple_quad_next(),
      _ => println!("We shall have next node and we shall not have next start node")
    }
  }

  fn cur_node(& self) -> Vec2 {
    get_coords(&self.elements[self.current_el]).clone()
  }

  fn next_node(&self) -> Vec2 {
    get_coords(&self.elements[self.current_el + 1]).clone()
  }

  fn prev_node(&self) -> Vec2 {
    get_coords(&self.elements[self.current_el - 1]).clone()
  }

  fn process_linear_node(&mut self) {
    let next_node = self.elements.get(self.current_el +1);
    match next_node {
      Some(Linear(_)) => self.linear_node_with_linear_next(),
      Some(SingleQuadrant(_)) => self.linear_node_with_single_quad(),
      Some(MultiQuadrant(_)) => self.linear_node_with_multiple_quad(),
      None => self.linear_node_last(),
      _ => println!("We shall not have start node after linear node")
    }
  }

  fn process_cicular_node(&mut self) {
    let next_node = self.elements.get(self.current_el +1);
    match next_node {
      Some(Linear(_)) => self.circular_node_with_linear_next(),
      Some(SingleQuadrant(_)) => self.circular_node_with_circular_next(),
      Some(MultiQuadrant(_)) => self.circular_node_with_circular_next(),
      None => self.circular_node_last(),
      _ => println!("We shall not have start node after linear node")
    }
  }

  fn circular_node_with_circular_next(&mut self) {
    unimplemented!();
  }
  fn circular_node_with_linear_next(&mut self) {
    unimplemented!();
  }

  fn linear_node_with_single_quad(&mut self) {
    unimplemented!()
  }

  fn linear_node_with_multiple_quad(&mut self) {
    unimplemented!()
  }

  fn get_perpendicular(&self, a: &Vec2, b: &Vec2, forward: bool) -> Vec2 {
    let dir = b - a;
    if forward {
      self.rot90 * dir.normalize()
    } else {
      self.rot90_minus * dir.normalize()
    }
  }

  fn get_rect_points(&self) -> [Vec2; 4] {
    match self.path_type {
      PathType::Rect(w, h) => {
        let hw = w / 2.0;
        let hh = w / 2.0;
        [
          Vec2::new(-hw, -hh),
          Vec2::new(-hw, hh),
          Vec2::new(hw, hh),
          Vec2::new(hw, -hh)
        ]
      },
      _ => {
        unreachable!()
      }
    }
  }

  fn get_from_path_point(&self, from: &Vec2, to: &Vec2) -> Vec2 {
    let is_forward = true;
    match self.path_type {
      PathType::Circle(half_width) => {
        let dir = self.get_perpendicular(from, to, is_forward);
        dir * half_width + from
      },
      PathType::Rect(w,h) => {
        use RectDir::*;
        let wh = Vec2::new(w, h);
        println!("calc direction {:?}", self.get_path_dir(&from, &to));
        from + match self.get_path_dir(&from, &to) {
          Left => RightBottom.vec(&wh),
          Right => LeftTop.vec(&wh),
          Top => LeftBottom.vec(&wh),
          Bottom => RightTop.vec(&wh),

          LeftBottom => RightBottom.vec(&wh),
          RightBottom => RightTop.vec(&wh),
          RightTop => LeftTop.vec(&wh),
          LeftTop => LeftBottom.vec(&wh),
        }
      }
    }
  }


  fn get_to_path_point(&self, from: &Vec2, to: &Vec2) -> Vec2 {
    let is_forward = false;
    match self.path_type {
      PathType::Circle(half_width) => {
        let dir = self.get_perpendicular(from, to, is_forward);
        dir * half_width + from
      },
      PathType::Rect(w, h) => {
        use RectDir::*;
        let wh = Vec2::new(w, h);
        to + match self.get_path_dir(&to, &from) {
          Left => RightTop.vec(&wh),
          Right => LeftBottom.vec(&wh),
          Top => RightBottom.vec(&wh),
          Bottom => LeftTop.vec(&wh),

          LeftBottom => LeftTop.vec(&wh),
          RightBottom => LeftBottom.vec(&wh),
          RightTop => RightBottom.vec(&wh),
          LeftTop => RightTop.vec(&wh),
        }
      }
    }
  }

  fn get_path_dir(&self, from: &Vec2, to: &Vec2) -> RectDir {
    use RectDir::*;
    match self.path_type {
      PathType::Rect(w,h) => {
        let wh = Vec2::new(w,h);
        let dir = (to - from).normalize();
        let normal = self.get_perpendicular(from, to, true);
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

      },
      _ => unreachable!()
    }
  }

  fn path_start_cap(&self) -> Vec<SvgCommand> {
    let cur_node = self.cur_node();
    let next_el = &self.elements[self.current_el +1];
    match self.path_type {
      PathType::Circle(half_width) => {
        let to = match next_el {
          Linear(next_coord) => self.get_from_path_point(&cur_node, &next_coord),
          MultiQuadrant(_) => {
            let Arc{from, ..} = self.arc_forward(&self.current_el + 1);
            from
          },
          SingleQuadrant(_) => {
            let arc = self.arc_forward(&self.current_el + 1);
            arc.from
          },
          _ => unreachable!()
        };
        
        vec!(SvgCommand::ArcTo(Arc::new(
            half_width,
            to.clone(),
            false,
            CircularDirection::CW
          )))
      },
      PathType::Rect(w,h) => {
        let wh = Vec2::new(w,h);
        use RectDir::*;
        match next_el {
          Linear(next_node) => {
            match self.get_path_dir(&cur_node, &next_node) {
              Left => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node))),
              Right => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node))),
              Top => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node))),
              Bottom => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node))),

              LeftTop => vec!(
                SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node)),
                SvgCommand::LineTo(RightBottom.vec(&wh) + cur_node)
                ),

              LeftBottom => vec!(
                SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node)),
                SvgCommand::LineTo(RightTop.vec(&wh) + cur_node)
                ),

              RightTop => vec!(
                SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node)),
                SvgCommand::LineTo(LeftBottom.vec(&wh) + cur_node)
                ),

              RightBottom => vec!(
                SvgCommand::LineTo(self.get_from_path_point(&cur_node, &next_node)),
                SvgCommand::LineTo(LeftTop.vec(&wh) + cur_node)
                )
            }
          },
          _ => unimplemented!()
        }
      }
    }
  }

  fn path_end_cap(&self)-> Vec<SvgCommand> {
    let prev_el = &self.elements[self.current_el];
    let prev_node = self.prev_node();
    let cur_node = self.cur_node();
    match self.path_type {
      PathType::Circle(half_width) => {
        let to = match prev_el {
          Linear(next_coord) => self.get_from_path_point(&cur_node, &next_coord),
          MultiQuadrant(_) => {
            let Arc{from, ..} = self.arc_backward(self.current_el);
            from
          },
          SingleQuadrant(_) => {
            let arc = self.arc_backward(self.current_el);
            arc.from
          },
          _ => unreachable!()
        };
        vec!(SvgCommand::ArcTo(Arc::new(
            half_width,
            to.clone(),
            false,
            CircularDirection::CW
          )))
      },
      PathType::Rect(w,h) => {
        let wh = Vec2::new(w,h);
        use RectDir::*;
        let cur_node = self.cur_node();
        println!("path dir for end cap {:?}", self.get_path_dir(&cur_node, &prev_node));
        match self.get_path_dir(&cur_node, &prev_node) {
          Left => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))),
          Right => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))),
          Top => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))),
          Bottom => vec!(SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))),

          LeftTop => vec!(
            SvgCommand::LineTo(RightBottom.vec(&wh) + cur_node),
            SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))
            ),

          LeftBottom => vec!(
            SvgCommand::LineTo(RightTop.vec(&wh) + cur_node),
            SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))
            ),

          RightTop => vec!(
            SvgCommand::LineTo(LeftBottom.vec(&wh) + cur_node),
            SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))
            ),

          RightBottom => vec!(
            SvgCommand::LineTo(LeftTop.vec(&wh) + cur_node),
            SvgCommand::LineTo(self.get_from_path_point(&cur_node, &prev_node))
            )
        } 
      }
    }
  }

  fn round_transition(&self, is_forward: bool) -> Option<SvgCommand> {
    match self.path_type {
      PathType::Circle(half_width) => {
        let prev_point = self.prev_node();
        let cur_point = self.cur_node();
        let next_point = self.next_node();
        let to = if is_forward {
          self.get_from_path_point(&cur_point, &next_point)
        } else {
          self.get_from_path_point(&cur_point, &prev_point)
        };


        Some(SvgCommand::ArcTo(Arc::new(
              half_width, 
              to,
              false,
              CircularDirection::CW
            )))
      },
      _ => None
    }
  }


  fn circular_node_last(&mut self) {
    let prev_point = self.prev_node();
    let arc = self.arc_forward(self.current_el);
    self.forward(SvgCommand::ArcTo(arc));
    let back_arc = self.arc_backward(self.current_el);
    println!("back_ark {:?}", back_arc);
    for item in self.path_end_cap() {
      self.forward(item);
    }
    self.backward(SvgCommand::ArcTo(back_arc));
  }

  fn linear_node_last(&mut self) {
    let prev_point = self.prev_node();
    let cur_point = self.cur_node();
    self.forward(SvgCommand::LineTo(self.get_to_path_point(&prev_point, &cur_point)));
    for item in self.path_end_cap() {
      self.forward(item);
    }
    self.backward( SvgCommand::LineTo(self.get_to_path_point(&cur_point, &prev_point)));
  }

  fn linear_node_with_linear_next(&mut self) {
    let prev_point = {self.prev_node()};
    let cur_point = self.cur_node();
    let next_point = self.next_node();
    let current_dir = cur_point - prev_point;
    let next_dir = next_point - cur_point;
    let angle = Rotation2::rotation_between(&current_dir, &next_dir).angle();
    if angle > 0.0 && angle < PI {
      // up
      let origin = self.get_from_path_point(&prev_point, &cur_point);
      let dir = current_dir.normalize();
      let initial_ray = Ray::new(origin, dir);
      let origin = self.get_from_path_point(&cur_point, &next_point);
      let dir = next_dir.normalize();
      let next_ray = Ray::new(origin, dir);
      let rays_intersection = initial_ray.intersects(&next_ray);
      self.forward( SvgCommand::LineTo(rays_intersection));
      self.backward( SvgCommand::LineTo(self.get_to_path_point(&cur_point, &prev_point)));
      match self.round_transition(false) {
        Some(arc) => self.backward(arc),
        None => ()
      };

    } else {
      //down
      self.forward(SvgCommand::LineTo(self.get_to_path_point(&prev_point, &cur_point)));
      match self.round_transition(true) {
        Some(arc) => {
          self.forward(arc)
        },
        None => ()
      };
      let origin = self.get_to_path_point(&cur_point, &prev_point);
      let dir = current_dir.normalize();
      let initial_ray = Ray::new(origin, dir);
      let origin = self.get_from_path_point(&next_point, &cur_point);
      let dir = -next_dir.normalize();
      let next_ray = Ray::new(origin,dir);
      let rays_intersection = initial_ray.intersects(&next_ray);
      self.backward(SvgCommand::LineTo(self.get_to_path_point(&cur_point, &prev_point)));
      self.backward(SvgCommand::LineTo(rays_intersection));
    }
  }

  fn get_arc_center(&self, node: usize) -> Vec2 {
    let from = get_coords(&self.elements[node-1]); 
    match &self.elements[node] {
      PathElement::MultiQuadrant(RawArc{to,dir, i, j}) => {
        let cx = i.map(|i| i + from.x).unwrap_or(from.x);
        let cy = j.map(|i| i + from.y).unwrap_or(from.y);
        Vec2::new(cx, cy)
      },
      PathElement::SingleQuadrant(RawArc{to, dir, i, j}) => {
        let cx = i.unwrap_or(0.0);
        let cy = j.unwrap_or(0.0);

        let center = vec!(
          from + Vec2::new(cx, cy),
          from + Vec2::new(cx, -cy),
          from + Vec2::new(-cx, cy),
          from + Vec2::new(-cx, -cy)
          ).into_iter().min_by(|c1, c2| {

          let arc_len1 = arc_len(dir, &c1, &from, &to);
          let arc_len2 = arc_len(dir, &c2, &from, &to);
          cmp(&arc_len1, &arc_len2)
        }).unwrap();
        center
      },
      _ => unreachable!()
    }
  }

  fn get_arc_direction(&self, node_id: usize) -> CircularDirection {
    match &self.elements[node_id] {
      PathElement::MultiQuadrant(RawArc{dir,..}) => dir.clone(),
      PathElement::SingleQuadrant(RawArc{dir,..}) => dir.clone(),
      _ => unreachable!()
    }
  }

  fn get_arc_end(&self, node_id: usize) -> Vec2 {
    match &self.elements[node_id] {
      PathElement::MultiQuadrant(RawArc{to,..}) => to.clone(),
      PathElement::SingleQuadrant(RawArc{to,..}) => to.clone(),
      _ => unreachable!()
    }
  }


  fn arc_forward(&self, for_node: usize) -> Arc {
    let from = get_coords(&self.elements[for_node -1]);
    match self.path_type {
      PathType::Circle(half_width) => {
        let current_arc_direction = self.get_arc_direction(for_node);
        let to = self.get_arc_end(for_node);
        let center = self.get_arc_center(for_node);
        let arc_start = from - center;
        let dir = arc_start.normalize();
        let radius = arc_start.magnitude();

        let resulting_radius = match current_arc_direction {
          CircularDirection::CW => radius + half_width,
          CircularDirection::CCW => radius - half_width
        };
        let start = dir * resulting_radius; 
        let arc_end = to - center;
        let dir = arc_end.normalize();

        let end = dir * resulting_radius;
        let (start, end) = (start + center, end + center);

        let arc_lenght = arc_len(&current_arc_direction, &center, &start, &end);
        println! ("forward {} center {}, current_arc_direction {:?} start {} end {}", arc_lenght, center, current_arc_direction, start, end);
        Arc::new_with_from (resulting_radius, start, end, arc_lenght > PI, current_arc_direction)

      },
      _=> unimplemented!()
    }
  }

  fn arc_backward(&self, for_node: usize) -> Arc {
    let to = get_coords(&self.elements[for_node -1]);
    match self.path_type {
      PathType::Circle(half_width) => {
        let current_arc_direction = self.get_arc_direction(for_node).reverse();
        let from = self.get_arc_end(for_node);
        let center = self.get_arc_center(for_node);
        let arc_start = from - center;
        let dir = arc_start.normalize();
        let radius = arc_start.magnitude();

        let resulting_radius = match current_arc_direction {
          CircularDirection::CW => radius + half_width,
          CircularDirection::CCW => radius - half_width
        };

        let start = dir * resulting_radius;
        let arc_end = to - center;
        let dir = arc_end.normalize();
        let end = dir * resulting_radius;

        let (start, end) = (start + center, end + center);

        let arc_lenght = arc_len(&current_arc_direction, &center, &end, &start);
        Arc::new_with_from(resulting_radius, start, end, arc_lenght > PI, current_arc_direction)

      },
      _=> unimplemented!()
    }
  }

  fn start_node_with_single_quad_next(&mut self) {
    let arc = self.arc_forward(self.current_el + 1);
    self.forward( SvgCommand::MoveTo(arc.from));
    for item in self.path_start_cap() {
      self.backward(item);
    }
  }

  fn start_node_with_multiple_quad_next(&mut self) {
    self.start_node_with_single_quad_next()
  }

  fn start_node_with_linear_next(&mut self)  {
    let base_start = self.cur_node();
    let base_end = self.next_node();

    let start_point = self.get_from_path_point(&base_start, &base_end);

    self.forward( SvgCommand::MoveTo(start_point));
    for item in self.path_start_cap() {
      self.backward(item);
    }
  }

  */


  pub fn collect_svg(mut self) -> SvgPath {
    /*
    self.prepare_paths();
    let (mut f, mut b) = self.paths;
    b.reverse();
    f.extend(b);
    for i in f.iter() {
      println!("P {:?}", i);
    };
    f
    */
  }
}
