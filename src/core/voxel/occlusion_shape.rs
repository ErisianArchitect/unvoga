#![allow(unused)]
/*
The OcclusionShape should be used for light and mesh occlusion.
You should logically map the x and y coordinates to the x/y/z coordinates.
If you're making an occluder on the X axis, use Z for X and Y for Y.
If you're making an occluder on the Y axis, use X for X and Z for Y.
If you're making an occluder on the Z axis, use X for X and Y for Y.

If you want to rotate an occluder, good luck.
*/

use crate::core::math::coordmap::{rotate_face_coord};
use crate::prelude::*;

use super::faces::Faces;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape16x16([u16; 16]);

impl OcclusionShape16x16 {
    
    pub const fn new(masks: [u16; 16]) -> Self {
        Self(masks)
    }

    pub const fn from_matrix(matrix: [[u8; 16]; 16]) -> Self {
        let mut masks = [0u16; 16];
        // Don't worry, I generated this with python.
        // I can't use loops in const functions, so this is what I did.
        masks[0] |= ((matrix[0][0] != 0) as u16) << 0;
        masks[0] |= ((matrix[0][1] != 0) as u16) << 1;
        masks[0] |= ((matrix[0][2] != 0) as u16) << 2;
        masks[0] |= ((matrix[0][3] != 0) as u16) << 3;
        masks[0] |= ((matrix[0][4] != 0) as u16) << 4;
        masks[0] |= ((matrix[0][5] != 0) as u16) << 5;
        masks[0] |= ((matrix[0][6] != 0) as u16) << 6;
        masks[0] |= ((matrix[0][7] != 0) as u16) << 7;
        masks[0] |= ((matrix[0][8] != 0) as u16) << 8;
        masks[0] |= ((matrix[0][9] != 0) as u16) << 9;
        masks[0] |= ((matrix[0][10] != 0) as u16) << 10;
        masks[0] |= ((matrix[0][11] != 0) as u16) << 11;
        masks[0] |= ((matrix[0][12] != 0) as u16) << 12;
        masks[0] |= ((matrix[0][13] != 0) as u16) << 13;
        masks[0] |= ((matrix[0][14] != 0) as u16) << 14;
        masks[0] |= ((matrix[0][15] != 0) as u16) << 15;
        masks[1] |= ((matrix[1][0] != 0) as u16) << 0;
        masks[1] |= ((matrix[1][1] != 0) as u16) << 1;
        masks[1] |= ((matrix[1][2] != 0) as u16) << 2;
        masks[1] |= ((matrix[1][3] != 0) as u16) << 3;
        masks[1] |= ((matrix[1][4] != 0) as u16) << 4;
        masks[1] |= ((matrix[1][5] != 0) as u16) << 5;
        masks[1] |= ((matrix[1][6] != 0) as u16) << 6;
        masks[1] |= ((matrix[1][7] != 0) as u16) << 7;
        masks[1] |= ((matrix[1][8] != 0) as u16) << 8;
        masks[1] |= ((matrix[1][9] != 0) as u16) << 9;
        masks[1] |= ((matrix[1][10] != 0) as u16) << 10;
        masks[1] |= ((matrix[1][11] != 0) as u16) << 11;
        masks[1] |= ((matrix[1][12] != 0) as u16) << 12;
        masks[1] |= ((matrix[1][13] != 0) as u16) << 13;
        masks[1] |= ((matrix[1][14] != 0) as u16) << 14;
        masks[1] |= ((matrix[1][15] != 0) as u16) << 15;
        masks[2] |= ((matrix[2][0] != 0) as u16) << 0;
        masks[2] |= ((matrix[2][1] != 0) as u16) << 1;
        masks[2] |= ((matrix[2][2] != 0) as u16) << 2;
        masks[2] |= ((matrix[2][3] != 0) as u16) << 3;
        masks[2] |= ((matrix[2][4] != 0) as u16) << 4;
        masks[2] |= ((matrix[2][5] != 0) as u16) << 5;
        masks[2] |= ((matrix[2][6] != 0) as u16) << 6;
        masks[2] |= ((matrix[2][7] != 0) as u16) << 7;
        masks[2] |= ((matrix[2][8] != 0) as u16) << 8;
        masks[2] |= ((matrix[2][9] != 0) as u16) << 9;
        masks[2] |= ((matrix[2][10] != 0) as u16) << 10;
        masks[2] |= ((matrix[2][11] != 0) as u16) << 11;
        masks[2] |= ((matrix[2][12] != 0) as u16) << 12;
        masks[2] |= ((matrix[2][13] != 0) as u16) << 13;
        masks[2] |= ((matrix[2][14] != 0) as u16) << 14;
        masks[2] |= ((matrix[2][15] != 0) as u16) << 15;
        masks[3] |= ((matrix[3][0] != 0) as u16) << 0;
        masks[3] |= ((matrix[3][1] != 0) as u16) << 1;
        masks[3] |= ((matrix[3][2] != 0) as u16) << 2;
        masks[3] |= ((matrix[3][3] != 0) as u16) << 3;
        masks[3] |= ((matrix[3][4] != 0) as u16) << 4;
        masks[3] |= ((matrix[3][5] != 0) as u16) << 5;
        masks[3] |= ((matrix[3][6] != 0) as u16) << 6;
        masks[3] |= ((matrix[3][7] != 0) as u16) << 7;
        masks[3] |= ((matrix[3][8] != 0) as u16) << 8;
        masks[3] |= ((matrix[3][9] != 0) as u16) << 9;
        masks[3] |= ((matrix[3][10] != 0) as u16) << 10;
        masks[3] |= ((matrix[3][11] != 0) as u16) << 11;
        masks[3] |= ((matrix[3][12] != 0) as u16) << 12;
        masks[3] |= ((matrix[3][13] != 0) as u16) << 13;
        masks[3] |= ((matrix[3][14] != 0) as u16) << 14;
        masks[3] |= ((matrix[3][15] != 0) as u16) << 15;
        masks[4] |= ((matrix[4][0] != 0) as u16) << 0;
        masks[4] |= ((matrix[4][1] != 0) as u16) << 1;
        masks[4] |= ((matrix[4][2] != 0) as u16) << 2;
        masks[4] |= ((matrix[4][3] != 0) as u16) << 3;
        masks[4] |= ((matrix[4][4] != 0) as u16) << 4;
        masks[4] |= ((matrix[4][5] != 0) as u16) << 5;
        masks[4] |= ((matrix[4][6] != 0) as u16) << 6;
        masks[4] |= ((matrix[4][7] != 0) as u16) << 7;
        masks[4] |= ((matrix[4][8] != 0) as u16) << 8;
        masks[4] |= ((matrix[4][9] != 0) as u16) << 9;
        masks[4] |= ((matrix[4][10] != 0) as u16) << 10;
        masks[4] |= ((matrix[4][11] != 0) as u16) << 11;
        masks[4] |= ((matrix[4][12] != 0) as u16) << 12;
        masks[4] |= ((matrix[4][13] != 0) as u16) << 13;
        masks[4] |= ((matrix[4][14] != 0) as u16) << 14;
        masks[4] |= ((matrix[4][15] != 0) as u16) << 15;
        masks[5] |= ((matrix[5][0] != 0) as u16) << 0;
        masks[5] |= ((matrix[5][1] != 0) as u16) << 1;
        masks[5] |= ((matrix[5][2] != 0) as u16) << 2;
        masks[5] |= ((matrix[5][3] != 0) as u16) << 3;
        masks[5] |= ((matrix[5][4] != 0) as u16) << 4;
        masks[5] |= ((matrix[5][5] != 0) as u16) << 5;
        masks[5] |= ((matrix[5][6] != 0) as u16) << 6;
        masks[5] |= ((matrix[5][7] != 0) as u16) << 7;
        masks[5] |= ((matrix[5][8] != 0) as u16) << 8;
        masks[5] |= ((matrix[5][9] != 0) as u16) << 9;
        masks[5] |= ((matrix[5][10] != 0) as u16) << 10;
        masks[5] |= ((matrix[5][11] != 0) as u16) << 11;
        masks[5] |= ((matrix[5][12] != 0) as u16) << 12;
        masks[5] |= ((matrix[5][13] != 0) as u16) << 13;
        masks[5] |= ((matrix[5][14] != 0) as u16) << 14;
        masks[5] |= ((matrix[5][15] != 0) as u16) << 15;
        masks[6] |= ((matrix[6][0] != 0) as u16) << 0;
        masks[6] |= ((matrix[6][1] != 0) as u16) << 1;
        masks[6] |= ((matrix[6][2] != 0) as u16) << 2;
        masks[6] |= ((matrix[6][3] != 0) as u16) << 3;
        masks[6] |= ((matrix[6][4] != 0) as u16) << 4;
        masks[6] |= ((matrix[6][5] != 0) as u16) << 5;
        masks[6] |= ((matrix[6][6] != 0) as u16) << 6;
        masks[6] |= ((matrix[6][7] != 0) as u16) << 7;
        masks[6] |= ((matrix[6][8] != 0) as u16) << 8;
        masks[6] |= ((matrix[6][9] != 0) as u16) << 9;
        masks[6] |= ((matrix[6][10] != 0) as u16) << 10;
        masks[6] |= ((matrix[6][11] != 0) as u16) << 11;
        masks[6] |= ((matrix[6][12] != 0) as u16) << 12;
        masks[6] |= ((matrix[6][13] != 0) as u16) << 13;
        masks[6] |= ((matrix[6][14] != 0) as u16) << 14;
        masks[6] |= ((matrix[6][15] != 0) as u16) << 15;
        masks[7] |= ((matrix[7][0] != 0) as u16) << 0;
        masks[7] |= ((matrix[7][1] != 0) as u16) << 1;
        masks[7] |= ((matrix[7][2] != 0) as u16) << 2;
        masks[7] |= ((matrix[7][3] != 0) as u16) << 3;
        masks[7] |= ((matrix[7][4] != 0) as u16) << 4;
        masks[7] |= ((matrix[7][5] != 0) as u16) << 5;
        masks[7] |= ((matrix[7][6] != 0) as u16) << 6;
        masks[7] |= ((matrix[7][7] != 0) as u16) << 7;
        masks[7] |= ((matrix[7][8] != 0) as u16) << 8;
        masks[7] |= ((matrix[7][9] != 0) as u16) << 9;
        masks[7] |= ((matrix[7][10] != 0) as u16) << 10;
        masks[7] |= ((matrix[7][11] != 0) as u16) << 11;
        masks[7] |= ((matrix[7][12] != 0) as u16) << 12;
        masks[7] |= ((matrix[7][13] != 0) as u16) << 13;
        masks[7] |= ((matrix[7][14] != 0) as u16) << 14;
        masks[7] |= ((matrix[7][15] != 0) as u16) << 15;
        masks[8] |= ((matrix[8][0] != 0) as u16) << 0;
        masks[8] |= ((matrix[8][1] != 0) as u16) << 1;
        masks[8] |= ((matrix[8][2] != 0) as u16) << 2;
        masks[8] |= ((matrix[8][3] != 0) as u16) << 3;
        masks[8] |= ((matrix[8][4] != 0) as u16) << 4;
        masks[8] |= ((matrix[8][5] != 0) as u16) << 5;
        masks[8] |= ((matrix[8][6] != 0) as u16) << 6;
        masks[8] |= ((matrix[8][7] != 0) as u16) << 7;
        masks[8] |= ((matrix[8][8] != 0) as u16) << 8;
        masks[8] |= ((matrix[8][9] != 0) as u16) << 9;
        masks[8] |= ((matrix[8][10] != 0) as u16) << 10;
        masks[8] |= ((matrix[8][11] != 0) as u16) << 11;
        masks[8] |= ((matrix[8][12] != 0) as u16) << 12;
        masks[8] |= ((matrix[8][13] != 0) as u16) << 13;
        masks[8] |= ((matrix[8][14] != 0) as u16) << 14;
        masks[8] |= ((matrix[8][15] != 0) as u16) << 15;
        masks[9] |= ((matrix[9][0] != 0) as u16) << 0;
        masks[9] |= ((matrix[9][1] != 0) as u16) << 1;
        masks[9] |= ((matrix[9][2] != 0) as u16) << 2;
        masks[9] |= ((matrix[9][3] != 0) as u16) << 3;
        masks[9] |= ((matrix[9][4] != 0) as u16) << 4;
        masks[9] |= ((matrix[9][5] != 0) as u16) << 5;
        masks[9] |= ((matrix[9][6] != 0) as u16) << 6;
        masks[9] |= ((matrix[9][7] != 0) as u16) << 7;
        masks[9] |= ((matrix[9][8] != 0) as u16) << 8;
        masks[9] |= ((matrix[9][9] != 0) as u16) << 9;
        masks[9] |= ((matrix[9][10] != 0) as u16) << 10;
        masks[9] |= ((matrix[9][11] != 0) as u16) << 11;
        masks[9] |= ((matrix[9][12] != 0) as u16) << 12;
        masks[9] |= ((matrix[9][13] != 0) as u16) << 13;
        masks[9] |= ((matrix[9][14] != 0) as u16) << 14;
        masks[9] |= ((matrix[9][15] != 0) as u16) << 15;
        masks[10] |= ((matrix[10][0] != 0) as u16) << 0;
        masks[10] |= ((matrix[10][1] != 0) as u16) << 1;
        masks[10] |= ((matrix[10][2] != 0) as u16) << 2;
        masks[10] |= ((matrix[10][3] != 0) as u16) << 3;
        masks[10] |= ((matrix[10][4] != 0) as u16) << 4;
        masks[10] |= ((matrix[10][5] != 0) as u16) << 5;
        masks[10] |= ((matrix[10][6] != 0) as u16) << 6;
        masks[10] |= ((matrix[10][7] != 0) as u16) << 7;
        masks[10] |= ((matrix[10][8] != 0) as u16) << 8;
        masks[10] |= ((matrix[10][9] != 0) as u16) << 9;
        masks[10] |= ((matrix[10][10] != 0) as u16) << 10;
        masks[10] |= ((matrix[10][11] != 0) as u16) << 11;
        masks[10] |= ((matrix[10][12] != 0) as u16) << 12;
        masks[10] |= ((matrix[10][13] != 0) as u16) << 13;
        masks[10] |= ((matrix[10][14] != 0) as u16) << 14;
        masks[10] |= ((matrix[10][15] != 0) as u16) << 15;
        masks[11] |= ((matrix[11][0] != 0) as u16) << 0;
        masks[11] |= ((matrix[11][1] != 0) as u16) << 1;
        masks[11] |= ((matrix[11][2] != 0) as u16) << 2;
        masks[11] |= ((matrix[11][3] != 0) as u16) << 3;
        masks[11] |= ((matrix[11][4] != 0) as u16) << 4;
        masks[11] |= ((matrix[11][5] != 0) as u16) << 5;
        masks[11] |= ((matrix[11][6] != 0) as u16) << 6;
        masks[11] |= ((matrix[11][7] != 0) as u16) << 7;
        masks[11] |= ((matrix[11][8] != 0) as u16) << 8;
        masks[11] |= ((matrix[11][9] != 0) as u16) << 9;
        masks[11] |= ((matrix[11][10] != 0) as u16) << 10;
        masks[11] |= ((matrix[11][11] != 0) as u16) << 11;
        masks[11] |= ((matrix[11][12] != 0) as u16) << 12;
        masks[11] |= ((matrix[11][13] != 0) as u16) << 13;
        masks[11] |= ((matrix[11][14] != 0) as u16) << 14;
        masks[11] |= ((matrix[11][15] != 0) as u16) << 15;
        masks[12] |= ((matrix[12][0] != 0) as u16) << 0;
        masks[12] |= ((matrix[12][1] != 0) as u16) << 1;
        masks[12] |= ((matrix[12][2] != 0) as u16) << 2;
        masks[12] |= ((matrix[12][3] != 0) as u16) << 3;
        masks[12] |= ((matrix[12][4] != 0) as u16) << 4;
        masks[12] |= ((matrix[12][5] != 0) as u16) << 5;
        masks[12] |= ((matrix[12][6] != 0) as u16) << 6;
        masks[12] |= ((matrix[12][7] != 0) as u16) << 7;
        masks[12] |= ((matrix[12][8] != 0) as u16) << 8;
        masks[12] |= ((matrix[12][9] != 0) as u16) << 9;
        masks[12] |= ((matrix[12][10] != 0) as u16) << 10;
        masks[12] |= ((matrix[12][11] != 0) as u16) << 11;
        masks[12] |= ((matrix[12][12] != 0) as u16) << 12;
        masks[12] |= ((matrix[12][13] != 0) as u16) << 13;
        masks[12] |= ((matrix[12][14] != 0) as u16) << 14;
        masks[12] |= ((matrix[12][15] != 0) as u16) << 15;
        masks[13] |= ((matrix[13][0] != 0) as u16) << 0;
        masks[13] |= ((matrix[13][1] != 0) as u16) << 1;
        masks[13] |= ((matrix[13][2] != 0) as u16) << 2;
        masks[13] |= ((matrix[13][3] != 0) as u16) << 3;
        masks[13] |= ((matrix[13][4] != 0) as u16) << 4;
        masks[13] |= ((matrix[13][5] != 0) as u16) << 5;
        masks[13] |= ((matrix[13][6] != 0) as u16) << 6;
        masks[13] |= ((matrix[13][7] != 0) as u16) << 7;
        masks[13] |= ((matrix[13][8] != 0) as u16) << 8;
        masks[13] |= ((matrix[13][9] != 0) as u16) << 9;
        masks[13] |= ((matrix[13][10] != 0) as u16) << 10;
        masks[13] |= ((matrix[13][11] != 0) as u16) << 11;
        masks[13] |= ((matrix[13][12] != 0) as u16) << 12;
        masks[13] |= ((matrix[13][13] != 0) as u16) << 13;
        masks[13] |= ((matrix[13][14] != 0) as u16) << 14;
        masks[13] |= ((matrix[13][15] != 0) as u16) << 15;
        masks[14] |= ((matrix[14][0] != 0) as u16) << 0;
        masks[14] |= ((matrix[14][1] != 0) as u16) << 1;
        masks[14] |= ((matrix[14][2] != 0) as u16) << 2;
        masks[14] |= ((matrix[14][3] != 0) as u16) << 3;
        masks[14] |= ((matrix[14][4] != 0) as u16) << 4;
        masks[14] |= ((matrix[14][5] != 0) as u16) << 5;
        masks[14] |= ((matrix[14][6] != 0) as u16) << 6;
        masks[14] |= ((matrix[14][7] != 0) as u16) << 7;
        masks[14] |= ((matrix[14][8] != 0) as u16) << 8;
        masks[14] |= ((matrix[14][9] != 0) as u16) << 9;
        masks[14] |= ((matrix[14][10] != 0) as u16) << 10;
        masks[14] |= ((matrix[14][11] != 0) as u16) << 11;
        masks[14] |= ((matrix[14][12] != 0) as u16) << 12;
        masks[14] |= ((matrix[14][13] != 0) as u16) << 13;
        masks[14] |= ((matrix[14][14] != 0) as u16) << 14;
        masks[14] |= ((matrix[14][15] != 0) as u16) << 15;
        masks[15] |= ((matrix[15][0] != 0) as u16) << 0;
        masks[15] |= ((matrix[15][1] != 0) as u16) << 1;
        masks[15] |= ((matrix[15][2] != 0) as u16) << 2;
        masks[15] |= ((matrix[15][3] != 0) as u16) << 3;
        masks[15] |= ((matrix[15][4] != 0) as u16) << 4;
        masks[15] |= ((matrix[15][5] != 0) as u16) << 5;
        masks[15] |= ((matrix[15][6] != 0) as u16) << 6;
        masks[15] |= ((matrix[15][7] != 0) as u16) << 7;
        masks[15] |= ((matrix[15][8] != 0) as u16) << 8;
        masks[15] |= ((matrix[15][9] != 0) as u16) << 9;
        masks[15] |= ((matrix[15][10] != 0) as u16) << 10;
        masks[15] |= ((matrix[15][11] != 0) as u16) << 11;
        masks[15] |= ((matrix[15][12] != 0) as u16) << 12;
        masks[15] |= ((matrix[15][13] != 0) as u16) << 13;
        masks[15] |= ((matrix[15][14] != 0) as u16) << 14;
        masks[15] |= ((matrix[15][15] != 0) as u16) << 15;
        Self(masks)
    }

    
    pub fn get(&self, x: usize, y: usize) -> bool {
        let sub = self.0[y];
        sub & (1 << x) != 0
    }

    
    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let sub = self.0[y];
        self.0[y] = if value {
            sub | (1 << x)
        } else {
            sub & !(1 << x)
        };
        sub & (1 << x) != 0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape8x8(u64);

impl OcclusionShape8x8 {
    
