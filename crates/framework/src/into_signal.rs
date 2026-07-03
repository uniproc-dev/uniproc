use app_core::signal::Signal;
use rpstate::{AccessMode, Field, Store};

pub trait IntoSignal<T> {
    fn into_signal(self) -> Signal<T>;
}

impl<T> IntoSignal<T> for Signal<T> {
    fn into_signal(self) -> Signal<T> {
        self
    }
}

impl<TValue, S, M> IntoSignal<TValue> for Field<TValue, S, M>
where
    TValue: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + Clone + 'static,
    S: Store,
    M: AccessMode,
{
    fn into_signal(self) -> Signal<TValue> {
        self.as_signal()
    }
}
