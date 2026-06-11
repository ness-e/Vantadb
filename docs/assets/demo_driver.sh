#!/usr/bin/env bash
# VantaDB terminal demo driver (typewriter effect) for asciinema recording.
set -u

CLI="${CLI:-vanta-cli}"
PROMPT_USER="\033[1;32m~/vantadb\033[0m \033[1;36m$\033[0m "

type_cmd() {
  # Print prompt, then "type" the command char by char.
  printf "%b" "$PROMPT_USER"
  local s="$1"
  for (( i=0; i<${#s}; i++ )); do
    printf "%s" "${s:$i:1}"
    sleep 0.022
  done
  printf "\n"
  sleep 0.35
}

run() {
  # $1 = displayed command, $2 = actual command to eval
  type_cmd "$1"
  eval "$2"
  sleep 1.1
}

clear
printf "\033[1;35m"
cat <<'BANNER'
  ╦  ╦┌─┐┌┐┌┌┬┐┌─┐╔╦╗╔╗
  ╚╗╔╝├─┤│││ │ ├─┤ ║║╠╩╗
   ╚╝ ┴ ┴┘└┘ ┴ ┴ ┴═╩╝╚═╝
BANNER
printf "\033[0m"
printf "  \033[2mEmbedded Rust engine · durable local memory + hybrid retrieval\033[0m\n\n"
sleep 1.4

run "vanta-cli put --namespace agent/main --key note-1 \\
    --payload \"HNSW graph index for fast vector search\" --vector 1,0,0" \
    "$CLI put --namespace agent/main --key note-1 --payload 'HNSW graph index for fast vector search' --vector 1,0,0 2>/dev/null"

run "vanta-cli put --namespace agent/main --key note-2 \\
    --payload \"BM25 keyword ranking over durable memory\" --vector 0,1,0" \
    "$CLI put --namespace agent/main --key note-2 --payload 'BM25 keyword ranking over durable memory' --vector 0,1,0 2>/dev/null"

run "vanta-cli put --namespace agent/main --key note-3 \\
    --payload \"RRF fuses lexical and semantic results\" --vector 0.5,0.5,0" \
    "$CLI put --namespace agent/main --key note-3 --payload 'RRF fuses lexical and semantic results' --vector 0.5,0.5,0 2>/dev/null"

run "vanta-cli list --namespace agent/main" \
    "$CLI list --namespace agent/main 2>/dev/null"

run "vanta-cli get --namespace agent/main --key note-1" \
    "$CLI get --namespace agent/main --key note-1 2>/dev/null"

run "vanta-cli rebuild-index" \
    "$CLI rebuild-index 2>/dev/null | tail -n +2"

run "vanta-cli status" \
    "$CLI status 2>/dev/null"

run "vanta-cli export --out memory.json" \
    "$CLI export --out '$PWD/memory.json' 2>/dev/null"

sleep 1.6
printf "\n  \033[1;32m✓ Local-first. Zero network. Crash-safe via WAL.\033[0m\n"
sleep 2.2
