use amethystate::{ReactiveMap, amethystate};
use app_contracts::features::activity::{Filter, Pick, Preset, Span};

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

    #[amestate(default = {})]
    presets: ReactiveMap<String, StoredPreset>,
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StoredPreset {
    order: u32,
    came: bool,
    went: bool,
    new_only: bool,
    series: bool,
    only: String,
    hidden: Vec<String>,
}

impl StoredPreset {
    fn of(order: usize, filter: &Filter) -> Self {
        Self {
            order: order as u32,
            came: filter.came,
            went: filter.went,
            new_only: filter.new_only,
            series: filter.series,
            only: filter.only.as_ref().map(Pick::id).unwrap_or_default(),
            hidden: filter.hidden.iter().map(Pick::id).collect(),
        }
    }

    fn filter(&self) -> Filter {
        Filter {
            came: self.came,
            went: self.went,
            new_only: self.new_only,
            series: self.series,
            text: String::new(),
            only: Pick::from_id(&self.only),
            hidden: self.hidden.iter().filter_map(|id| Pick::from_id(id)).collect(),
        }
    }
}

pub fn remembered_presets(settings: &ActivitySettings) -> Vec<Preset> {
    let mut stored: Vec<(String, StoredPreset)> = settings.presets().entries().collect();
    stored.sort_by_key(|(_, preset)| preset.order);
    stored
        .into_iter()
        .map(|(name, preset)| Preset {
            filter: preset.filter(),
            name,
        })
        .collect()
}

pub fn remember_presets(settings: &ActivitySettings, presets: &[Preset]) -> anyhow::Result<()> {
    let stored = settings.presets();
    for (name, _) in stored.entries() {
        if !presets.iter().any(|preset| preset.name == name) {
            stored.remove(&name)?;
        }
    }
    for (order, preset) in presets.iter().enumerate() {
        stored.insert(preset.name.clone(), &StoredPreset::of(order, &preset.filter))?;
    }
    Ok(())
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

    #[test]
    fn presets_are_read_back_in_the_order_they_were_kept_and_a_dropped_one_is_gone() {
        let (store, _) = StoreBuilder::in_memory().migrate().unwrap();
        let settings = ActivitySettings::new_with(&store).unwrap();
        let tooling = Preset {
            name: "tooling".into(),
            filter: Filter {
                series: false,
                hidden: vec![Pick::Launcher("claude.exe".into()), Pick::Exe(r"c:\git\git.exe".into())],
                ..Filter::default()
            },
        };
        let quiet = Preset {
            name: "quiet".into(),
            filter: Filter {
                went: false,
                only: Some(Pick::Folder(r"c:\work".into())),
                ..Filter::default()
            },
        };
        let gone = Preset {
            name: "gone".into(),
            filter: Filter::default(),
        };

        remember_presets(&settings, &[gone, tooling.clone(), quiet.clone()]).unwrap();
        remember_presets(&settings, &[tooling.clone(), quiet.clone()]).unwrap();

        assert_eq!(
            remembered_presets(&ActivitySettings::new_with(&store).unwrap()),
            [tooling, quiet]
        );
    }
}
