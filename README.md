# bunnies

[![Crates.io](https://img.shields.io/crates/v/bunnies)](https://crates.io/crates/bunnies)

A fast chess library for chess engines. **Currently in alpha**.

## Current Features
- Game state representation
- Legal move generation
- Forwards-backwards move application
- FEN parsing and generation
- UCI generation
- Full PGN parsing and generation
  - Support for NAGs and variations (comments are parsed, but not generated)
  - Support for nested variations
- Perft tests
- Game termination detection (and draw detection)

## Optimizations
`bunnies` makes use of the following optimizations:
- Bitboard-based board representation
- Direct legal move generation that takes pins and checks into account (no pseudolegal step and validation)
- Make-unmake move application (eliminating the need to copy game state during search)
- Magic bitboards for sliding piece attack generation
- Precomputed attacks for knights and kings
- Zobrist hashing for board state hashing

`bunnies` is currently able to generate ~23M moves/thread/second (as measured on an M1 Macbook Pro),
which is not nearly as fast as programs like `Stockfish` (which is able to generate
~120M moves/thread/sec under the same conditions).

## Contribution
`bunnies` needs the following to make 1.0.0 happen:
- Multithreading
- Further performance improvement
  - Figure out bottlenecks
  - Solve inefficiencies
- More tests wouldn't hurt
- More benchmarks wouldn't hurt
- Support for PGN comments

Feedback and contributions welcome!