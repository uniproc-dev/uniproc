use amethystate::{ReactiveMap, amethystate};
use app_contracts::features::activity::{Filter, Pick, Span};

#[amethystate(prefix = "activity")]
pub struct ActivitySettings {
    #[amestate(default = Span::default().id().to_string())]
    span: String,

    #[amestate(default = true)]
    came: bool,

    #[amestate(default = true)]
    went: bool,

    #[amestate(default = false)]
    new_only: bool,

    #[amestate(default = true)]
    series: bool,

    #[amestate(default = String::new())]
    only: String,

    #[amestate(default = {})]
    hidden: ReactiveMap<String, u32>,
}

pub fn remembered(settings: &ActivitySettings) -> (Span, Filter) {
    let span = Span::from_id(&settings.span().get()).unwrap_or_default();
    let mut hidden: Vec<(u32, Pick)> = settings
        .hidden()
        .entries()
        .filter_map(|(id, order)| Pick::from_id(&id).map(|pick| (order, pick)))
        .collect();
    hidden.sort_by_key(|(order, _)| *order);
    let filter = Filter {
        came: settings.came().get(),
        went: settings.went().get(),
        new_only: settings.new_only().get(),
        series: settings.series().get(),
        text: String::new(),
        only: Pick::from_id(&settings.only().get()),
        hidden: hidden.into_iter().map(|(_, pick)| pick).collect(),
    };
    (span, filter)
}

pub fn remember(settings: &ActivitySettings, span: Span, filter: &Filter) -> anyhow::Result<()> {
    settings.span().set(span.id().to_string())?;
    settings.came().set(filter.came)?;
    settings.went().set(filter.went)?;
    settings.new_only().set(filter.new_only)?;
    settings.series().set(filter.series)?;
    settings.only().set(filter.only.as_ref().map(Pick::id).unwrap_or_default())?;

    let hidden = settings.hidden();
    let kept: Vec<String> = filter.hidden.iter().map(Pick::id).collect();
    for (id, _) in hidden.entries() {
        if !kept.contains(&id) {
            hidden.remove(&id)?;
        }
    }
    for (order, id) in kept.into_iter().enumerate() {
        hidden.insert(id, &(order as u32))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use amethystate::store::builder::StoreBuilder;

    use super::*;

    #[test]
    fn what_was_chosen_is_read_back_as_it_was_but_the_search() {
        let (store, _) = StoreBuilder::in_memory().migrate().unwrap();
        let chosen = Filter {
            came: false,
            went: true,
            new_only: true,
            series: false,
            text: "git".into(),
            only: Some(Pick::Launcher("explorer.exe".into())),
            hidden: vec![Pick::Folder(r"c:\a".into()), Pick::Exe(r"c:\b\b.exe".into())],
        };

        remember(&ActivitySettings::new_with(&store).unwrap(), Span::Quarter, &chosen).unwrap();
        let (span, filter) = remembered(&ActivitySettings::new_with(&store).unwrap());

        assert_eq!(span, Span::Quarter);
        assert_eq!(
            filter,
            Filter {
                text: String::new(),
                ..chosen
            }
        );
    }
}
