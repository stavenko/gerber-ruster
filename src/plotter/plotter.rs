extern crate nalgebra as na;
use crate::parser::*;
use std::collections::HashMap;
use std::convert::From;
use na::{ Vector2};
use super::line::Line;
use super::arc::Arc;
use super::circular_direction::*;
use super::stroke_path::{ StrokePathElement, to_stroke_around_path };

type Vec2 = Vector2<f32>;


#[derive(Debug, PartialEq, Clone)]
pub struct RawArc{
  pub to: Vec2,
  pub dir: CircularDirection,
  pub i: Option<f32>,
  pub j: Option<f32>
}

#[derive(Clone, Debug, PartialEq)]
enum RawPathElement {
  SingleQuadrant(RawArc),
  MultiQuadrant(RawArc),
  Linear(Vec2),
  Start(Vec2)
}

#[derive(Clone, Debug, PartialEq)]
pub enum PathType {
  Rect(f32, f32),
  Circle(f32),
  Stroke
}


#[derive(Debug, PartialEq)]
struct RawPath {
  tp: PathType,
  elements: Vec<RawPathElement>,
}

pub struct Path{ 
  pub(in super) tp: PathType,
  pub elements: Vec<Box<dyn StrokePathElement>>
}

impl Path {
  fn new(tp: PathType) -> Path {
    Path {
      tp,
      elements: Vec::new()
    }
  }

  pub fn add(&mut self, element: Box<dyn StrokePathElement>) {
    self.elements.push(element);
  }
}

impl From<RawPath> for Path {
  fn from(path: RawPath) -> Self {
    use RawPathElement::*;
    let mut path_elements: Path = Path::new(path.tp);
    let mut start_point: Option<Vec2> = None;
    for element in path.elements.into_iter() {
      match element {
        Start(v) => {start_point.replace(v);},
        Linear(v) => {
          match start_point { 
            Some(from) => path_elements.add(Box::new(Line::new(v, from))),
            None => unreachable!("Must be start_point")
          };
          start_point.replace(v);
        },
        SingleQuadrant(RawArc{ to, dir, i, j }) => {
          match start_point { 
            Some(from) => path_elements.add(Box::new(Arc::new(
              to,
              from,
              i, 
              j,
              true,
              dir
            ))),
            None => unreachable!("Must be start_point")
          };
          start_point.replace(to);
        },
        MultiQuadrant(RawArc{ to, dir, i, j }) => {
          match start_point { 
            Some(from) => path_elements.add(Box::new(Arc::new(
              to,
              from,
              i, 
              j,
              false,
              dir
            ))),
            None => unreachable!("Must be start_point")
          };
          start_point.replace(to);
        }
      }
    };
    path_elements
  }
}


impl RawPath {
  pub fn start_with(tp: PathType, start: Vec2) -> Self {
    RawPath {
      tp,
      elements: vec!(RawPathElement::Start(start))
    }
  }
  pub fn start(tp: PathType,) -> Self {
    RawPath {
      tp,
      elements: Vec::new()
    }
  }

  pub fn push(&mut self, el: RawPathElement) {
    self.elements.push(el);
  }
}




pub struct Plotter {
  unit: Option<Unit>,
  format: Option<FormatSpecification>,
  tools: HashMap<String, ApertureTemplatePrimitive>,
  interpolation: Option<Interpolation>,
  circular_direction: Option<CircularDirection>,
  selected_aperture: Option<(String, ApertureTemplatePrimitive)>,
  collected_paths: Vec<Path>,
  current_path: Option<RawPath>
}

impl Plotter {
  pub fn new() -> Self {
    Plotter {
      interpolation: None,
      circular_direction: None,
      selected_aperture: None,
      unit: None, 
      format: None, 
      tools: HashMap::new(),
      collected_paths: Vec::new(),
      // bounding_box: BoundingBox::default(),
      current_path: None
    }
  }

  fn set_unit(&mut self, u: Unit) {
    self.unit.replace(u);
  }

  fn set_format(&mut self, f: FormatSpecification) {
    self.format.replace(f);
  }

  fn set_circular_direction(&mut self, dir: CircularDirection) {
    self.circular_direction.replace(dir);
  }

  fn add_aperture(&mut self, a: Aperture) {
    self.tools.insert(a.name, a.template);
  }

  fn apply_aperture(&mut self, name: String) {
    let aperture = self.tools.remove_entry(&name);
    match aperture {
      Some(ap) => {
        let prev_aperture = self.selected_aperture.replace(ap);
        if let Some(ap) = prev_aperture {
          self.tools.insert(ap.0, ap.1);
        }
      },
      None => {
        panic! ("Aperture {} not found", name);
      }
    }
  }

