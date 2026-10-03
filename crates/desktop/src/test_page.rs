use std::ops::{Deref, DerefMut};

use guinea::app::{Harness, Segment};
use guinea::winui::harness::Mounted;

pub struct Below<'h, P> {
    page: Mounted<'h, P>,
    _segment: Segment<'h>,
}

impl<'h, P> Below<'h, P> {
    pub fn mount(h: &'h Harness, mount: impl FnOnce(&Segment<'h>) -> anyhow::Result<Mounted<'h, P>>) -> Self {
        let segment = h.child();
        let page = mount(&segment).unwrap();
        Self { page, _segment: segment }
    }
}

impl<'h, P> Deref for Below<'h, P> {
    type Target = Mounted<'h, P>;

    fn deref(&self) -> &Self::Target {
        &self.page
    }
}

impl<P> DerefMut for Below<'_, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.page
    }
}
