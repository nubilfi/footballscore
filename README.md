# footballscore

[![version](https://img.shields.io/crates/v/footballscore?color=blue&logo=rust&style=flat-square)](https://crates.io/crates/footballscore)
[![Build Status](https://github.com/nubilfi/footballscore/actions/workflows/rust.yml/badge.svg)](https://github.com/nubilfi/footballscore/actions?branch=main)
[![Documentation](https://docs.rs/footballscore/badge.svg)](https://docs.rs/footballscore/latest/footballscore/)
[![codecov](https://codecov.io/gh/nubilfi/footballscore/graph/badge.svg?token=SRGOFSB31Q)](https://codecov.io/gh/nubilfi/footballscore)

a CLI tool to retrieve football scores and fixtures.

Uses [api-football.com](https://api-football.com) (paid) for live scores and next fixtures, and [football-data.org](https://www.football-data.org) (free) for upcoming fixtures.

Usage:

```bash
footballscore
Retreive football score and fixture of your favorite team

USAGE:
    footballscore [COMMAND]

Commands (paid - api-football.com):
  live      Retrieve live score for a club
  next      Show next fixture for a club
  team      Look up a club ID by name

Commands (free - football-data.org):
  upcoming  Show upcoming fixture with live score
  find      Search for a team ID by name
  leagues   List league IDs
```

Output:

```bash
Match: Barcelona 0 vs 0 Girona
```

To retrieve _live score_ data, you only need to use `live` command:

```bash
# live score
footballscore live -k=api_key_value -c=club_id_value
```

More example:

```bash
# next fixture
footballscore next -k=api_key_value -c=club_id_value

# look up a club ID
footballscore team -k=api_key_value -n="barcelona"
```

### football-data.org

Free tier: 10 requests/minute, covering the major European leagues and cups. Sign up at [football-data.org](https://www.football-data.org) to get a token. Fixtures come from `/teams/{id}/matches`, so `upcoming` only needs a team ID, `find` lists teams per competition and needs a league ID. Work from `leagues` down:

```bash
# 1. find the league
footballscore leagues england -t=auth_token_value
#   2021	Premier League	England

# 2. find the team within that league
footballscore find "Liverpool" -t=auth_token_value -l 2021
#   64	Liverpool

# 3. next fixture, with the live score when the match is running
footballscore upcoming -t=auth_token_value --team-id=64
```

Set `LEAGUE_ID` in the config file to drop the `-l` flag from `find`. `--season` (or `SEASON`, e.g. `2026`) is optional, without it the API serves the season it has current

Note `upcoming` shows a match that is currently in play ahead of anything scheduled later, so a single command covers both the fixture list and the live score. Kickoff times are UTC.

Want to stay updated regularly? Set up an `interval` for specific durations on your panel item.

**Update Frequency** : The data is updated every 15 seconds. Although the data is updated every 15 seconds, depending on the competition there may be a delay between reality and the availability of data in the API.

Or, you might want to use it on `i3wm + Polybar + dunstify` or something similar, here's an example of mine.

![image](https://github.com/nubilfi/footballscore/blob/main/i3wm/footballscore-i3wm.png "image")

## Config file

Credentials and endpoints can be set in `~/.config/footballscore/config` so you don't need to pass them on every invocation:

```env
API_KEY=xxxxx
API_ENDPOINT=v3.football.api-sports.io
FOOTBALL_DATA_ENDPOINT=api.football-data.org/v4
FOOTBALL_DATA_TOKEN=xxxxx
LEAGUE_ID=2021
SEASON=2026
```

## Development

```bash
git clone git@github.com:nubilfi/footballscore.git


# Build
cd footballscore

cargo build -r              # build with cargo
make                        # build docker image with Makefile
./scripts/build_package.sh  # build packages for ArchLinux
./scripts/setup.sh          # setup application environment variables

# Run the application
cargo run -- -h

# Run unit tests and integration tests
cargo test

# Run benchmark
cargo bench
```

## License

[MIT](https://github.com/nubilfi/footballscore/blob/main/LICENSE)
