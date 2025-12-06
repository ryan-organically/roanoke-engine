//! Procedural Synthesizer for Roanoke Engine
//!
//! Real-time audio synthesis inspired by Jeremy Soule's ambient orchestral style.
//! Generates evolving drones, pads, and melodic fragments algorithmically.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::Mutex;
use rand::Rng;

/// Oscillator waveform types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Triangle,
    SoftSquare,  // Square with rounded edges
    Saw,
    Noise,       // For texture/breath sounds
}

/// ADSR Envelope state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

/// ADSR Envelope generator
#[derive(Debug, Clone)]
pub struct Envelope {
    pub attack: f32,   // seconds
    pub decay: f32,    // seconds
    pub sustain: f32,  // level 0-1
    pub release: f32,  // seconds
    state: EnvelopeState,
    level: f32,
    time: f32,
}

impl Envelope {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            state: EnvelopeState::Idle,
            level: 0.0,
            time: 0.0,
        }
    }

    pub fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.time = 0.0;
    }

    pub fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.time = 0.0;
        }
    }

    pub fn process(&mut self, dt: f32) -> f32 {
        self.time += dt;

        match self.state {
            EnvelopeState::Attack => {
                if self.attack > 0.0 {
                    self.level = (self.time / self.attack).min(1.0);
                    if self.time >= self.attack {
                        self.state = EnvelopeState::Decay;
                        self.time = 0.0;
                    }
                } else {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                    self.time = 0.0;
                }
            }
            EnvelopeState::Decay => {
                if self.decay > 0.0 {
                    let decay_amount = (self.time / self.decay).min(1.0);
                    self.level = 1.0 - decay_amount * (1.0 - self.sustain);
                    if self.time >= self.decay {
                        self.state = EnvelopeState::Sustain;
                    }
                } else {
                    self.level = self.sustain;
                    self.state = EnvelopeState::Sustain;
                }
            }
            EnvelopeState::Sustain => {
                self.level = self.sustain;
            }
            EnvelopeState::Release => {
                if self.release > 0.0 {
                    let release_progress = (self.time / self.release).min(1.0);
                    self.level = self.sustain * (1.0 - release_progress);
                    if self.time >= self.release {
                        self.state = EnvelopeState::Idle;
                        self.level = 0.0;
                    }
                } else {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
            EnvelopeState::Idle => {
                self.level = 0.0;
            }
        }

        self.level
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }
}

/// Single oscillator with phase accumulator
#[derive(Debug, Clone)]
pub struct Oscillator {
    pub frequency: f32,
    pub waveform: Waveform,
    phase: f32,
    phase_increment: f32,
    sample_rate: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            frequency: 440.0,
            waveform: Waveform::Sine,
            phase: 0.0,
            phase_increment: 0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
        self.phase_increment = freq / self.sample_rate;
    }

    pub fn process(&mut self) -> f32 {
        let sample = match self.waveform {
            Waveform::Sine => (self.phase * std::f32::consts::TAU).sin(),
            Waveform::Triangle => {
                let t = self.phase;
                if t < 0.5 {
                    4.0 * t - 1.0
                } else {
                    3.0 - 4.0 * t
                }
            }
            Waveform::SoftSquare => {
                // Square wave with smoothing
                let raw: f32 = if self.phase < 0.5 { 1.0 } else { -1.0 };
                // Soft clip
                (raw * 2.0).tanh() * 0.5
            }
            Waveform::Saw => 2.0 * self.phase - 1.0,
            Waveform::Noise => rand::thread_rng().gen_range(-1.0..1.0),
        };

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

/// Low-pass filter (simple one-pole)
#[derive(Debug, Clone)]
pub struct LowPassFilter {
    cutoff: f32,
    resonance: f32,
    sample_rate: f32,
    prev_sample: f32,
    alpha: f32,
}

impl LowPassFilter {
    pub fn new(sample_rate: f32, cutoff: f32) -> Self {
        let mut filter = Self {
            cutoff,
            resonance: 0.0,
            sample_rate,
            prev_sample: 0.0,
            alpha: 0.0,
        };
        filter.set_cutoff(cutoff);
        filter
    }

    pub fn set_cutoff(&mut self, freq: f32) {
        self.cutoff = freq.clamp(20.0, self.sample_rate * 0.49);
        let rc = 1.0 / (self.cutoff * std::f32::consts::TAU);
        let dt = 1.0 / self.sample_rate;
        self.alpha = dt / (rc + dt);
    }

    pub fn process(&mut self, input: f32) -> f32 {
        self.prev_sample = self.prev_sample + self.alpha * (input - self.prev_sample);
        self.prev_sample
    }
}

/// Reverb effect using simple delay network
#[derive(Debug, Clone)]
pub struct SimpleReverb {
    delay_lines: Vec<Vec<f32>>,
    write_positions: Vec<usize>,
    decay: f32,
    wet_mix: f32,
}

impl SimpleReverb {
    pub fn new(sample_rate: f32, decay: f32, wet_mix: f32) -> Self {
        // Prime-number delay lengths for diffusion
        let delay_samples: Vec<usize> = vec![
            (sample_rate * 0.0297) as usize,  // ~30ms
            (sample_rate * 0.0371) as usize,  // ~37ms
            (sample_rate * 0.0411) as usize,  // ~41ms
            (sample_rate * 0.0437) as usize,  // ~44ms
        ];

        let delay_lines = delay_samples.iter().map(|&len| vec![0.0; len]).collect();
        let write_positions = vec![0; 4];

        Self {
            delay_lines,
            write_positions,
            decay,
            wet_mix,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut wet = 0.0;

        for (i, delay_line) in self.delay_lines.iter_mut().enumerate() {
            let len = delay_line.len();
            let read_pos = self.write_positions[i];

            // Read from delay line
            let delayed = delay_line[read_pos];
            wet += delayed;

            // Write input + feedback to delay line
            delay_line[read_pos] = input + delayed * self.decay;

            // Advance write position
            self.write_positions[i] = (read_pos + 1) % len;
        }

        wet *= 0.25; // Average the 4 delay lines
        input * (1.0 - self.wet_mix) + wet * self.wet_mix
    }

    pub fn set_wet_mix(&mut self, mix: f32) {
        self.wet_mix = mix.clamp(0.0, 1.0);
    }
}

/// A single synthesizer voice (drone, pad, etc.)
#[derive(Debug, Clone)]
pub struct SynthVoice {
    pub name: String,
    oscillators: Vec<Oscillator>,
    envelope: Envelope,
    filter: LowPassFilter,
    pub volume: f32,
    pub target_volume: f32,
    pub target_frequency: f32,
    detune_cents: Vec<f32>,  // Detune for richness
    filter_envelope_amount: f32,
    base_cutoff: f32,
}

impl SynthVoice {
    pub fn new_drone(sample_rate: f32) -> Self {
        // Drone: multiple detuned oscillators for rich pad sound
        let mut oscillators = Vec::new();
        let detune = vec![-8.0, -3.0, 0.0, 3.0, 8.0]; // cents

        for _ in &detune {
            let mut osc = Oscillator::new(sample_rate);
            osc.waveform = Waveform::Sine;
            oscillators.push(osc);
        }

        Self {
            name: "drone".to_string(),
            oscillators,
            envelope: Envelope::new(4.0, 1.0, 0.8, 6.0), // Very slow attack/release
            filter: LowPassFilter::new(sample_rate, 800.0),
            volume: 0.0,
            target_volume: 0.0,
            target_frequency: 110.0,
            detune_cents: detune,
            filter_envelope_amount: 400.0,
            base_cutoff: 600.0,
        }
    }

    pub fn new_pad(sample_rate: f32) -> Self {
        let mut oscillators = Vec::new();
        let detune = vec![-5.0, 0.0, 5.0, 12.0]; // With octave doubling

        for _ in &detune {
            let mut osc = Oscillator::new(sample_rate);
            osc.waveform = Waveform::Triangle;
            oscillators.push(osc);
        }

        Self {
            name: "pad".to_string(),
            oscillators,
            envelope: Envelope::new(3.0, 0.5, 0.7, 4.0),
            filter: LowPassFilter::new(sample_rate, 2000.0),
            volume: 0.0,
            target_volume: 0.0,
            target_frequency: 220.0,
            detune_cents: detune,
            filter_envelope_amount: 800.0,
            base_cutoff: 1200.0,
        }
    }

    pub fn new_melody(sample_rate: f32) -> Self {
        let mut oscillators = Vec::new();
        let detune = vec![0.0, 0.1]; // Very slight detune for warmth

        for _ in &detune {
            let mut osc = Oscillator::new(sample_rate);
            osc.waveform = Waveform::Sine;
            oscillators.push(osc);
        }

        Self {
            name: "melody".to_string(),
            oscillators,
            envelope: Envelope::new(0.3, 0.2, 0.6, 2.0),
            filter: LowPassFilter::new(sample_rate, 4000.0),
            volume: 0.0,
            target_volume: 0.0,
            target_frequency: 440.0,
            detune_cents: detune,
            filter_envelope_amount: 1000.0,
            base_cutoff: 3000.0,
        }
    }

    pub fn new_shimmer(sample_rate: f32) -> Self {
        let mut oscillators = Vec::new();
        let detune = vec![-7.0, 0.0, 7.0];

        for _ in &detune {
            let mut osc = Oscillator::new(sample_rate);
            osc.waveform = Waveform::Sine;
            oscillators.push(osc);
        }

        Self {
            name: "shimmer".to_string(),
            oscillators,
            envelope: Envelope::new(0.5, 0.3, 0.4, 3.0),
            filter: LowPassFilter::new(sample_rate, 6000.0),
            volume: 0.0,
            target_volume: 0.0,
            target_frequency: 880.0,
            detune_cents: detune,
            filter_envelope_amount: 500.0,
            base_cutoff: 5000.0,
        }
    }

    pub fn new_bass(sample_rate: f32) -> Self {
        let mut oscillators = Vec::new();
        let detune = vec![0.0, 0.0]; // Sub + fundamental

        for (i, _) in detune.iter().enumerate() {
            let mut osc = Oscillator::new(sample_rate);
            osc.waveform = if i == 0 { Waveform::Sine } else { Waveform::Triangle };
            oscillators.push(osc);
        }

        Self {
            name: "bass".to_string(),
            oscillators,
            envelope: Envelope::new(0.1, 0.3, 0.5, 1.5),
            filter: LowPassFilter::new(sample_rate, 300.0),
            volume: 0.0,
            target_volume: 0.0,
            target_frequency: 55.0,
            detune_cents: detune,
            filter_envelope_amount: 200.0,
            base_cutoff: 200.0,
        }
    }

    pub fn set_frequency(&mut self, freq: f32) {
        self.target_frequency = freq;
        for (i, osc) in self.oscillators.iter_mut().enumerate() {
            let detune_ratio = 2.0_f32.powf(self.detune_cents[i] / 1200.0);
            osc.set_frequency(freq * detune_ratio);
        }
    }

    pub fn trigger(&mut self) {
        self.envelope.trigger();
    }

    pub fn release(&mut self) {
        self.envelope.release();
    }

    pub fn process(&mut self, dt: f32) -> f32 {
        // Update volume smoothly
        self.volume += (self.target_volume - self.volume) * dt * 2.0;

        // Process envelope
        let env = self.envelope.process(dt);

        // Sum oscillators
        let mut sample = 0.0;
        for osc in &mut self.oscillators {
            sample += osc.process();
        }
        sample /= self.oscillators.len() as f32;

        // Apply filter with envelope modulation
        let cutoff = self.base_cutoff + env * self.filter_envelope_amount;
        self.filter.set_cutoff(cutoff);
        sample = self.filter.process(sample);

        // Apply envelope and volume
        sample * env * self.volume
    }

    pub fn is_active(&self) -> bool {
        self.envelope.is_active() || self.target_volume > 0.01
    }
}

/// Shared state for audio thread communication
#[derive(Debug)]
pub struct SynthState {
    pub voices: Vec<SynthVoice>,
    pub reverb_mix: f32,
    pub master_volume: f32,
    pub is_playing: AtomicBool,
}

impl SynthState {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            voices: vec![
                SynthVoice::new_drone(sample_rate),
                SynthVoice::new_pad(sample_rate),
                SynthVoice::new_melody(sample_rate),
                SynthVoice::new_shimmer(sample_rate),
                SynthVoice::new_bass(sample_rate),
            ],
            reverb_mix: 0.4,
            master_volume: 0.5,
            is_playing: AtomicBool::new(true),
        }
    }
}

/// The procedural synthesizer - generates audio samples
pub struct ProceduralSynth {
    pub state: Arc<Mutex<SynthState>>,
    reverb: SimpleReverb,
    sample_rate: f32,
}

impl ProceduralSynth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state: Arc::new(Mutex::new(SynthState::new(sample_rate))),
            reverb: SimpleReverb::new(sample_rate, 0.6, 0.4),
            sample_rate,
        }
    }

    /// Get shared state for external control
    pub fn get_state(&self) -> Arc<Mutex<SynthState>> {
        Arc::clone(&self.state)
    }

    /// Update voice parameters from audio system layer data
    pub fn update_from_layers(&mut self, layer_data: &[(String, f32, f32)]) {
        if let Ok(mut state) = self.state.lock() {
            for (name, volume, frequency) in layer_data {
                if let Some(voice) = state.voices.iter_mut().find(|v| &v.name == name) {
                    voice.target_volume = *volume;
                    if *frequency > 20.0 {
                        voice.set_frequency(*frequency);
                    }

                    // Auto-trigger envelope when volume rises
                    if *volume > 0.05 && !voice.envelope.is_active() {
                        voice.trigger();
                    } else if *volume < 0.02 && voice.envelope.is_active() {
                        voice.release();
                    }
                }
            }
        }
    }

    /// Set reverb amount
    pub fn set_reverb(&mut self, mix: f32) {
        self.reverb.set_wet_mix(mix);
        if let Ok(mut state) = self.state.lock() {
            state.reverb_mix = mix;
        }
    }

    /// Generate a single audio sample (called at sample rate)
    pub fn process_sample(&mut self) -> f32 {
        let dt = 1.0 / self.sample_rate;
        let mut sample = 0.0;
        let mut master_vol = 0.5;

        if let Ok(mut state) = self.state.lock() {
            if !state.is_playing.load(Ordering::Relaxed) {
                return 0.0;
            }

            master_vol = state.master_volume;

            for voice in &mut state.voices {
                if voice.is_active() {
                    sample += voice.process(dt);
                }
            }
        }

        // Apply reverb
        sample = self.reverb.process(sample);

        // Soft clip to prevent harsh distortion
        (sample * master_vol).tanh() * 0.8
    }

    /// Fill a buffer with generated audio (stereo interleaved)
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for chunk in buffer.chunks_mut(2) {
            let sample = self.process_sample();
            chunk[0] = sample; // Left
            chunk[1] = sample; // Right (mono for now, could add stereo spread)
        }
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        if let Ok(mut state) = self.state.lock() {
            state.master_volume = volume.clamp(0.0, 1.0);
        }
    }

    /// Pause/resume synthesis
    pub fn set_playing(&mut self, playing: bool) {
        if let Ok(state) = self.state.lock() {
            state.is_playing.store(playing, Ordering::Relaxed);
        }
    }
}

/// Helper to convert semitones to frequency ratio
pub fn semitones_to_ratio(semitones: f32) -> f32 {
    2.0_f32.powf(semitones / 12.0)
}

/// Helper to convert frequency to MIDI note number
pub fn freq_to_midi(freq: f32) -> f32 {
    69.0 + 12.0 * (freq / 440.0).log2()
}

/// Helper to convert MIDI note number to frequency
pub fn midi_to_freq(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}
