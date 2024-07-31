use std::ops::{Index, IndexMut};

use rand::Rng;

pub enum Direction {
    North,
    South,
    East,
    West,
}

pub struct MazeCell {
    pub north: bool,
    pub south: bool,
    pub east: bool,
    pub west: bool,
    pub visited: bool,
}

pub struct Maze {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<MazeCell>,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Maze {
        let mut cells = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            cells.push(MazeCell {
                north: false,
                south: false,
                east: false,
                west: false,
                visited: false,
            });
        }
        Maze {
            width,
            height,
            cells,
        }
    }

    fn get_cell_neighbors_when(
        &self,
        x: usize,
        y: usize,
        f: impl Fn(&MazeCell) -> bool,
    ) -> Vec<(usize, usize)> {
        let mut neighbors: Vec<(usize, usize)> = Vec::new();
        if x > 0 && f(&self[(x - 1, y)]) {
            neighbors.push((x - 1, y));
        }
        if x < self.width - 1 && f(&self[(x + 1, y)]) {
            neighbors.push((x + 1, y));
        }
        if y > 0 && f(&self[(x, y - 1)]) {
            neighbors.push((x, y - 1));
        }
        if y < self.height - 1 && f(&self[(x, y + 1)]) {
            neighbors.push((x, y + 1));
        }
        neighbors
    }

    fn get_cell_unvisited_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        self.get_cell_neighbors_when(x, y, |cell| !cell.visited)
    }

    fn get_cell_visited_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        self.get_cell_neighbors_when(x, y, |cell| cell.visited)
    }

    fn get_cell_unvisited_neighbors_with_visited_neighbors(
        &self,
        x: usize,
        y: usize,
    ) -> Vec<(usize, usize)> {
        let mut neighbors = Vec::new();
        for (nx, ny) in self.get_cell_unvisited_neighbors(x, y) {
            if !self.get_cell_visited_neighbors(nx, ny).is_empty() {
                neighbors.push((nx, ny));
            }
        }
        neighbors
    }

    pub fn generate(&mut self, x: usize, y: usize) {
        let mut stack = Vec::with_capacity(self.width * self.height);
        stack.push((x, y));

        // Pop the current cell
        while let Some((x, y)) = stack.pop() {
            // Mark the current cell as visited
            self[(x, y)].visited = true;
            // Get the unvisited neighbors with visited neighbors
            let mut neighbors = self.get_cell_unvisited_neighbors_with_visited_neighbors(x, y);
            // If there are no unvisited neighbors with visited neighbors, continue
            if neighbors.is_empty() {
                continue;
            }
            // Push the current cell back onto the stack
            stack.push((x, y));

            // Choose a random unvisited neighbor and remove walls between the current cell and the neighbor
            let (nx, ny) = neighbors.remove(rand::thread_rng().gen_range(0..neighbors.len()));
            let direction = if nx > x {
                Direction::East
            } else if nx < x {
                Direction::West
            } else if ny > y {
                Direction::South
            } else {
                Direction::North
            };
            match direction {
                Direction::North => {
                    self[(x, y)].north = true;
                    self[(nx, ny)].south = true;
                }
                Direction::East => {
                    self[(x, y)].east = true;
                    self[(nx, ny)].west = true;
                }
                Direction::West => {
                    self[(x, y)].west = true;
                    self[(nx, ny)].east = true;
                }
                Direction::South => {
                    self[(x, y)].south = true;
                    self[(nx, ny)].north = true;
                }
            }
            // Push the neighbor onto the stack as the current cell for the next iteration
            stack.push((nx, ny));
        }
        // Clear all visited flags
        for cell in &mut self.cells {
            cell.visited = false;
        }
    }
}

impl Index<(usize, usize)> for Maze {
    type Output = MazeCell;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.cells[index.1 * self.width + index.0]
    }
}

impl IndexMut<(usize, usize)> for Maze {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.cells[index.1 * self.width + index.0]
    }
}
