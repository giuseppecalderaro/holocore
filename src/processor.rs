use crate::components::control::{ self, Controls, ControlState };
use crate::components::processing_units::{ self, PUs };
use crate::components::sinks::{ self, Sinks };
use crate::components::sources::{ self, Sources};
use crate::config::Config;
use crate::inject;
use crate::objects::Objects;

use flume::{ Receiver, Sender, bounded, unbounded };
use futures::future;
use std::collections::HashMap;
use tokio::task::spawn;

macro_rules! init {
    ($component:ident) => {
        {
            let mut unlocked_component = $component.lock().await;
            match unlocked_component.init().await {
                Ok(()) => log::info!("Initialized component {}", &unlocked_component.get_name()),
                Err(e) => log::error!("Failed to initialize component {} - {}", &unlocked_component.get_name(), e)
            }
        }
    };
}

macro_rules! run_source {
    ($component:ident, $self:ident, $ctrl:ident, $tasks:ident) => {
        {
            let unlocked_component = $component.lock().await;

            $ctrl = unlocked_component.endpoints($ctrl);
            let ctrl_rx = $self.ctrl_rxs.remove(unlocked_component.get_name()).expect(&format!("Cannot find ctrl rx for source {}", unlocked_component.get_name()));

            let txs = $self.txs.remove(unlocked_component.get_name()).expect(&format!("Cannot find txs for source {}", unlocked_component.get_name()));
            $tasks.push(recv!($component, txs, ctrl_rx));
        }
    };
}

macro_rules! run_pu {
    ($component:ident, $self:ident, $ctrl:ident, $tasks:ident) => {
        {
            let unlocked_component = $component.lock().await;

            $ctrl = unlocked_component.endpoints($ctrl);
            let ctrl_rx = $self.ctrl_rxs.remove(unlocked_component.get_name()).expect(&format!("Cannot find ctrl rx for source {}", unlocked_component.get_name()));

            let txs = $self.txs.remove(unlocked_component.get_name()).expect(&format!("Cannot find txs for PU {}", unlocked_component.get_name()));
            let rxs = $self.rxs.remove(unlocked_component.get_name()).expect(&format!("Cannot find rxs for PU {}", unlocked_component.get_name()));
            $tasks.push(execute!($component, txs, rxs, ctrl_rx));
        }
    };
}

macro_rules! run_sink {
    ($component:ident, $self:ident, $ctrl:ident, $tasks:ident) => {
        {
            let unlocked_component = $component.lock().await;

            $ctrl = unlocked_component.endpoints($ctrl);
            let ctrl_rx = $self.ctrl_rxs.remove(unlocked_component.get_name()).expect(&format!("Cannot find ctrl rx for source {}", unlocked_component.get_name()));

            let rxs = $self.rxs.remove(unlocked_component.get_name()).expect(&format!("Cannot find txs for source {}", unlocked_component.get_name()));
            $tasks.push(send!($component, rxs, ctrl_rx));
        }
    };
}

