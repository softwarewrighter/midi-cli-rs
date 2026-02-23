//! Orchestral preset - Full symphonic orchestration
//!
//! Creates rich orchestral textures with:
//! - Full string section (violins, violas, cellos, basses)
//! - Woodwind choir (flutes, oboes, clarinets, bassoons)
//! - Brass section (horns, trumpets, trombones)
//! - Orchestral percussion (timpani, cymbals)
//! - Harp for color

use crate::midi::{Note, NoteSequence};
use crate::preset::{Key, MoodGenerator, PresetConfig, PresetVariation, create_rng};
use rand::Rng;

/// GM program numbers for orchestral instruments
const STRING_SECTION: &[u8] = &[48, 49, 44, 45]; // Strings, tremolo, pizzicato
const CONTRABASS: &[u8] = &[43, 32];  // Contrabass, acoustic bass
const WOODWINDS: &[u8] = &[73, 74, 68, 71, 70]; // Flute, recorder, oboe, clarinet, bassoon
const BRASS: &[u8] = &[60, 56, 57, 58]; // Horn, trumpet, trombone, tuba
const HARP: u8 = 46;
const TIMPANI: u8 = 47;

pub struct OrchestralPreset;

impl MoodGenerator for OrchestralPreset {
    fn generate(&self, config: &PresetConfig) -> Vec<NoteSequence> {
        let mut rng = create_rng(config.seed);
        let variation = PresetVariation::from_seed(config.seed);
        let mut sequences = Vec::new();

        let beats = config.duration_secs * (config.tempo as f64 / 60.0);
        let root = config.key.root();
        let scale = config.key.scale_intervals();
        let tempo = config.tempo;

        // Layer 0: String section (always present)
        sequences.push(generate_strings(&mut rng, &variation, beats, root, scale, tempo));

        // Layer 1: Bass section (high probability)
        if variation.include_layer(1, config.intensity, 20) {
            sequences.push(generate_bass(&mut rng, &variation, beats, root, scale, tempo));
        }

        // Layer 2: Woodwinds (intensity dependent)
        if variation.include_layer(2, config.intensity, 35) {
            sequences.push(generate_woodwinds(&mut rng, &variation, beats, root, scale, tempo));
        }

        // Layer 3: Brass (moderate probability)
        if variation.include_layer(3, config.intensity, 45) {
            sequences.push(generate_brass(&mut rng, &variation, beats, root, scale, tempo));
        }

        // Layer 4: Harp (adds color)
        if variation.include_layer(4, config.intensity, 55) {
            sequences.push(generate_harp(&mut rng, &variation, beats, root, scale, tempo));
        }

        // Layer 5: Timpani
        if variation.include_layer(5, config.intensity, 50) {
            sequences.push(generate_timpani(&mut rng, &variation, beats, root, tempo));
        }

        sequences
    }

    fn name(&self) -> &'static str { "orchestral" }

    fn description(&self) -> &'static str {
        "Full symphonic orchestra with strings, woodwinds, brass, and percussion"
    }
}

fn generate_strings(
    rng: &mut impl Rng, variation: &PresetVariation, beats: f64, root: u8, scale: &[u8], tempo: u16,
) -> NoteSequence {
    let instrument = variation.pick_instrument(0, STRING_SECTION);
    let mut notes = Vec::new();
    let style = variation.pick_style(0, 4);
    let mut t = 0.0;

    while t < beats {
        match style {
            0 => {
                // Sustained chords
                let dur = rng.gen_range(2.0..4.0);
                let vel = variation.adjust_velocity(rng.gen_range(55..75));
                for (i, idx) in [0, 2, 4, 6].iter().enumerate() {
                    let interval = scale[*idx % scale.len()];
                    let octave: i8 = match i { 0 => -12, 3 => 12, _ => 0 };
                    let pitch = (root as i8 + interval as i8 + octave).clamp(36, 96) as u8;
                    notes.push(Note::new(pitch, dur, vel, t));
                }
                t += dur;
            }
            1 => {
                // Tremolo
                let section_dur = rng.gen_range(1.5..3.0);
                let base = root + scale[rng.gen_range(0..scale.len())];
                let mut nt = t;
                while nt < t + section_dur && nt < beats {
                    let pitch = (base as i8 + rng.gen_range(-2..3)).clamp(48, 84) as u8;
                    notes.push(Note::new(pitch, 0.125, variation.adjust_velocity(rng.gen_range(50..70)), nt));
                    nt += 0.125;
                }
                t += section_dur;
            }
            2 => {
                // Melodic unison
                let contour = variation.get_contour(variation.phrase_length as usize);
                let mut scale_idx = variation.scale_offset as usize;
                for (i, dir) in contour.iter().enumerate() {
                    if t >= beats { break; }
                    let pitch = (root as i8 + scale[scale_idx % scale.len()] as i8).clamp(55, 79) as u8;
                    let dur = if i == contour.len() - 1 { rng.gen_range(1.0..2.0) } else { rng.gen_range(0.4..0.8) };
                    let vel = variation.adjust_velocity(65);
                    notes.push(Note::new(pitch, dur, vel, t));
                    notes.push(Note::new(pitch.saturating_add(12).min(96), dur, vel.saturating_sub(10), t));
                    match *dir { 1 => scale_idx = (scale_idx + 1) % scale.len(), -1 => scale_idx = (scale_idx + scale.len() - 1) % scale.len(), _ => {} }
                    t += dur;
                }
            }
            _ => {
                // Polyphonic
                let section: f64 = rng.gen_range(3.0_f64..5.0_f64).min(beats - t);
                for voice in 0..3 {
                    let base = root as i8 + (voice as i8 - 1) * 7;
                    let mut vt = t;
                    while vt < t + section {
                        let pitch = (base + scale[rng.gen_range(0..scale.len())] as i8).clamp(40, 88) as u8;
                        let dur = rng.gen_range(0.5..1.5);
                        notes.push(Note::new(pitch, dur, variation.adjust_velocity(rng.gen_range(50..70)), vt));
                        vt += dur + rng.gen_range(0.0..0.3);
                    }
                }
                t += section;
            }
        }
    }
    NoteSequence::new(notes, instrument, tempo)
}

