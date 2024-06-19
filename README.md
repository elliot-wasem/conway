# Conway's Game of Life

This is a simple implementation of Conway's Game of Life, written in Rust, with wrapping edges.

There are a few controls:
| Input | Effect |
|-|-|
| q | quit |
| a | increases frame timeout |
| s | decreases frame timeout |

There are also a few command line options:
| Option | Effect |
|-|-|
| -t/--timeout | Set the timeout in milliseconds of each frame. Min: 10, Max: 1000, Increments: 10, Default: 100 |
| -a/--alive | Initial number of cells randomly generated on the board. Ignored if -s/--seed is passed. Default: 1000 |
| -s/--seed | Seed file to be used for the initial state of the board. Aligns file with top-left corner, and truncates lines/columns that won't fit on screen. Overrides -a/--alive. |
| -c/--character | Character used to draw cells. Default: * |
