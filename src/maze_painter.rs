use std::cmp::min;

use embedded_graphics::{
    prelude::{PixelColor, Point, Size},
    primitives::{Circle, Line, Primitive, PrimitiveStyle, StyledDrawable},
    transform::Transform,
    Drawable,
};
use log::info;

use crate::maze::Maze;

const CLICK_APPROXIMATION: u32 = 20;

pub struct MazePainter<C: PixelColor> {
    pub maze: Maze,
    pub style: PrimitiveStyle<C>,
    pub cell_size: Size,
    pub offset: Point,
}

impl<C: PixelColor> MazePainter<C> {
    pub fn new(mut maze: Maze, style: PrimitiveStyle<C>, cell_size: Size, offset: Point) -> Self {
        maze[(0, 0)].visited = true;
        Self {
            maze,
            style,
            cell_size,
            offset,
        }
    }

    pub fn draw_marker<D>(
        &self,
        x: usize,
        y: usize,
        style: &PrimitiveStyle<D::Color>,
        target: &mut D,
    ) where
        D: embedded_graphics::prelude::DrawTarget<Color = C>,
    {
        Circle::new(
            Point::new(
                x as i32 * self.cell_size.width as i32 + 2,
                y as i32 * self.cell_size.height as i32 + 2,
            ),
            min(self.cell_size.width, self.cell_size.height) - 3,
        )
        .translate(self.offset)
        .draw_styled(style, target)
        .ok();
    }

    fn get_cell_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut neighbors: Vec<(usize, usize)> = Vec::new();
        if x > 0 && self.maze[(x, y)].west {
            neighbors.push((x - 1, y));
        }
        if x < self.maze.width - 1 && self.maze[(x, y)].east {
            neighbors.push((x + 1, y));
        }
        if y > 0 && self.maze[(x, y)].north {
            neighbors.push((x, y - 1));
        }
        if y < self.maze.height - 1 && self.maze[(x, y)].south {
            neighbors.push((x, y + 1));
        }
        neighbors
    }

    fn is_cell_clickable(&self, x: usize, y: usize) -> bool {
        if x >= self.maze.width || y >= self.maze.height {
            return false;
        }
        if self.maze[(x, y)].visited {
            return false;
        }
        let neighbors = self.get_cell_neighbors(x, y);
        for n in neighbors {
            if self.maze[(n.0, n.1)].visited {
                return true;
            }
        }
        false
    }

    fn get_cell_central_point(&self, x: usize, y: usize) -> (i32, i32) {
        (
            x as i32 * self.cell_size.width as i32
                + self.cell_size.width as i32 / 2
                + self.offset.x,
            y as i32 * self.cell_size.height as i32
                + self.cell_size.height as i32 / 2
                + self.offset.y,
        )
    }

    /**
     * Extend the point by `+/- extent` in both directions
     * and return clickable cells that are covered by the range.
     */
    fn get_covered_cells_with_approximation(
        &self,
        x: i32,
        y: i32,
        extent: u32,
    ) -> Vec<(usize, usize)> {
        let mut cells: Vec<(usize, usize)> = Vec::new();

        let central_cell_col = (x - self.offset.x) as u32 / self.cell_size.width;
        let central_cell_row = (y - self.offset.y) as u32 / self.cell_size.height;

        let cell_col_range = extent / self.cell_size.width;
        let cell_row_range = extent / self.cell_size.height;

        let min_col = if central_cell_col > cell_col_range {
            central_cell_col - cell_col_range
        } else {
            0
        };

        let max_col = if central_cell_col + cell_col_range < self.maze.width as u32 {
            central_cell_col + cell_col_range
        } else {
            self.maze.width as u32 - 1
        };

        let min_row = if central_cell_row > cell_row_range {
            central_cell_row - cell_row_range
        } else {
            0
        };

        let max_row = if central_cell_row + cell_row_range < self.maze.height as u32 {
            central_cell_row + cell_row_range
        } else {
            self.maze.height as u32 - 1
        };

        for col in min_col..=max_col {
            for row in min_row..=max_row {
                if self.is_cell_clickable(col as usize, row as usize) {
                    cells.push((col as usize, row as usize));
                }
            }
        }
        cells
    }

    fn point_to_cell(&self, mut x: i32, mut y: i32) -> Option<(usize, usize)> {
        if x < self.offset.x {
            x = self.offset.x;
        }
        if y < self.offset.y {
            y = self.offset.y;
        }

        let cell_col = (x - self.offset.x) as u32 / self.cell_size.width;
        let cell_row = (y - self.offset.y) as u32 / self.cell_size.height;
        if cell_col >= self.maze.width as u32 || cell_row >= self.maze.height as u32 {
            return None;
        }
        Some((cell_col as usize, cell_row as usize))
    }

    /**
     * The touch screen on the board is too small, so we need to approximate the touch point,
     * make it easier to click on the cell
     */
    fn get_closest_clickable_cell(&self, x: i32, y: i32, extent: u32) -> Option<(usize, usize)> {
        // Do not apply extent if the current cell is already visited.
        if let Some(cell) = self.point_to_cell(x, y) {
            if self.maze[cell].visited {
                return None;
            }
        } else {
            return None;
        };

        // Get all clickable cells in the extent
        let mut cells = self.get_covered_cells_with_approximation(x, y, extent);
        if cells.is_empty() {
            return None;
        }
        // Pick the closest one
        cells.sort_by(|a, b| {
            let a_dist = dist_sq(self.get_cell_central_point(a.0, a.1), (x, y));
            let b_dist = dist_sq(self.get_cell_central_point(b.0, b.1), (x, y));
            a_dist.cmp(&b_dist)
        });
        info!("Closest cell is {:?}", cells[0]);
        Some(cells[0])
    }

    pub fn on_click<D>(
        &mut self,
        x: i32,
        y: i32,
        style: PrimitiveStyle<D::Color>,
        target: &mut D,
    ) -> bool
    where
        D: embedded_graphics::prelude::DrawTarget<Color = C>,
    {
        let cell = self.get_closest_clickable_cell(x, y, CLICK_APPROXIMATION);
        if cell.is_none() {
            return false;
        }
        let (x, y) = cell.unwrap();
        if x >= self.maze.width || y >= self.maze.height {
            return false;
        }
        let neighbors = self.get_cell_neighbors(x, y);
        info!("Cell {:?} has neighbors {:?}", (x, y), neighbors);
        let mut ret = false;
        for n in neighbors {
            if self.maze[(n.0, n.1)].visited {
                // Draw a line from (x, y) to (n.0, n.1)
                let line = Line::new(
                    Point::new(
                        x as i32 * self.cell_size.width as i32 + self.cell_size.width as i32 / 2,
                        y as i32 * self.cell_size.height as i32 + self.cell_size.height as i32 / 2,
                    ),
                    Point::new(
                        n.0 as i32 * self.cell_size.width as i32 + self.cell_size.width as i32 / 2,
                        n.1 as i32 * self.cell_size.height as i32
                            + self.cell_size.height as i32 / 2,
                    ),
                );
                info!("Drawing line from {:?} to {:?}", (x, y), n);
                line.translate(self.offset).draw_styled(&style, target).ok();
                self.maze[(x, y)].visited = true;
                ret = true;
            }
        }
        ret
    }
}

