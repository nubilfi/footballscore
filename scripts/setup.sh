#!/bin/bash

API_KEY="1e5765fc0c22df4e4ccf20581c2ef3d7"

mkdir -p ${HOME}/.config/footballscore
cat > ${HOME}/.config/footballscore/config.env <<EOL
API_KEY=$API_KEY
API_ENDPOINT=v3.football.api-sports.io
EOL
