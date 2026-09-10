#!/bin/bash

API_KEY="1e5765fc0c22df4e4ccf20581c2ef3d7" # revoked :D

mkdir -p ${HOME}/.config/footballscore
cat >${HOME}/.config/footballscore/config <<EOL
API_KEY=$API_KEY
API_ENDPOINT=v3.football.api-sports.io
FOOTBALL_DATA_ENDPOINT=$FOOTBALL_DATA_ENDPOINT
FOOTBALL_DATA_TOKEN=$FOOTBALL_DATA_TOKEN
LEAGUE_ID=$LEAGUE_ID
EOL

FOOTBALL_DATA_ENDPOINT=api.football-data.org
FOOTBALL_DATA_TOKEN=xxxxx
LEAGUE_ID=2021
