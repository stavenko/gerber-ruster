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



type ParseResult<I> = Result<Cmd, GerberError<I>>;

impl<'a> Iterator for GerberReader<'a> {
  type Item = ParseResult<&'a str>;
  fn next<'b>(&mut self) -> Option<Self::Item> {
    if self.pointer == "" {
      None
    } else {
      match alt((simple_command_block, extended_command))(self.pointer) {
        Ok((rest, command)) => {
          self.pointer = rest;
          Some(Ok(command))
        },
        Err(Error(err)) => {
          Some(Err(err))
        },

        Err(Incomplete(_err)) => {
          Some(Err(GerberError::Incomplete))
        },

        Err(Failure(_err)) => {
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
  let result = iter.map(|x| x.unwrap()).collect::<Vec<Cmd>>();

  assert_eq!(result[0], Cmd::One(GerberCommand::ApertureMacro(String::from(r"OC8*
5,1,8,0,0,1.08239X$1,22.5*"))));
  assert_eq!(result[2], Cmd::One(GerberCommand::ApertureDefinition(Aperture { 
    name: String::from("10"), 
    template: ApertureTemplatePrimitive::R(Rect { 
      width: 0.8, 
      height: -1.2, 
      hole_diameter: None }) })));
  assert_eq!(result[3], Cmd::One(GerberCommand::ApertureDefinition(Aperture { name: String::from("11"), template: ApertureTemplatePrimitive::C(Circle { diameter: 0.1524, hole_diameter: None }) })));
}


#[test]
fn read_comment() {
  let comment = "G04 EAGLE Gerber RS-274X export*\n";
  let mut iter = GerberReader::new(comment);
  let r = iter.next();
  let r = r.unwrap().unwrap();
  if let Cmd::Many(v) = r {
    assert_eq!(v[0], GerberCommand::Comment(String::from("EAGLE Gerber RS-274X export")));
  } else {
    panic!("wrong");
  }
  
}

#[test]
fn read_operation() {
  let cmds = "X02Y01D03*";
  let mut iter = GerberReader::new(cmds);
  let c = iter.next().unwrap().unwrap();
  if let Cmd::Many(v) = c {
    assert_eq!(v[0], GerberCommand::Coordinate{coord: Coordinate::X, value: "02".into()});
    assert_eq!(v[1], GerberCommand::Coordinate { coord: Coordinate::Y, value: "01".into() });
    assert_eq!(v[2], GerberCommand::Operation(OperationType::Flash));
  } else {
    panic!("wrong");
  }
}
