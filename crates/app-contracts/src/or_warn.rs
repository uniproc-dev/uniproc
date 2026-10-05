use std::fmt::{Debug, Display};
use std::panic::Location;

pub trait OrWarn<T> {
    fn or_warn(self, what: impl Display) -> Option<T>;
}

impl<T, E: Debug> OrWarn<T> for Result<T, E> {
    #[track_caller]
    fn or_warn(self, what: impl Display) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                let caller = Location::caller();
                let at = format_args!("{}:{}", caller.file(), caller.line());
                tracing::warn!(?err, %at, "{what}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tracing::field::{Field, Visit};
    use tracing::{Event, Level, Subscriber};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::Layer;

    use super::*;

    #[derive(Default, Clone)]
    struct Seen(Arc<Mutex<Vec<(Level, Vec<(String, String)>)>>>);

    struct Fields(Vec<(String, String)>);

    impl Visit for Fields {
        fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
            self.0.push((field.name().to_string(), format!("{value:?}")));
        }
    }

    impl<S: Subscriber> Layer<S> for Seen {
        fn on_event(&self, event: &Event<'_>, _cx: Context<'_, S>) {
            let mut fields = Fields(Vec::new());
            event.record(&mut fields);
            self.0.lock().unwrap().push((*event.metadata().level(), fields.0));
        }
    }

    fn seen(write: impl FnOnce()) -> Vec<(Level, Vec<(String, String)>)> {
        let seen = Seen::default();
        tracing::subscriber::with_default(tracing_subscriber::registry().with(seen.clone()), write);
        let events = seen.0.lock().unwrap().clone();
        events
    }

    fn field<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
        fields.iter().find(|(field, _)| field == name).map(|(_, value)| value.as_str())
    }

    #[test]
    fn a_failure_is_one_warning_with_the_error_what_failed_and_where() {
        let line = line!() + 2;
        let events = seen(|| {
            Err::<(), _>("disk full").or_warn(format_args!("could not keep the {}", "theme"));
        });

        assert_eq!(events.len(), 1, "{:?}", events.iter().map(|(_, fields)| fields).collect::<Vec<_>>());
        let (level, fields) = &events[0];
        assert_eq!(*level, Level::WARN);
        assert_eq!(field(fields, "message"), Some("could not keep the theme"));
        assert_eq!(field(fields, "err"), Some("\"disk full\""));
        assert_eq!(field(fields, "at"), Some(format!("{}:{line}", file!()).as_str()));
    }

    #[test]
    fn a_success_says_nothing_and_hands_the_value_on() {
        let mut opened = None;
        let events = seen(|| opened = Ok::<u8, &str>(5).or_warn("could not open the settings"));

        assert!(events.is_empty());
        assert_eq!(opened, Some(5));
    }
}
