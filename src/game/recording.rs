use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::fs::{File};
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Duration;
use std::fmt;
use std::str::FromStr;
use strum::{EnumString, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
pub enum RecordedKey {
    MoveLeft,
    MoveRight,
    SoftDrop,
    HardDrop,
    RotateClockwise,
    RotateAnticlockwise,
    Hold,
}

/// Represents a single recorded game input with timing information
#[derive(Debug, Clone)]
pub struct RecordedInput {
    /// Accumulated game time when this input occurred (not real-time)
    pub timestamp: Duration,
    /// The game input that was triggered
    pub keys: Vec<RecordedKey>,
}

// Custom serialization for RecordedInput
impl Serialize for RecordedInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format: "{timestamp in microseconds}:{comma separated list of keys}"
        let micros = self.timestamp.as_micros();

        // Use strum's Display trait to convert keys to strings
        let keys_str: Vec<String> = self.keys.iter()
            .map(|key| key.to_string())
            .collect();

        let serialized = format!("{}:{}", micros, keys_str.join(","));
        serializer.serialize_str(&serialized)
    }
}

// Custom deserialization for RecordedInput
impl<'de> Deserialize<'de> for RecordedInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RecordedInputVisitor;

        impl<'de> serde::de::Visitor<'de> for RecordedInputVisitor {
            type Value = RecordedInput;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format '{timestamp}:{key1,key2,...}'")
            }

            fn visit_str<E>(self, value: &str) -> Result<RecordedInput, E>
            where
                E: serde::de::Error,
            {
                let parts: Vec<&str> = value.split(':').collect();
                if parts.len() != 2 {
                    return Err(E::custom(format!("invalid format: {}", value)));
                }

                // Parse timestamp
                let micros = u64::from_str(parts[0])
                    .map_err(|_| E::custom(format!("invalid timestamp: {}", parts[0])))?;
                let timestamp = Duration::from_micros(micros);

                // Parse keys
                let mut keys = Vec::new();
                if !parts[1].is_empty() {
                    for key_str in parts[1].split(',') {
                        // Use strum's EnumString trait to parse the key
                        match RecordedKey::from_str(key_str) {
                            Ok(key) => keys.push(key),
                            Err(_) => return Err(E::custom(format!("invalid key: {}", key_str))),
                        }
                    }
                }

                Ok(RecordedInput { timestamp, keys })
            }
        }

        deserializer.deserialize_str(RecordedInputVisitor)
    }
}

/// Manages the recording of a game session
pub struct GameRecording {
    /// Total elapsed game time (accumulated delta)
    current_timestamp: Duration,
    /// The recorded inputs
    inputs: Vec<RecordedInput>,
    next_update_buffer: Vec<RecordedKey>,
}

impl GameRecording {
    /// Create a new GameRecorder in inactive state
    pub fn new() -> Self {
        Self {
            current_timestamp: Duration::ZERO,
            inputs: Vec::new(),
            next_update_buffer: Vec::new(),
        }
    }

    /// Update the recorder with the game's time delta
    pub fn update(&mut self, delta: Duration) {
        self.current_timestamp += delta;
        // empty the next update buffer into inputs
        if !self.next_update_buffer.is_empty() {
            let input = RecordedInput {
                timestamp: self.current_timestamp,
                keys: std::mem::take(&mut self.next_update_buffer),
            };
            self.inputs.push(input);
        }
    }

    /// Record an input if recording is active
    pub fn record_input(&mut self, key: RecordedKey) {
        self.next_update_buffer.push(key);
    }

    /// Save the recording to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        // Serialize the entire inputs vector using serde_json
        serde_json::to_writer(writer, &self.inputs).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

        Ok(())
    }

    /// Get the recorded inputs
    pub fn inputs(&self) -> &[RecordedInput] {
        &self.inputs
    }
}

/// For playing back a recorded game
pub struct GamePlayback {
    /// The recorded inputs
    inputs: Vec<RecordedInput>,
    /// Current position in the playback
    current_index: usize,
    /// Current game time in the playback
    current_timestamp: Duration,
}

