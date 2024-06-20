use anyhow::Result;
use ncurses::*;
use std::fs;
use std::path::PathBuf;

use crate::conway::{initialize, run_frame};

use super::conway::{Cell, InputHandler, InputType};
use super::window::{Color, ColorPair, Window};
use super::Cli;

fn collect_seed_files(sidebar_width: usize) -> Result<Vec<String>> {
    let files = fs::read_dir("seeds")?;

    let mut samples: Vec<String> = files
        .map(|file| {
            let file: PathBuf = file.unwrap().path();
            let mut name: String = file.file_name().unwrap().to_str().unwrap().to_string();
            if name.len() > sidebar_width - 4 {
                name = format!("{}...", &name[..sidebar_width - 7]);
            }
            name
        })
        .collect::<Vec<String>>();
    samples.sort();

    Ok(samples)
}

pub fn run(args: &Cli) -> Result<()> {
    ncurses::setlocale(ncurses::LcCategory::all, "")?;

    initscr();

    curs_set(ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();

    noecho();

    refresh();

    timeout(args.timeout);

    // Get the number of rows and columns for the entirety of the terminal
    let max_rows: i32 = LINES();
    let max_cols: i32 = COLS();

    // width of the sidebar. This is the width of the sidebar in characters
    let sidebar_width = 20;

    // Create the sidebar and display windows
    let sidebar: Window = Window::new(max_rows, sidebar_width, 0, 0);
    let mut display: Window =
        Window::new(max_rows, max_cols - sidebar_width - 1, 0, sidebar_width + 1);

    // collect the seed files for the sample display
    let samples: Vec<String> = collect_seed_files(sidebar_width as usize)?;

    // which sample is selected at the moment
    let mut cur_sample: isize = 0;

    // Initialize the grid with the first sample
    let mut cur_input: InputType = InputType::Continue;
    let mut input_handler: InputHandler = InputHandler::new(args.timeout, args.character);
    let mut filename: String = format!("seeds/{}", &samples[cur_sample as usize]);
    let mut grid: Vec<Vec<Cell>> = initialize(&mut display, args.alive, &Some(filename))?;

    // color for the selected sample
    let selected_color: ColorPair = ColorPair::new(Color::Black, Color::White);

    while cur_input != InputType::Quit {
        // handle arrow keys
        if cur_input == InputType::Down || cur_input == InputType::Up {
            // update the sample based on the arrow key
            if cur_input == InputType::Down {
                cur_sample += 1;
            } else if cur_input == InputType::Up {
                cur_sample -= 1;
            }

            // wrap around the samples
            if cur_sample >= samples.len() as isize {
                cur_sample = 0;
            } else if cur_sample < 0 {
                cur_sample = samples.len() as isize - 1;
            }

            // populate the grid with the new sample
            filename = format!("seeds/{}", &samples[cur_sample as usize]);
            grid = initialize(&mut display, args.alive, &Some(filename))?;
        }

        // clear the windows
        sidebar.erase();
        display.erase();

        // draw the sidebar's border
        sidebar.draw_border()?;

        // draw the sample names
        for (i, sample) in samples.iter().enumerate() {
            if cur_sample == i as isize {
                sidebar.print(2, i as i32 + 1, sample, Some(&selected_color))?;
            } else {
                sidebar.print(2, i as i32 + 1, sample, None)?;
            }
        }

        // run a single frame, collecting input and the updated grid.
        let (input, new_grid) = run_frame(&mut display, &grid, &mut input_handler)?;

        // refresh just the sidebar. The display window will be refreshed as
        // part of the call to 'run_frame()'
        sidebar.refresh();

        // update the input and grid for the next iteration
        cur_input = input;
        grid = new_grid;
    }

    endwin();

    Ok(())
}
