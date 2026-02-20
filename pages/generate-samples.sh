#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
AUDIO_DIR="$SCRIPT_DIR/audio"
CLI="${1:-midi-cli-rs}"

mkdir -p "$AUDIO_DIR"

echo "Generating mood preset demos (3 seeds each)..."

for mood in suspense eerie upbeat calm ambient jazz; do
    for seed in 1 2 3; do
        echo "  - $mood seed $seed"
        $CLI preset -m $mood -d 5 --seed $seed -o "$AUDIO_DIR/${mood}-${seed}.wav"
    done
done

echo ""
echo "Generating melody demos..."

# Piano - "Twinkle Twinkle" inspired (public domain)
echo "  - piano (nursery rhyme style, 110 BPM)"
$CLI generate --notes "C4:1:80@0,C4:1:80@1,G4:1:85@2,G4:1:85@3,A4:1:90@4,A4:1:90@5,G4:2:85@6,F4:1:80@8,F4:1:80@9,E4:1:85@10,E4:1:85@11,D4:1:80@12,D4:1:80@13,C4:2:90@14" -i piano -t 110 \
    -o "$AUDIO_DIR/melody-piano.wav"

# Strings - Lyrical sweeping melody (slower, expressive)
echo "  - strings (lyrical sweep, 66 BPM)"
$CLI generate --notes "E4:3:70@0,G4:1:75@3,A4:2:80@4,G4:2:75@6,E4:2:70@8,D4:2:75@10,E4:4:80@12,C4:3:70@16,E4:1:75@19,G4:2:80@20,A4:2:85@22,G4:4:80@24" -i strings -t 66 \
    -o "$AUDIO_DIR/melody-strings.wav"

# Bass - Funky walking bass line (upbeat, rhythmic)
echo "  - bass (walking groove, 128 BPM)"
$CLI generate --notes "E2:1:90@0,G2:1:85@1,A2:1:90@2,B2:1:85@3,C3:1:95@4,B2:1:85@5,A2:1:90@6,G2:1:85@7,E2:1:90@8,D2:1:85@9,C2:1:90@10,D2:1:85@11,E2:2:95@12,G2:1:85@14,E2:1:90@15" -i bass -t 128 \
    -o "$AUDIO_DIR/melody-bass.wav"

# Cello - Deep expressive melody (slow, emotional)
echo "  - cello (deep expression, 54 BPM)"
$CLI generate --notes "G2:4:75@0,B2:2:80@4,D3:2:85@6,G3:4:90@8,F#3:2:85@12,E3:2:80@14,D3:4:85@16,C3:2:80@20,B2:2:75@22,G2:6:80@24" -i cello -t 54 \
    -o "$AUDIO_DIR/melody-cello.wav"

echo ""
echo "Done! Generated $(ls -1 "$AUDIO_DIR"/*.wav 2>/dev/null | wc -l | tr -d ' ') audio files in $AUDIO_DIR"
