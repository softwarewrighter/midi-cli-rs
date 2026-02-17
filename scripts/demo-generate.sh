#!/bin/bash
# Demo script showing midi-cli-rs usage patterns for AI coding agents
# This script generates various audio samples and places them in ./preview/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
CLI="$PROJECT_DIR/target/release/midi-cli-rs"
PREVIEW_DIR="$PROJECT_DIR/preview"

# Build if needed
if [[ ! -f "$CLI" ]]; then
    echo "Building midi-cli-rs..."
    cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"
fi

# Create preview directory
mkdir -p "$PREVIEW_DIR"

echo "Generating audio samples..."
echo ""

# ============================================
# Example 1: Simple C Major Chord (arpeggiated)
# ============================================
echo "1. C Major Chord (arpeggiated) - piano"
"$CLI" generate \
    --notes "C4:0.5:80@0,E4:0.5:80@0.5,G4:0.5:80@1,C5:1:80@1.5" \
    --instrument piano \
    --tempo 120 \
    --output "$PREVIEW_DIR/01-c-major-arpeggio.wav"

# ============================================
# Example 2: Simple melody
# ============================================
echo "2. Simple melody - piano"
"$CLI" generate \
    --notes "C4:0.5:80,D4:0.5:70,E4:0.5:80,F4:0.5:70,G4:1:90,E4:0.5:80,C4:1:100" \
    --instrument piano \
    --tempo 100 \
    --output "$PREVIEW_DIR/02-simple-melody.wav"

# ============================================
# Example 3: Low drone (suspense element)
# ============================================
echo "3. Low drone - cello"
"$CLI" generate \
    --notes "C2:4:60,G2:4:50@0" \
    --instrument cello \
    --tempo 60 \
    --output "$PREVIEW_DIR/03-low-drone.wav"

# ============================================
# Example 4: String pad
# ============================================
echo "4. String pad - strings"
"$CLI" generate \
    --notes "C3:4:50,E3:4:50@0,G3:4:50@0,C4:4:40@0" \
    --instrument strings \
    --tempo 60 \
    --output "$PREVIEW_DIR/04-string-pad.wav"

# ============================================
# Example 5: Upbeat rhythm
# ============================================
echo "5. Upbeat rhythm - piano"
"$CLI" generate \
    --notes "C4:0.25:90@0,C4:0.25:70@0.5,E4:0.25:90@1,E4:0.25:70@1.5,G4:0.25:90@2,G4:0.25:70@2.5,C5:0.5:100@3" \
    --instrument piano \
    --tempo 140 \
    --output "$PREVIEW_DIR/05-upbeat-rhythm.wav"

# ============================================
# Example 6: Bass line
# ============================================
echo "6. Bass line - electric bass"
"$CLI" generate \
    --notes "C2:1:100@0,G2:0.5:80@1,C2:0.5:90@1.5,E2:1:100@2,G2:1:80@3" \
    --instrument bass \
    --tempo 100 \
    --output "$PREVIEW_DIR/06-bass-line.wav"

# ============================================
# Example 7: Bells/chimes
# ============================================
echo "7. Bells/chimes - vibraphone"
"$CLI" generate \
    --notes "C5:2:60@0,E5:2:50@1,G5:2:50@2,C6:3:40@3" \
    --instrument vibraphone \
    --tempo 80 \
    --output "$PREVIEW_DIR/07-bells.wav"

# ============================================
# Example 8: Minor key (eerie)
# ============================================
echo "8. Minor key (eerie) - strings"
"$CLI" generate \
    --notes "A3:2:50,C4:2:50@0,E4:2:40@0,A4:4:30@2" \
    --instrument strings \
    --tempo 50 \
    --output "$PREVIEW_DIR/08-minor-eerie.wav"

# ============================================
# Example 9: JSON input demo
# ============================================
echo "9. JSON input - multi-note"
echo '{"tempo":90,"instrument":"piano","notes":[
    {"pitch":"C4","duration":0.5,"velocity":80,"offset":0},
    {"pitch":"E4","duration":0.5,"velocity":80,"offset":0.5},
    {"pitch":"G4","duration":0.5,"velocity":80,"offset":1},
    {"pitch":"B4","duration":0.5,"velocity":70,"offset":1.5},
    {"pitch":"C5","duration":1,"velocity":90,"offset":2}
]}' | "$CLI" generate --json --output "$PREVIEW_DIR/09-json-demo.wav"