impl<C: PixelColor> Drawable for MazePainter<C> {
    type Color = C;

    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
    {
        for y in 0..self.maze.height {
            for x in 0..self.maze.width {
                let cell = &self.maze[(x, y)];
                let x = x as i32;
                let y = y as i32;
                if !cell.north {
                    Line::new(
                        Point::new(
                            x * self.cell_size.width as i32,
                            y * self.cell_size.height as i32,
                        ),
                        Point::new(
                            (x + 1) * self.cell_size.width as i32,
                            y * self.cell_size.height as i32,
                        ),
                    )
                    .translate(self.offset)
                    .into_styled(self.style)
                    .draw(target)
                    .ok();
                }
                if !cell.south {
                    Line::new(
                        Point::new(
                            x * self.cell_size.width as i32,
                            (y + 1) * self.cell_size.height as i32,
                        ),
                        Point::new(
                            (x + 1) * self.cell_size.width as i32,
                            (y + 1) * self.cell_size.height as i32,
                        ),
                    )
                    .translate(self.offset)
                    .into_styled(self.style)
                    .draw(target)
                    .ok();
                }
                if !cell.east {
                    Line::new(
                        Point::new(
                            (x + 1) * self.cell_size.width as i32,
                            y * self.cell_size.height as i32,
                        ),
                        Point::new(
                            (x + 1) * self.cell_size.width as i32,
                            (y + 1) * self.cell_size.height as i32,
                        ),
                    )
                    .translate(self.offset)
                    .into_styled(self.style)
                    .draw(target)
                    .ok();
                }
                if !cell.west {
                    Line::new(
                        Point::new(
                            x * self.cell_size.width as i32,
                            y * self.cell_size.height as i32,
                        ),
                        Point::new(
                            x * self.cell_size.width as i32,
                            (y + 1) * self.cell_size.height as i32,
                        ),
                    )
                    .translate(self.offset)
                    .into_styled(self.style)
                    .draw(target)
                    .ok();
                }
            }
        }
        // Draw start and end markers
        self.draw_marker(0, 0, &self.style, target);
        self.draw_marker(
            self.maze.width - 1,
            self.maze.height - 1,
            &self.style,
            target,
        );
        Ok(())
    }
}

fn dist_sq<X1, Y1, X2, Y2>(a: (X1, Y1), b: (X2, Y2)) -> u32
where
    X1: Into<i32>,
    Y1: Into<i32>,
    X2: Into<i32>,
    Y2: Into<i32>,
{
    ((a.0.into() - b.0.into()).pow(2) + (a.1.into() - b.1.into()).pow(2))
        .try_into()
        .unwrap()
}
