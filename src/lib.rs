pub mod parser;

#[cfg(test)]
mod test {
  use crate::parser;

  #[test]
  fn read_whole_file() {
    let contents = include_str!("../test_files/simple.gbr");
    println!("{}", contents);
    let reader = parser::GerberReader::new(contents);
    let commands = reader.collect::<Vec<_>>();
    assert_eq!(commands.len(), 303);
  }
  #[test]
  fn read_whole_big_file() {
    let contents = include_str!("../test_files/hard_one.gbr");
    println!("{}", contents);
    let reader = parser::GerberReader::new(contents);
    let commands = reader.collect::<Vec<_>>();
    assert_eq!(commands.len(), 5689);
  }
}
