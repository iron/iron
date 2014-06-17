#!/bin/sh

# This script will recompile a rust project using `make`
# every time something in the specified directory changes.
#
# It is designed to be used in a rust-empty style crate.
# $1: Directory to watch
#     src by default
# $2: Command to execute
#     `make` by default

# Determine target architecture
TARGET=`rustc --version 2> /dev/null | awk '/host:/ { FS = " "; print $2 }'`

# Watch files in infinite loop
watch () {
  if [ -e "$1" ]; then
    echo "Watching files in $1.."
    while inotifywait -q -r -e modify $1; do
      echo Rebuilding:
      echo '~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~'
      $2
      if [ ! $? -eq 0 ]; then
        echo ""
      fi
    done
  else
    echo "$1 is not a valid directory"
  fi
}

# Capture user input with defaults
DIR=${1:-src}
CMD=${2:-make}

watch "$DIR" "$CMD"