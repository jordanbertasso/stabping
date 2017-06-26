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
            _ => unreachable!()
        }
    }

    pub fn default_options_bootstrap(&self) -> (&'static str, u32) {
        match *self {
            Kind::TcpPing => ("google.com:80", 10_000)
        }
    }
}

