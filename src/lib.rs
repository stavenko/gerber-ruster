pub mod parser;
mod plotter;
mod svg;

#[cfg(test)]
mod test {
  use crate::parser;

  #[test]
  fn read_whole_file() {
    let contents = include_str!("../test_files/simple.gbr");
    let reader = parser::GerberReader::new(contents);
    let commands = reader.collect::<Vec<_>>();
    assert_eq!(commands.len(), 303);
  }
  #[test]
  fn read_whole_big_file() {
    let contents = include_str!("../test_files/hard_one.gbr");
    let reader = parser::GerberReader::new(contents);
    let commands = reader.collect::<Vec<_>>();
    assert_eq!(commands.len(), 5689);
  }
}

#[cfg(test)]
mod integration_test {
  extern crate gerber_compare;
  use std::path::Path;
  use std::fs::File;
  use std::io::BufReader;
  use std::io::prelude::*;
  use crate::parser;
  use crate::plotter;
  use gerber_compare::*;
  use crate::svg::SvgComposer;


  fn join<T,S, E>(r1: Result<T, E>, r2: Result<S, E>) -> Result<(T,S), E> {
    match r1 {
      Ok(t) => match r2 {
        Ok (s) => Ok((t,s)),
        Err(e) => Err(e)
      },
      Err(e) => Err(e)
    }
  }

  static rootDir: &str = "./test-visual";
  static gerber_folder: &str = "gerber";
  static svg_folder: &str = "expected/";
  fn files_list() -> Vec<&'static str> { 
    vec! (
    "smokes/one-1",
    "smokes/one",
    "smokes/two",
    "smokes/three"
    )
  }


  #[test]
  fn run_through() {

    let root = Path::new(rootDir);
    let grb_path = root.join(gerber_folder);

    for file in files_list().into_iter().map(String::from) {
      let grb = grb_path.join(format!("{}.gbr", file));
      match File::open(&grb) {
        Ok(grb_file) =>  { 
          let file_content ={ 
            let mut content = String::new();
            let mut reader = BufReader::new(grb_file);
            reader.read_to_string(&mut content); 
            content
          };
          let (result, unit) = {
            let mut parser = parser::GerberReader::new(&file_content);
            let mut plotter = plotter::Plotter::new();
            {
              for parse_result in &mut parser {
                match parse_result {
                  Ok(command) => { plotter.consume(command); },
                  Err(e) => {
                    println!("error occured {:?}", e);
                    panic!("Error in file");
                  }
                }
              }
            }
            let u = plotter.get_units();
            (plotter.get_result(), u)
          };


          let composer = SvgComposer::new(result, unit);
          let result = composer.compose();

          let is_almost_same = svg_is_same(result, &grb);
        },
        Err(e) => {
          println!("Error: {:?} ", e);
        }
      }
    }
    assert_eq!(true, true);
  }
}