fn generate_bass(
    rng: &mut impl Rng, variation: &PresetVariation, beats: f64, root: u8, scale: &[u8], tempo: u16,
) -> NoteSequence {
    let instrument = variation.pick_instrument(1, CONTRABASS);
    let mut notes = Vec::new();
    let bass_root = root.saturating_sub(24).max(28);
    let style = variation.pick_style(1, 3);
    let mut t = 0.0;

    while t < beats {
        match style {
            0 => {
                let dur = rng.gen_range(2.0..4.0);
                let pitch = bass_root + scale[rng.gen_range(0..3) % scale.len()];
                notes.push(Note::new(pitch, dur, variation.adjust_velocity(rng.gen_range(60..80)), t));
                t += dur;
            }
            1 => {
                let contour = variation.get_contour(4);
                let mut scale_idx = 0;
                for dir in contour.iter() {
                    if t >= beats { break; }
                    let pitch = (bass_root as i8 + scale[scale_idx % scale.len()] as i8).clamp(28, 55) as u8;
                    notes.push(Note::new(pitch, 0.9, variation.adjust_velocity(rng.gen_range(65..85)), t));
                    match *dir { 1 => scale_idx = (scale_idx + 1) % scale.len(), -1 => scale_idx = (scale_idx + scale.len() - 1) % scale.len(), _ => {} }
                    t += 1.0;
                }
            }
            _ => {
                if (t % 4.0) < 0.1 || ((t % 4.0) - 2.0).abs() < 0.1 {
                    notes.push(Note::new(bass_root + scale[0], 0.3, variation.adjust_velocity(rng.gen_range(70..90)), t));
                }
                t += 0.5;
            }
        }
    }
    NoteSequence::new(notes, instrument, tempo)
}