  fn set_interpolation(&mut self, i: Interpolation) {
    self.interpolation.replace(i);
  }

  pub fn consume(&mut self, command: GerberCommand) {
    match command {
      GerberCommand::Unit(u) => self.set_unit(u),
      GerberCommand::FormatSpecification(f) => self.set_format(f),
      GerberCommand::ApertureDefinition(a) => self.add_aperture(a),
      GerberCommand::Interpolation(i) => self.set_interpolation(i),
      GerberCommand::ApplyAperture(a) => self.apply_aperture(a),
      GerberCommand::Operation(op) => self.operation(op),
      GerberCommand::CounterClockWiseArc => self.set_circular_direction(CircularDirection::CCW),
      GerberCommand::ClockWiseArc => self.set_circular_direction(CircularDirection::CW),
      _ => ()
    }
  }


  fn operation(&mut self, op: Operation) {
    match op.op_type {
      OperationType::Interpolation => self.interpolation(op.coords),
      OperationType::Move => self.start_new_path(Some(op.coords)),
      _ => ()
    }
  }

  fn path_type(&self) -> PathType {
    let ap = self.selected_aperture.as_ref();
    use ApertureTemplatePrimitive::*;
    match ap {
      Some((_, ap)) => {
        match ap {
          R(r) => PathType::Rect(r.width, r.height),
          C(c) => PathType::Circle(c.diameter / 2.0),
          P(p) => PathType::Circle(p.outer_diameter/2.0),
          O(o) => PathType::Circle(o.width/2.0),
          M(_) => PathType::Circle(0.5)

        }
      },
      _ => panic!("Aperture is not selected")
    }
  }

  fn path_width(&self) -> f32 {
    let ap = self.selected_aperture.as_ref();
    use ApertureTemplatePrimitive::*;
    match ap {
      Some((_, ap)) => {
        match ap {
          C(c) => c.diameter,
          R(r) => r.width,
          O(r) => r.width,
          P(p) => p.outer_diameter,
          _ => panic!("Cannot calculate width from aperture")
        }
      },
      _ => panic!("Aperture is not selected")
    }
  }


  fn start_new_path(&mut self, coords: Option<Coordinates>) {
    let ap = self.selected_aperture.as_ref();
    match ap {
      Some((_, _)) => {
        let coords = coords.map(|c| self.construct_coords(c))
          .unwrap_or(Vec2::new(0.0, 0.0));
        let pt = self.path_type();
        let current_path = self.current_path.replace(
          RawPath::start_with(pt, coords)
        );

        match current_path {
          Some(path) => self.collected_paths.push(path.into()),
          None=> ()
        };
      },
      _ => panic!("Aperture is not selected")
    }
  }

  fn apply_format(&self, value: String) -> f32 {
    match &self.format {
      Some(FormatSpecification{x, ..}) => x.parse(value),
      _ => unreachable!()
    }
  }

  fn construct_coords(&mut self, coords: Coordinates) -> Vec2 {
    match & self.format {
      Some(FormatSpecification{x, ..}) => {
        Vec2::new(
          coords.x.map(|v| x.parse(v)).unwrap_or(0f32), 
          coords.y.map(|v| x.parse(v)).unwrap_or(0f32)
        )
      },
      None=> panic!("FormatSpecification is not in state on first place where it is needed")
    }
  }


  fn interpolation(&mut self, coords: Coordinates) {
    if self.current_path.is_none() {
      self.start_new_path(None);
    }

    let i = coords.i.clone().map(|i| self.apply_format(i));
    let j = coords.j.clone().map(|i| self.apply_format(i));
    let direction = self.circular_direction.clone().unwrap_or(CircularDirection::CW);
    let coords = self.construct_coords(coords);
    let path_element = match self.interpolation {
      Some(Interpolation::Linear) => RawPathElement::Linear(coords),
      Some(Interpolation::SingleQuadrant) => RawPathElement::SingleQuadrant(RawArc{to:coords, dir:direction, i, j}),
      Some(Interpolation::MultiQuadrant) => RawPathElement::MultiQuadrant(RawArc{to:coords, dir:direction, i, j}),
      None => panic!(format!("Unknown interpolation type"))
    };

    self.current_path.as_mut().map(|path| path.push(path_element));
  }


  pub fn get_units(&mut self) -> Unit {
    match self.unit.take() {
      Some(u) => u,
      None => Unit::Inches
    }
  }



  pub fn get_result(mut self) -> Vec<Path> {
    match self.current_path.take() {
      Some(path) => self.collected_paths.push(path.into()),
      None => ()
    };

    self.collected_paths.into_iter()
      .map(|p| to_stroke_around_path(p))
      .collect()
  }
}


