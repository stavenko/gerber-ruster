extern crate nom;

use nom::{
  Err::{ Error }, IResult,
  bytes::complete::{ take_while, take, tag, take_till },
  number::complete::{ float },
  combinator::{ iterator },
  character::{
    complete::{ char, one_of },
  },
  branch::{ alt },
  sequence::{ separated_pair, pair, preceded, delimited, terminated },
  error::{ ErrorKind, ParseError },
};

#[derive(Debug, PartialEq)]
pub enum Unit{
  Millimeters,
  Inches
}

#[derive(Debug, PartialEq)]
pub struct NumberSpec {
  pub integer: u8,
  pub rational: u8
}

type Coordinates = [Option<i32>; 2];

#[derive(Debug, PartialEq)]
pub struct FormatSpecification {
  pub x: NumberSpec,
  pub y: NumberSpec

}

#[derive(Debug, PartialEq)]
pub enum OperationType {
  Move,
  Interpolation,
  Flash
}

#[derive(Debug, PartialEq)]
pub struct Operation {
  pub op_type: OperationType,
  pub coords: Coordinates
}

#[derive(Debug, PartialEq)]
pub struct Circle {
  pub diameter: f32,
  pub hole_diameter: Option<f32>
}

#[derive(Debug, PartialEq)]
pub struct Rect {
  pub width: f32,
  pub height: f32,
  pub hole_diameter: Option<f32>
}

#[derive(Debug, PartialEq)]
pub struct Polygon {
  pub outer_diameter: f32,
  pub number_of_vertices: i32,
  pub rotation: f32,
  pub hole_diameter: Option<f32>
}

#[derive(Debug, PartialEq)]
pub enum ApertureTemplatePrimitive {
  C(Circle),
  R(Rect),
  O(Rect),
  P(Polygon),
  M(String)

}

#[derive(Debug, PartialEq)]
pub struct Aperture {
  pub name: String,
  pub template: ApertureTemplatePrimitive
}

#[derive(Debug, PartialEq)]
pub enum Polarity {
  Dark,
  Clear
}

#[derive(Debug, PartialEq)]
pub enum ImagePolarity {
  Positive,
  Negative
}

#[derive(Debug, PartialEq)]
pub enum GerberCommand {
  Stop,
  Operation(Operation),
  ApertureMacro(String), // just put it's contents in there for now
  ApertureDefinition(Aperture),
  Unit(Unit),
  FormatSpecification(FormatSpecification),
  Comment(String),
  LinearInterpolation,
  ClockWiseArc,
  CounterClockWiseArc,
  SingleQuadrantInterpolation,
  MultiQuadrantInterpolation,
  StartContourMode,
  FinishConrourMode,
  ApplyAperture(String),
  LevelPolarity(Polarity),
  ImagePolarity(ImagePolarity),
  ImageName(String)

}

#[derive(Debug, PartialEq)]
pub enum GerberError<I>{
  IncorrectOpCode(u8),
  IncorrectGCode,
  Incomplete,
  UnexpectedUnit,
  UnexpectedPolarity(String),
  Faulure,
  Nom(I, ErrorKind)
}



impl<I> ParseError<I> for GerberError<I> {
  fn from_error_kind(input: I, kind: ErrorKind) -> Self {
    GerberError::Nom(input, kind)
  }

  fn append(_: I, _: ErrorKind, other: Self) -> Self {
    other
  }
}

fn spaces(i: &str) -> IResult<&str, &str, GerberError<&str>> {
  let chars = " \n\t";
  take_while(move |c| chars.contains(c))(i)
}

fn str_until_the_end_of_command(i: & str) -> IResult<& str, & str, GerberError<&str>> {
  take_till(move |c| c == '*')(i)
}

fn comment(i: & str) -> IResult<& str, GerberCommand, GerberError<&str>> {
  let (i, _) = tag("G04 ")(i)?;
  let (rest, comment) = str_until_the_end_of_command(i)?;
  Ok((rest, GerberCommand::Comment(String::from(comment))))
}

