#!/usr/bin/env bash

# ============================ [ CONFIGURATION ] ============================ 

DEBUG=0

CHANGES_IN_CLIENT=1 # no changes
CHANGES_IN_CORE=1 # no changes

# ============================ [ DECLARATION ] ============================ 

function debug() {
  if [[ "${DEBUG}" -eq 0 ]]; then
    echo "[DEBUG] $1"
  fi
}

function read_changes() {
  local -r changed_files=$(git status --porcelain | cut -d' ' -f2)
  debug "changed_files = ${changed_files}"
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
  cd ./core && cargo make clip && cargo make clip && cd ../
}

function client_changed() {
  debug "Checking client changed..."
  return ${CHANGES_IN_CLIENT}
}

function client_build() {
  debug "Building client..."
  cd ./client && dart format . && flutter test && cd ../
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
