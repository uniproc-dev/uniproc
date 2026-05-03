pub use framework::icons::{IconFamily, IconKey, IconRef, IconVariant};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/icons.rs"));
}

pub use generated::*;
