# footballscore

a CLI tool to retreive football score from api-football.com. You will need to obtain an `API_KEY` by signing up at api.football.com.

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
    -d, --club-id <club-id>             Your favorite Club ID (optional), if not specified `529 (Barcelona)` will be assumed
```

Output:

```bash
Match: Le Havre 0 vs 1 Paris Saint German
```

Or, you might want to use it on `i3wm + Polybar` or something similar, here's an example of mine.

![image](https://github.com/nubilfi/footballscore/blob/main/i3wm/footballscore-i3wm.png "image")
