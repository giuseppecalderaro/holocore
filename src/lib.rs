#![feature(generators, generator_trait)]
pub mod components;
pub mod config;
pub mod objects;
pub mod processor;
pub mod utils;

use std::collections::HashMap;

fn usage() {
    println!("Usage: \n./main <main class> <config file>");
}

fn register(url: &str, cfg: &config::Config) {
    match &cfg.gateway {
        Some(gateway) => {
            // Retrieve gateway url
            match utils::redis::get(url, gateway) {
                Ok(gateway_url) => {
                    // Register service with gateway
                    let body = HashMap::from([
                        (String::from("name"), String::from(&cfg.name)),
                        (String::from("url"), format!("http://{}:{}", &cfg.ctrl_host, cfg.ctrl_port))
                    ]);
                    utils::http::post(&gateway_url, body);
                    log::info!("Registered service {} with gateway {}", &cfg.name, gateway);
                },
                Err(e) => log::error!("Failed to access discovery service - {}", e)
            }
        },
        None => log::error!("Discovery service is set but gateway is not. Cannot continue")
    };
}

pub fn entry_point(argc: usize, argv: Vec<String>) -> std::io::Result<()> {
    if argc != 3 {
        usage();
        std::process::exit(0);
    }

    env_logger::init();

    let cfg: config::Config = confy::load_path(&argv[2]).expect("Cannot read config file");
    println!("*** CONFIG ***: \n{}\n*** ****** ***", serde_json::to_string_pretty(&cfg).expect("Cannot serialize Config to Json"));

    if let Some(url) = &cfg.discovery_service {
        register(url, &cfg);
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(cfg.workers)
            .thread_name(&cfg.name)
            .thread_stack_size(cfg.stack_size * 1024 * 1024)
            .enable_all()
            .build()
            .expect("Cannot build runtime");

    runtime.block_on(async move {
        let mut processor = processor::Processor::new(&cfg.name, &cfg);
        match processor.init().await {
            Ok(()) => log::info!("Processor {} initialized successfully", &cfg.name),
            Err(e) => panic!("Cannot initializing processor {} - {}", &cfg.name, e)
        }

        match processor.run().await {
            Ok(()) => log::info!("Processor {} completed successfully", &cfg.name),
            Err(e) => panic!("Processor {} exited with error: {}", &cfg.name, e)
        };
    });

    Ok(())
}
