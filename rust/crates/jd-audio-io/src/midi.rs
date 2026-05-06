use crossbeam_channel::{unbounded, Receiver, Sender};
use jd_core::Result;
use midir::{MidiInput as RawMidiInput, MidiInputConnection};

/// Counterpart of `juce::MidiMessage`. Owns the raw bytes for portability.
#[derive(Debug, Clone)]
pub struct MidiMessage {
    pub timestamp_us: u64,
    pub data: Vec<u8>,
}

impl MidiMessage {
    pub fn status(&self) -> Option<u8> {
        self.data.first().copied()
    }

    pub fn is_note_on(&self) -> bool {
        matches!(self.status(), Some(s) if (s & 0xF0) == 0x90 && self.data.get(2).copied().unwrap_or(0) > 0)
    }

    pub fn is_note_off(&self) -> bool {
        match self.status() {
            Some(s) if (s & 0xF0) == 0x80 => true,
            Some(s) if (s & 0xF0) == 0x90 => self.data.get(2).copied().unwrap_or(0) == 0,
            _ => false,
        }
    }

    pub fn note_number(&self) -> Option<u8> {
        self.data.get(1).copied()
    }

    pub fn velocity(&self) -> Option<u8> {
        self.data.get(2).copied()
    }
}

/// Counterpart of `juce::MidiInput`. Holds the connection and a receiver of
/// parsed messages.
pub struct MidiInput {
    _conn: MidiInputConnection<Sender<MidiMessage>>,
    rx: Receiver<MidiMessage>,
}

impl MidiInput {
    pub fn open_first_available(client_name: &str) -> Result<Self> {
        let mut input = RawMidiInput::new(client_name)
            .map_err(|e| jd_core::Error::Other(e.to_string()))?;
        input.ignore(midir::Ignore::None);

        let ports = input.ports();
        let port = ports
            .first()
            .ok_or_else(|| jd_core::Error::Other("no MIDI input ports".into()))?;

        let (tx, rx) = unbounded();
        let conn = input
            .connect(
                port,
                "jd-audio-io",
                |stamp, msg, sender| {
                    let _ = sender.send(MidiMessage {
                        timestamp_us: stamp,
                        data: msg.to_vec(),
                    });
                },
                tx,
            )
            .map_err(|e| jd_core::Error::Other(e.to_string()))?;

        Ok(Self { _conn: conn, rx })
    }

    pub fn try_recv(&self) -> Option<MidiMessage> {
        self.rx.try_recv().ok()
    }
}
