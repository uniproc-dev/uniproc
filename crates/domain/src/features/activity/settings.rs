use amethystate::{ReactiveMap, amethystate};
use app_contracts::features::activity::{Filter, Group, Hue, Pick, Span};

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

    #[amestate(default = true)]
    other: bool,

    #[amestate(default = {})]
    groups: ReactiveMap<String, StoredGroup>,
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StoredGroup {
    order: u32,
    name: String,
    hue: String,
    rules: Vec<String>,
    shown: bool,
}

pub fn remembered_groups(settings: &ActivitySettings) -> Vec<Group> {
    let mut stored: Vec<(String, StoredGroup)> = settings.groups().entries().collect();
    stored.sort_by_key(|(_, group)| group.order);
    let mut groups: Vec<Group> = stored
        .into_iter()
        .map(|(id, group)| Group {
            id,
            name: group.name,
            hue: Hue::from_id(&group.hue).unwrap_or_default(),
            rules: group.rules.iter().filter_map(|rule| Pick::from_id(rule)).collect(),
            shown: group.shown,
        })
        .collect();
    if !groups.iter().any(Group::is_built_in) {
        groups.insert(0, Group::windows_background());
    }
    groups
}

pub fn remember_groups(settings: &ActivitySettings, groups: &[Group]) -> anyhow::Result<()> {
    let stored = settings.groups();
    for (id, _) in stored.entries() {
        if !groups.iter().any(|group| group.id == id) {
            stored.remove(&id)?;
        }
    }
    for (order, group) in groups.iter().enumerate() {
        let kept = StoredGroup {
            order: order as u32,
            name: group.name.clone(),
            hue: group.hue.id().to_string(),
            rules: group.rules.iter().map(Pick::id).collect(),
            shown: group.shown,
        };
        stored.insert(group.id.clone(), &kept)?;
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
        other: settings.other().get(),
    };
    (span, filter)
}

pub fn remember(settings: &ActivitySettings, span: Span, filter: &Filter) -> anyhow::Result<()> {
    settings.span().set(span.id().to_string());
    settings.came().set(filter.came);
    settings.went().set(filter.went);
    settings.new_only().set(filter.new_only);
    settings.series().set(filter.series);
    settings.only().set(filter.only.as_ref().map(Pick::id).unwrap_or_default());
    settings.other().set(filter.other);

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
            only: Some(Pick::Under("explorer.exe".into())),
            hidden: vec![Pick::Folder(r"c:\a".into()), Pick::Exe("b.exe".into())],
            other: false,
        };

        remember(&ActivitySettings::new_with(&store), Span::Quarter, &chosen).unwrap();
        let (span, filter) = remembered(&ActivitySettings::new_with(&store));

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
    fn with_nothing_kept_the_windows_background_group_is_there() {
        let (store, _) = StoreBuilder::in_memory().migrate().unwrap();

        assert_eq!(
            remembered_groups(&ActivitySettings::new_with(&store)),
            [Group::windows_background()]
        );
    }

    #[test]
    fn groups_are_read_back_in_their_order_and_a_dropped_one_is_gone() {
        let (store, _) = StoreBuilder::in_memory().migrate().unwrap();
        let settings = ActivitySettings::new_with(&store);
        let tooling = Group {
            id: "g1".into(),
            name: "Dev tooling".into(),
            hue: Hue::Purple,
            rules: vec![Pick::Under("claude.exe".into()), Pick::Exe("cargo.exe".into())],
            shown: false,
        };
        let background = Group {
            rules: vec![],
            ..Group::windows_background()
        };
        let dropped = Group {
            id: "g2".into(),
            name: "gone".into(),
            hue: Hue::Coral,
            rules: vec![],
            shown: true,
        };

        remember_groups(&settings, &[dropped, tooling.clone(), background.clone()]).unwrap();
        remember_groups(&settings, &[tooling.clone(), background.clone()]).unwrap();

        assert_eq!(
            remembered_groups(&ActivitySettings::new_with(&store)),
            [tooling, background]
        );
    }
}