    pub const fn new(mask: u64) -> Self {
        Self(mask)
    }

    pub const fn from_matrix(matrix: [[u8; 8]; 8]) -> Self {
        let mut mask = 0u64;
        mask |= ((matrix[0][0] != 0) as u64) << 0;
        mask |= ((matrix[0][1] != 0) as u64) << 1;
        mask |= ((matrix[0][2] != 0) as u64) << 2;
        mask |= ((matrix[0][3] != 0) as u64) << 3;
        mask |= ((matrix[0][4] != 0) as u64) << 4;
        mask |= ((matrix[0][5] != 0) as u64) << 5;
        mask |= ((matrix[0][6] != 0) as u64) << 6;
        mask |= ((matrix[0][7] != 0) as u64) << 7;
        mask |= ((matrix[1][0] != 0) as u64) << 8;
        mask |= ((matrix[1][1] != 0) as u64) << 9;
        mask |= ((matrix[1][2] != 0) as u64) << 10;
        mask |= ((matrix[1][3] != 0) as u64) << 11;
        mask |= ((matrix[1][4] != 0) as u64) << 12;
        mask |= ((matrix[1][5] != 0) as u64) << 13;
        mask |= ((matrix[1][6] != 0) as u64) << 14;
        mask |= ((matrix[1][7] != 0) as u64) << 15;
        mask |= ((matrix[2][0] != 0) as u64) << 16;
        mask |= ((matrix[2][1] != 0) as u64) << 17;
        mask |= ((matrix[2][2] != 0) as u64) << 18;
        mask |= ((matrix[2][3] != 0) as u64) << 19;
        mask |= ((matrix[2][4] != 0) as u64) << 20;
        mask |= ((matrix[2][5] != 0) as u64) << 21;
        mask |= ((matrix[2][6] != 0) as u64) << 22;
        mask |= ((matrix[2][7] != 0) as u64) << 23;
        mask |= ((matrix[3][0] != 0) as u64) << 24;
        mask |= ((matrix[3][1] != 0) as u64) << 25;
        mask |= ((matrix[3][2] != 0) as u64) << 26;
        mask |= ((matrix[3][3] != 0) as u64) << 27;
        mask |= ((matrix[3][4] != 0) as u64) << 28;
        mask |= ((matrix[3][5] != 0) as u64) << 29;
        mask |= ((matrix[3][6] != 0) as u64) << 30;
        mask |= ((matrix[3][7] != 0) as u64) << 31;
        mask |= ((matrix[4][0] != 0) as u64) << 32;
        mask |= ((matrix[4][1] != 0) as u64) << 33;
        mask |= ((matrix[4][2] != 0) as u64) << 34;
        mask |= ((matrix[4][3] != 0) as u64) << 35;
        mask |= ((matrix[4][4] != 0) as u64) << 36;
        mask |= ((matrix[4][5] != 0) as u64) << 37;
        mask |= ((matrix[4][6] != 0) as u64) << 38;
        mask |= ((matrix[4][7] != 0) as u64) << 39;
        mask |= ((matrix[5][0] != 0) as u64) << 40;
        mask |= ((matrix[5][1] != 0) as u64) << 41;
        mask |= ((matrix[5][2] != 0) as u64) << 42;
        mask |= ((matrix[5][3] != 0) as u64) << 43;
        mask |= ((matrix[5][4] != 0) as u64) << 44;
        mask |= ((matrix[5][5] != 0) as u64) << 45;
        mask |= ((matrix[5][6] != 0) as u64) << 46;
        mask |= ((matrix[5][7] != 0) as u64) << 47;
        mask |= ((matrix[6][0] != 0) as u64) << 48;
        mask |= ((matrix[6][1] != 0) as u64) << 49;
        mask |= ((matrix[6][2] != 0) as u64) << 50;
        mask |= ((matrix[6][3] != 0) as u64) << 51;
        mask |= ((matrix[6][4] != 0) as u64) << 52;
        mask |= ((matrix[6][5] != 0) as u64) << 53;
        mask |= ((matrix[6][6] != 0) as u64) << 54;
        mask |= ((matrix[6][7] != 0) as u64) << 55;
        mask |= ((matrix[7][0] != 0) as u64) << 56;
        mask |= ((matrix[7][1] != 0) as u64) << 57;
        mask |= ((matrix[7][2] != 0) as u64) << 58;
        mask |= ((matrix[7][3] != 0) as u64) << 59;
        mask |= ((matrix[7][4] != 0) as u64) << 60;
        mask |= ((matrix[7][5] != 0) as u64) << 61;
        mask |= ((matrix[7][6] != 0) as u64) << 62;
        mask |= ((matrix[7][7] != 0) as u64) << 63;
        OcclusionShape8x8(mask)
    }
    
