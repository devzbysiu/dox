#!/usr/bin/env bash

# This script builds core and/or client when file changes are found.
# It reads the changes from git, then checks if paths start with
# `client/` or `core/` respectively. If changed files are not visible
# in git or do not start with `core/` or `client/` then nothing happens.

set -o errexit
set -o pipefail
set -o nounset

# ============================ [ CONFIGURATION ] ============================ 

DEBUG=0 # enable debug logs

CHANGES_IN_CLIENT=1 # no changes
CHANGES_IN_CORE=1 # no changes

# colors

BOLD="\e[1";
ENDCOLOR="\e[0m";

# font color selection
CYAN="36";

# bg color
BG_DEFAULT="49";

# ============================ [ DECLARATION ] ============================ 

function debug() {
  if [[ "${DEBUG}" -eq 0 ]]; then
    local -r message=$1
    echo -e "${BOLD};${BG_DEFAULT};${CYAN}m [DEBUG] ${ENDCOLOR}${message}"
  fi
}

function read_changes() {
  local -r changed_files=$(git status --porcelain | sed 's/^ //g' | tr -s ' ' | cut -d' ' -f2)
  debug "changed_files = [ ${changed_files} ]"
  while IFS= read -r line; do
    if [[ ${line} = client/* ]]; then
      debug "line ${line} starts with client/*"
      CHANGES_IN_CLIENT=0
    fi
    if [[ ${line} = core/* ]]; then
      debug "line ${line} starts with core/*"
      CHANGES_IN_CORE=0
    fi
  done <<< "${changed_files}"
}

function core_changed() {
  debug "Checking core changed..."
  return ${CHANGES_IN_CORE}
}

function core_build() {
  debug "Building core..."
  pushd ./core
  cargo make clip 
  cargo make test
  popd
}

function client_changed() {
  debug "Checking client changed..."
  return ${CHANGES_IN_CLIENT}
}

function client_build() {
  debug "Building client..."
  pushd ./client
  dart format .
  flutter test
  popd
}

function main() {
  read_changes
  if core_changed; then
    core_build
  fi
  if client_changed; then
    client_build
  fi
  exit 0
}

# ============================ [ EXECUTION ] ============================ 

main
