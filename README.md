# Chesnaught

Toy Chess and [Chess960] engine that speaks [UCI].

## Installation

This is a toy project that I don't feel like providing prebuilt executables. Compile it yourself using [Cargo], remember to put `--release`.

Chesnaught is a command line software. You'll need a GUI in order to use Chesnaught. Here are my recommendation:

- [En Croissant] &ndash; User friendly option
- [Cute Chess] &ndash; Lightweight option

## Playing Motif

(It's currently in development, it may change)

## Options

Before playing, consider configuring it first to give it more computation advantage.

### Thread

The amount of threads that Chesnaught will use during search. Optimal setting: the same number as your CPU cores. You can provide less if you want but giving more than the number of your CPU cores may not help.

Chesnaught actual uses more than alloted threads but the extra threads should be fairly lowly processed.

### Hash

The amount of memory in MiB that Chesnaught will use for storing previously computed board position to avoid duplicate computation. Chesnaught will not allocate it right away but it'll start with none then it'll grow as needed. Chesnaught could limit itself to a lower setting but it is guaranteed to never exceed provided limit. Optimal setting: as high as you're willing to give.

## Playing

If you want to play against Chesnaught. You'll need to put a limit as otherwise it'll not play as it searches forever. You can limit it by:

- Time (Recommended)
- Depth &ndash; The number of plies it'll search
- Nodes &ndash; The number of positions it'll search. (Poorly supported, it'll overcount)

## Analysis

You can perform analysis with Chesnaught to see what it thinks. It shouldn't be used for analyzing games, use [Stockfish] instead. Chesnaught doesn't provide multiple lines as it's mainly designed for playing.

Chesnaught does provide centipawn analysis but this is an approximation. Chesnaught's actual score is a compound number that is hard to condense into a single number.

[Chess960]: https://en.wikipedia.org/wiki/Chess960
[UCI]: https://en.wikipedia.org/wiki/Universal_Chess_Interface
[Cargo]: https://rust-lang.org/
[En Croissant]: https://encroissant.org/
[Cute Chess]: https://cutechess.com/
[Stockfish]: https://stockfishchess.org/
