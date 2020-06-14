use super::commands::*;
use nom::{
  Err::{ Incomplete, Failure, Error },
  branch::{ alt },
};


pub struct GerberReader<'a> {
  pointer: &'a str
}
impl<'a> GerberReader<'a> {
  pub fn new(input: &'a str) -> Self {
    GerberReader {
      pointer: input
    }
  }
}
type ParseResult<I> = Result<GerberCommand, GerberError<I>>;

impl<'a> Iterator for GerberReader<'a> {
  type Item = ParseResult<&'a str>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pointer == "" {
      None
    } else {
      match alt((simple_command, extended_command))(self.pointer) {
        Ok((rest, command)) => {
          if command == GerberCommand::Stop {
            None
          } else {
            self.pointer = rest;
            Some(Ok(command))
          }
        },
        Err(Error(err)) => {
          println!("emit error {:?}", err);
          Some(Err(err))
        },

        Err(Incomplete(err)) => {
          println!("emit error 1 {:?}", err);
          Some(Err(GerberError::Incomplete))
        },

        Err(Failure(err)) => {
          println!("emit error 2 {:?}", err);
          Some(Err(GerberError::Faulure))
        }
      }
    }
  }
}

#[test]
fn read_several_aperture_extended_commands() {
  let cmds = r"%AMOC8*
5,1,8,0,0,1.08239X$1,22.5*%
G01*

%ADD10R,0.800000X-1.200000*%
%ADD11C,0.152400*%
";

  let iter = GerberReader::new(cmds);
  let result = iter.map(|x| x.unwrap()).collect::<Vec<GerberCommand>>();

  assert_eq!(result[0], GerberCommand::ApertureMacro(String::from(r"OC8*
5,1,8,0,0,1.08239X$1,22.5*")));
  assert_eq!(result[2], GerberCommand::ApertureDefinition(Aperture { name: String::from("10"), template: ApertureTemplatePrimitive::R(Rect { width: 0.8, height: -1.2, hole_diameter: None }) }));
  assert_eq!(result[3], GerberCommand::ApertureDefinition(Aperture { name: String::from("11"), template: ApertureTemplatePrimitive::C(Circle { diameter: 0.1524, hole_diameter: None }) }));
}

#[test]
fn read_several_commands() {
  let cmds = "G01*\nG02*\nG03*\n";
  let mut iter = GerberReader::new(cmds);
  let c1 = iter.next().unwrap().unwrap();
  let c2 = iter.next().unwrap().unwrap();
  let c3 = iter.next().unwrap().unwrap();
  assert_eq!(c1, GerberCommand::LinearInterpolation);
  assert_eq!(c2, GerberCommand::ClockWiseArc);
  assert_eq!(c3, GerberCommand::CounterClockWiseArc);
}
#[test]
fn read_several_extended_commands() {
  let cmds = "%MOMM*%\n%FSLAX21Y21*%\nG03*\n";

  let mut iter = GerberReader::new(cmds);
  let c1 = iter.next();
  let c2 = iter.next();
  let c3 = iter.next().unwrap().unwrap();
  let c1 = c1.unwrap().unwrap();
  let c2 = c2.unwrap().unwrap();
  assert_eq!(c1, GerberCommand::Unit(Unit::Millimeters));
  assert_eq!(c2, GerberCommand::FormatSpecification(FormatSpecification{
    x: NumberSpec{
      integer: 2, rational: 1
    },
    y: NumberSpec{
      integer: 2, rational: 1
    }
  }));
  assert_eq!(c3, GerberCommand::CounterClockWiseArc);
}

#[test]
fn read_comment() {
  let comment = "G04 EAGLE Gerber RS-274X export*\n";
  let mut iter = GerberReader::new(comment);
  let r = iter.next();
  let r = r.unwrap().unwrap();
  assert_eq!(r, GerberCommand::Comment(String::from("EAGLE Gerber RS-274X export")));
}

#[test]
fn read_operation() {
  let cmds = "X562500Y558800D02*\nX546100D01*\nY546100D03*";
  let mut iter = GerberReader::new(cmds);
  let c1 = iter.next();
  let c2 = iter.next();
  let c3 = iter.next().unwrap().unwrap();
  let c1 = c1.unwrap().unwrap();
  let c2 = c2.unwrap().unwrap();
  assert_eq!(c1, GerberCommand::Operation(
    Operation { 
      op_type: OperationType::Move, 
      coords: [Some(562500), Some(558800)] 
  }));
  assert_eq!(c2, GerberCommand::Operation(Operation { 
    op_type: OperationType::Interpolation, 
    coords: [Some(546100), None] 
  }));
  assert_eq!(c3, GerberCommand::Operation( Operation { 
    op_type: OperationType::Flash, coords: [None, Some(546100)] 
  }));
}
