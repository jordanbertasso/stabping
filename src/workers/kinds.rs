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
    pub fn id(&self) -> usize { *self as usize }
    pub fn name(&self) -> &'static str { ALL_NAMES[self.id()] }

    pub fn for_name<'a>(name: &'a str) -> Self {
        match name {
            "tcpping" => Kind::TcpPing,
        }
    }

    pub fn default_options_bootstrap(&self) -> (&'static str, u32) {
        match *self {
            Kind::TcpPing => ("google.com:80", 10_000)
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

