extern crate nalgebra as na;
use na::{  Rotation2, Vector2 };
use std::f32::consts::PI;
use super::circular_direction::*;

type Vec2 = Vector2<f32>;

#[derive(Debug, PartialEq, Clone)]
pub enum IntersectorEnum {
  Ray(Ray),
  Arc(Arc),
  Segment(Segment)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Segment {
  from: Vec2,
  to: Vec2
}

impl Segment{
  pub fn new(from: Vec2, to: Vec2) -> Self {
    Segment {
      from, to
    }
  }
  fn make_ray(&self) -> Ray {
    Ray::new(self.from, self.to - self.from)
  }

  fn spot(&self, s: f32) -> Vec2 {
    self.make_ray().spot(s)
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ray {
  pub origin: Vec2,
  pub dir: Vec2
}


#[derive(Debug, PartialEq, Clone)]
pub struct Arc {
  pub center: Vec2,
  pub from: Vec2,
  pub to: Vec2,
  pub direction: CircularDirection
}

pub trait Intersects {
  fn get_intersector(&self) -> IntersectorEnum;
}


#[derive(Debug, PartialEq, Clone)]
enum LinearIntersectResult {
  SameLines,
  ParallelLines,
  Parameters(f32, f32)
}

impl Ray {
  pub fn new(origin: Vec2, dir: Vec2) -> Self {
    Ray {
      origin, dir
    }
  }

  pub fn spot(&self, t: f32) -> Vec2 {
    self.origin + t * self.dir
  }
}

impl IntersectorEnum {
  pub fn intersects(&self, other: IntersectorEnum) -> Vec<Vec2> {
    use IntersectorEnum::*;
    match self {
      Ray(left_ray) => {
        match other {
          Ray(right_ray) => IntersectorEnum::intersects_ray_ray(&left_ray, &right_ray),
          Arc(right_arc) => IntersectorEnum::intersects_ray_arc(&left_ray, &right_arc),
          Segment(s) => IntersectorEnum::ray_segment_intersection( &left_ray, &s)
        }
      },
      Arc(left_arc) => {
        match other {
          Ray(right_ray) => IntersectorEnum::intersects_ray_arc(&right_ray, &left_arc),
          Arc(right_arc) => IntersectorEnum::intersects_arc_arc(&left_arc, &right_arc),
          Segment(s) => IntersectorEnum::intersects_segment_arc(&s, &left_arc)
        }
      },
      Segment(left_segment) => {
        match other {
          Ray(right_ray) => IntersectorEnum::ray_segment_intersection(&right_ray, &left_segment),
          Arc(right_arc) => IntersectorEnum::intersects_segment_arc(&left_segment, &right_arc),
          Segment(s) => IntersectorEnum::segment_segment_intersection(&s, &left_segment)
        }
      },
    }
  }

  fn ray_segment_intersection(ray: &Ray, segment: &Segment) -> Vec<Vec2>{
    match Self::linear_components(ray, &segment.make_ray()) {
      LinearIntersectResult::SameLines => Vec::new(),
      LinearIntersectResult::ParallelLines => Vec::new(),
      LinearIntersectResult::Parameters(s, t) => {
        // println!("ray segment {}, {}",  s, t);
        if s > 0.0 && t > 0.0 && t < 1.0 {
          vec!(ray.spot(s))
        } else {
          Vec::new()
        }
      }
    }
  }

  fn linear_components(ray_one: &Ray, ray_two: &Ray) -> LinearIntersectResult {
    let d = ray_two.origin - ray_one.origin;

    let dir_kross = Self::kross(ray_one.dir, ray_two.dir);
    let one_kross = Self::kross(d, ray_one.dir);

    if dir_kross.abs() <= f32::EPSILON {
      if one_kross.abs() < f32::EPSILON {
        LinearIntersectResult::SameLines
      } else {
        LinearIntersectResult::ParallelLines
      }
    } else {
      let two_kross = Self::kross(d, ray_two.dir);
      let s = two_kross / dir_kross;
      let t = one_kross / dir_kross;
      LinearIntersectResult::Parameters(s, t)
    }
  }

  fn segment_segment_intersection(main: &Segment, segment: &Segment) -> Vec<Vec2>{
    match Self::linear_components(&main.make_ray(), &segment.make_ray()) {
      LinearIntersectResult::SameLines => Vec::new(),
      LinearIntersectResult::ParallelLines => Vec::new(),
      LinearIntersectResult::Parameters(s, t) => {
        if s > 0.0 && s < 1.0 && t > 0.0 && t < 1.0 {
          vec!(main.spot(s))
        } else {
          Vec::new()
        }
      }
    }
  }

  fn intersects_ray_ray(left: &Ray, other: &Ray) -> Vec<Vec2> {
    match Self::linear_components(left, other) {
      LinearIntersectResult::SameLines => Vec::new(),
      LinearIntersectResult::ParallelLines => Vec::new(),
      LinearIntersectResult::Parameters(s, t) => {
        if s > 0.0 && t > 0.0 {
          vec!(left.spot(s))
        } else {
          Vec::new()
        }
      }
    }
  }

  fn ray_ray_find_t(main: &Ray, intersector: &Ray) -> f32 {
    let left = intersector;
    let other = main;

    let distance = other.origin - left.origin;
    let basis = left.dir.normalize(); 
    println!("basis: {} {}", basis.x, basis.y);
    let projection = other.dir.dot(&basis);
    let distance_projection = distance.dot(&basis);


    let ortho_projection = other.dir - projection * basis;
    let ortho_distance_projection = distance - distance_projection * basis;

    println!("projection: distance {}", distance);
    println!("projection: dir {}  pos {}", ortho_projection, ortho_distance_projection);

    let sign = -ortho_projection.dot(&ortho_distance_projection).signum();
    println!("sign {}", ortho_projection.dot(&ortho_distance_projection));
    sign * ortho_distance_projection.magnitude() / ortho_projection.magnitude()
  }


  fn get_angle(point: &Vec2, dir: &CircularDirection) -> f32 {
    let angle_raw = point.y.atan2(point.x);
    use CircularDirection::*;
    match dir {
      CCW => if angle_raw < 0.0 {
          angle_raw.abs()
        } else {
          2.0 * PI - angle_raw
        },

      CW => if angle_raw < 0.0 {
        2.0 * PI + angle_raw
        } else {
          angle_raw
        }
    }
  }

  fn kross(a: Vec2, b: Vec2) -> f32 {
    a.x * b.y - b.x * a.y
  }

  fn is_on_arc(arc: &Arc, point: &Vec2) -> bool {
    let kross = Self::kross(point - arc.from, arc.to - arc.from); 
    match arc.direction {
      CircularDirection::CCW => kross >= 0.0,
      CircularDirection::CW  => kross <= 0.0
    }
  }

  fn is_on_segment(segment: &Segment, point: &Vec2) -> bool {
    let related = point - segment.from;
    let dir = segment.to - segment.from;
    let angle = Rotation2::rotation_between(&dir, &related).angle();
    let projection = dir.dot(&related) / dir.magnitude();
    angle.abs() <= f32::EPSILON  && projection >= 0.0 && projection <= 1.0 
  }

   fn intersects_arc_arc(left: &Arc, other: &Arc) -> Vec<Vec2> {
    let u = other.center - left.center;
    let v = Vec2::new(u.y, -u.x);
    let r0 = (left.from - left.center).magnitude();
    let r1 = (other.from - other.center).magnitude();

    let s = 0.5 * ( 1.0 + (r0.powi(2) - r1.powi(2)) / u.dot(&u)); 
    let t_squared = r0.powi(2) / u.dot(&u) - s.powi(2);
    if t_squared >= 0.0 {

      let t = t_squared.abs().sqrt();
      let point1 = left.center + s * u + t * v;
      let point2 = left.center + s * u - t * v;
      vec![point1, point2].into_iter()
        .filter(|p| {
          IntersectorEnum::is_on_arc(left, &p) && IntersectorEnum::is_on_arc(other, &p)
        })
        .collect::<Vec<_>>()
    } else {
      Vec::new()
    }
  }

   fn intersects_segment_arc(segment: &Segment, arc: &Arc)->Vec<Vec2> {
     let ray = segment.make_ray();
     let mut unchecked_intersection = Self::ray_arc_both_points(&ray, arc);
     let mut result = Vec::new();
     while let Some(vec) = unchecked_intersection.pop() {
       if Self::is_on_arc(&arc, &vec) && Self::is_on_segment(&segment, &vec) {
         result.push(vec)
       }
     }
     result
   }

   fn ray_arc_both_points(ray: &Ray, arc: &Arc) -> Vec<Vec2> {
    let delta = ray.origin - arc.center;
    let radius = (arc.from - arc.center).magnitude();
    let dir = ray.dir;

    // square equation
    let a = dir.dot(&dir);
    let b = 2.0 * delta.dot(&dir);
    let c = delta.dot(&delta) - radius.powi(2);

    let descr = b.powi(2) - 4.0 * a * c;
    if descr < 0.0 {
      Vec::new()
    } else if descr <= f32::EPSILON {
      vec!(ray.origin + ray.dir * (-b / (2.0 * a)))
    } else {
      let t1 = (-b + descr.sqrt()) / (2.0 * a);
      let t2 = (-b - descr.sqrt()) / (2.0 * a);
      let (t1, t2) = { (t1.min(t2), t1.max(t2)) };
      vec!(ray.spot(t1), ray.spot(t2))
    }
   }

   fn intersects_ray_arc(left: &Ray, other: &Arc) -> Vec<Vec2> {
    let delta = left.origin - other.center;
    let radius = (other.from - other.center).magnitude();
    let dir = left.dir;

    // square equation
    let a = dir.dot(&dir);
    let b = 2.0 * delta.dot(&dir);
    let c = delta.dot(&delta) - radius.powi(2);
    let mut result = Vec::new();

    let descr = b.powi(2) - 4.0 * a * c;
    println!("descr {}", descr);
    if descr.abs() <= f32::EPSILON {
      result.push(left.origin + left.dir * (-b / (2.0 * a)))
    } else {
      let t1 = (-b + descr.sqrt()) / (2.0 * a);
      let t2 = (-b - descr.sqrt()) / (2.0 * a);
      let (t1, t2) = { (t1.max(t2), t1.min(t2)) };
      
      if t1 > 0.0 {
        result.push(left.origin + left.dir * t1)
      } 
      if t2 > 0.0 {
        result.push(left.origin + left.dir * t2) // latest element will be popped first 
      }
    }
    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn linear_components_same() {
    let ray1= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
    let ray2= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));

    assert_eq!(IntersectorEnum::linear_components(&ray1, &ray2), LinearIntersectResult::SameLines);
  }

  #[test]
  fn linear_components_same_1() {
    let ray1= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 2.0).normalize());
    let ray2= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 2.0).normalize());

    assert_eq!(IntersectorEnum::linear_components(&ray1, &ray2), LinearIntersectResult::SameLines);
  }

  #[test]
  fn linear_components_parallel() {
    let ray1= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
    let ray2= Ray::new(Vec2::new(0.0, 1.0), Vec2::new(1.0, 0.0));

    assert_eq!(IntersectorEnum::linear_components(&ray1, &ray2), LinearIntersectResult::ParallelLines);
  }

  #[test]
  fn linear_components_parallel_2() {
    let ray1= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 2.0).normalize());
    let ray2= Ray::new(Vec2::new(0.0, 1.0), Vec2::new(1.0, 2.0).normalize());

    assert_eq!(IntersectorEnum::linear_components(&ray1, &ray2), LinearIntersectResult::ParallelLines);
  }

  #[test]
  fn linear_components_intersect() {
    let ray1= Ray::new(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
    let ray2= Ray::new(Vec2::new(0.0, 1.0), Vec2::new(1.0, -0.4).normalize());

    assert_eq!(IntersectorEnum::linear_components(&ray1, &ray2), LinearIntersectResult::Parameters(2.6925821, 2.5));
  }

  #[test]
  fn linear_components_intersect_2() {
    let ray1= Ray::new(Vec2::new(1.0, 0.0), Vec2::new(1.0, 2.0).normalize());
    let ray2= Ray::new(Vec2::new(0.0, 1.0), Vec2::new(1.9, 2.0).normalize());

    assert_eq!(IntersectorEnum::linear_components(&ray1, &ray2), LinearIntersectResult::Parameters(4.5977044, 4.844814));
  }

#[test]
  fn linear_components_intersect_dang() {
    let ray1= Ray::new(Vec2::new(12.2, 25.7), Vec2::new(0.4288417,0.9033797));
    let ray2= Segment::new(Vec2::new(12.5, 26.0), Vec2::new(12.9, 26.0)).make_ray();
    println!("RAy 1 {:?}", ray1);
    println!("RAy 2 {:?}", ray2);


    match IntersectorEnum::linear_components(&ray1, &ray2) {
      LinearIntersectResult::Parameters(s, t) => {
        println!("adfasdff {} {}", s, t);
        assert_eq!(ray1.spot(s), ray2.spot(t))
      },
      _ => unreachable!()
    }
  }

}

