use crate::plotter::{ Path, AlgebraicPathElement };
use crate::parser::{ Unit };
use super::bounding_box::*;
use super::serializable::*;

struct SvgElement(Box<dyn Serializable>);

impl SvgElement {
  fn new(path_element: AlgebraicPathElement) -> Self {
    SvgElement(match path_element {
      AlgebraicPathElement::Line(l) => Box::new(l),
      AlgebraicPathElement::Arc(a) => Box::new(a),
    })
  }
}

type SvgPath = Vec<SvgElement>;

pub struct SvgComposer {
  paths: Vec<SvgPath>,
  unit: Unit,
  bb: BoundingBox
}

impl SvgComposer {
  pub fn new (paths: Vec<Path>, unit: Unit) -> Self {
    let paths = paths.into_iter().map(|path| {
      let svg_path: SvgPath = path.elements.into_iter().map(|item| {
        SvgElement::new(item.algebraic())
      }).collect();
      svg_path
    }).collect();

    SvgComposer{ bb: BoundingBox::default(), paths , unit }
  }

  fn calculate_bounding_box(&mut self) {
    for path in &self.paths {
      let mut bb = BoundingBox::default();
      for item in path {
        bb += item.0.get_bounding_box();
      }
      self.bb += bb;
    }
  }

  pub fn compose(mut self)->String {

    self.calculate_bounding_box();
    let paths: Vec<String> = self.paths.iter().map(|path| {
      self.serialize(path)
    }).collect();

    let wh = self.bb.max - self.bb.min;
    // let one_unit = self.unit.to_points(1.0);

    // let left = -self.unit.to_points(self.bb.min.x);
    // let top = self.unit.to_points(wh.y - self.bb.min.y);
    let w = self.unit.to_points(wh.x).ceil() as i32;
    let h = self.unit.to_points(wh.y).ceil() as i32;

    format!(
        r#"<svg version="2.0" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}pt" height="{}pt" viewBox="0 0 {} {}"> 
        {} 
        </svg>"#, 
        w, h, w, h,
        paths.join("\n"))
    
  }


  fn serialize(&self, svg_path: &SvgPath) -> String {
    let one_unit = self.unit.to_points(1.0);
    let wh = self.bb.max - self.bb.min;
    let left = -self.unit.to_points(self.bb.min.x);
    let top = self.unit.to_points(wh.y + self.bb.min.y);
    let matrix = format!("matrix({},0,0,-{}, {}, {})", one_unit, one_unit, left, top);
    let mut items: Vec<String> = svg_path.iter().map(|p| p.0.serialize()).collect();
    items.insert(0, svg_path.first().unwrap().0.initial());


    format!("<path d=\"{}\" fill=\"black\" stroke=\"red\" stroke-width=\"0.02\" transform=\"{}\"/>", items.join(" "), matrix)
  }
}

