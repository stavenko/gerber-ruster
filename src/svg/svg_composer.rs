use crate::plotter::{ Tree, Path, Region, AlgebraicPathElement };
use crate::parser::{ Unit, Polarity };
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

struct SvgPath {
  polarity: Polarity,
  elements: Vec<SvgElement>
}

pub struct SvgComposer {
  paths: Vec<SvgPath>,
  unit: Unit,
  bb: BoundingBox
}

impl SvgComposer {

  fn region_to_svg_paths(starting_polirity: Polarity, region: Tree<Path>) -> Vec<SvgPath> {
    let mut result: Vec<SvgPath> = Vec::new();
    let Path{ elements, ..} = &region.data;
    result.push(
      SvgPath {
        polarity: starting_polirity.clone(),
        elements: elements.iter()
          .map(|item| SvgElement::new(item.algebraic()))
          .collect()
      });

    let others = region.into_iter().map(|child| Self::region_to_svg_paths(starting_polirity.switch(), *child)).flatten().collect::<Vec<_>>();
    result.extend(others);
    result
      

    /*
    for child in region.into_iter() {
      result.push(
        SvgPath {
          polarity: starting_polirity.clone(),
          elements: elements.iter()
            .map(|item| SvgElement::new(item.algebraic()))
            .collect()
        });

      let mut new_paths = Self::region_to_svg_paths(starting_polirity.switch(), *child);
      result.append(&mut new_paths);
    }
    result

    */

    // let mut polarity = region.starting_polirity;


    
    /*
    region.paths.into_iter().map(|path| {
      let svg_path: Vec<SvgElement> = path.elements.into_iter().map(|item| {
        SvgElement::new(item.algebraic())
      }).collect();

      SvgPath {
        polarity: polarity.switch(),
        elements: svg_path
      }
    }).collect()
    */
  }
  pub fn new (regions: Vec<Region>, unit: Unit) -> Self {
    println!("REGs: {}", regions.len());
    let paths = regions.into_iter()
      .map(|r|  {
        Self::region_to_svg_paths(r.starting_polirity, r.paths)
      })
      .flatten()
      .collect::<Vec<SvgPath>>();

    /*
    let paths = paths.into_iter().map(|path| {
      let svg_path: SvgPath = path.elements.into_iter().map(|item| {
        SvgElement::new(item.algebraic())
      }).collect();
      svg_path
    }).collect();
    */

    SvgComposer{ bb: BoundingBox::default(), paths , unit }
  }

  fn calculate_bounding_box(&mut self) {
    for path in &self.paths {
      let mut bb = BoundingBox::default();
      for item in &path.elements {
        bb += item.0.get_bounding_box();
      }
      self.bb += bb;
    }
  }

  pub fn compose(mut self)->String {
    println!("----------------------compose svg-------------------");

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
    println!("----------------------ser path-------------------");
    let one_unit = self.unit.to_points(1.0);
    let wh = self.bb.max - self.bb.min;
    let left = -self.unit.to_points(self.bb.min.x);
    let top = self.unit.to_points(wh.y + self.bb.min.y);
    let matrix = format!("matrix({},0,0,-{}, {}, {})", one_unit, one_unit, left, top);
    let mut items: Vec<String> = svg_path.elements.iter().map(|p| p.0.serialize()).collect();
    items.insert(0, svg_path.elements.first().unwrap().0.initial());

    let color = match svg_path.polarity {
      Polarity::Dark => "black",
      Polarity::Clear => "yellow"
    };

    format!("<path d=\"{}\" fill=\"{}\" stroke=\"red\" stroke-width=\"0.02\" transform=\"{}\"/>", items.join(" "), color, matrix)
  }
}

