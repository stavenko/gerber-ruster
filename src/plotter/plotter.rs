extern crate nalgebra as na;
use crate::parser::*;
use std::collections::HashMap;
use std::convert::From;
use na::{ Vector2};
use super::line::Line;
use super::arc::Arc;
use super::path::{ PathType, Path };
use super::circular_direction::*;
use super::region::Region;

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
  SingleQuadrant{ x: Option<f32>, y: Option<f32>, i: Option<f32>, j: Option<f32>, dir: CircularDirection},
  MultiQuadrant{ x: Option<f32>, y: Option<f32>, i: Option<f32>, j: Option<f32>, dir: CircularDirection},
  Linear{x: Option<f32>, y: Option<f32>},
  Start{x: Option<f32>, y: Option<f32>}
}



#[derive(Debug, PartialEq)]
struct RawPath {
  tp: PathType,
  elements: Vec<RawPathElement>,
}


impl From<RawPath> for Path {
  fn from(path: RawPath) -> Self {
    use RawPathElement::*;
    let mut path_elements: Path = Path::new(path.tp);
    let mut start_point: Option<Vec2> = None;
    for element in path.elements.into_iter() {
      match element {
        Start{x, y} => {start_point.replace(Vec2::new(x.unwrap_or(0.0), y.unwrap_or(0.0)));},
        Linear{ x, y } => {
          match start_point { 
            Some(from) => {
              let v = Vec2::new(x.unwrap_or(from.x), y.unwrap_or(from.y));
              path_elements.add(Box::new(Line::new(v, from)));
              start_point.replace(v);
            },
            None => unreachable!("Must be start_point")
          };
        },
        SingleQuadrant{x, y, i, j, dir} => {
          match start_point { 
            Some(from) => {
              let to = Vec2::new(x.unwrap_or(from.x), y.unwrap_or(from.y));
              start_point.replace(to);
              path_elements.add(Box::new(Arc::new(
              to,
              from,
              i, 
              j,
              true,
              dir
            )))
            },
            None => unreachable!("Must be start_point")
          };
        },
        MultiQuadrant{ x, y, i, j, dir } => {
          match start_point { 
            Some(from) => {
              let to = Vec2::new(x.unwrap_or(from.x), y.unwrap_or(from.y));
              start_point.replace(to);

              path_elements.add(Box::new(Arc::new(
              to,
              from,
              i, 
              j,
              false,
              dir
            )))
            },
            None => unreachable!("Must be start_point")
          };
        }
      }
    };
    path_elements
  }
}

enum SelectedTool {
  Aperture{ key: String, template: ApertureTemplatePrimitive },
  Region
}

impl RawPath {
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
  selected_aperture: Option<SelectedTool>,
  collected_paths: Vec<Path>,
  current_path: Option<RawPath>,
  coords_accumulator: HashMap<Coordinate, f32>
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
      current_path: None,
      coords_accumulator: HashMap::new()

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

  fn start_contour(&mut self) {
    let last_ap = self.selected_aperture.replace(SelectedTool::Region);
    if let Some(SelectedTool::Aperture{ key, template }) = last_ap {
      self.tools.insert(key, template);
    }
    self.terminate_path();
  }

  fn finish_contour(&mut self) {
    let last_ap = self.selected_aperture.take();
    if let Some(SelectedTool::Aperture{ key, template }) = last_ap {
      self.tools.insert(key, template);
    }
    self.terminate_path();
  }

  fn apply_aperture(&mut self, name: String) {
    let aperture = self.tools.remove_entry(&name);
    match aperture {
      Some(ap) => {
        self.terminate_path();
        let prev_aperture = self.selected_aperture.replace(SelectedTool::Aperture{key: ap.0, template: ap.1 });
        if let Some(SelectedTool::Aperture{key, template}) = prev_aperture {
          self.tools.insert(key, template);
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

  fn set_coordinate(&mut self, coord: Coordinate, value: String) {
    let value = if let Some(FormatSpecification{x, ..})  = &self.format  {
      x.parse(value)
    } else {
      panic!("Cannot parse coordinate - incorrect state - no coordinate format specified");
    };
    self.coords_accumulator.insert(coord, value);
  }


  pub fn consume(&mut self, command: GerberCommand) {
    println!("cmd: {:?}", command);
    match command {
      GerberCommand::StartContourMode => self.start_contour(),
      GerberCommand::FinishConrourMode => self.finish_contour(),
      GerberCommand::Unit(u) => self.set_unit(u),
      GerberCommand::Coordinate{coord, value} => self.set_coordinate(coord, value),
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

  fn operation(&mut self, op: OperationType) {
    match op {
      OperationType::Interpolation => self.interpolation(),
      OperationType::Move => self.start_new_path(),
      _ => ()
    }
  }

  fn path_type(&self) -> PathType {
    let ap = self.selected_aperture.as_ref();
    use ApertureTemplatePrimitive::*;
    match ap {
      Some(SelectedTool::Region) => {
        PathType::Stroke
      },
      Some(SelectedTool::Aperture{template, ..}) => {
        match template {
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

  fn start_new_path(&mut self) {
    let x = self.coords_accumulator.remove(&Coordinate::X);
    let y = self.coords_accumulator.remove(&Coordinate::Y);

    println!("start path {:?}, {:?}", x, y);
    let ap = self.selected_aperture.as_ref();
    match ap {
      Some(_) => {
        let pt = self.path_type();
        let current_path = self.current_path.replace(
          RawPath::start(pt)
        );
        if let Some(path) = &mut self.current_path {
          path.push(RawPathElement::Start{x, y})
        }

        if let Some(path) =  current_path {
          self.collected_paths.push(path.into());
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

  fn interpolation(&mut self) {
    let i = self.coords_accumulator.remove(&Coordinate::I);
    let j = self.coords_accumulator.remove(&Coordinate::J);
    let x = self.coords_accumulator.remove(&Coordinate::X);
    let y = self.coords_accumulator.remove(&Coordinate::Y);

    if self.current_path.is_none() {
      self.start_new_path();
    }

    let dir= self.circular_direction.clone().unwrap_or(CircularDirection::CW);
    let path_element = match self.interpolation {
      Some(Interpolation::Linear) => RawPathElement::Linear{x, y},
      Some(Interpolation::SingleQuadrant) => RawPathElement::SingleQuadrant{x, y, i, j, dir},
      Some(Interpolation::MultiQuadrant) => RawPathElement::MultiQuadrant{x, y, i, j, dir},
      None => panic!("Unknown interpolation type")
    };

    if let Some(path) = self.current_path.as_mut() {
      path.push(path_element);
    }
  }


  pub fn get_units(&mut self) -> Unit {
    match self.unit.take() {
      Some(u) => u,
      None => Unit::Inches
    }
  }


  fn terminate_path(&mut self) {
    if let Some(path) = self.current_path.take() {
      if !path.elements.is_empty() {
        self.collected_paths.push(path.into());
      }
    };
  }

  pub fn get_result(mut self) -> Vec<Region> {
    self.terminate_path();

    self.collected_paths.into_iter()
      .map(|p| Region::from_raw_region(p))
      .flatten()
      .collect()
  }
}


