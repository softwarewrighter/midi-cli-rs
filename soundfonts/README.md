# SoundFonts

Place SoundFont (.sf2) files here for MIDI-to-WAV rendering.

## Recommended (MIT Licensed)

For commercial use with clear licensing:

1. **FluidR3_GM.sf2** (~140MB) - MIT licensed
   - macOS: Included with `brew install fluid-synth`
   - Copy from `/opt/homebrew/share/sounds/sf2/FluidR3_GM.sf2`

2. **GeneralUser_GS.sf2** (~30MB) - Permissive license
   - Download: https://schristiancollins.com/generaluser.php
   - Explicitly allows commercial music production

## Auto-Detection

The CLI searches for soundfonts in this order:
1. `./soundfonts/FluidR3_GM.sf2`
2. `./soundfonts/GeneralUser_GS.sf2`
3. `./soundfonts/MuseScore_General.sf2`
4. `./soundfonts/default.sf2`
5. System paths (`/opt/homebrew/...`, `/usr/share/...`)

## License Warning

Avoid GPL-licensed soundfonts (e.g., TimGM6mb) if you need unambiguous
commercial rights for rendered audio.
