use serde::Serialize;

#[derive(Serialize)]
pub struct LibraryStatus {
    configured: bool,
    provider: Option<String>,
    root: Option<String>,
    records: u32,
}

pub fn status() -> LibraryStatus {
    LibraryStatus {
        configured: false,
        provider: None,
        root: None,
        records: 0,
    }
}
