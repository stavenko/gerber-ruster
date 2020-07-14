use std::ops::{ Div, DivAssign };
use std::convert::From;

pub type Forest<T> = Vec<Box<Tree<T>>>;

pub struct Tree<T> {
  pub data: T,
  children: Forest<T>
}

impl<T> From<Tree<T>> for Forest<T> {
  fn from(tree: Tree<T>) -> Self {
    vec!(Box::new(tree))
  }
}


impl<T>  Tree<T> {
  pub fn new(data: T) -> Self {
    Tree::<T>{
      children: Vec::new(),
      data
    }
  }

  pub fn is_leaf(&self) -> bool {
    self.children.is_empty()
  }

  pub fn push(&mut self, item: T) {
    self.children.push(Box::new(Tree::new(item)));
  }

  pub fn extend(&mut self, sub_forest: Forest<T>) 
  {
    self.children.extend(sub_forest)
  }

  pub fn forest_mut(&mut self) -> &mut Forest<T> {
    &mut self.children
  }
}

impl<T> IntoIterator for Tree<T> {
  type Item = Box<Tree<T>>;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.children.into_iter()
  }
}


pub fn tr<T>(item: T) -> Tree<T> {
  Tree::<T>::new(item)
}

impl<T> DivAssign for Tree<T> {
  fn div_assign(&mut self, rhs: Self) {
    self.children.push(Box::new(rhs))
  }
}


impl<T> Div for Tree<T> {
  type Output = Self;

  fn div(self, rhs: Self) -> Self::Output {
    let mut out = Tree::new(self.data);
    out /= rhs;
    out
  }
}


