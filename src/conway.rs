use crate::window::ArrowKeys;

use super::window::Window;
use anyhow::Result;
use ncurses::*;
use rand::{rngs::ThreadRng, Rng};
use std::{collections::HashSet, path::Path};

/// A cell in the grid of the game.
/// Contains the x and y coordinates of the cell, and whether the cell is alive or dead.
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    /// x-coordinate of the cell
    pub x: usize,
    /// y-coordinate of the cell
    pub y: usize,
    /// Whether the cell is alive or dead
    pub alive: bool,
}

impl Cell {
    pub fn new(x: usize, y: usize, alive: bool) -> Cell {
        Cell { x, y, alive }
    }

    pub fn is_alive(&self) -> bool {
        self.alive
    }

    pub fn set_alive(&mut self) {
        self.alive = true;
    }

    pub fn set_dead(&mut self) {
        self.alive = false;
    }

    pub fn count_alive_neighbors(&self, grid: &[Vec<Cell>]) -> usize {
        //! Counts the number of alive neighbors of the cell.
        //! A neighbor can be immediately next to the cell, or diagonally adjacent to it.
        //! A neighbor can also wrap around the edges of the grid.
        let nrows: usize = grid.len();
        let ncols: usize = grid[0].len();
        let mut count: usize = 0;
        for i in -1..=1 {
            for j in -1..=1 {
                // Skip the cell itself
                if i == 0 && j == 0 {
                    continue;
                }
                // calculate the next cell to check
                let mut x = self.x as i32 + j;
                let mut y = self.y as i32 + i;

                // wrap around the edges
                if x < 0 {
                    x = nrows as i32 - 1;
                } else if x >= nrows as i32 {
                    x = 0;
                }
                if y < 0 {
                    y = ncols as i32 - 1;
                } else if y >= ncols as i32 {
                    y = 0;
                }

                // check if the cell is alive
                if grid[x as usize][y as usize].is_alive() {
                    count += 1;
                }
            }
        }
        count
    }
}

pub fn draw(window: &mut Window, grid: &[Vec<Cell>], state: &State) -> Result<()> {
    //! Draws the grid on the screen
    //!
    //! # Arguments
    //! * `grid` - The grid to draw
    //! * `nrows` - Number of rows in the grid
    //! * `ncols` - Number of columns in the grid
    //! * `input_handler` - Input handler to get the character to draw for alive cells
    for i in 0..grid.len() {
        for j in 0..grid[0].len() {
            let output = format!(
                "{}",
                if grid[i][j].is_alive() {
                    state.draw_char
                } else {
                    ' '
                }
            );
            window.print(j as i32 * 2, i as i32, &output, None)?;
        }
    }
    let num_alive: usize = grid.iter().flatten().filter(|cell| cell.is_alive()).count();
    window.print(
        0,
        grid.len() as i32,
        &format!(
            "Alive: {}, Timeout: {} | q: Quit, a: increase timeout, s: decrease timeout",
            num_alive, state.timeout
        ),
        None,
    )
}

pub struct State {
    timeout: i32,
    draw_char: char,
}

impl State {
    pub fn new(timeout: i32, draw_char: char) -> State {
        State { timeout, draw_char }
    }

    pub fn get_timeout(&self) -> i32 {
        self.timeout
    }

    pub fn get_draw_char(&self) -> char {
        self.draw_char
    }

    pub fn set_timeout(&mut self, timeout: i32) {
        self.timeout = timeout;
    }

    pub fn set_draw_char(&mut self, draw_char: char) {
        self.draw_char = draw_char;
    }
}

