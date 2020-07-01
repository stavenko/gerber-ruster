extern crate nalgebra as na;
use na::{  Vector2 };

type Vec2 = Vector2<f32>;

#[derive(Debug, PartialEq)]
pub struct Ray {
  origin: Vec2,
  dir: Vec2
}

impl Ray {
  pub fn new(origin: Vec2, dir: Vec2) -> Self {
    Ray {
      origin, dir
    }
  }

  pub fn intersects(&self, other: &Ray) -> Vec2 {
    let distance = other.origin - self.origin;
    let basis = self.dir.normalize();
    let projection = other.dir.dot(&basis);
    let distance_projection = distance.dot(&basis);

    let ortho_projection = other.dir - projection * basis;
    let ortho_distance_projection = distance - distance_projection * basis;

    let sign = -ortho_projection.dot(&ortho_distance_projection).signum();
    let t = sign * ortho_distance_projection.magnitude() / ortho_projection.magnitude();

    other.origin + other.dir * t
  }
}



#[test]
fn ray_intersect_neg() {
  let o = Ray::new(
    Vec2::new(0.0, 0.0), 
    Vec2::new(1.0, 0.0)
  );
  let i = Ray::new(
    Vec2::new(0.5, -1.0), 
    Vec2::new(0.0, -1.0)
  );
  let n = o.intersects(&i);
  assert_eq!(n, Vec2::new(0.5, 0.0));
}

#[test]
fn ray_intersect() {
  let o = Ray::new(
    Vec2::new(0.0, 0.0), 
    Vec2::new(1.0, 0.0)
  );
  let i = Ray::new(
    Vec2::new(0.5, -1.0), 
    Vec2::new(0.0, 1.0)
  );
  let n = o.intersects(&i);
  assert_eq!(n, Vec2::new(0.5, 0.0));
}

fn normalize() {
  let t = Vec2::new(0.0, 500.0);
  let n = t.normalize();
  assert_eq!(n, Vec2::new(0.0, 1.0));
}
