#[derive(Clone)]
pub enum Kind {
    TcpPing,
    // HttpDownload,
}

struct AssociatedData {
    id: u32,
    name: &'static str,
    default_options_bootstrap: (&'static str, u32),
}

impl Kind {
    fn _associated_data(&self) -> AssociatedData {
        match *self {
            Kind::TcpPing => AssociatedData {
                id: 0,
                name: "tcpping",
                default_options_bootstrap: ("google.com:80", 10_000),
            },
        }
    }

    pub fn id(&self) -> u32 {
        self._associated_data().id
    }

    pub fn name(&self) -> &'static str {
        self._associated_data().name
    }

    pub fn default_options_bootstrap(&self) -> (&'static str, u32) {
        self._associated_data().default_options_bootstrap
    }
}

