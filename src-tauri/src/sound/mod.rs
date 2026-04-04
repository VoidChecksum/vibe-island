use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Sound manager — plays event sounds and manages sound packs
pub struct SoundManager {
    enabled: bool,
    volume: f32,
    sounds: HashMap<String, Vec<u8>>,
}

impl SoundManager {
    pub fn new() -> Self {
        Self {
            enabled: true,
            volume: 0.5,
            sounds: HashMap::new(),
        }
    }

    pub fn play(&self, sound_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.enabled {
            return Ok(());
        }

        // Use rodio for cross-platform audio
        let (_stream, stream_handle) = rodio::OutputStream::try_default()
            .map_err(|e| format!("Audio output error: {}", e))?;

        if let Some(data) = self.sounds.get(sound_name) {
            let cursor = std::io::Cursor::new(data.clone());
            let source = rodio::Decoder::new(cursor)
                .map_err(|e| format!("Decode error: {}", e))?;
            let sink = rodio::Sink::try_new(&stream_handle)
                .map_err(|e| format!("Sink error: {}", e))?;
            sink.set_volume(self.volume);
            sink.append(source);
            sink.sleep_until_end();
        }

        Ok(())
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn load_pack(&mut self, pack_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.sounds.clear();

        if !pack_path.exists() {
            return Ok(());
        }

        // Look for sound files in the pack directory
        let entries = fs::read_dir(pack_path)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ["wav", "mp3", "ogg", "flac", "m4a"].contains(&ext.as_str()) {
                    if let Some(stem) = path.file_stem() {
                        let name = stem.to_string_lossy().to_string();
                        let data = fs::read(&path)?;
                        self.sounds.insert(name, data);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the sound packs directory
    pub fn packs_dir() -> PathBuf {
        let dir = dirs::data_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".local/share"))
            .join("vibe-island/sound-packs");
        fs::create_dir_all(&dir).ok();
        dir
    }

    /// List available sound packs
    pub fn list_packs() -> Vec<String> {
        let dir = Self::packs_dir();
        let mut packs = vec!["default".to_string()];
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        packs.push(name.to_string());
                    }
                }
            }
        }
        packs
    }
}
