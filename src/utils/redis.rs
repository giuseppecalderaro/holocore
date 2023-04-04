use redis::Commands;

pub fn get(url: &str, key: &str) -> Result<String, String> {
    match redis::Client::open(format!("redis://{}/", url)) {
        Ok(client) => match client.get_connection() {
            Ok(mut connection) => {
                match connection.get(key) {
                    Ok(value) => Ok(value),
                    Err(e) => {
                        let err_msg = format!("Cannot get value from key {} - {}", key, e);
                        log::error!("{}", &err_msg);
                        Err(err_msg)
                    }
                }
            },
            Err(e) => {
                let err_msg = format!("Cannot connect to redis - {}", e);
                log::error!("{}", &err_msg);
                Err(err_msg)
            }
        },
        Err(e) => {
            let err_msg = format!("Cannot open client with {} - {}", url, e);
            log::error!("{}", &err_msg);
            Err(err_msg)
        }
    }
}
