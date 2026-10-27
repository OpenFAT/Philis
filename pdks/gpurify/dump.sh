#!/usr/bin/env bash
# Regenerates each <pdk>.deck.json from <pdk>.deck with the gpurify the flake
# pins (Q61): `nix develop -c pdks/gpurify/dump.sh`. Pdk reads the dumps;
# verify's decks::dumps_are_fresh fails until they match the decks again.
set -eu
cd "$(dirname "$0")"
for deck in *.deck; do
  gpurify deck "$deck" --format json > "$deck.json"
done
