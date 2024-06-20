/*
 * Rules:
 * - Any live cell with fewer than two live neighbours dies, as if by underpopulation.
 * - Any live cell with two or three live neighbours lives on to the next generation.
 * - Any live cell with more than three live neighbours dies, as if by overpopulation.
 * - Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
 *
 * Due to the "infinite" nature of the game, this implementation simply uses wrapping edges.
 * */

pub mod conway;
pub mod demo;
pub mod window;

use anyhow::Result;
use clap::Parser;
use conway::{initialize, run_frame, Cell, InputHandler, InputType};
use ncurses::*;
use window::Window;

/// Conway's Game of Life
///
/// A simple implementation of Conway's Game of Life using ncurses.
#[derive(Parser)]
pub struct Cli {
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
    /// Run a demo program to see the various seeds
    #[clap(short = 'd', long = "demo")]
    demo: bool,
}

fn main() -> Result<()> {
    let mut args = Cli::parse();

    if args.demo {
        demo::run(&args)?;
        return Ok(());
    }

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
    let nrows: usize = LINES() as usize - 1;
    let ncols: usize = COLS() as usize - 1;

    let mut input_handler: InputHandler = InputHandler::new(args.timeout, args.character);

    let mut win: Window = Window::new(nrows as i32, ncols as i32, 0, 0);

    /* initialize the grid */
    let mut grid: Vec<Vec<Cell>> = initialize(&mut win, args.alive, &args.seed_file)?;

    loop {
        let (input, new_grid) = run_frame(&mut win, &grid, &mut input_handler)?;
        grid = new_grid;
        if input == InputType::Quit {
            break;
        }
    }

    endwin();

    Ok(())
}
