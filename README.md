# uglychild

A fast chess library for chess engines or any chess program that values performance. **API is not stable; currently in alpha**.

`uglychild` is currently one of the fastest chess move generators that exists. Without any specialized AVX/BMI2 instructions like PEXT, `uglychild` is able to generate up to 1.5 billion nodes/thread/sec, as measured on an Mac M3 chip.

This library has 2 goals.

1. Evolve into a fully functional MCTS-powered UCI chess engine
2. Offer powerful APIs for chess, including:
    - A chess engine API with Rust, C, and WASM bindings
    - A core chess API with Rust, C, and WASM bindings

Feedback and contributions welcome!