# ============================================
# Example 10: Flute melody
# ============================================
echo "10. Flute melody"
"$CLI" generate \
    --notes "G5:0.5:70,A5:0.5:75,B5:0.5:80,C6:1:85,B5:0.5:75,A5:0.5:70,G5:1:80" \
    --instrument flute \
    --tempo 88 \
    --output "$PREVIEW_DIR/10-flute-melody.wav"

# ============================================
# MOOD PRESETS
# ============================================
echo ""
echo "=== Mood Presets ==="
echo ""

# ============================================
# Example 11: Suspense preset
# ============================================
echo "11. Suspense preset (5 sec)"
"$CLI" preset \
    --mood suspense \
    --duration 5 \
    --intensity 70 \
    --seed 42 \
    --output "$PREVIEW_DIR/11-preset-suspense.wav"

# ============================================
# Example 12: Eerie preset
# ============================================
echo "12. Eerie preset (5 sec)"
"$CLI" preset \
    --mood eerie \
    --duration 5 \
    --intensity 60 \
    --seed 42 \
    --output "$PREVIEW_DIR/12-preset-eerie.wav"

# ============================================
# Example 13: Upbeat preset
# ============================================
echo "13. Upbeat preset (5 sec)"
"$CLI" preset \
    --mood upbeat \
    --duration 5 \
    --key C \
    --intensity 80 \
    --seed 42 \
    --output "$PREVIEW_DIR/13-preset-upbeat.wav"

# ============================================
# Example 14: Calm preset
# ============================================
echo "14. Calm preset (5 sec)"
"$CLI" preset \
    --mood calm \
    --duration 5 \
    --key G \
    --intensity 50 \
    --seed 42 \
    --output "$PREVIEW_DIR/14-preset-calm.wav"

# ============================================
# Example 15: Ambient preset
# ============================================
echo "15. Ambient preset (7 sec)"
"$CLI" preset \
    --mood ambient \
    --duration 7 \
    --key Em \
    --intensity 40 \
    --seed 42 \
    --output "$PREVIEW_DIR/15-preset-ambient.wav"

# ============================================
# Example 16: Jazz preset (Bb - classic jazz key)
# ============================================
echo "16. Jazz preset - Bb (8 sec)"
"$CLI" preset \
    --mood jazz \
    --duration 8 \
    --key Bb \
    --intensity 70 \
    --seed 42 \
    --output "$PREVIEW_DIR/16-preset-jazz-bb.wav"

# ============================================
# Example 17: Jazz preset (F - mellow)
# ============================================
echo "17. Jazz preset - F mellow (8 sec)"
"$CLI" preset \
    --mood jazz \
    --duration 8 \
    --key F \
    --intensity 50 \
    --tempo 80 \
    --seed 123 \
    --output "$PREVIEW_DIR/17-preset-jazz-mellow.wav"

# ============================================
# Example 18: Jazz preset (Eb - uptempo)
# ============================================
echo "18. Jazz preset - Eb uptempo (6 sec)"
"$CLI" preset \
    --mood jazz \
    --duration 6 \
    --key Eb \
    --intensity 85 \
    --tempo 140 \
    --seed 456 \
    --output "$PREVIEW_DIR/18-preset-jazz-uptempo.wav"

# ============================================
# Example 19: Walking bass only (Bb) - deep, plucked
# ============================================
echo "19. Walking bass - Bb deep (8 sec)"
"$CLI" generate \
    --notes "Bb1:0.7:95@0,D2:0.7:90@1,F2:0.7:95@2,A1:0.7:90@3,Bb1:0.7:95@4,G1:0.7:90@5,F1:0.7:95@6,E1:0.7:90@7,Eb1:0.7:95@8,G1:0.7:90@9,Bb1:0.7:95@10,D2:0.7:90@11,Eb2:0.7:95@12,D2:0.7:90@13,C2:0.7:95@14,Bb1:0.7:100@15" \
    --instrument bass \
    --tempo 120 \
    --output "$PREVIEW_DIR/19-walking-bass.wav"

echo ""
echo "Generated samples in $PREVIEW_DIR:"
ls -la "$PREVIEW_DIR"/*.wav 2>/dev/null || echo "No WAV files found"
echo ""
echo "Open $PREVIEW_DIR/index.html in a browser to preview."
