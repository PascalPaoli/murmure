use log::error;
use rodio::buffer::SamplesBuffer;
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub enum PlayerCommand {
    Play(Vec<f32>),
    Stop,
    IsPlaying(std::sync::mpsc::Sender<bool>),
}

pub struct TtsPlayer {
    tx: Sender<PlayerCommand>,
}

impl TtsPlayer {
    pub fn new() -> Self {
        let (tx, rx) = channel::<PlayerCommand>();

        thread::spawn(move || {
            let handle = match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(h) => h,
                Err(e) => {
                    error!("Failed to open TTS audio stream: {}", e);
                    return;
                }
            };

            let mut player = rodio::Player::connect_new(&handle.mixer());

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    PlayerCommand::Play(samples) => {
                        let source = SamplesBuffer::new(
                            std::num::NonZeroU16::new(1).unwrap(), 
                            std::num::NonZeroU32::new(24000).unwrap(), 
                            samples
                        );
                        player.append(source);
                    }
                    PlayerCommand::Stop => {
                        player.stop();
                        player = rodio::Player::connect_new(&handle.mixer());
                    }
                    PlayerCommand::IsPlaying(reply_tx) => {
                        let is_playing = !player.empty();
                        let _ = reply_tx.send(is_playing);
                    }
                }
            }
        });

        Self { tx }
    }

    pub fn play(&self, samples: Vec<f32>) {
        let _ = self.tx.send(PlayerCommand::Play(samples));
    }

    pub fn stop(&self) {
        let _ = self.tx.send(PlayerCommand::Stop);
    }

    pub fn is_playing(&self) -> bool {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        if self.tx.send(PlayerCommand::IsPlaying(reply_tx)).is_ok() {
            reply_rx.recv().unwrap_or(false)
        } else {
            false
        }
    }
}
