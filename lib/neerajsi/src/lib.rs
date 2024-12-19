use std::fmt;
use num_derive::FromPrimitive;
use nalgebra::Vector2;

pub fn to_vector2<T>(val: &[T;2]) -> Vector2<T> 
    where T: Clone + Copy
{
    Vector2::new(val[0], val[1])
}

pub fn to_vector2_cast(val: &[usize;2]) -> Vector2<i64> 
{
    Vector2::new(val[0] as i64, val[1] as i64)
}

pub fn vector2_from_tuple<T>(val: (T, T)) -> Vector2<T>
    where T: Clone + Copy
{
    Vector2::new(val.0, val.1)
}

#[macro_export]
macro_rules! index2d {
    ($m:expr, $v:expr) => { ($m)[$v.x as usize][$v.y as usize] };
}

#[macro_export]
macro_rules! index2d_array {
    ($m:expr, $v:expr) => { ($m)[$v[0] as usize][$v[1] as usize] };
}


#[macro_export]
macro_rules! vec2d {
    ($r:expr, $c:expr, $defval: expr) => { vec![vec![$defval; $c]; $r] }
}

pub struct PositionIterator<I, J, F>
{
    data: I,
    predicate: F,
    current_iter: Option<J>,
    row: usize,
    col: usize,
}

impl<I: fmt::Debug, J: fmt::Debug, F> fmt::Debug for PositionIterator<I, J, F>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PositionIterator")
         .field("data", &self.data)
         .field("current_iter", &self.current_iter)
         .field("row", &self.row)
         .field("col", &self.col)
         .finish()
    }
}

impl<I, J, F> Iterator for PositionIterator<I, J, F>
where
    I: Iterator,
    I::Item: IntoIterator<IntoIter = J>,
    J: Iterator,
    F: Fn(J::Item) -> bool,
{
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut iter) = self.current_iter {
                while let Some(item) = iter.next() {
                    self.col += 1;
                    if (self.predicate)(item) {
                        return Some((self.row, self.col - 1));
                    }
                }
                self.current_iter = None;
                self.row += 1;
                self.col = 0;
            }

            match self.data.next() {
                Some(next_iterable) => {
                    self.current_iter = Some(next_iterable.into_iter());
                }
                None => return None,
            }
        }
    }
}

pub trait Iterable2d : Iterator
    where Self: Sized,
          Self::Item: IntoIterator
{
    fn positions2d<P>(self, predicate: P) -> PositionIterator<Self, <Self::Item as IntoIterator>::IntoIter, P>
        where P: Fn(<<Self as Iterator>::Item as IntoIterator>::Item) -> bool
    {
        PositionIterator{
            data: self,
            predicate,
            current_iter: None,
            row: 0,
            col: 0
        }
    }
}

impl<T> Iterable2d for T where T: Iterator<Item: IntoIterator> + Sized {}

pub const DIRECTION_VECTORS: [[i64;2]; 8] = [
    [0, -1], 
    [0, 1],
    [-1, 0],
    [1, 0],
    [-1, -1],
    [-1, 1],
    [1, 1],
    [1,-1]
];

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum DirectionName {
    W = 0,
    E = 1,
    N = 2,
    S = 3,
    NW = 4,
    NE = 5,
    SE = 6,
    SW = 7
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum CardinalDirectionName {
    W = DirectionName::W as isize,
    E = DirectionName::E as isize,
    N = DirectionName::N as isize,
    S = DirectionName::S as isize,
}

impl From<CardinalDirectionName> for DirectionName {
    fn from(value: CardinalDirectionName) -> Self {
        DirectionName::from_isize(value as isize).unwrap()
    }
}

use num_traits::FromPrimitive;
use DirectionName::*;

pub const DIRECTIONS4: [DirectionName; 4] = [W, E, N, S];
pub const DIRECTIONS8: [DirectionName; 8] = [W, E, N, S, NW, NE, SE, SW];

#[test]
fn check_directions() {

    const GRID: [[DirectionName; 3];3] = [
        [NW, N, NE],
        [W, SW, E,],
        [SW, S, SE],
    ];

    let pos = to_vector2(&[1, 1]);
    for d in DIRECTIONS8 {
        let dir = next_pos(pos, d);
        assert_eq!(index2d!(GRID, dir), d);
    }
}

pub type VectorType = Vector2<i64>;

pub fn direction_vector(direction: DirectionName) -> VectorType {
    to_vector2(&DIRECTION_VECTORS[direction as usize])
}

pub fn next_pos_cardinal(pos: VectorType, direction: CardinalDirectionName) -> VectorType {
    next_pos(pos, direction.into())
}

pub fn next_pos(pos: VectorType, direction: DirectionName) -> VectorType {
    pos + direction_vector(direction)
}

pub type Location = [usize;2];

#[derive(Debug, Default)]
pub struct DirectionIterator {
    location: Location,
    rows: usize,
    cols: usize,
    current_dir: usize,
    dir_count: usize
}

impl DirectionIterator {
    pub fn new_cardinal(location: Location, rows: usize, cols: usize) -> Self
    {
        Self{location, rows, cols, current_dir:0, dir_count:4}
    }

    pub fn new_all_dirs(location: Location, rows: usize, cols: usize) -> Self
    {
        Self{location, rows, cols, current_dir:0, dir_count:8}
    }
}

impl Iterator for DirectionIterator {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_dir < self.dir_count {
            let d = DIRECTION_VECTORS[self.current_dir];
            self.current_dir += 1;

            let new_loc: [Option<usize>;2] = std::array::from_fn(|a| self.location[a].checked_add_signed(d[a] as isize));
                    
            if let [Some(r), Some(c)] = new_loc {
                if (r < self.rows) && (c < self.cols) {
                    return Some([r, c]);
                }
            }
        }

        None
    }
}

impl std::iter::FusedIterator for DirectionIterator {}

