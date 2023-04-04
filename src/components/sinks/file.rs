use crate::config::sinks::FileSinkConfig;
use crate::objects::Objects;
use crate::utils::time::{ datetime_from_ts, now };

use rocket::{ self, Build, Rocket };
use tokio::{fs::File, io::AsyncWriteExt};

pub struct FileSink {
    name: String,
    sent_objs: u64,
    filename: String,
    fd: Option<File>
}

impl FileSink {
    pub fn new(cfg: &FileSinkConfig) -> Self {
        Self {
            name: String::from(&cfg.name),
            sent_objs: 0,
            filename: format!("{}_{}", cfg.filename, datetime_from_ts(now())),
            fd: None
        }
    }

    pub async fn init(&mut self) -> Result<(), String> {
        self.fd = Some(File::create(&self.filename).await.expect("FileSink: cannot create file"));
        Ok(())
    }

    pub async fn send(&mut self, obj: Objects) -> Result<(), String> {
        match serde_json::to_string(&obj) {
            Ok(wire) => {
                match self.fd.as_mut() {
                    Some(fd) => {
                        match fd.write_all(wire.as_bytes()).await {
                            Ok(()) => {
                                match fd.write_u8(0xA).await {
                                    Ok(()) => {
                                        self.sent_objs += 1;
                                        Ok(())
                                    },
                                    Err(e) => {
                                        log::error!("{}: failed to write newline", &self.name);
                                        Err(e.to_string())
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("{}: failed to write to file", &self.name);
                                Err(e.to_string())
                            }
                        }
                    },
                    None => {
                        let err_msg = format!("{}: Cannot find the file descriptor", &self.name);
                        log::error!("{}", &err_msg);
                        Err(err_msg)
                    }
                }
            },
            Err(e) => {
                log::error!("{}: Cannot serialize object - {}", &self.name, e);
                Err(e.to_string())
            }
        }
    }

    // Getters
    pub fn get_name(&self) -> &str {
        &self.name
    }

    // Endpoints
    pub fn endpoints(&self, ctrl: Rocket<Build>) -> Rocket<Build> {
        ctrl
    }
}
