# Conway's Game of Life

This is a simple implementation of Conway's Game of Life, written in rust, with wrapping edges.

There are a few controls:
| Input | Effect |
|-|-|
| q | quit |
| a | increases frame timeout |
| s | decreases frame timeout |

There are also a few command line options:
| Option | Effect |
|-|-|
| -t/--timeout | Set the timeout in milliseconds of each frame.
Min: 10, Max: 1000, Increments: 10, Default: 100 |
| -a/--alive | Initial number of cells randomly generated on the board.\nIgnored if -s/--seed is passed. Default: 1000 |
