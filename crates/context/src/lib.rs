#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::all)]
mod bindings;
mod encode;
mod extract;

use std::collections::HashMap;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

pub use extract::has_own_icon;

pub(crate) const ICON_SIZE: i32 = 32;

pub struct IconRequest<'a> {
    pub path: &'a str,
    pub package_full_name: Option<&'a str>,
}

enum Slot {
    Pending,
    Ready(Arc<[u8]>),
    Missing,
}

enum Source {
    File(String),
    Package(String),
    Window(isize),
}

struct Extraction {
    key: String,
    source: Source,
}

type Slots = Arc<Mutex<HashMap<String, Slot>>>;

pub struct IconCache {
    slots: Slots,
    requests: Sender<Extraction>,
}

impl IconCache {
    pub fn new() -> Self {
        let slots: Slots = Arc::default();
        let (requests, pending) = channel::<Extraction>();

        let filled = slots.clone();
        let spawned = std::thread::Builder::new()
            .name("uniproc-icons".into())
            .spawn(move || {
                for extraction in pending {
                    let slot = match extract_png(&extraction) {
                        Some(png) => Slot::Ready(png.into()),
                        None => Slot::Missing,
                    };
                    filled
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .insert(extraction.key, slot);
                }
            });
        if let Err(err) = spawned {
            tracing::warn!(?err, "context: icon extraction thread did not start");
        }

        Self { slots, requests }
    }

    pub fn icon(&self, req: IconRequest) -> Option<Arc<[u8]>> {
        let package = req.package_full_name.filter(|s| !s.is_empty());
        match package {
            Some(package) => self.lookup(package.to_string(), || Source::Package(package.to_string())),
            None if req.path.is_empty() => None,
            None => self.lookup(req.path.to_string(), || Source::File(req.path.to_string())),
        }
    }

    pub fn window_icon(&self, handle: isize) -> Option<Arc<[u8]>> {
        if handle == 0 {
            return None;
        }
        self.lookup(format!("window:{handle}"), || Source::Window(handle))
    }

    fn lookup(&self, key: String, source: impl FnOnce() -> Source) -> Option<Arc<[u8]>> {
        let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        match slots.get(&key) {
            Some(Slot::Ready(png)) => Some(png.clone()),
            Some(Slot::Pending | Slot::Missing) => None,
            None => {
                slots.insert(key.clone(), Slot::Pending);
                let queued = self.requests.send(Extraction {
                    key: key.clone(),
                    source: source(),
                });
                if queued.is_err() {
                    slots.insert(key, Slot::Missing);
                }
                None
            }
        }
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_png(extraction: &Extraction) -> Option<Vec<u8>> {
    let image = match &extraction.source {
        Source::Package(package) => extract::extract_appx_icon_rgba(package, ICON_SIZE),
        Source::File(path) => extract::extract_icon_rgba(path),
        Source::Window(handle) => extract::extract_window_icon_rgba(*handle),
    }?;
    encode::encode_png(image.width, image.height, &image.pixels)
        .inspect_err(|err| tracing::warn!(?err, key = extraction.key, "context: icon did not encode"))
        .ok()
}
