ROANOKE ENGINE - AUDIO ASSETS
==============================

Directory Structure:
--------------------
assets/audio/
├── music/           - Background music tracks
│   ├── menu.ogg     - Main menu theme
│   └── exploration/ - In-game exploration tracks
├── ambience/        - Weather and environmental loops
│   ├── wind.ogg     - Wind ambience (loopable)
│   ├── rain.ogg     - Rain ambience (loopable)
│   ├── thunder.ogg  - Thunder sounds
│   ├── birds.ogg    - Birdsong ambience (loopable)
│   └── crickets.ogg - Cricket ambience for night (loopable)
└── sfx/             - Sound effects
    ├── footsteps/   - Footstep sounds per surface
    └── ui/          - UI interaction sounds

Supported Formats:
------------------
- OGG Vorbis (recommended - good quality, small size)
- WAV (uncompressed, large files)
- MP3 (supported but OGG preferred)
- FLAC (lossless, larger than OGG)

Audio Guidelines:
-----------------
- Ambience tracks should be seamlessly loopable
- Normalize audio levels to prevent clipping
- Sample rate: 44100 Hz recommended
- Bit depth: 16-bit minimum

Procedural Soundtrack Notes:
----------------------------
The engine includes a procedural soundtrack system inspired by Jeremy Soule's
atmospheric compositions. It generates music using:

- Layered synthesis (drone, pad, melody, shimmer, bass)
- Musical scales: Dorian (exploration), Aeolian (tension), Lydian (discovery)
- Dynamic response to weather, time of day, and game state
- Smooth crossfades between intensity levels

The procedural system can work standalone or alongside sampled music tracks.
