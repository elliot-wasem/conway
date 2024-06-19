/*
 * Rules:
 * - Any live cell with fewer than two live neighbours dies, as if by underpopulation.
 * - Any live cell with two or three live neighbours lives on to the next generation.
 * - Any live cell with more than three live neighbours dies, as if by overpopulation.
 * - Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
 *
 * Due to the "infinite" nature of the game, this implementation simply uses wrapping edges.
 * */

use anyhow::Result;
use clap::Parser;
use ncurses::*;
use rand::{rngs::ThreadRng, Rng};
use std::{collections::HashSet, path::Path};

/// Conway's Game of Life
///
/// A simple implementation of Conway's Game of Life using ncurses.
#[derive(Parser)]
struct Cli {
    /// Number of alive cells to start with
    #[clap(short = 'a', long = "alive", default_value = "1000")]
    alive: Option<usize>,
    /// Seed file to start with
    #[clap(short = 's', long = "seed", default_value = "None")]
    seed_file: Option<String>,
    /// Timeout in milliseconds
    #[clap(short = 't', long = "timeout", default_value = "100")]
    timeout: i32,
    /// What character to use to draw each cell
    #[clap(short = 'c', long = "character", default_value = "*")]
    character: char,
}

fn draw(grid: &[Vec<Cell>], input_handler: &InputHandler) -> Result<()> {
    //! Draws the grid on the screen
    //!
    //! # Arguments
    //! * `grid` - The grid to draw
    //! * `nrows` - Number of rows in the grid
    //! * `ncols` - Number of columns in the grid
    for i in 0..grid.len() {
        for j in 0..grid[0].len() {
            let output = format!(
                "{}",
                if grid[i][j].is_alive() {
                    input_handler.draw_char
                } else {
                    ' '
                }
            );
            mvaddstr(i as i32, (j * 2) as i32, &output)?;
        }
    }
    let num_alive = grid.iter().flatten().filter(|cell| cell.is_alive()).count();
    mvaddstr(
        grid.len() as i32,
        0,
        &format!(
            "Alive: {}, Timeout: {} | q: Quit, a: increase timeout, s: decrease timeout",
            num_alive, input_handler.timeout
        ),
    )?;
    Ok(())
}

fn get_next_char() -> char {
    //! Gets the next character from the user
    getch() as u8 as char
}

/// A cell in the grid of the game.
/// Contains the x and y coordinates of the cell, and whether the cell is alive or dead.
#[derive(Debug, Clone, Copy)]
struct Cell {
    /// x-coordinate of the cell
    x: usize,
    /// y-coordinate of the cell
    y: usize,
    /// Whether the cell is alive or dead
    alive: bool,
}

impl Cell {
    fn new(x: usize, y: usize, alive: bool) -> Cell {
        Cell { x, y, alive }
    }

    fn is_alive(&self) -> bool {
        self.alive
    }

    fn set_alive(&mut self) {
        self.alive = true;
    }

    fn set_dead(&mut self) {
        self.alive = false;
    }

