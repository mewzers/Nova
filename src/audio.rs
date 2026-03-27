use rodio::source::SineWave;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use crate::debug;

// EN: Minimal audio backend driving the CHIP-8 buzzer tone.
// FR: Backend audio minimal pilotant la tonalite du buzzer CHIP-8.
#[derive(Default)]
pub struct AudioEngine {
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
}

// EN: Do not clone runtime audio handles; recreate a fresh idle engine.
// FR: Ne clone pas les handles audio runtime; recree un moteur inactif propre.
impl Clone for AudioEngine {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl AudioEngine {
    // EN: Lazily initialize default output stream/handle.
    // FR: Initialise a la demande le flux/sortie audio par defaut.
    fn ensure_output(&mut self) -> bool {
        if self.handle.is_some() {
            return true;
        }
        match OutputStream::try_default() {
            Ok((stream, handle)) => {
                self.stream = Some(stream);
                self.handle = Some(handle);
                debug::log("audio_output_initialized");
                true
            }
            Err(_) => {
                debug::log("audio_output_failed");
                false
            }
        }
    }

    // EN: Start/refresh buzzer playback and map UI volume (0..100) to audio gain.
    // FR: Demarre/actualise le buzzer et mappe le volume UI (0..100) vers le gain audio.
    pub fn set_buzzer(&mut self, active: bool, volume_percent: u8) {
        if !active {
            self.stop();
            return;
        }
        if !self.ensure_output() {
            return;
        }

        let volume = (volume_percent.min(100) as f32 / 100.0) * 0.25;

        if self.sink.is_none() {
            if let Some(handle) = &self.handle
                && let Ok(sink) = Sink::try_new(handle)
            {
                let source = SineWave::new(880.0).amplify(volume).repeat_infinite();
                sink.append(source);
                sink.play();
                self.sink = Some(sink);
                debug::log("audio_playback_started");
            }
            return;
        }

        if let Some(sink) = &self.sink {
            sink.set_volume(volume);
            sink.play();
        }
    }

    // EN: Stop current buzzer sink if active.
    // FR: Arrete le sink du buzzer courant s il est actif.
    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
            debug::log("audio_playback_stopped");
        }
    }
}