fn g_command(i: & str) -> IResult<& str, GerberCommand, GerberError<&str>> {
  let (i, _) = tag("G")(i)?;
  let (rest, number) = take(2usize)(i)?;
  match number {
    "01" => Ok((rest, GerberCommand::LinearInterpolation)),
    "02" => Ok((rest, GerberCommand::ClockWiseArc)),
    "03" => Ok((rest, GerberCommand::CounterClockWiseArc)),
    "74" => Ok((rest, GerberCommand::SingleQuadrantInterpolation)),
    "75" => Ok((rest, GerberCommand::MultiQuadrantInterpolation)),
    "36" => Ok((rest, GerberCommand::StartContourMode)),
    "37" => Ok((rest, GerberCommand::FinishConrourMode)),
    _ => Err(Error(GerberError::IncorrectGCode))
  }
}
fn stop_command(i: & str) -> IResult<& str, GerberCommand, GerberError<&str>> {
  let (rest, _) = tag("M02")(i)?;
  Ok((rest, GerberCommand::Stop))
}

fn d_command(i: & str) -> IResult<& str, GerberCommand, GerberError<&str>> {
  let (i, _) = tag("D")(i)?;
  let (rest, number) = take(2usize)(i)?;
  match number {
    s => Ok((rest, GerberCommand::ApplyAperture(String::from(s))))
  }
}

pub fn simple_command(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  terminated(
    alt(( 
      g_command, 
      d_command,
      comment,
      operation,
      stop_command
    )), 
    delimited(spaces, char('*'), spaces)
  )
  (i)
}
pub fn extended_command(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  terminated(
    alt(( 
      unit_command,
      format_specification,
      aperture_macro,
      aperture_definition,
      level_polarity, image_polarity, image_name
    )), 
    spaces
  )
  (i)
}

fn unit_command(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let (rest, unit) = delimited(
    char('%'), 
    preceded(tag("MO"), str_until_the_end_of_command), 
    tag("*%")
  )(i)?;
  match unit {
    "MM" => Ok((rest, GerberCommand::Unit(Unit::Millimeters))),
    "IN" => Ok((rest, GerberCommand::Unit(Unit::Inches))),
    _ => Err(Error(GerberError::UnexpectedUnit))
  }
}

fn level_polarity(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let (rest, unit) = delimited(
    char('%'), 
    preceded(tag("LP"), str_until_the_end_of_command), 
    tag("*%")
  )(i)?;
  match unit {
    "D" => Ok((rest, GerberCommand::LevelPolarity(Polarity::Dark))),
    "C" => Ok((rest, GerberCommand::LevelPolarity(Polarity::Clear))),
    s => Err(Error(GerberError::UnexpectedPolarity(String::from(s))))
  }
}

fn image_name(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let (rest, name) = delimited(
    char('%'), 
    preceded(tag("IN"), str_until_the_end_of_command), 
    tag("*%")
  )(i)?;

  Ok((rest, GerberCommand::ImageName(String::from(name))))
}
fn image_polarity(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let (rest, unit) = delimited(
    char('%'), 
    preceded(tag("IP"), str_until_the_end_of_command), 
    tag("*%")
  )(i)?;
  match unit {
    "POS" => Ok((rest, GerberCommand::ImagePolarity(ImagePolarity::Positive))),
    "NEG" => Ok((rest, GerberCommand::ImagePolarity(ImagePolarity::Negative))),
    s => Err(Error(GerberError::UnexpectedPolarity(String::from(s))))
  }
}
fn is_digit(c: char) -> bool {
  c.is_digit(10)
}