fn generate_woodwinds(
    rng: &mut impl Rng, variation: &PresetVariation, beats: f64, root: u8, scale: &[u8], tempo: u16,
) -> NoteSequence {
    let instrument = variation.pick_instrument(2, WOODWINDS);
    let mut notes = Vec::new();
    let style = variation.pick_style(2, 4);
    let ww_root = root + 12;
    let mut t = if beats > 4.0 { 1.0 } else { 0.0 };

    while t < beats {
        match style {
            0 => {
                let dur = rng.gen_range(2.0..3.5);
                let vel = variation.adjust_velocity(rng.gen_range(55..70));
                for i in 0..2 {
                    let pitch = (ww_root as i8 + scale[(i * 2) % scale.len()] as i8).clamp(60, 96) as u8;
                    notes.push(Note::new(pitch, dur, vel, t));
                }
                t += dur + rng.gen_range(0.5..1.5);
            }
            1 => {
                let contour = variation.get_contour(variation.phrase_length as usize);
                let mut scale_idx = variation.scale_offset as usize;
                for (i, dir) in contour.iter().enumerate() {
                    if t >= beats { break; }
                    if variation.should_rest(rng) { t += rng.gen_range(0.3..0.6); continue; }
                    let pitch = (ww_root as i8 + scale[scale_idx % scale.len()] as i8).clamp(60, 91) as u8;
                    let dur = if i == contour.len() - 1 { rng.gen_range(1.0..1.5) } else { rng.gen_range(0.3..0.7) };
                    notes.push(Note::new(pitch, dur, variation.adjust_velocity(rng.gen_range(60..80)), t));
                    match *dir { 1 => scale_idx = (scale_idx + 1) % scale.len(), -1 => scale_idx = (scale_idx + scale.len() - 1) % scale.len(), _ => {} }
                    t += dur;
                }
            }
            2 => {
                let section = rng.gen_range(1.0..2.0);
                let base = ww_root + scale[rng.gen_range(0..scale.len())];
                let trill: i8 = if rng.gen_bool(0.5) { 1 } else { 2 };
                let mut nt = t;
                let mut alt = false;
                while nt < t + section && nt < beats {
                    let pitch = if alt { (base as i8 + trill).clamp(60, 96) as u8 } else { base };
                    notes.push(Note::new(pitch, 0.1, variation.adjust_velocity(rng.gen_range(50..65)), nt));
                    nt += 0.1;
                    alt = !alt;
                }
                t += section + rng.gen_range(1.0..2.0);
            }
            _ => {
                let high = ww_root + scale[rng.gen_range(2..scale.len())];
                let low = root + scale[rng.gen_range(0..3)];
                notes.push(Note::new(high, 0.75, variation.adjust_velocity(rng.gen_range(65..80)), t));
                if t + 1.0 < beats {
                    notes.push(Note::new(low, 0.75, variation.adjust_velocity(rng.gen_range(55..70)), t + 1.0));
                }
                t += 2.5;
            }
        }
    }
    NoteSequence::new(notes, instrument, tempo)
}

fn generate_brass(
    rng: &mut impl Rng, variation: &PresetVariation, beats: f64, root: u8, scale: &[u8], tempo: u16,
) -> NoteSequence {
    let instrument = variation.pick_instrument(3, BRASS);
    let mut notes = Vec::new();
    let style = variation.pick_style(3, 4);
    let mut t = if beats > 6.0 { beats * 0.3 } else if beats > 3.0 { 1.0 } else { 0.0 };

    while t < beats {
        match style {
            0 => {
                // Fanfare
                for (interval, dur) in [(0i8, 0.5), (4, 0.5), (7, 1.0), (12, 1.5)] {
                    if t >= beats { break; }
                    let pitch = (root as i8 + interval).clamp(48, 84) as u8;
                    notes.push(Note::new(pitch, dur, variation.adjust_velocity(rng.gen_range(75..95)), t));
                    t += dur;
                }
                t += rng.gen_range(2.0..4.0);
            }
            1 => {
                let dur = rng.gen_range(2.0..4.0);
                let vel = variation.adjust_velocity(rng.gen_range(70..90));
                for interval in [0i8, 7, 12] {
                    let pitch = (root as i8 + interval).clamp(48, 84) as u8;
                    notes.push(Note::new(pitch, dur, vel, t));
                }
                t += dur + rng.gen_range(1.0..3.0);
            }
            2 => {
                if (t % 4.0) < 0.1 {
                    let pitch = (root as i8 + scale[rng.gen_range(0..3) % scale.len()] as i8).clamp(48, 79) as u8;
                    notes.push(Note::new(pitch, 0.3, variation.adjust_velocity(rng.gen_range(80..100)), t));
                }
                t += 1.0;
            }
            _ => {
                let pattern: &[(i8, f64)] = match rng.gen_range(0..3) {
                    0 => &[(0, 0.75), (7, 0.25), (12, 1.0)],
                    1 => &[(7, 0.5), (12, 0.5), (7, 1.0)],
                    _ => &[(0, 1.0), (4, 0.5), (7, 0.5)],
                };
                for (interval, dur) in pattern {
                    if t >= beats { break; }
                    let pitch = (root as i8 + interval).clamp(48, 79) as u8;
                    notes.push(Note::new(pitch, *dur, variation.adjust_velocity(rng.gen_range(70..85)), t));
                    t += dur;
                }
                t += rng.gen_range(2.0..4.0);
            }
        }
    }
    NoteSequence::new(notes, instrument, tempo)
}