    pub fn get(self, x: usize, y: usize) -> bool {
        let index = y * 8 + x;
        self.0 & (1 << index) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 8 + x;
        let old = self.0 & (1 << index) != 0;
        self.0 = if value {
            self.0 | (1 << index)
        } else {
            self.0 & !(1 << index)
        };
        old
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape4x4(u16);

impl OcclusionShape4x4 {
    
    pub const fn new(mask: u16) -> Self {
        Self(mask)
    }

    pub const fn from_matrix(matrix: [[u8; 4]; 4]) -> Self {
        let mut mask = 0u16;
        mask |= ((matrix[0][0] != 0) as u16) << 0;
        mask |= ((matrix[0][1] != 0) as u16) << 1;
        mask |= ((matrix[0][2] != 0) as u16) << 2;
        mask |= ((matrix[0][3] != 0) as u16) << 3;
        mask |= ((matrix[1][0] != 0) as u16) << 4;
        mask |= ((matrix[1][1] != 0) as u16) << 5;
        mask |= ((matrix[1][2] != 0) as u16) << 6;
        mask |= ((matrix[1][3] != 0) as u16) << 7;
        mask |= ((matrix[2][0] != 0) as u16) << 8;
        mask |= ((matrix[2][1] != 0) as u16) << 9;
        mask |= ((matrix[2][2] != 0) as u16) << 10;
        mask |= ((matrix[2][3] != 0) as u16) << 11;
        mask |= ((matrix[3][0] != 0) as u16) << 12;
        mask |= ((matrix[3][1] != 0) as u16) << 13;
        mask |= ((matrix[3][2] != 0) as u16) << 14;
        mask |= ((matrix[3][3] != 0) as u16) << 15;
        Self(mask)
    }

    pub fn get(self, x: usize, y: usize) -> bool {
        let index = y * 4 + x;
        self.0 & (1 << index) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 4 + x;
        let old = self.0 & (1 << index) != 0;
        self.0 = if value {
            self.0 | (1 << index)
        } else {
            self.0 & !(1 << index)
        };
        old
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionShape2x2(u8);

impl OcclusionShape2x2 {
    
    pub const fn new(mask: u8) -> Self {
        Self(mask)
    }

    
    pub const fn from_matrix(matrix: [[u8; 2]; 2]) -> Self {
        let mut mask = 0u8;
        mask |= ((matrix[0][0] != 0) as u8) << 0;
        mask |= ((matrix[0][1] != 0) as u8) << 1;
        mask |= ((matrix[1][0] != 0) as u8) << 2;
        mask |= ((matrix[1][1] != 0) as u8) << 3;
        Self(mask)
    }

    pub fn get(self, x: usize, y: usize) -> bool {
        let index = y * 2 + x;
        self.0 & (1 << index) != 0
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) -> bool {
        let index = y * 2 + x;
        let old = self.0 & (1 << index) != 0;
        self.0 = if value {
            self.0 | (1 << index)
        } else {
            self.0 & !(1 << index)
        };
        old
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OcclusionRect {
    left: u8,
    top: u8,
    right: u8,
    bottom: u8
}

impl OcclusionRect {
    const FULL: OcclusionRect = OcclusionRect {
        left: 0,
        top: 0,
        right: 16,
        bottom: 16
    };
    pub fn new(x: u8, y: u8, width: u8, height: u8) -> Self {
        Self {
            left: x,
            top: y,
            right: x.checked_add(width).expect("Overflow on X axis"),
            bottom: y.checked_add(height).expect("Overflow on Y axis")
        }
    }

    pub fn from_min_max(min: (u8, u8), max: (u8, u8)) -> Self {
        Self {
            left: min.0,
            top: min.1,
            right: max.0,
            bottom: max.1
        }
    }

    pub fn contains(self, point: (u8, u8)) -> bool {
        self.left <= point.0
        && self.right > point.0
        && self.top <= point.1
        && self.bottom > point.1
    }

    pub fn intersects(self, other: OcclusionRect) -> bool {
        self.left < other.right
        && other.left < self.right
        && self.top < other.bottom
        && other.top < self.bottom
    }

    pub fn contains_rect(self, other: OcclusionRect) -> bool {
        self.left <= other.left
        && self.top <= other.top
        && self.right >= other.right
        && self.bottom >= other.bottom
    }

    /// Reduces the size of the rectangle to fit within the sample size. This stretches the bounds to fit the most space.
    pub fn downsample(self, sample_size: u8) -> Self {
        fn reduce_min(value: u8, reduction: u8) -> u8 {
            value / (16 / reduction)
        }
        fn reduce_max(value: u8, reduction: u8) -> u8 {
            let div = 16 / reduction;
            value / div + if value % div == 0 { 0 } else { 1 }
        }
        Self {
            left: reduce_min(self.left, sample_size),
            top: reduce_min(self.top, sample_size),
            right: reduce_max(self.right, sample_size),
            bottom: reduce_max(self.bottom, sample_size)
        }
    }

    
    pub fn rotate(self, angle: u8) -> Self {
        let (x1, y1) = rotate_face_coord(angle, self.left as usize, self.top as usize, 16);
        let (x2, y2) = rotate_face_coord(angle, self.right as usize, self.bottom as usize, 16);
        let xmin = x1.min(x2);
        let ymin = y1.min(y2);
        let xmax = x1.max(x2);
        let ymax = x1.max(x2);
        Self {
            left: xmin as u8,
            top: ymin as u8,
            right: xmax as u8,
            bottom: ymax as u8
        }
    }
}

pub enum OcclusionShape {
    S16x16(OcclusionShape16x16),
    S8x8(OcclusionShape8x8),
    S4x4(OcclusionShape4x4),
    S2x2(OcclusionShape2x2),
    Rect(OcclusionRect),
    Full,
    Empty
}

impl OcclusionShape {
    pub const FULL_FACES: Faces<OcclusionShape> = Faces::new(
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
        OcclusionShape::Full,
    );
    pub const EMPTY_FACES: Faces<OcclusionShape> = Faces::new(
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
        OcclusionShape::Empty,
    );

    
    pub fn is_empty(&self) -> bool {
        matches!(self, OcclusionShape::Empty)
    }

    
    pub fn is_full(&self) -> bool {
        matches!(self, OcclusionShape::Full)
    }

    
    pub fn fully_occluded(&self) -> bool {
        self.is_full() || {
            match self {
                OcclusionShape::S16x16(shape) => shape.0.iter().find(|&&v| v != u16::MAX).is_none(),
                OcclusionShape::S8x8(shape) => shape.0 == u64::MAX,
                OcclusionShape::S4x4(shape) => shape.0 == u16::MAX,
                OcclusionShape::S2x2(shape) => shape.0 == 0b1111,
                &OcclusionShape::Rect(rect) => rect == OcclusionRect::new(0, 0, 16, 16),
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => false,
            }
        }
    }

    
    pub fn occludes(&self) -> bool {
        match self {
            OcclusionShape::S16x16(shape) => shape.0.iter().find(|&&sub| sub != 0).is_some(),
            OcclusionShape::S8x8(shape) => shape.0 != 0,
            OcclusionShape::S4x4(shape) => shape.0 != 0,
            OcclusionShape::S2x2(shape) => shape.0 != 0,
            OcclusionShape::Rect(shape) => true,
            OcclusionShape::Full => true,
            OcclusionShape::Empty => false,
        }
    }

    pub fn occluded_by(&self, other: &OcclusionShape, angle: u8, other_angle: u8) -> bool {
        // What I wanted to do for occlusion is rather complicated combinatorically, so nested match
        // expressions is the way to go. There may be a better way to do it, but I'm not smart enough to know it.
        if other.is_empty() {
            return false;
        }
        if other.is_full() {
            return true;
        }
        match self {
            OcclusionShape::Full => match other {
                OcclusionShape::S16x16(shape) => shape.0.iter().find(|&&sub| sub != u16::MAX).is_none(),
                OcclusionShape::S8x8(shape) => shape.0 == u64::MAX,
                OcclusionShape::S4x4(shape) => shape.0 == u16::MAX,
                OcclusionShape::S2x2(shape) => shape.0 & 0xF == 0xF,
                OcclusionShape::Rect(shape) => *shape == OcclusionRect::FULL,
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::Rect(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.rotate(angle);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (x, y) = rotate_face_coord(other_angle, x as usize, y as usize, 16);
                            if !other.get(x, y) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.rotate(angle);
                    let sample = shape.downsample(8);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (x, y) = rotate_face_coord(other_angle, x as usize, y as usize, 8);
                            if !other.get(x, y) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.rotate(angle);
                    let sample = shape.downsample(4);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (x, y) = rotate_face_coord(other_angle, x as usize, y as usize, 4);
                            if !other.get(x, y) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::Rect(shape) => match other {
                    let shape = shape.rotate(angle);
                    let sample = shape.downsample(2);
                    for y in shape.top..shape.bottom {
                        for x in shape.left..shape.right {
                            let (x, y) = rotate_face_coord(other_angle, x as usize, y as usize, 2);
                            if !other.get(x, y) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    let shape = shape.rotate(angle);
                    let other = other.rotate(other_angle);
                    other.contains_rect(shape)
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S16x16(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        for x in 0..16 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 16);
                            let (ox, oy) = rotate_face_coord(other_angle, x, y, 16);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        let oy = y / 2;
                        for x in 0..16 {
                            let ox = x / 2;
                            let (sx, sy) = rotate_face_coord(angle, x, y, 16);
                            let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 8);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        let oy = y / 4;
                        for x in 0..16 {
                            let ox = x / 4;
                            let (sx, sy) = rotate_face_coord(angle, x, y, 16);
                            let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 4);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    for y in 0..16 {
                        let oy = y / 8;
                        for x in 0..16 {
                            let ox = x / 8;
                            let (sx, sy) = rotate_face_coord(angle, x, y, 16);
                            let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 2);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S16x16(shape) => match other {
                    let other = other.rotate(other_angle);
                    for y in 0..16 {
                        for x in 0..16 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 16);
                            if shape.get(sx, sy) && !other.contains((x as u8, y as u8)) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S8x8(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y * 2;
                        for x in 0..8 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 8);
                            if shape.get(sx, sy) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 16);
                                        if !other.get(ox, oy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },            
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        for x in 0..8 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 8);
                            let (ox, oy) = rotate_face_coord(other_angle, x, y, 8);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y / 2;
                        for x in 0..8 {
                            let ox = x / 2;
                            let (sx, sy) = rotate_face_coord(angle, x, y, 8);
                            let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 4);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    for y in 0..8 {
                        let oy = y / 4;
                        for x in 0..8 {
                            let ox = x / 4;
                            let (sx, sy) = rotate_face_coord(angle, x, y, 8);
                            let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 2);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S8x8(shape) => match other {
                    let other = other.rotate(other_angle);
                    for y in 0..8 {
                        for x in 0..8 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 8);
                            if shape.get(sx, sy) {
                                let inner = OcclusionRect::from_min_max(
                                    (x as u8 * 2, y as u8 * 2),
                                    (x as u8 * 2 + 2, y as u8 * 2 + 2)
                                );
                                if !other.contains_rect(inner) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S4x4(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y * 4;
                        for x in 0..4 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 4);
                            if shape.get(sx, sy) {
                                let ox = x * 4;
                                for oy in oy..oy+4 {
                                    for ox in ox..ox+4 {
                                        let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 16);
                                        if !other.get(ox, oy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y * 2;
                        for x in 0..4 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 4);
                            if shape.get(sx, sy) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 8);
                                        if !other.get(ox, oy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        for x in 0..4 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 4);
                            let (ox, oy) = rotate_face_coord(other_angle, x, y, 4);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    for y in 0..4 {
                        let oy = y / 2;
                        for x in 0..4 {
                            let ox = x / 2;
                            let (sx, sy) = rotate_face_coord(angle, x, y, 4);
                            let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 2);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S4x4(shape) => match other {
                    let other = other.rotate(other_angle);
                    for y in 0..4 {
                        for x in 0..4 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 4);
                            if shape.get(sx, sy) {
                                let inner = OcclusionRect::from_min_max(
                                    (x as u8 * 4, y as u8 * 4),
                                    (x as u8 * 4 + 4, y as u8 * 4 + 4)
                                );
                                if !other.contains_rect(inner) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::S2x2(shape) => match other {
                OcclusionShape::S16x16(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 8;
                        for x in 0..2 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 2);
                            if shape.get(sx, sy) {
                                let ox = x * 8;
                                for oy in oy..oy+8 {
                                    for ox in ox..ox+8 {
                                        let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 16);
                                        if !other.get(ox, oy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S8x8(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 4;
                        for x in 0..2 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 2);
                            if shape.get(sx, sy) {
                                let ox = x * 4;
                                for oy in oy..oy+4 {
                                    for ox in ox..ox+4 {
                                        let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 8);
                                        if !other.get(ox, oy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S4x4(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        let oy = y * 2;
                        for x in 0..2 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 2);
                            if shape.get(sx, sy) {
                                let ox = x * 2;
                                for oy in oy..oy+2 {
                                    for ox in ox..ox+2 {
                                        let (ox, oy) = rotate_face_coord(other_angle, ox, oy, 4);
                                        if !other.get(ox, oy) {
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::S2x2(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    for y in 0..2 {
                        for x in 0..2 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 2);
                            let (ox, oy) = rotate_face_coord(other_angle, x, y, 2);
                            if shape.get(sx, sy) && !other.get(ox, oy) {
                                return false;
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Rect(other) => {
                    // OcclusionShape::S2x2(shape) => match other {
                    let other = other.rotate(other_angle);
                    for y in 0..2 {
                        for x in 0..2 {
                            let (sx, sy) = rotate_face_coord(angle, x, y, 2);
                            if shape.get(sx, sy) {
                                let inner = OcclusionRect::from_min_max(
                                    (x as u8 * 8, y as u8 * 8),
                                    (x as u8 * 8 + 8, y as u8 * 8 + 8)
                                );
                                if !other.contains_rect(inner) {
                                    return false;
                                }
                            }
                        }
                    }
                    true
                },
                OcclusionShape::Full => unreachable!(),
                OcclusionShape::Empty => unreachable!(),
            },
            OcclusionShape::Empty => {
                other.fully_occluded()
            },
        }
    }
}

impl From<[[u8; 2]; 2]> for OcclusionShape {
    fn from(value: [[u8; 2]; 2]) -> Self {
        let mut occluder = OcclusionShape2x2::default();
        for y in 0..2 {
            for x in 0..2 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S2x2(occluder)
    }
}

impl From<[[u8; 4]; 4]> for OcclusionShape {
    fn from(value: [[u8; 4]; 4]) -> Self {
        let mut occluder = OcclusionShape4x4::default();
        for y in 0..4 {
            for x in 0..4 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S4x4(occluder)
    }
}

impl From<[[u8; 8]; 8]> for OcclusionShape {
    fn from(value: [[u8; 8]; 8]) -> Self {
        let mut occluder = OcclusionShape8x8::default();
        for y in 0..8 {
            for x in 0..8 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S8x8(occluder)
    }
}

impl From<[[u8; 16]; 16]> for OcclusionShape {
    fn from(value: [[u8; 16]; 16]) -> Self {
        let mut occluder = OcclusionShape16x16::default();
        for y in 0..16 {
            for x in 0..16 {
                if value[y][x] != 0 {
                    occluder.set(x, y, true);
                }
            }
        }
        OcclusionShape::S16x16(occluder)
    }
}

impl From<OcclusionShape2x2> for OcclusionShape {
    fn from(value: OcclusionShape2x2) -> Self {
        OcclusionShape::S2x2(value)
    }
}

impl From<OcclusionShape4x4> for OcclusionShape {
    fn from(value: OcclusionShape4x4) -> Self {
        OcclusionShape::S4x4(value)
    }
}

impl From<OcclusionShape8x8> for OcclusionShape {
    fn from(value: OcclusionShape8x8) -> Self {
        OcclusionShape::S8x8(value)
    }
}

impl From<OcclusionShape16x16> for OcclusionShape {
    fn from(value: OcclusionShape16x16) -> Self {
        OcclusionShape::S16x16(value)
    }
}

impl From<OcclusionRect> for OcclusionShape {
    fn from(value: OcclusionRect) -> Self {
        OcclusionShape::Rect(value)
    }
}

#[test]
fn occlusion_test() {
    let occluder_a = OcclusionShape::from([
        [1,1,0,0],
        [1,1,0,0],
        [1,1,0,0],
        [1,1,0,0]
    ]);
    let occluder_b = OcclusionShape::from([
        [1,0],
        [1,0],
    ]);
    if occluder_a.occluded_by(&occluder_b, 0, 0) {
        println!("A Occluded by B");
    }
    if occluder_b.occluded_by(&occluder_a, 0, 0) {
        println!("B Occluded by A");
    }
}

#[test]
fn reduce_rect_test() {
    fn reduce_min(value: i32, base: i32, reduction: i32) -> i32 {
        value / (base / reduction)
    }
    fn reduce_max(value: i32, base: i32, reduction: i32) -> i32 {
        let div = base / reduction;
        value / div + if value % div == 0 { 0 } else { 1 }
    }
    let base = 16;
    let reduction = 8;
    let (left, right) = (2, 14);
    let (rl, rr) = (
        reduce_min(left, base, reduction),
        reduce_max(right, base, reduction)
    );
    println!("{rl}, {rr}");
}