pub struct InputHandler {
    input: InputType,
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler {
            input: InputType::Continue,
        }
    }

    pub fn handle_input(&mut self, state: &mut State) -> Result<InputType> {
        let c: i32 = getch();
        self.input = if c == ArrowKeys::Down as i32 || c == 'j' as i32 {
            InputType::Down
        } else if c == ArrowKeys::Up as i32 || c == 'k' as i32 {
            InputType::Up
        } else {
            match c as u8 as char {
                'q' => InputType::Quit,
                'a' => InputType::IncreaseTimeout,
                's' => InputType::DecreaseTimeout,
                _ => InputType::Continue,
            }
        };

        match self.input {
            InputType::Quit | InputType::Continue => (),
            InputType::IncreaseTimeout => {
                // Increase timeout
                if state.timeout < 1000 {
                    state.timeout += 10;
                }
                timeout(state.timeout);
            }
            InputType::DecreaseTimeout => {
                // Decrease timeout
                if state.timeout > 10 {
                    state.timeout -= 10;
                }
                timeout(state.timeout);
            }
            _ => (),
        }

        Ok(self.input)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputType {
    Quit,
    Continue,
    IncreaseTimeout,
    DecreaseTimeout,
    Up,
    Down,
}

pub fn initialize(
    window: &mut Window,
    num_alive: Option<usize>,
    seed_file: &Option<String>,
) -> Result<Vec<Vec<Cell>>> {
    //! Initializes the grid with the given number of alive cells or seed file.
    let mut grid: Vec<Vec<Cell>> = vec![];
    let nrows: usize = window.get_rows() as usize - 1; // -1 to account for status bar at bottom
    let ncols: usize = window.get_cols() as usize;
    for i in 0..nrows {
        grid.push(vec![]);
        for j in 0..ncols / 2 {
            // /2 to account for space between characters
            grid[i].push(Cell::new(i, j, false));
        }
    }

    if seed_file.is_some() && Path::new(&seed_file.clone().unwrap()).exists() {
        // Read the seed file and set the cells to alive based on the seed file.
        let seed: String = std::fs::read_to_string(seed_file.clone().unwrap()).unwrap();
        for (rownum, line) in seed.lines().enumerate() {
            if rownum >= nrows {
                break;
            }
            let cells: Vec<char> = line.chars().collect::<Vec<char>>();
            for (colnum, cell) in cells.iter().enumerate() {
                if colnum >= ncols {
                    break;
                }
                if *cell == '*' {
                    grid[rownum][colnum].set_alive();
                }
            }
        }
    } else if (seed_file.is_some() && !Path::new(&seed_file.clone().unwrap()).exists())
        || num_alive.is_some()
    {
        // Set the cells to alive randomly based on the number of alive cells.
        if num_alive.unwrap() > grid.len() * grid[0].len() {
            endwin();
            return Err(anyhow::anyhow!(
                "Number of alive cells cannot be greater than the number of cells in the grid."
            ));
        }
        let mut rng: ThreadRng = rand::thread_rng();
        let mut alive_cells: HashSet<(usize, usize)> = HashSet::new();
        while alive_cells.len() < num_alive.unwrap() {
            let i: usize = rng.gen::<usize>() % nrows;
            let j: usize = rng.gen::<usize>() % (ncols / 2);
            alive_cells.insert((i, j));
        }
        for (i, j) in alive_cells {
            grid[i][j].set_alive();
        }
    } else {
        endwin();
        return Err(anyhow::anyhow!("Invalid arguments."));
    }

    Ok(grid)
}

pub fn calc_next_frame(grid: &[Vec<Cell>]) -> Vec<Vec<Cell>> {
    //! Calculates the next frame of the game, returning a new grid.
    let mut next_frame: Vec<Vec<Cell>> = grid.to_vec();

    grid.iter().for_each(|row| {
        row.iter().for_each(|cell| {
            let count = cell.count_alive_neighbors(grid);
            if cell.is_alive() {
                if !(2..=3).contains(&count) {
                    next_frame[cell.x][cell.y].set_dead();
                }
            } else if count == 3 {
                next_frame[cell.x][cell.y].set_alive();
            }
        })
    });
    next_frame
}

pub fn run_frame(
    window: &mut Window,
    grid: &[Vec<Cell>],
    input_handler: &mut InputHandler,
    state: &mut State,
) -> Result<(InputType, Vec<Vec<Cell>>)> {
    //! Runs a single loop of the game, drawing the grid, calculating the next
    //! frame, and getting input from the user.
    window.erase();
    draw(window, grid, state)?;
    window.refresh();
    let next_grid = calc_next_frame(grid);
    let input: InputType = input_handler.handle_input(state)?;
    Ok((input, next_grid))
}
