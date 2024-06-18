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
use gettextrs::*;
use ncursesw::*;
use rand::Rng;
use std::{collections::HashSet, path::Path};
use clap::Parser;

/// Conway's Game of Life
///
/// A simple implementation of Conway's Game of Life using ncurses.
#[derive(Parser)]
struct Cli {
    /// Number of alive cells to start with
    #[clap(short='a', long="alive", default_value = "1000")]
    alive: Option<usize>,
    /// Seed file to start with
    #[clap(short='s', long="seed", default_value = "None")]
    seed_file: Option<String>,
    /// Timeout in milliseconds
    #[clap(short='t', long="timeout", default_value = "100")]
    timeout: u64,
}

fn draw(grid: &Vec<Vec<Cell>>, args: &Cli) -> Result<()> {
    //! Draws the grid on the screen
    //!
    //! # Arguments
    //! * `grid` - The grid to draw
    //! * `nrows` - Number of rows in the grid
    //! * `ncols` - Number of columns in the grid
    for i in 0..grid.len() {
        for j in 0..grid[0].len() {
            let output = WideString::from(
                format!("{}", if grid[i][j].is_alive() { '*' } else { ' ' }).as_str(),
            );
            let origin = Origin {
                y: i as i32,
                x: (j * 2) as i32,
            };
            mvaddwstr(origin, &output)?;
        }
    }
    let num_alive = grid.iter().flatten().filter(|cell| cell.is_alive()).count();
    let origin = Origin {
        y: grid.len() as i32,
        x: 0,
    };
    mvaddstr(origin, format!("Alive: {}, Timeout: {} | q: Quit, a: increase timeout, s: decrease timeout", num_alive, args.timeout).as_str())?;
    Ok(())
}

fn get_next_char() -> CharacterResult<char> {
    //! Gets the next character from the user
    match getch() {
        Ok(c) => c,
        Err(_) => CharacterResult::Character('\0'),
    }
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

    fn count_alive_neighbors(&self, grid: &Vec<Vec<Cell>>) -> usize {
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
    seed_file: Option<String>,
) -> Vec<Vec<Cell>> {
    //! Initializes the grid with the given number of alive cells or seed file.
    let mut grid: Vec<Vec<Cell>> = vec![];
    for i in 0..nrows {
        grid.push(vec![]);
        for j in 0..ncols {
            grid[i].push(Cell::new(i, j, false));
        }
    }

    if seed_file.is_some() {
        let seed = std::fs::read_to_string(seed_file.unwrap()).unwrap();
        for (rownum, line) in seed.lines().enumerate() {
            if rownum >= nrows {
                break;
            }
            let cells = line.chars().collect::<Vec<char>>();
            for (colnum, cell) in cells.iter().enumerate() {
                if colnum >= ncols {
                    break;
                }
                if *cell == '*' {
                    grid[rownum][colnum].set_alive();
                }
            }
        }
    } else if num_alive.is_some() {
        let mut rng = rand::thread_rng();
        let mut alive_cells: HashSet<(usize, usize)> = HashSet::new();
        while alive_cells.len() < num_alive.unwrap() {
            let i: usize = rng.gen::<usize>() % nrows;
            let j: usize = rng.gen::<usize>() % ncols;
            alive_cells.insert((i, j));
        }
        for (i, j) in alive_cells {
            grid[i][j].set_alive();
        }
    }

    grid
}

fn calc_next_frame(grid: &Vec<Vec<Cell>>) -> Vec<Vec<Cell>> {
    //! Calculates the next frame of the game, returning a new grid.
    let mut next_frame: Vec<Vec<Cell>> = grid.clone();

    for row in grid.iter() {
        for cell in row.iter() {
            let count = cell.count_alive_neighbors(&grid);
            if cell.is_alive() {
                if count < 2 || count > 3 {
                    next_frame[cell.x][cell.y].set_dead();
                }
            } else {
                if count == 3 {
                    next_frame[cell.x][cell.y].set_alive();
                }
            }
        }
    }
    next_frame
}

enum InputType {
    Quit,
    Continue,
    IncreaseTimeout,
    DecreaseTimeout,
}

fn run_frame(
    grid: &Vec<Vec<Cell>>, args: &Cli
) -> Result<(InputType, Vec<Vec<Cell>>)> {
    //! Runs a single loop of the game, drawing the grid, calculating the next
    //! frame, and getting the next character from the user.
    erase()?;
    draw(&grid, args)?;
    refresh()?;
    let next_grid = calc_next_frame(grid);
    let c = get_next_char();
    let input: InputType = if c == CharacterResult::Character('q') {
        InputType::Quit
    } else if c == CharacterResult::Character('a') {
        InputType::IncreaseTimeout
    } else if c == CharacterResult::Character('s') {
        InputType::DecreaseTimeout
    } else {
        InputType::Continue
    };
    Ok((input, next_grid))
}

fn main() -> Result<()> {

    let mut args = Cli::parse();

    setlocale(LocaleCategory::LcAll, "");

    /* initialize screen */
    initscr()?;

    /* hide cursor */
    curs_set(CursorType::Invisible)?;

    /* enables colors */
    start_color()?;

    /* initially refreshes screen, emptying it */
    refresh()?;

    /* keypresses will not be displayed on screen */
    noecho()?;

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
    timeout(std::time::Duration::from_millis(args.timeout as u64))?;

    /* get the number of rows and columns */
    let nrows: usize = LINES() as usize - 2; // -2 to account for status bar at bottom
    let ncols: usize = (COLS() as usize - 1) / 2; // /2 to account for spacing characters at every
                                                  // other cell
    /* initialize the grid */
    let mut grid = if args.seed_file.is_some() && Path::new(&args.seed_file.clone().unwrap()).exists() {
        initialize(nrows, ncols, None, Some(args.seed_file.clone().unwrap()))
    } else if args.alive.is_some() {
        initialize(nrows, ncols, Some(args.alive.unwrap()), None)
    } else {
        std::process::exit(1);
    };

    loop {
        let (input, new_grid) = run_frame(&grid, &args)?;
        grid = new_grid;
        match input {
            InputType::Quit => break,
            InputType::Continue => continue,
            InputType::IncreaseTimeout => {
                if args.timeout < 1000 {
                    args.timeout += 10;
                    timeout(std::time::Duration::from_millis(args.timeout as u64))?;
                }
            },
            InputType::DecreaseTimeout => {
                if args.timeout > 10 {
                    args.timeout -= 10;
                    timeout(std::time::Duration::from_millis(args.timeout as u64))?;
                }
            }
        }
    }

    endwin()?;

    Ok(())
}