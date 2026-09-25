use macroquad::audio::{
    PlaySoundParams, Sound as MacroquadSound, load_sound_from_bytes, play_sound, stop_sound,
};

/// Literally write the code to make a WAV file
fn generate_beep_wav(frequency: f32, duration_secs: f32) -> Vec<u8> {
    const SAMPLE_RATE: u32 = 44_100;
    const AMPLITUDE: f32 = 16_384.0;

    let num_samples = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let data_size = num_samples * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample = (t * frequency * 2.0 * std::f32::consts::PI).sin();
        let amplitude = (sample * AMPLITUDE) as i16;

        wav.extend_from_slice(&amplitude.to_le_bytes());
    }

    wav
}

pub struct Sound {
    sound: MacroquadSound,
    playing: bool,
}

impl Sound {
    pub async fn new() -> Self {
        // The duration doesn't matter because we loop it.
        let bytes = generate_beep_wav(440.0, 0.1);
        let sound = load_sound_from_bytes(&bytes).await.unwrap();

        Self {
            sound,
            playing: false,
        }
    }

    pub fn update(&mut self, sound_timer: u8) {
        if sound_timer > 0 && !self.playing {
            play_sound(
                &self.sound,
                PlaySoundParams {
                    looped: true,
                    volume: 1.0,
                },
            );

            self.playing = true;
        } else if sound_timer == 0 && self.playing {
            stop_sound(&self.sound);
            self.playing = false;
        }
    }
}