macro_rules! recv {
    ($source:ident, $txs:ident, $ctrl_rx:ident) => {
        {
            let cloned = $source.clone();

            spawn(async move {
                let mut unlocked_component = cloned.lock().await;
                let mut disabled = vec![];
                let num_txs = $txs.len();

                loop {
                    tokio::select! {
                        obj = $ctrl_rx.recv_async() => {
                            match obj {
                                Ok(Controls::Inject(obj)) => {
                                    inject!(unlocked_component, $txs, obj, disabled);
                                },
                                Ok(Controls::SleepTime(sleep_time)) => {
                                    unlocked_component.set_sleep_time(sleep_time);
                                },
                                Err(e) => {
                                    log::error!("Task {} bad control request received - {}", unlocked_component.get_name(), e);
                                    break;
                                }
                            }
                        },
                        result = unlocked_component.recv() => match result {
                            Ok((obj, completed)) => match obj {
                                Some(obj) => {
                                    let mut futures = $txs.iter()
                                                          .enumerate()
                                                          .filter(|&(idx, _)| !disabled.contains(&idx))
                                                          .map(|(_, tx)| tx.send_async(obj.clone()))
                                                          .collect::<Vec<_>>();

                                    while !futures.is_empty() {
                                        match future::select_all(futures).await {
                                            (Ok(_value), _index, remaining) => futures = remaining,
                                            (Err(e), index, remaining) => {
                                                log::error!("{}: failed while sending at index {} - {}", unlocked_component.get_name(), index, e);
                                                futures = remaining;
                                                disabled.push(index);
                                                if disabled.len() == num_txs {
                                                    log::error!("{}: all the txs are disabled. Returning...", unlocked_component.get_name());
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                },
                                None => {
                                    if completed == true {
                                        log::info!("Task {} completed", unlocked_component.get_name());
                                        break;
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!("Task {} exited - {}", unlocked_component.get_name(), e);
                                break;
                            }
                        }
                    };
                }
            })
        }
    };
}

macro_rules! execute {
    ($component:ident, $txs:ident, $rxs:ident, $ctrl_rx:ident) => {
        {
            let cloned = $component.clone();

            spawn(async move {
                let mut unlocked_component = cloned.lock().await;
                let mut disabled_txs = vec![];
                let mut disabled_rxs = vec![];
                let num_txs = $txs.len();
                let num_rxs = $rxs.len();

                loop {
                    tokio::select! {
                        obj = $ctrl_rx.recv_async() => {
                            match obj {
                                Ok(Controls::Inject(obj)) => {
                                    inject!(unlocked_component, $txs, obj, disabled_txs);
                                },
                                Ok(Controls::SleepTime(_sleep_time)) => (),
                                Err(e) => {
                                    log::error!("Task {} bad control request received - {}", unlocked_component.get_name(), e);
                                    return;
                                }
                            }
                        },
                        _ = async {
                            let futures = $rxs.iter()
                                              .enumerate()
                                              .filter(|&(idx, _)| !disabled_rxs.contains(&idx))
                                              .map(|(_, rx)| rx.recv_async())
                                              .collect::<Vec<_>>();

                            match future::select_all(futures).await {
                                (Ok(obj), _index, _remaining) => match unlocked_component.execute(obj).await {
                                    Ok((obj, completed)) => match obj {
                                        Some(obj) => {
                                            let mut futures = $txs.iter()
                                                                  .enumerate()
                                                                  .filter(|&(idx, _)| !disabled_txs.contains(&idx))
                                                                  .map(|(_, tx)| tx.send_async(obj.clone()))
                                                                  .collect::<Vec<_>>();

                                            while !futures.is_empty() {
                                                match future::select_all(futures).await {
                                                    (Ok(_value), _index, remaining) => futures = remaining,
                                                    (Err(e), index, remaining) => {
                                                        log::error!("{}: failed while sending at index {} - {}", unlocked_component.get_name(), index, e);
                                                        futures = remaining;
                                                        disabled_txs.push(index);
                                                        if disabled_txs.len() == num_txs {
                                                            log::error!("{}: all the txs are disabled. Returning...", unlocked_component.get_name());
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                        },
                                        None => {
                                            if completed == true {
                                                log::info!("Task {} completed", unlocked_component.get_name());
                                                return;
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        log::error!("Task {} exited - {}", unlocked_component.get_name(), e);
                                        return;
                                    }
                                },
                                (Err(e), index, _remaining) => {
                                    log::error!("{}: failed while receiving at index {} - {}", unlocked_component.get_name(), index, e);
                                    disabled_rxs.push(index);
                                    if disabled_rxs.len() == num_rxs {
                                        log::error!("{}: all the rxs are disabled. Returning...", unlocked_component.get_name());
                                        return;
                                    }
                                }
                            }
                        } => ()
                    }
                }
            })
        }
    };
}

macro_rules! send {
    ($source:ident, $rxs:ident, $ctrl_rx:ident) => {
        {
            let cloned = $source.clone();

            spawn(async move {
                let mut unlocked_component = cloned.lock().await;
                let mut disabled = vec![];
                let num_rxs = $rxs.len();

                loop {
                    tokio::select! {
                        obj = $ctrl_rx.recv_async() => {
                            match obj {
                                Ok(Controls::SleepTime(_sleep_time)) => (),
                                Ok(_) => (),
                                Err(e) => {
                                    log::error!("Task {} bad control request received - {}", unlocked_component.get_name(), e);
                                    return;
                                }
                            }
                        },
                        () = async {
                            let futures = $rxs.iter()
                                              .enumerate()
                                              .filter(|&(idx, _)| !disabled.contains(&idx))
                                              .map(|(_, rx)| rx.recv_async())
                                              .collect::<Vec<_>>();

                            match future::select_all(futures).await {
                                (Ok(obj), _index, _remaining) => match unlocked_component.send(obj).await {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log::error!("Task {} exited - {}", unlocked_component.get_name(), e);
                                        return;
                                    }
                                },
                                (Err(e), index, _remaining) => {
                                    log::error!("{}: failed while receiving at index {} - {}", unlocked_component.get_name(), index, e);
                                    disabled.push(index);
                                    if disabled.len() == num_rxs {
                                        log::error!("{}: all the rxs are disabled. Returning...", unlocked_component.get_name());
                                        return;
                                    }
                                }
                            }
                        } => ()
                    }
                }
            })
        }
    };
}

pub(crate) struct Processor {
    name: String,
    sources: Vec<Sources>,
    pus: Vec<PUs>,
    sinks: Vec<Sinks>,
    txs: HashMap<String, Vec<Sender<Objects>>>,
    rxs: HashMap<String, Vec<Receiver<Objects>>>,

    // Config
    cfg: Config,

    // Control
    ctrl_txs: HashMap<String, Sender<Controls>>,
    ctrl_rxs: HashMap<String, Receiver<Controls>>
}

impl Processor {
    pub fn new(processor_name: &str, cfg: &Config) -> Self {
        log::info!("Creating processor {} - version {}", processor_name, cfg.version);

        // Channels store
        let mut txs = HashMap::<String, Vec<Sender<Objects>>>::new();
        let mut rxs = HashMap::<String, Vec<Receiver<Objects>>>::new();
        let mut ctrl_txs = HashMap::<String, Sender<Controls>>::new();
        let mut ctrl_rxs = HashMap::<String, Receiver<Controls>>::new();

        // Populate channels
        for cfg in &cfg.sources {
            // Data
            txs.insert(String::from(cfg.name()), vec![]);

            // Control
            let (ctrl_tx, ctrl_rx) = unbounded::<Controls>();
            ctrl_txs.insert(String::from(cfg.name()), ctrl_tx);
            ctrl_rxs.insert(String::from(cfg.name()), ctrl_rx);
        }

        match &cfg.pus {
            Some(cfgs) => {
                for cfg in cfgs {
                    // Data
                    txs.insert(String::from(cfg.name()), vec![]);
                    rxs.insert(String::from(cfg.name()), vec![]);

                    // Control
                    let (ctrl_tx, ctrl_rx) = unbounded::<Controls>();
                    ctrl_txs.insert(String::from(cfg.name()), ctrl_tx);
                    ctrl_rxs.insert(String::from(cfg.name()), ctrl_rx);
                }
            },
            None => ()
        }

        for cfg in &cfg.sinks {
            // Data
            rxs.insert(String::from(cfg.name()), vec![]);

            // Control
            let (ctrl_tx, ctrl_rx) = unbounded::<Controls>();
            ctrl_txs.insert(String::from(cfg.name()), ctrl_tx);
            ctrl_rxs.insert(String::from(cfg.name()), ctrl_rx);
        }

        // Create sources
        let mut sources = Vec::<Sources>::new();
        for cfg in &cfg.sources {
            let source = sources::factory::make(cfg);
            sources.push(source);

            for downstream in cfg.downstreams() {
                let (tx, rx) = bounded(cfg.queue_size());
                match txs.get_mut(cfg.name()) {
                    Some(txs) => txs.push(tx),
                    None => panic!("Cannot find source {} inlet", cfg.name())
                }

                match rxs.get_mut(downstream) {
                    Some(rxs) => rxs.push(rx),
                    None => panic!("Cannot find source {} outlet for {}", cfg.name(), downstream)
                }
            }
        }

        // Create processing units
        let mut pus = Vec::<PUs>::new();
        match &cfg.pus {
            Some(cfgs) => {
                for cfg in cfgs {
                    let proc = processing_units::factory::make(cfg);
                    pus.push(proc);

                    for downstream in cfg.downstreams() {
                        let (tx, rx) = bounded(cfg.queue_size());
                        match txs.get_mut(cfg.name()) {
                            Some(txs) => txs.push(tx),
                            None => panic!("Cannot find source {} inlet", cfg.name())
                        }

                        match rxs.get_mut(downstream) {
                            Some(rxs) => rxs.push(rx),
                            None => panic!("Cannot find source {} outlet for {}", cfg.name(), downstream)
                        }
                    }
                }
            }
            None => ()
        }

        // Create sinks
        let mut sinks = Vec::<Sinks>::new();
        for cfg in &cfg.sinks {
            let sink = sinks::factory::make(cfg);
            sinks.push(sink);
        }

        Self {
            name: String::from(processor_name),
            sources,
            pus,
            sinks,
            txs,
            rxs,
            cfg: Config::clone(cfg),
            ctrl_txs,
            ctrl_rxs
        }
    }

    pub async fn init(&mut self) -> std::io::Result<()> {
        log::info!("Initializing processor {}", self.name);

        for source_enum in &self.sources {
            match source_enum {
                Sources::BinanceSpotSource(source) => init!(source),
                Sources::FileSource(source) => init!(source),
                Sources::MockSource(source) => init!(source)
            }
        }

        for processing_enum in &self.pus {
            match processing_enum {
                PUs::MockPU(processing) => init!(processing),
                PUs::OrderManagerPU(processing) => init!(processing),
                PUs::OrderbookManagerPU(processing) => init!(processing),
                PUs::StrategyPU(processing) => init!(processing)
            }
        }

        for sink_enum in &self.sinks {
            match sink_enum {
                Sinks::BinanceSpotSink(sink) => init!(sink),
                Sinks::FileSink(sink) => init!(sink),
                Sinks::MockSink(sink) => init!(sink)
            }
        }

        Ok(())
    }

    pub async fn run(mut self) -> std::io::Result<()> {
        log::info!("Running processor {}", self.name);

        let control_state = ControlState::new(self.cfg, self.ctrl_txs);
        let mut ctrl = control::build_control(control_state);
        let mut tasks = Vec::new();

        for source_enum in &self.sources {
            match source_enum {
                Sources::BinanceSpotSource(source) => run_source!(source, self, ctrl, tasks),
                Sources::FileSource(source) => run_source!(source, self, ctrl, tasks),
                Sources::MockSource(source) => run_source!(source, self, ctrl, tasks)
            };
        }

        for processing_enum in &self.pus {
            match processing_enum {
                PUs::MockPU(punit) => run_pu!(punit, self, ctrl, tasks),
                PUs::OrderManagerPU(punit) => run_pu!(punit, self, ctrl, tasks),
                PUs::OrderbookManagerPU(punit) => run_pu!(punit, self, ctrl, tasks),
                PUs::StrategyPU(punit) => run_pu!(punit, self, ctrl, tasks)
            };
        }

        for sink_enum in &self.sinks {
            match sink_enum {
                Sinks::BinanceSpotSink(sink) => run_sink!(sink, self, ctrl, tasks),
                Sinks::FileSink(sink) => run_sink!(sink, self, ctrl, tasks),
                Sinks::MockSink(sink) => run_sink!(sink, self, ctrl, tasks)
            };
        }

        tokio::select! {
            _ = futures::future::join_all(tasks) => (),
            _ = ctrl.launch() => ()
        };

        Ok(())
    }
}
