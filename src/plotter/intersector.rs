extern crate nalgebra as na;
use na::{  Rotation2, Vector2 };
use std::f32::consts::PI;
use super::circular_direction::*;

type Vec2 = Vector2<f32>;

pub enum IntersectorEnum {
  Ray(Ray),
  Arc(Arc),
  Segment(Segment)
}

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
    Ray::new(self.from, self.from - self.to)
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
  pub fn intersects(&self, other: IntersectorEnum) -> Option<Vec2> {
    use IntersectorEnum::*;
    match self {
      Ray(left_ray) => {
        match other {
          Ray(right_ray) => IntersectorEnum::intersects_ray_ray(&left_ray, &right_ray),
          Arc(right_arc) => IntersectorEnum::intersects_ray_arc_2(&left_ray, &right_arc),
          Segment(s) => IntersectorEnum::ray_segment_intersection( &left_ray, &s)
        }
      },
      Arc(left_arc) => {
        match other {
          Ray(right_ray) => IntersectorEnum::intersects_ray_arc_2(&right_ray, &left_arc),
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

  fn ray_segment_intersection(ray: &Ray, segment: &Segment) -> Option<Vec2>{
    let segment_ray = Ray::new(segment.from, segment.to - segment.from);
    let t = Self::ray_ray_find_t(&segment_ray, ray);
    if t >= 0.0 && t <= 1.0 {
      Some(segment_ray.spot(t))
    } else {
      None
    }
  }

  fn segment_segment_intersection(main: &Segment, segment: &Segment) -> Option<Vec2>{
    let segment_ray = Ray::new(segment.from, segment.to - segment.from);
    let main_segment_ray = Ray::new(main.from, main.to - main.from);
    let t1 = Self::ray_ray_find_t(&segment_ray, &main_segment_ray);
    let t2 = Self::ray_ray_find_t(&main_segment_ray, &segment_ray); // Ineffective...
    if t1 >= 0.0 && t1 <= 1.0  && t2 >= 0.0 && t2 <= 1.0 {
      Some(segment_ray.spot(t1))
    } else {
      None
    }
  }

  fn ray_ray_find_t(main: &Ray, intersector: &Ray) -> f32 {
    let left = intersector;
    let other = main;

    let distance = other.origin - left.origin;
    let basis = left.dir.normalize(); 
    let projection = other.dir.dot(&basis);
    let distance_projection = distance.dot(&basis);

    let ortho_projection = other.dir - projection * basis;
    let ortho_distance_projection = distance - distance_projection * basis;


    let sign = -ortho_projection.dot(&ortho_distance_projection).signum();
    sign * ortho_distance_projection.magnitude() / ortho_projection.magnitude()
  }

  fn intersects_ray_ray(left: &Ray, other: &Ray) -> Option<Vec2> {
    let t = Self::ray_ray_find_t(other, left);
    Some(other.origin + other.dir * t)
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

   fn intersects_arc_arc(left: &Arc, other: &Arc) -> Option<Vec2> {
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
      if IntersectorEnum::is_on_arc(left, &point1) && IntersectorEnum::is_on_arc(other, &point1)
      {
        Some(point1)
      } else if IntersectorEnum::is_on_arc(left, &point2) && IntersectorEnum::is_on_arc(other, &point2)
      {
        Some(point2)
      } else {
        None
      }

    } else {
      None
    }
  }

   fn intersects_segment_arc(segment: &Segment, arc: &Arc)->Option<Vec2> {
     let ray = segment.make_ray();
     let mut unchecked_intersection = Self::ray_arc_both_points(&ray, arc);
     let mut result = None;
     while let Some(vec) = unchecked_intersection.pop() {
       if Self::is_on_arc(&arc, &vec) && Self::is_on_segment(&segment, &vec) {
         result = Some(vec);
         break;
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
      println!("NNNNNNNNNOOOOOO");
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
   fn intersects_ray_arc_2(left: &Ray, other: &Arc) -> Option<Vec2> {
    let delta = left.origin - other.center;
    let radius = (other.from - other.center).magnitude();
    let dir = left.dir;

    // square equation
    let a = dir.dot(&dir);
    let b = 2.0 * delta.dot(&dir);
    let c = delta.dot(&delta) - radius.powi(2);

    let descr = b.powi(2) - 4.0 * a * c;
    if descr < 0.0 {
      None
    } else if descr <= f32::EPSILON {
      Some(left.origin + left.dir * (-b / (2.0 * a)))
    } else {
      let t1 = (-b + descr.sqrt()) / (2.0 * a);
      let t2 = (-b - descr.sqrt()) / (2.0 * a);
      let (t1, t2) = { (t1.min(t2), t1.max(t2)) };
      
      if t1 > 0.0 {
        Some(left.origin + left.dir * t1)
      } else if t2 > 0.0 {
        Some(left.origin + left.dir * t2)
      } else {
        None
      }
    }
  }

   fn intersects_ray_arc(left: &Ray, other: &Arc) -> Option<Vec2> {
    let radius = (other.from - other.center).magnitude();
    let arc_center = other.center - left.origin;
    let basis_x = left.dir.normalize();
    let basis_y = Rotation2::new(PI / 2.0) * basis_x;
    let to_center_y = arc_center.dot(&basis_y);
    let to_center_x = arc_center.dot(&basis_x);

    if to_center_y > radius {
      None
    } else {

      let delta_t = (radius.powi(2) - to_center_y.powi(2)).sqrt();

      let t_to_center = to_center_x / left.dir.magnitude();
      let t_from_center = delta_t / left.dir.magnitude();

      let (t1, t2) = (t_to_center - t_from_center, t_to_center + t_from_center);

      if t1 > 0.0 {
        Some(left.origin + left.dir * t1)
      } else if t2 > 0.0 {
        Some(left.origin + left.dir * t2)
      } else {
        None
      }
    }
  }
}
