use app_contracts::features::activity::ActivityState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::activity::{ActivityGroupsMsg, ActivityGroupsPage};
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

use crate::routes::Route;

#[derive(Default)]
pub struct ActivityGroups(ActivityGroupsPage);

#[page]
impl Page for ActivityGroups {
    type Params = crate::routes::ActivityGroupsParams;
    type Installs = ();
    type Message = ActivityGroupsMsg;

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, message: ActivityGroupsMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (state, dispatch) = cx.read::<ActivityState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ActivityGroupsMsg| message);
        let nav = cx.navigate::<Route>();
        let back = Callback::new(move |()| nav.to(Route::Activity {}));
        self.0.view(&state, &dispatch, &l10n, palette, forward, back)
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::activity::{Clock, Group, Hide, Hue, NewGroup, Pick};
    use domain::features::activity::settings::{remembered_groups, ActivitySettings};
    use domain::features::activity::{ActivityDeps, ActivityFeature};
    use guinea::app::Harness;
    use guinea::winui::harness::Mounted;
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::{StoreAccess, StorePlugin};
    use ui::pages::activity::ActivityGroupsMark;

    use super::*;

    fn start(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ActivityDeps {
                now: || 0,
                clock: |_| Clock::default(),
            });
        h.feature(ActivityFeature).unwrap();
    }

    fn mount(h: &Harness) -> Mounted<'_, ActivityGroups> {
        let mut page = Mounted::mount_at(
            h.child(),
            crate::routes::ActivityGroupsParams::default(),
            Route::ActivityGroups {},
        )
        .unwrap();
        page.settle();
        page
    }

    fn stored(h: &Harness) -> ActivitySettings {
        ActivitySettings::new_with(&h.segment().store().unwrap()).unwrap()
    }

    fn groups(h: &Harness) -> Vec<Group> {
        h.state::<ActivityState>().groups.clone()
    }

    fn click(page: &mut Mounted<'_, ActivityGroups>, mark: ActivityGroupsMark) {
        assert!(page.find(mark).is_some(), "no {mark:?} in {:#?}", page.tree());
        page.click(mark).settle();
        page.settle();
    }

    fn expand(page: &mut Mounted<'_, ActivityGroups>, group: &str) {
        page.send(ActivityGroupsMsg::Expand(group.to_string(), true));
        page.settle();
    }

    #[guinea::test(iterations = 4)]
    fn the_built_in_group_is_shown_as_built_in_and_cannot_be_renamed_or_deleted(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        expand(&mut page, Group::WINDOWS_BACKGROUND);

        assert!(page.find(ActivityGroupsMark::Group).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::BuiltIn).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Shown).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Rule).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Colour).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Name).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Delete).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Up).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::Down).is_none(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_group_switched_off_here_is_off_on_the_page_and_next_run(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);

        click(&mut page, ActivityGroupsMark::Shown);

        assert!(!groups(h)[0].shown, "{:#?}", groups(h));
        assert!(!remembered_groups(&stored(h))[0].shown);
    }

    #[guinea::test(iterations = 4)]
    fn a_group_is_moved_recoloured_emptied_and_deleted_from_its_card(h: &mut Harness) {
        start(h);
        let h = &*h;
        let tool = Pick::Exe("tool.exe".into());
        h.act::<ActivityState>(NewGroup(tool)).settle();
        let mut page = mount(h);
        let id = groups(h)[1].id.clone();
        assert_ne!(groups(h)[1].hue, Hue::Teal);
        expand(&mut page, &id);
        assert!(page.find(ActivityGroupsMark::Name).is_some(), "{:#?}", page.tree());

        click(&mut page, ActivityGroupsMark::Up);
        assert_eq!(groups(h)[0].id, id);

        click(&mut page, ActivityGroupsMark::Colour);
        assert_eq!(groups(h)[0].hue, Hue::Teal);

        click(&mut page, ActivityGroupsMark::Rule);
        assert!(groups(h)[0].rules.is_empty(), "{:#?}", groups(h));
        assert_eq!(groups(h)[1], Group::windows_background());

        click(&mut page, ActivityGroupsMark::Down);
        assert_eq!(groups(h)[1].id, id);

        click(&mut page, ActivityGroupsMark::Delete);
        assert_eq!(groups(h), [Group::windows_background()]);
        assert_eq!(remembered_groups(&stored(h)), [Group::windows_background()]);
    }

    #[guinea::test(iterations = 4)]
    fn what_is_hidden_is_listed_here_and_shown_again_from_here(h: &mut Harness) {
        start(h);
        let h = &*h;
        h.act::<ActivityState>(Hide(Pick::Exe("conhost.exe".into()))).settle();
        let mut page = mount(h);
        assert!(page.find(ActivityGroupsMark::Hidden).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::NothingHidden).is_none(), "{:#?}", page.tree());

        click(&mut page, ActivityGroupsMark::ShowAgain);

        assert!(h.state::<ActivityState>().filter.hidden.is_empty());
        assert!(page.find(ActivityGroupsMark::Hidden).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityGroupsMark::NothingHidden).is_some(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn the_breadcrumb_leads_back_to_activity(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);

        page.click(ActivityGroupsMark::Back).settle();

        assert_eq!(page.navigated::<Route>(), [Route::Activity {}]);
    }
}