fn op_code(i: &str) -> IResult<&str, OperationType, GerberError<&str>> {
  let (rest, op) = preceded(tag("D"), take_while(is_digit))(i)?;
  let op = String::from(op).parse::<u8>().unwrap();
  match op {
    1 => Ok((rest, OperationType::Interpolation)),
    2 => Ok((rest, OperationType::Move)),
    3 => Ok((rest, OperationType::Flash)),
    x => Err(Error(GerberError::IncorrectOpCode(x)))
  }
}
fn coordinate_data(i: &str) -> IResult<&str, Coordinates, GerberError<&str>> {
  let mut iter = iterator(i, pair(one_of("XY"), take_while(is_digit)));
  let mut coords: Coordinates = [None, None];
  for (coord, digit) in iter.collect::<Vec<_>>() {
    match coord {
      'X' => {coords[0] = Some(String::from(digit).parse().unwrap());},
      'Y' => {coords[1] = Some(String::from(digit).parse().unwrap());}
      _ => {}
    }
  }

  let (rest, _) = iter.finish()?;



  /*
  let (rest, (x, y)) = pair(
    preceded(tag("X"), take_while(is_digit)),
    preceded(tag("Y"), take_while(is_digit))
  )(i)?;
  */

  // let x = String::from(x).parse::<i32>().unwrap();
  // let y = String::from(y).parse::<i32>().unwrap();

  Ok((rest, coords))
}


fn operation(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let(rest, (coords, op_type)) = pair(coordinate_data, op_code)(i)?;

  Ok((rest, GerberCommand::Operation(Operation {
    op_type,
    coords
  })))

}

fn coordinate_spec(i: &str) -> IResult<&str, [NumberSpec; 2], GerberError<&str>> {
  let (rest, ((x_int, x_ratio), (y_int, y_ratio))) = pair(
    preceded(tag("X"), pair(take(1usize), take(1usize))),
    preceded(tag("Y"), pair(take(1usize), take(1usize)))
  )(i)?;

  Ok((rest, [
      NumberSpec{
        integer: String::from(x_int).parse::<u8>().unwrap(),
        rational: String::from(x_ratio).parse::<u8>().unwrap()
      }
      , 
      NumberSpec{
        integer: String::from(y_int).parse::<u8>().unwrap(),
        rational: String::from(y_ratio).parse::<u8>().unwrap()
      }
]))
}

fn aperture_template_definition(i: &str) -> IResult<&str, ApertureTemplatePrimitive, GerberError<&str>> {
  let (input, (name_and_type, definition)) = separated_pair(
    take_till(|c| c == ','),
    tag(","),
    take_till(|c| c == '*')
  )(i)?;
  let mut it = iterator(definition, terminated(float, tag("X")));
  let mut items = it
    .collect::<Vec<f32>>();
  let (leftover, _) = it.finish()?;
  items.push(String::from(leftover).parse().unwrap());

  // let (_, (name, and_type)) = pair(take_while(is_digit), nom::combinator::rest)(i)?;

  // let name = String::from(name).parse::<i32>().unwrap();
  let aperture = match name_and_type {
    "C" => ApertureTemplatePrimitive::C(Circle{
      diameter: items[0],
      hole_diameter: if items.len() == 2 {Some(items[1]) } else {None}
    }), 
    "R" => ApertureTemplatePrimitive::R(Rect{
      width: items[0],
      height: items[1],
      hole_diameter: if items.len() == 3 {Some(items[2]) } else {None}
    }), 
    "O" => ApertureTemplatePrimitive::O(Rect{
      width: items[0],
      height: items[1],
      hole_diameter: if items.len() == 3 {Some(items[2]) } else {None}
    }), 
    "P" => ApertureTemplatePrimitive::P(Polygon{
      outer_diameter: items[0],
      number_of_vertices: items[1] as i32,
      rotation: items[2],
      hole_diameter: if items.len() == 4 { Some(items[3]) } else {None}
    }), 
    s => ApertureTemplatePrimitive::M(String::from(s))
  };
  Ok((input, aperture))
}
fn aperture_macro(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let (rest, contents) = delimited(
    char('%'),
    preceded(tag("AM"), take_till(|c| c == '%')),
    char('%')
  )(i)?;

  Ok((rest, GerberCommand::ApertureMacro(String::from(contents))))
}

