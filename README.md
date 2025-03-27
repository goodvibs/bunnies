# bunnies

[![Crates.io](https://img.shields.io/crates/v/bunnies)](https://crates.io/crates/bunnies)

A fast chess library for chess engines. **Currently in alpha**.

## Current Features
- Game state representation
- (Pseudo-legal) move generation and application (and un-application)
- FEN parsing and generation
- UCI generation
- Full PGN parsing and generation
  - Support for NAGs and variations (comments are parsed, but not generated)
  - Support for nested variations
- Perft testing
- Game termination detection (and draw detection)

## Optimizations
`bunnies` makes use of the following optimizations:
- Bitboard-based board representation
- Pseudo-legal move generation
- Make-unmake move application (eliminating the need to copy game state)
- Magic bitboards for sliding piece attack generation
- Precomputed attacks for knights and kings
- Zobrist hashing for board state hashing

`bunnies` is currently able to generate ~15M moves/thread/second (as measured on an M1 Macbook Pro),
which is not nearly as fast as programs like `Stockfish` (which is able to generate
~120M moves/thread/sec under the same conditions).

## Contribution
`bunnies` needs the following to make 1.0.0 happen:
- Identify and remove bottlenecks
- Some more benchmarks
- Some more integration tests
- Support for PGN comments
- Unit tests for draw detection

Feedback and contributions welcome!