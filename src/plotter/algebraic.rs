use super::Arc;
use super::Line;

pub enum AlgebraicPathElement {
  Arc(Arc),
  Line(Line)
}

pub trait Algebraic<T> 
{
  fn algebraic(&self) -> T;
}

impl Algebraic<AlgebraicPathElement> for Arc {
  fn algebraic(&self) -> AlgebraicPathElement {
    AlgebraicPathElement::Arc((*self).clone())
  }
}

impl Algebraic<AlgebraicPathElement> for Line {
  fn algebraic(&self) -> AlgebraicPathElement {
    AlgebraicPathElement::Line((*self).clone())
  }
}

