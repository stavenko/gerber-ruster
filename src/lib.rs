pub mod parser;
mod plotter;
mod svg;

pub use svg::SvgComposer;

#[cfg(test)]
mod test {
  use crate::parser;

  #[test]
  fn read_whole_file() {
    let contents = include_str!("../test_files/simple.gbr");
    let reader = parser::GerberReader::new(contents);
    let commands = reader.collect::<Vec<_>>();
    assert_eq!(commands.len(), 304);
  }
  #[test]
  fn read_whole_big_file() {
    let contents = include_str!("../test_files/hard_one.gbr");
    let reader = parser::GerberReader::new(contents);
    let commands = reader.collect::<Vec<_>>();
    assert_eq!(commands.len(), 5690);
  }
}

#[cfg(test)]
mod integration_test {
  use std::path::Path;
  use std::fs::File;
  use std::io::BufReader;
  use std::io::prelude::*;
  use crate::parser;
  use crate::plotter;
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

  static ROOT_DIR: &str = "./test-visual";
  static GERBER_FOLDER: &str = "gerber";
  static SVG_FOLDER: &str = "expected/";
  fn files_list() -> Vec<&'static str> { 
    vec! (
      /*
    "smokes/one-1",
    "smokes/one",
    "smokes/two",
    "smokes/three",
    "strokes/circle-tool-multi-segment",
    "strokes/circle-tool-single-segment",
    "strokes/circle-tool-zero-length",
    "strokes/rect-tool-multi-segment",
    "strokes/rect-tool-single-segment",
    "strokes/rect-tool-zero-length",
    "arc-strokes/single-quadrant-I-to-II",
    "arc-strokes/single-quadrant-II-to-III",
    "arc-strokes/single-quadrant-III-to-IV",
    "arc-strokes/single-quadrant-IV-to-I",

    "arc-strokes/multi-quadrant-I-to-II",
    "arc-strokes/multi-quadrant-I-to-III",
    "arc-strokes/multi-quadrant-I-to-IV",

    "arc-strokes/multi-quadrant-II-to-III",
    "arc-strokes/multi-quadrant-II-to-IV",
    "arc-strokes/multi-quadrant-II-to-I",

    "arc-strokes/multi-quadrant-III-to-IV",
    "arc-strokes/multi-quadrant-III-to-II",
    "arc-strokes/multi-quadrant-III-to-I",

    "arc-strokes/multi-quadrant-IV-to-II",
    "arc-strokes/multi-quadrant-IV-to-III",
    "arc-strokes/multi-quadrant-IV-to-I",
    "arc-strokes/zero-length",
    "arc-strokes/full-circle",

      "real-world/simple",
      "real-world/hard_one",
    "regions/region-with-arc-cut-in",
    "regions/region-with-arcs",
    "regions/region-with-cut-in-line",
    */
    "regions/region-with-lines",
    /*
    "regions/shitty-region-with-lines",
    */
    
    )
  }


  #[test]
  fn run_through() {
    let root = Path::new(ROOT_DIR);
    let grb_path = root.join(GERBER_FOLDER);

    for file in files_list().into_iter().map(String::from) {
      let grb = grb_path.join(format!("{}.gbr", file));
      match File::open(&grb) {
        Ok(grb_file) =>  { 
          let file_content ={ 
            let mut content = String::new();
            let mut reader = BufReader::new(grb_file);
            reader.read_to_string(&mut content).unwrap(); 
            content
          };
          let (result, unit) = {
            let mut parser = parser::GerberReader::new(&file_content);
            let mut plotter = plotter::Plotter::new();
            {
              for parse_result in &mut parser {
                match parse_result {
                  Ok(parser::Cmd::One(command)) => { plotter.consume(command); },
                  Ok(parser::Cmd::Many(commands)) => { for c in commands {plotter.consume(c);}; },
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
          let file_name_to_save = root.join(SVG_FOLDER).join(format!("{}-result.svg", file));
          println!("path to save {:?}", file_name_to_save);
          if !file_name_to_save.exists() {
            std::fs::create_dir_all(file_name_to_save.parent().unwrap()).unwrap();
          }
          File::create(file_name_to_save).map(move |mut f| f.write_all(result.as_bytes())).unwrap().unwrap();
        },
        Err(e) => {
          println!("Error: {:?} ", e);
        }
      }
    }
    assert_eq!(true, true);
  }
}