    fn count_alive_neighbors(&self, grid: &[Vec<Cell>]) -> usize {
        //! Counts the number of alive neighbors of the cell.
        //! A neighbor can be immediately next to the cell, or diagonally adjacent to it.
        //! A neighbor can also wrap around the edges of the grid.
        let nrows = grid.len();
        let ncols = grid[0].len();
        let mut count: usize = 0;
        for i in -1..=1 {
            for j in -1..=1 {
                // Skip the cell itself
                if i == 0 && j == 0 {
                    continue;
                }
                // calculate the next cell to check
                let mut x = self.x as i32 + i;
                let mut y = self.y as i32 + j;

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

fn initialize(
    nrows: usize,
    ncols: usize,
    num_alive: Option<usize>,
    seed_file: &Option<String>,
) -> Result<Vec<Vec<Cell>>> {
    //! Initializes the grid with the given number of alive cells or seed file.
    let mut grid: Vec<Vec<Cell>> = vec![];
    for i in 0..nrows {
        grid.push(vec![]);
        for j in 0..ncols {
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
        if num_alive.unwrap() > nrows * ncols {
            return Err(anyhow::anyhow!(
                "Number of alive cells cannot be greater than the number of cells in the grid."
            ));
        }
        let mut rng: ThreadRng = rand::thread_rng();
        let mut alive_cells: HashSet<(usize, usize)> = HashSet::new();
        while alive_cells.len() < num_alive.unwrap() {
            let i: usize = rng.gen::<usize>() % nrows;
            let j: usize = rng.gen::<usize>() % ncols;
            alive_cells.insert((i, j));
        }
        for (i, j) in alive_cells {
            grid[i][j].set_alive();
        }
    } else {
        return Err(anyhow::anyhow!("Invalid arguments."));
    }

    Ok(grid)
}

fn calc_next_frame(grid: &[Vec<Cell>]) -> Vec<Vec<Cell>> {
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

struct InputHandler {
    input: InputType,
    timeout: i32,
    draw_char: char,
}

impl InputHandler {
    fn new(timeout: i32, draw_char: char) -> InputHandler {
        InputHandler {
            input: InputType::Continue,
            timeout,
            draw_char,
        }
    }

    fn handle_input(&mut self) -> Result<InputType> {
        let c: char = get_next_char();
        self.input = match c {
            'q' => InputType::Quit,
            'a' => InputType::IncreaseTimeout,
            's' => InputType::DecreaseTimeout,
            _ => InputType::Continue,
        };

        match self.input {
            InputType::Quit | InputType::Continue => (),
            InputType::IncreaseTimeout => {
                // Increase timeout
                if self.timeout < 1000 {
                    self.timeout += 10;
                }
                timeout(self.timeout);
            }
            InputType::DecreaseTimeout => {
                // Decrease timeout
                if self.timeout > 10 {
                    self.timeout -= 10;
                }
                timeout(self.timeout);
            }
        }

        Ok(self.input)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum InputType {
    Quit,
    Continue,
    IncreaseTimeout,
    DecreaseTimeout,
}

fn run_frame(
    grid: &[Vec<Cell>],
    input_handler: &mut InputHandler,
) -> Result<(InputType, Vec<Vec<Cell>>)> {
    //! Runs a single loop of the game, drawing the grid, calculating the next
    //! frame, and getting input from the user.
    erase();
    draw(grid, input_handler)?;
    refresh();
    let next_grid = calc_next_frame(grid);
    let input: InputType = input_handler.handle_input()?;
    Ok((input, next_grid))
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

    ncurses::setlocale(LcCategory::all, "")?;

    /* initialize screen */
    initscr();

    /* hide cursor */
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    /* enables colors */
    start_color();

    /* initially refreshes screen, emptying it */
    refresh();

    /* keypresses will not be displayed on screen */
    noecho();

    /*
     * Set minimum timeout to 10ms, maximum timeout to 1000ms, and makes
     * timeout is in increment of 10.
     * */
    if args.timeout < 10 {
        args.timeout = 10;
    } else if args.timeout > 1000 {
        args.timeout = 1000;
    } else if args.timeout % 10 != 0 {
        args.timeout = (args.timeout / 10) * 10;
    }

    /* Applies timeout value */
    timeout(args.timeout);

    /* get the number of rows and columns */
    let nrows: usize = LINES() as usize - 2; // -2 to account for status bar at bottom
    let ncols: usize = (COLS() as usize - 1) / 2; // /2 to account for spacing characters at every
                                                  // other cell
                                                  /* initialize the grid */
    let mut grid: Vec<Vec<Cell>> = initialize(nrows, ncols, args.alive, &args.seed_file)?;

    let mut input_handler: InputHandler = InputHandler::new(args.timeout, args.character);

    loop {
        let (input, new_grid) = run_frame(&grid, &mut input_handler)?;
        grid = new_grid;
        if input == InputType::Quit {
            break;
        }
    }

    endwin();

    Ok(())
}