impl GamePlayback {
    /// Load a recording from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        // Deserialize the inputs vector from the file using serde_json
        let inputs: Vec<RecordedInput> = serde_json::from_reader(reader).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        Ok(Self {
            inputs,
            current_index: 0,
            current_timestamp: Duration::ZERO,
        })
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.current_timestamp = Duration::ZERO;
    }

    /// Update the playback with the game's time delta
    /// Returns inputs that should be triggered at the current time
    pub fn update(&mut self, delta: Duration) -> Vec<RecordedKey> {
        if self.is_finished() {
            return vec![];
        }

        // Update current game time
        self.current_timestamp += delta;
        let mut result = Vec::new();

        // Check for inputs that should be triggered
        while self.current_index < self.inputs.len() {
            let input = &self.inputs[self.current_index];

            if input.timestamp <= self.current_timestamp {
                for key in input.keys.iter() {
                    result.push(*key);
                }
                self.current_index += 1;
            } else {
                break;
            }
        }

        dbg!(&result);
        result
    }

    /// Check if playback has finished
    pub fn is_finished(&self) -> bool {
        self.current_index >= self.inputs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Helper function to create a temporary file path
    fn temp_file_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("rustris_test_{}.json", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()));
        path
    }

    #[test]
    fn test_game_recorder_new() {
        let recorder = GameRecording::new();
        assert_eq!(recorder.current_timestamp, Duration::ZERO);
        assert!(recorder.inputs.is_empty());
        assert!(recorder.next_update_buffer.is_empty());
    }

    #[test]
    fn test_game_recorder_record_input() {
        let mut recorder = GameRecording::new();

        // Record some inputs
        recorder.record_input(RecordedKey::MoveLeft);
        recorder.record_input(RecordedKey::RotateClockwise);

        // Inputs should be in the next_update_buffer but not yet in inputs
        assert_eq!(recorder.next_update_buffer.len(), 2);
        assert_eq!(recorder.next_update_buffer[0], RecordedKey::MoveLeft);
        assert_eq!(recorder.next_update_buffer[1], RecordedKey::RotateClockwise);
        assert!(recorder.inputs.is_empty());
    }

    #[test]
    fn test_game_recorder_update() {
        let mut recorder = GameRecording::new();

        // Record some inputs
        recorder.record_input(RecordedKey::MoveLeft);
        recorder.record_input(RecordedKey::RotateClockwise);

        // Update the recorder
        let delta = Duration::from_millis(100);
        recorder.update(delta);

        // Inputs should now be in the inputs vector with correct timestamp
        // Both keys are grouped into a single RecordedInput now
        assert!(recorder.next_update_buffer.is_empty());
        assert_eq!(recorder.inputs.len(), 1);
        assert_eq!(recorder.inputs[0].timestamp, delta);
        assert_eq!(recorder.inputs[0].keys.len(), 2);
        assert_eq!(recorder.inputs[0].keys[0], RecordedKey::MoveLeft);
        assert_eq!(recorder.inputs[0].keys[1], RecordedKey::RotateClockwise);

        // Record more inputs and update again
        recorder.record_input(RecordedKey::HardDrop);
        let delta2 = Duration::from_millis(50);
        recorder.update(delta2);

        // Now we have a second input record with the new timestamp
        assert_eq!(recorder.inputs.len(), 2);
        assert_eq!(recorder.inputs[1].keys.len(), 1);
        assert_eq!(recorder.inputs[1].keys[0], RecordedKey::HardDrop);
        assert_eq!(recorder.inputs[1].timestamp, delta + delta2);
    }

    #[test]
    fn test_game_recorder_save_and_load() -> io::Result<()> {
        let mut recorder = GameRecording::new();

        // Record some inputs with timing
        recorder.record_input(RecordedKey::MoveLeft);
        recorder.update(Duration::from_millis(100));

        recorder.record_input(RecordedKey::RotateClockwise);
        recorder.update(Duration::from_millis(150));

        recorder.record_input(RecordedKey::HardDrop);
        recorder.update(Duration::from_millis(50));

        // Save to a temporary file
        let file_path = temp_file_path();
        recorder.save_to_file(&file_path)?;

        // Load the recording
        let player = GamePlayback::load_from_file(&file_path)?;

        // Check that the loaded inputs match what we recorded
        assert_eq!(player.inputs.len(), 3);
        assert_eq!(player.inputs[0].keys.len(), 1);
        assert_eq!(player.inputs[0].keys[0], RecordedKey::MoveLeft);
        assert_eq!(player.inputs[1].keys.len(), 1);
        assert_eq!(player.inputs[1].keys[0], RecordedKey::RotateClockwise);
        assert_eq!(player.inputs[2].keys.len(), 1);
        assert_eq!(player.inputs[2].keys[0], RecordedKey::HardDrop);

        // Clean up
        fs::remove_file(&file_path)?;

        Ok(())
    }

    #[test]
    fn test_game_player_update() {
        let mut player = GamePlayback {
            inputs: vec![
                RecordedInput {
                    keys: vec![RecordedKey::MoveLeft],
                    timestamp: Duration::from_millis(100),
                },
                RecordedInput {
                    keys: vec![RecordedKey::RotateClockwise],
                    timestamp: Duration::from_millis(250),
                },
                RecordedInput {
                    keys: vec![RecordedKey::HardDrop],
                    timestamp: Duration::from_millis(400),
                },
            ],
            current_index: 0,
            current_timestamp: Duration::ZERO,
        };

        // Update less than the first input's time
        let keys = player.update(Duration::from_millis(50));
        assert!(keys.is_empty());
        assert_eq!(player.current_index, 0);

        // Update past the first input's time
        let keys = player.update(Duration::from_millis(60));
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], RecordedKey::MoveLeft);
        assert_eq!(player.current_index, 1);

        // Update past multiple inputs
        let keys = player.update(Duration::from_millis(300));
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], RecordedKey::RotateClockwise);
        assert_eq!(keys[1], RecordedKey::HardDrop);
        assert_eq!(player.current_index, 3);
        assert!(player.is_finished());

        // Update after all inputs are consumed
        let keys = player.update(Duration::from_millis(100));
        assert!(keys.is_empty());
    }

    #[test]
    fn test_game_player_reset() {
        let mut player = GamePlayback {
            inputs: vec![
                RecordedInput {
                    keys: vec![RecordedKey::MoveLeft],
                    timestamp: Duration::from_millis(100),
                },
                RecordedInput {
                    keys: vec![RecordedKey::RotateClockwise],
                    timestamp: Duration::from_millis(250),
                },
            ],
            current_index: 0,
            current_timestamp: Duration::ZERO,
        };

        // Consume all inputs
        player.update(Duration::from_millis(300));
        assert!(player.is_finished());

        // Reset the player
        player.reset();

        // Check that the state is reset
        assert_eq!(player.current_index, 0);
        assert_eq!(player.current_timestamp, Duration::ZERO);
        assert!(!player.is_finished());
    }

    #[test]
    fn test_game_player_is_finished() {
        let mut player = GamePlayback {
            inputs: vec![
                RecordedInput {
                    keys: vec![RecordedKey::MoveLeft],
                    timestamp: Duration::from_millis(100),
                },
            ],
            current_index: 0,
            current_timestamp: Duration::ZERO,
        };

        // Initially not finished
        assert!(!player.is_finished());

        // After consuming all inputs
        player.update(Duration::from_millis(200));
        assert!(player.is_finished());

        // Empty player is finished
        let empty_player = GamePlayback {
            inputs: vec![],
            current_index: 0,
            current_timestamp: Duration::ZERO,
        };
        assert!(empty_player.is_finished());
    }

    #[test]
    fn test_grouped_keys() {
        let mut recorder = GameRecording::new();

        // Record multiple inputs before updating
        recorder.record_input(RecordedKey::MoveLeft);
        recorder.record_input(RecordedKey::MoveRight);
        recorder.record_input(RecordedKey::HardDrop);

        // Update the recorder once
        let delta = Duration::from_millis(100);
        recorder.update(delta);

        // All three inputs should be in one RecordedInput
        assert_eq!(recorder.inputs.len(), 1);
        assert_eq!(recorder.inputs[0].keys.len(), 3);
        assert_eq!(recorder.inputs[0].keys[0], RecordedKey::MoveLeft);
        assert_eq!(recorder.inputs[0].keys[1], RecordedKey::MoveRight);
        assert_eq!(recorder.inputs[0].keys[2], RecordedKey::HardDrop);
        assert_eq!(recorder.inputs[0].timestamp, delta);

        // Create a player with this input
        let mut player = GamePlayback {
            inputs: recorder.inputs().to_vec(),
            current_index: 0,
            current_timestamp: Duration::ZERO,
        };

        // Update and check that all keys are returned
        let keys = player.update(Duration::from_millis(100));
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], RecordedKey::MoveLeft);
        assert_eq!(keys[1], RecordedKey::MoveRight);
        assert_eq!(keys[2], RecordedKey::HardDrop);
    }
}
