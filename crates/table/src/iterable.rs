use windows_collections::{IIterable, IIterator};
use windows_core::{Interface, Result, RuntimeType};

use crate::bindings as c;

pub(crate) trait First<T: RuntimeType + 'static> {
    #[allow(non_snake_case)]
    fn First(&self) -> Result<IIterator<T>>;
}

impl<C: Interface + IterableOf<T>, T: RuntimeType + 'static> First<T> for C {
    fn First(&self) -> Result<IIterator<T>> {
        self.cast::<IIterable<T>>()?.First()
    }
}

pub(crate) trait IterableOf<T> {}

impl IterableOf<c::Visual> for c::VisualCollection {}
impl IterableOf<c::ICompositionInteractionSource> for c::CompositionInteractionSourceCollection {}
