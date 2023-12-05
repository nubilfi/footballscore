# footballscore

[![version](https://img.shields.io/crates/v/footballscore?color=blue&logo=rust&style=flat-square)](https://crates.io/crates/footballscore)
[![Build Status](https://github.com/nubilfi/footballscore/workflows/Rust/badge.svg?branch=master)](https://github.com/nubilfi/footballscore/actions?branch=main)
[![Documentation](https://docs.rs/footballscore/badge.svg)](https://docs.rs/footballscore/latest/footballscore/)
[![codecov](https://codecov.io/gh/nubilfi/footballscore/branch/master/graph/badge.svg)](https://codecov.io/gh/nubilfi/footballscore)

a CLI tool to retreive football score from api-football.com. You will need to obtain an `API_KEY` by signing up at dashboard.api-football.com.

Usage:

```bash
footballscore
Utility to retreive football score of your favorite team from api-football.com

USAGE:
    footballscore [OPTIONS]

FLAGS:
    -h, --help      Prints help information
    -V, --version   Prints version information

OPTIONS:
    -k, --api-key <api-key>             Api key (optional but either this or API_KEY environemnt variable must exist)
        --next-match <next-match>       Show next match, 1 = true, 0 = false (optional)
    -c, --club-id <club-id>             Your favorite Club ID (optional), if not specified `529 (Barcelona)` will be assumed
```

Output:

```bash
Match: Barcelona 0 vs 0 Girona
```

Or, you might want to use it on `i3wm + Polybar + dunstify` or something similar, here's an example of mine.

![image](https://github.com/nubilfi/footballscore/blob/main/i3wm/footballscore-i3wm.png "image")

## Development

```bash
git clone git@github.com:nubilfi/footballscore.git

# Build
cd footballscore
cargo build -r

# Run unit tests and integration tests
cargo test

# Run benchmark
cargo bench
```

## License

[MIT](https://github.com/nubilfi/footballscore/blob/main/LICENSE)