fn format_specification(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {
  let (rest, [x, y]) = delimited(
    char('%'), 
    preceded( tag("FSLA"), coordinate_spec),
    tag("*%")
  )(i)?;

  Ok((rest, GerberCommand::FormatSpecification(FormatSpecification{ x, y })))
}

fn aperture_definition(i: &str) -> IResult<&str, GerberCommand, GerberError<&str>> {

  let (rest, (name, aperture)) = delimited(
    char('%'),
    preceded(tag("ADD"), pair(take_while(is_digit), aperture_template_definition)),
    tag("*%")
    )(i)?;

  Ok((rest, GerberCommand::ApertureDefinition(Aperture{
    name: String::from(name),
    template: aperture
  })))
}



#[test]
fn test_format() {
  let command = "%FSLAX34Y34*%";
  let (_, r) = format_specification(command).unwrap();
  assert_eq!(r, GerberCommand::FormatSpecification(
    FormatSpecification{
      x: NumberSpec{ integer: 3, rational: 4},
      y: NumberSpec{ integer: 3, rational: 4}
    })
  );
}


#[test]
fn read_aperture_macro() {
  let cmds = r"%AMOC8*
5,1,8,0,0,1.08239X$1,22.5*%
G01*
%ADD10R,0.800000X1.200000*%
%ADD11C,0.152400*%
";

  let (_, cmd) = aperture_macro(cmds).unwrap();

  assert_eq!(cmd, GerberCommand::ApertureMacro(String::from(r"OC8*
5,1,8,0,0,1.08239X$1,22.5*")));
}
#[test]
fn read_aperture_def() {
  let cmds = r"%ADD10R,0.800000X1.200000*%
%ADD11C,0.152400*%
";

  let (_, cmd) = aperture_definition(cmds).unwrap();

  assert_eq!(cmd, GerberCommand::ApertureDefinition(Aperture { 
    name: String::from("10"), 
    template: ApertureTemplatePrimitive::R(Rect { 
      width: 0.8, 
      height: 1.2, 
      hole_diameter: None 
    }) 
  }));
}





#[test]
fn read_simple_command() {
  let comment = "G04 EAGLE Gerber RS-274X export*\n";
  let code = "G75*\n";
  let (_, comment) = simple_command(comment).unwrap();
  let (_, code) = simple_command(code).unwrap();
  assert_eq!(code, GerberCommand::MultiQuadrantInterpolation);
  assert_eq!(comment, GerberCommand::Comment(String::from("EAGLE Gerber RS-274X export")));
}


#[test]
fn read_g01() {
  let input = "G01*";
  let (_, item) = g_command(input).unwrap();
  assert_eq!(item, GerberCommand::LinearInterpolation);
}

#[test]
fn read_g02() {
  let input = "G02*";
  let (_, item) = g_command(input).unwrap();
  assert_eq!(item, GerberCommand::ClockWiseArc);
}

#[test]
fn read_g03() {
  let input = "G03*";
  let (_, item) = g_command(input).unwrap();
  assert_eq!(item, GerberCommand::CounterClockWiseArc);
}

#[test]
fn read_g74() {
  let input = "G74*";
  let (_, item) = g_command(input).unwrap();
  assert_eq!(item, GerberCommand::SingleQuadrantInterpolation);
}
#[test]
fn read_g75() {
  let input = "G75*";
  let (_, item) = g_command(input).unwrap();
  assert_eq!(item, GerberCommand::MultiQuadrantInterpolation);
}
#[test]
fn read_unit_command() {
  let input = "%MOMM*%";
  let (_, item) = unit_command(input).unwrap();
  assert_eq!(item, GerberCommand::Unit(Unit::Millimeters));
}

#[test]
fn read_incorrect() {
  let input = "G88*";
  let err = g_command(input).unwrap_err();
  match err {
    Error(e) => assert_eq!(e, GerberError::IncorrectGCode),
    _ => panic!("Unexpected error: {:?}", err),
  }
}

#[test]
fn read_aperture() {
  let input1 = "O,0.800000X1.200000*";
  let input2 = "O,0.800000X1.200000X0.5*";
  let (_, ap) = aperture_template_definition(input1).unwrap();
  let (_, ap1) = aperture_template_definition(input2).unwrap();
  assert_eq!(ap, ApertureTemplatePrimitive::O(Rect{
    width: 0.8,
    height: 1.2,
    hole_diameter: None
  }));
  assert_eq!(ap1, ApertureTemplatePrimitive::O(Rect{
    width: 0.8,
    height: 1.2,
    hole_diameter: Some(0.5)
  }));
}


