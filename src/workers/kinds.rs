use workers::Options;

pub enum Kind {
    TcpPing = 0,
    // HttpDownload = 1,
}

static ALL_NAMES: [&'static str; 1] = [
    "tcpping",
    // "httpdownload",
];

impl Kind {
    pub fn kind_id(&self) -> usize {
        *self as usize
    }

    pub fn name(&self) -> &'static str {
        match *self {
            Kind::TcpPing => "tcpping",
        }
    }

    pub fn default_options(&self) -> Options {
        match *self {
            Kind::TcpPing =>
                Options {
                    addrs: Vec::new(),
                    interval: 10_000,
                },
        }
    }

    pub fn run_worker(&self, manager: Arc<TargetManager>,
                             results_out: Sender<TargetResults>) -> thread::JoinHandle<()> {
        match *self {
            TargetKind::TcpPing => run_tcpping_worker(manager, results_out),
        }
    }

    pub fn new_managers_for_all<'a>(data_path: &'a Path) -> Result<Vec<Arc<TargetManager>>, ManagerError> {
        let mut targets = Vec::with_capacity(ALL_KINDS.len());
        for k in ALL_KINDS.iter() {
            targets.push(
                Arc::new(try!(TargetManager::new(k, data_path)))
            );
        }
        Ok(targets)
    }
}

