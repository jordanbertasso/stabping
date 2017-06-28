#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum Feed {
    Raw,
    Hourly,
    Daily,
    Weekly,
}
