use crate::config::sources::FileSourceConfig;
use crate::objects::Objects;

use rocket::{ self, Build, Rocket };
use tokio::fs::File;
use tokio::io::{ AsyncBufReadExt, BufReader };

pub struct FileSource {
    name: String,
    queue_size: usize,
    sequence_number: u64,
    filename: String,
    reader: Option<BufReader<File>>
}

impl FileSource {
    pub fn new(cfg: &FileSourceConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            queue_size: cfg.queue_size,
            sequence_number: 0,
            filename: String::from(&cfg.filename),
            reader: None
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        let fd = File::open(&self.filename).await.unwrap_or_else(|_| panic!("FileSource: cannot open file {}", &self.filename));
        self.reader = Some(BufReader::new(fd));
        Ok(())
    }

    pub async fn recv(&mut self) -> Result<(Option<Objects>, bool), String> {
        if let Some(ref mut reader) = self.reader {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(done) => {
                    if done == 0 {
                        log::info!("{}: read {} messages", &self.name, self.sequence_number);
                        return Ok((None, true));
                    }

                    match serde_json::from_str(&line) {
                        Ok(obj) => {
                            self.sequence_number += 1;
                            return Ok((obj, false));
                        },
                        Err(e) => {
                            log::error!("{}: Cannot deserialize object - {}", &self.name, e);
                            return Err(e.to_string());
                        }
                    }
                },
                Err(e) => {
                    let err_msg = format!("{}: cannot read file {} - {}", &self.name, &self.filename, e);
                    log::error!("{}", &err_msg);
                    return Err(err_msg);
                }
            }
        }

        Err(format!("{}: reader is not set", self.name))
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_queue_size(&self) -> usize {
        self.queue_size
    }

    // Setters
    pub fn set_sleep_time(&mut self, _sleep_time: u64) {}

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl
    }
}