fn generate_harp(
    rng: &mut impl Rng, variation: &PresetVariation, beats: f64, root: u8, scale: &[u8], tempo: u16,
) -> NoteSequence {
    let mut notes = Vec::new();
    let style = variation.pick_style(4, 3);
    let mut t = 0.0;

    while t < beats {
        match style {
            0 => {
                // Glissando
                let up = rng.gen_bool(0.6);
                let num = rng.gen_range(6..12);
                for i in 0..num {
                    let idx = if up { i % scale.len() } else { (scale.len() - 1 - (i % scale.len())) % scale.len() };
                    let oct = (i / scale.len()) as i8 * 12;
                    let offset = if up { oct } else { -oct };
                    let pitch = (root as i8 + scale[idx] as i8 + offset).clamp(36, 96) as u8;
                    notes.push(Note::new(pitch, 1.5, variation.adjust_velocity(rng.gen_range(50..70)), t + i as f64 * 0.1));
                }
                t += num as f64 * 0.1 + rng.gen_range(2.0..4.0);
            }
            1 => {
                // Rolled chord
                for (i, deg) in [0, 2, 4, 6].iter().enumerate() {
                    let pitch = (root as i8 + scale[*deg % scale.len()] as i8).clamp(48, 84) as u8;
                    notes.push(Note::new(pitch, 2.0, variation.adjust_velocity(rng.gen_range(55..70)), t + i as f64 * 0.08));
                }
                t += 3.0;
            }
            _ => {
                if rng.gen_bool(0.4) {
                    let oct: i8 = if rng.gen_bool(0.5) { 0 } else { 12 };
                    let pitch = (root as i8 + scale[rng.gen_range(0..scale.len())] as i8 + oct).clamp(48, 91) as u8;
                    notes.push(Note::new(pitch, rng.gen_range(1.0..2.0), variation.adjust_velocity(rng.gen_range(45..65)), t));
                }
                t += rng.gen_range(1.0..2.0);
            }
        }
    }
    NoteSequence::new(notes, HARP, tempo)
}

fn generate_timpani(
    rng: &mut impl Rng, variation: &PresetVariation, beats: f64, root: u8, tempo: u16,
) -> NoteSequence {
    let mut notes = Vec::new();
    let style = variation.pick_style(5, 3);
    let timp_root = root.saturating_sub(24).max(36);
    let timp_fifth = timp_root + 7;
    let mut t = 0.0;

    while t < beats {
        match style {
            0 => {
                if (t % 4.0) < 0.1 {
                    let dur = rng.gen_range(0.5..1.0);
                    let mut rt = t;
                    while rt < t + dur && rt < beats {
                        notes.push(Note::new(timp_root, 0.08, variation.adjust_velocity(rng.gen_range(50..70)), rt));
                        rt += 0.08;
                    }
                }
                t += 2.0;
            }
            1 => {
                let patterns: &[&[f64]] = &[&[0.0, 2.0], &[0.0, 1.0, 3.0], &[0.0, 0.5, 2.0, 2.5]];
                let pattern = patterns[rng.gen_range(0..patterns.len())];
                for &offset in pattern {
                    if t + offset < beats {
                        let pitch = if rng.gen_bool(0.7) { timp_root } else { timp_fifth };
                        notes.push(Note::new(pitch, 0.4, variation.adjust_velocity(rng.gen_range(70..90)), t + offset));
                    }
                }
                t += 4.0;
            }
            _ => {
                if rng.gen_bool(0.25) {
                    let pitch = if rng.gen_bool(0.8) { timp_root } else { timp_fifth };
                    notes.push(Note::new(pitch, 0.5, variation.adjust_velocity(rng.gen_range(75..95)), t));
                }
                t += rng.gen_range(1.0..2.0);
            }
        }
    }
    NoteSequence::new(notes, TIMPANI, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestral_generates_sequences() {
        let config = PresetConfig { duration_secs: 8.0, key: Key::C, intensity: 50, seed: 42, tempo: 80 };
        let sequences = OrchestralPreset.generate(&config);
        assert!(!sequences.is_empty());
    }

    #[test]
    fn test_orchestral_different_seeds_vary() {
        let c1 = PresetConfig { seed: 42, ..Default::default() };
        let c2 = PresetConfig { seed: 43, ..Default::default() };
        let s1 = OrchestralPreset.generate(&c1);
        let s2 = OrchestralPreset.generate(&c2);
        let different = s1.len() != s2.len() || s1.iter().zip(s2.iter()).any(|(a, b)| a.instrument != b.instrument || a.notes.len() != b.notes.len());
        assert!(different);
    }

    #[test]
    fn test_orchestral_instruments_vary() {
        let mut found = std::collections::HashSet::new();
        for seed in 1..20 {
            let config = PresetConfig { seed, duration_secs: 5.0, intensity: 70, ..Default::default() };
            for seq in OrchestralPreset.generate(&config) { found.insert(seq.instrument); }
        }
        assert!(found.len() > 4);
    }
}
