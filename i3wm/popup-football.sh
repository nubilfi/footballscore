#!/bin/bash

TMP=${XDG_RUNTIME_DIR:-/tmp}/"$UID"_footballscore_notification
touch "$TMP"

DIFF=${DIFF:-0}

send_notification() {
	BODY=$(footballscore "-k=1e5765fc0c22df4e4ccf20581c2ef3d7" "-c=529" "--next-match=1" | head -n7)
	dunstify -h string:x-canonical-private-synchronous:footballscore \
		"$BODY" -u NORMAL
}

send_notification "$DIFF"
