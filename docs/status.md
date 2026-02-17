# Project Status: midi-cli-rs

**Last Updated**: 2026-02-16

## Current Phase

**Phase 0: Documentation and Design** - COMPLETE

## Overall Progress

```
[##--------] 20% Complete
```

| Phase | Status | Progress |
|-------|--------|----------|
| Phase 0: Documentation | Complete | 100% |
| Phase 1: Core MIDI Generation | Not Started | 0% |
| Phase 2: CLI Interface | Not Started | 0% |
| Phase 3: WAV Rendering | Not Started | 0% |
| Phase 4: Mood Presets | Not Started | 0% |
| Phase 5: Polish | Not Started | 0% |

## Completed Work

### Phase 0: Documentation (2026-02-16)

- [x] Created docs/prd.md - Product requirements document
- [x] Created docs/architecture.md - System architecture with licensing analysis
- [x] Created docs/design.md - Detailed design decisions
- [x] Created docs/plan.md - Implementation plan with phases
- [x] Created docs/status.md - This file
- [x] Initial project structure (Cargo.toml, src/main.rs)

## In Progress

None - ready to begin Phase 1 implementation.

## Blocked

None.

## Recent Decisions

### Licensing Strategy (2026-02-16)

**Decision**: Use only permissively-licensed dependencies and SoundFonts.

**Rationale**: Output audio must be commercially usable (YouTube monetization).

**Key Points**:
- All Rust crates: MIT/Apache-2.0 dual licensed
- FluidSynth: LGPL-2.1 (OK for generated output)
- SoundFonts: MIT licensed (FluidR3_GM, MuseScore_General)

### FluidSynth Integration (2026-02-16)

**Decision**: Use FluidSynth as external process, not library binding.

**Rationale**:
- Simpler build (no C library linking)
- Clearer LGPL license boundary
- Easier cross-platform support

### Input Format (2026-02-16)

**Decision**: Support both CLI arguments and JSON stdin.

**Rationale**: CLI args for simple cases, JSON for complex multi-track sequences.

### Output Format (2026-02-16)

**Decision**: Support MIDI and WAV output, determined by file extension.

**Rationale**: MIDI for flexibility, WAV for immediate playback.

## Next Steps

### Immediate (Phase 1)

1. Add dependencies to Cargo.toml (midly, thiserror)
2. Implement Note struct with parsing
3. Implement NoteSequence struct
4. Implement MIDI file writer
5. Add unit tests

### Short-term (Phases 2-3)

1. Add CLI with clap
2. Implement JSON input parsing
3. Integrate FluidSynth for WAV rendering

### Medium-term (Phase 4)

1. Implement mood preset generators
2. Add preset command

## Risks and Concerns

| Risk | Status | Mitigation |
|------|--------|------------|
| FluidSynth availability | Low | Clear error messages, optional feature |
| Cross-platform compatibility | Medium | Test early on Linux and macOS |
| MIDI complexity | Low | Using well-tested midly crate |

## Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Test coverage | > 80% | N/A |
| Clippy warnings | 0 | N/A |
| Generation time (5s clip) | < 10ms | N/A |
| Render time (5s clip) | < 500ms | N/A |

## Notes for AI Agents

This project is designed to be used by AI coding agents. Key points:

1. **Determinism**: Always use `--seed` for reproducible output
2. **Single-instrument**: Output one instrument per invocation, layer with sox/ffmpeg
3. **Error messages**: Formatted for easy parsing
4. **Documentation**: Check `--help` for full command reference

## Contact

Project maintained by Software Wrighter.
