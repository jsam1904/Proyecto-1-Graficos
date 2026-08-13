use raylib::audio::{Music, RaylibAudio, Sound};

pub struct AudioManager<'a> {
    shot_sound: Option<Sound<'a>>,
    music: Option<Music<'a>>,
}

impl<'a> AudioManager<'a> {
    pub fn new(audio: &'a RaylibAudio) -> Self {
        let shot_sound = match audio.new_sound("assets/audio/shot.wav") {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("[audio] No se pudo cargar shot.wav, se sigue sin sonido de disparo: {e}");
                None
            }
        };

        let music = match audio.new_music("assets/audio/theme.ogg") {
            Ok(m) => {
                m.set_volume(0.4);
                Some(m)
            }
            Err(e) => {
                eprintln!("[audio] No se pudo cargar theme.ogg, se sigue sin musica de fondo: {e}");
                None
            }
        };

        AudioManager { shot_sound, music }
    }

    pub fn start_music(&mut self) {
        if let Some(music) = &mut self.music {
            music.play_stream();
        }
    }

    pub fn update(&mut self) {
        if let Some(music) = &mut self.music {
            music.update_stream();
        }
    }

    pub fn play_shot(&self) {
        if let Some(shot_sound) = &self.shot_sound {
            shot_sound.play();
        }
    }
}