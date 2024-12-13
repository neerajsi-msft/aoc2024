use std::ops::Index;

use enum_iterator::Sequence;
use num::Zero;

pub const GRID_DIRECTION8_VECTORS: [[i32;2]; 8] =
[
    [-1, 0],
    [-1, 1],
    [0, 1],
    [1, 1],
    [1, 0],
    [1, -1],
    [-1, 0],
    [-1, -1]
];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Sequence)]
pub enum GridDirection8 {
    N = 0,
    NE = 1,
    E = 2,
    SE = 3,
    S = 4,
    SW = 5,
    W = 6,
    NW = 7
}

pub const GRID_DIRECTION4_VECTORS: [[i32;2];4] =
[
    [-1, 0],
    [0, 1],
    [1, 0],
    [0, -1]
];

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Sequence)]
pub enum GridDirection4 {
    N = 0,
    E = 1,
    S = 2,
    W = 3
}

pub trait Grid<IndexType>
    where IndexType: num::Num  + Copy + Clone + Ord + Eq {
    
    fn get_origin() -> [IndexType; 2] {
        [Zero::zero(); 2]
    }

    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
