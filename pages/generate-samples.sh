#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
AUDIO_DIR="$SCRIPT_DIR/audio"
CLI="${1:-midi-cli-rs}"

mkdir -p "$AUDIO_DIR"

echo "Generating preset demos..."

# Suspense
echo "  - suspense"
$CLI preset -m suspense -d 5 --seed 20260220 -k Am --intensity 60 -t 70 \
    -o "$AUDIO_DIR/suspense-demo.wav"

# Eerie
echo "  - eerie"
$CLI preset -m eerie -d 5 --seed 20260220 -k Dm --intensity 50 -t 60 \
    -o "$AUDIO_DIR/eerie-demo.wav"

# Upbeat
echo "  - upbeat"
$CLI preset -m upbeat -d 5 --seed 20260220 -k C --intensity 70 -t 120 \
    -o "$AUDIO_DIR/upbeat-demo.wav"

# Calm
echo "  - calm"
$CLI preset -m calm -d 5 --seed 20260220 -k G --intensity 30 -t 72 \
    -o "$AUDIO_DIR/calm-demo.wav"

# Ambient
echo "  - ambient"
$CLI preset -m ambient -d 8 --seed 20260220 -k Em --intensity 40 -t 60 \
    -o "$AUDIO_DIR/ambient-demo.wav"

# Jazz
echo "  - jazz"
$CLI preset -m jazz -d 6 --seed 20260220 -k F --intensity 50 -t 100 \
    -o "$AUDIO_DIR/jazz-demo.wav"

echo "Generating melody demos..."

# Piano melody - C major arpeggio with D minor passing (sequential with @offset)
echo "  - piano"
$CLI generate --notes "C4:1:80@0,E4:1:80@1,G4:1:80@2,C5:1:90@3,G4:1:80@4,E4:1:80@5,C4:1:80@6,D4:1:75@7,F4:1:75@8,A4:1:75@9,D5:1:85@10,A4:1:75@11,F4:1:75@12,D4:1:75@13,C4:2:90@14" -i piano -t 100 \
    -o "$AUDIO_DIR/melody-piano.wav"

# Strings melody - G major with Am passing (sequential with @offset)
echo "  - strings"
$CLI generate --notes "G3:2:70@0,B3:1:70@2,D4:1:70@3,G4:2:80@4,D4:1:70@6,B3:1:70@7,G3:2:70@8,A3:2:70@10,C4:1:70@12,E4:1:70@13,A4:2:80@14,E4:1:70@16,C4:1:70@17,A3:2:70@18,G3:4:85@20" -i strings -t 80 \
    -o "$AUDIO_DIR/melody-strings.wav"

echo ""
echo "Done! Generated $(ls -1 "$AUDIO_DIR"/*.wav 2>/dev/null | wc -l | tr -d ' ') audio files in $AUDIO_DIR"
