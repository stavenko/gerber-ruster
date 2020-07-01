#[derive(Debug, PartialEq, Clone)]
pub enum CircularDirection {
  CW,
  CCW
}

impl CircularDirection {
  pub fn reverse(self) -> CircularDirection {
    match self {
      CircularDirection::CW => CircularDirection::CCW,
      CircularDirection::CCW => CircularDirection::CW,
    }

  }
}

