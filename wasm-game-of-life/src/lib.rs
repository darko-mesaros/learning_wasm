mod utils;

use core::fmt;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)] // makes each cell a single byte
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

// Exposing to JavaScript
#[wasm_bindgen]
impl Universe {
    // The new function is rather simple. It generates a new Universe, with some pre-defined alive
    // out cells. 
    //
    // This should maybe be a "Default" implementation, but okay
    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let cells = (0..width * height) // This makes a Range
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
        Universe {
            width,
            height,
            cells,
        }
    }

    // Just adding the render function that returns a String
    pub fn render(&self) -> String {
        self.to_string()
    }

    // How the tick function works: 
    //
    // This is rather simple, but I will note a few things here.
    // We clone the `cells` property as we are replacing it after we are done. Hence the
    // `self.cells.clone()`.
    //
    // We perform all the operations, and checks and then either return the `next_cell` as Alive or
    // Dead.
    // Then we replace the value in the position (`idx`) of the cloned `cells`, with that new
    // value. And lastly we replace the self.cells with the updated `cells` (from `next`)
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbors dies, as if caused
                    // by underpopulation
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbors lives on to the next
                    // generation
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than 3 live neighbors, dies, as if caused by
                    // overpopulation
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly 3 live neighbors becomes a live cell, as
                    // if by reproduction
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain the same state
                    (otherwise, _) => otherwise,
                };
                next[idx] = next_cell;
            }
        }

        self.cells = next;

    }
}

// These are not directly exposed to JavaScript
impl Universe {
    // Here is how the get_index function works:
    //
    // BOTH ROWS AND COLUMNS ARE ZERO INDEXED
    //
    // 3 x 3 universe presented linearly
    //   0   1   2   3   4   5   6   7   8
    // |---|---|---|---|---|---|---|---|---|
    // |   Row 0   |   Row 1   |   Row 2   |
    //
    // 3 x 3 universe presented 2D
    //
    //             c (1)
    //       |---|---|---|
    //       |---|---|---|
    // r (2) |---|-x-|---|
    //
    // (row * self.width + column) = (2 * 3 + 1) = 6 + 2 = 8
    //                               x (8)
    // |---|---|---|---|---|---|---|---|---|
    // |   Row 0   |   Row 1   |   Row 2   |
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize

    }

    // Here is how the live_neighbor_count function works:
    // It checks the neightbors of a given point in the universe. Basically everything around the X
    // here.
    //
    // |-*-|-*-|-*-|
    // |-*-|-X-|-*-|
    // |-*-|-*-|-*-|
    //
    // If it gets 0 for both delta_row and delta_col it skips the check as that is the center (the
    // cell itself).
    //
    // There is a big deal here with modulo and making sure the universe wraps. Say we have a 5x5
    // universe, and our cell is at X:
    //     0   1   2   3
    // 0 |-a-|-b-|-c-|-d-|
    // 1 |-e-|-f-|-g-|-X-|
    // 2 |-i-|-j-|-k-|-l-|
    // 3 |-m-|-n-|-o-|-p-|
    // 4 |-q-|-r-|-s-|-t-|
    //
    // let neighbor_col = (3 + (-1)) % 4; // = 2; where the 'c,g,k' chars are
    // let neighbor_col = (3 + 0) % 4; // = 3; where the 'd,l' chars are
    // let neighbor_col = (3 + 1) % 4; // = 0; wraps around, where the 'a,e,i' chars are
    //
    // This means if we were to check the diagonal up from X:
    // let neighbor_row = (0 + (-1)) % 4; // = 0; wraps around, where the 'a,b,c,d' chars are
    // let neighbor_col = (3 + 1) % 4; // = 0; wraps around, where the 'a,e,i' chars are
    // Meaning we get the get_index(0, 0);
    //
    //
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() { // Represents moving UP (-1)
                                                                   // staying (0), and moving DOWN (+1)
            for delta_col in [self.width - 1, 0, 1].iter().cloned() { // Same here moving LEFT (-1)
                                                                      // staying (0), moving RIGHT (+1)
                 if delta_row == 0 && delta_col == 0 {
                     continue;
                 }

                 let neighbor_row = (row + delta_row) % self.height;
                 let neighbor_col = (column + delta_col) % self.width;
                 let idx = self.get_index(neighbor_row, neighbor_col);
                 count += self.cells[idx] as u8;
            }
        }
        count
    }

}

impl fmt::Display for Universe {
    // A but on how the fmt is implemented here:
    //
    // First off Slicing - we are slicing the `self.cells` so we can work better with the chunks.
    // Meaning we will take a Vector and convert it into a slice:
    // let vec = vec![1,2,3,4,5,6,7,8,9];
    // let slice = vec.as_slice(); // &[1,2,3,4,5,6,7,8,9]
    //
    // Now when telling it to `chunk(self.width as usize)` - we are basically telling it to take
    // that 1D slice (array) and cut it up in to what ever the width of the universe is. 
    // Let's say the with is 2, we get something like this:
    // [1, 2, 3]
    // [4, 5, 6]
    // [7, 8, 9]
    //
    // Then with the first `write!()` macro we are writing either of the two characters. And after
    // each line (chunk) we are just adding a new line.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                // let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                let symbol = if cell == Cell::Dead { '⬜' } else { '🟪' };
                // let symbol = if cell == Cell::Dead { '0' } else { '1' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
