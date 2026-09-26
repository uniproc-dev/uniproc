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

struct Extraction {
    key: String,
    path: String,
    package_full_name: Option<String>,
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
        let key = package.unwrap_or(req.path);
        if key.is_empty() {
            return None;
        }

        let mut slots = self.slots.lock().unwrap_or_else(PoisonError::into_inner);
        match slots.get(key) {
            Some(Slot::Ready(png)) => Some(png.clone()),
            Some(Slot::Pending | Slot::Missing) => None,
            None => {
                slots.insert(key.to_string(), Slot::Pending);
                let queued = self.requests.send(Extraction {
                    key: key.to_string(),
                    path: req.path.to_string(),
                    package_full_name: package.map(str::to_string),
                });
                if queued.is_err() {
                    slots.insert(key.to_string(), Slot::Missing);
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
    let image = match &extraction.package_full_name {
        Some(package) => extract::extract_appx_icon_rgba(package, ICON_SIZE),
        None => extract::extract_icon_rgba(&extraction.path),
    }?;
    encode::encode_png(image.width, image.height, &image.pixels)
        .inspect_err(|err| tracing::warn!(?err, key = extraction.key, "context: icon did not encode"))
        .ok()
}
