use app_contracts::features::activity::ActivityState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::activity::{ActivityPresetsMsg, ActivityPresetsPage};
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

use crate::routes::Route;

#[derive(Default)]
pub struct ActivityPresets(ActivityPresetsPage);

#[page]
impl Page for ActivityPresets {
    type Params = crate::routes::ActivityPresetsParams;
    type Installs = ();
    type Message = ActivityPresetsMsg;

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, message: ActivityPresetsMsg, cx: &mut UpdateCx<'_, Self>) {
        let (_, dispatch) = cx.read::<ActivityState>();
        self.0.update(message, &dispatch);
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (state, dispatch) = cx.read::<ActivityState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ActivityPresetsMsg| message);
        let nav = cx.navigate::<Route>();
        let back = Callback::new(move |()| nav.to(Route::Activity {}));
        self.0.view(&state, &dispatch, &l10n, palette, forward, back)
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::activity::{Clock, Filter, Hide, Pick, Preset, ShowWent, Unhide};
    use domain::features::activity::settings::{remember_presets, remembered_presets, ActivitySettings};
    use domain::features::activity::{ActivityDeps, ActivityFeature};
    use guinea::app::Harness;
    use guinea::winui::harness::Mounted;
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::{StoreAccess, StorePlugin};
    use ui::pages::activity::ActivityPresetsMark;

    use super::*;

    fn tools() -> Pick {
        Pick::Folder(r"c:\tools".into())
    }

    fn base(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ActivityDeps {
                now: || 0,
                clock: |_| Clock::default(),
            });
    }

    fn start(h: &mut Harness) {
        base(h);
        h.feature(ActivityFeature).unwrap();
    }

    fn mount(h: &Harness) -> Mounted<'_, ActivityPresets> {
        let mut page = Mounted::mount_at(
            h.child(),
            crate::routes::ActivityPresetsParams::default(),
            Route::ActivityPresets {},
        )
        .unwrap();
        page.settle();
        page
    }

    fn stored(h: &Harness) -> ActivitySettings {
        ActivitySettings::new_with(&h.segment().store().unwrap()).unwrap()
    }

    fn chosen() -> Filter {
        Filter {
            went: false,
            hidden: vec![tools()],
            ..Filter::default()
        }
    }

    fn save_tooling(h: &Harness, page: &mut Mounted<'_, ActivityPresets>) {
        h.act::<ActivityState>(Hide(tools())).settle();
        h.act::<ActivityState>(ShowWent(false)).settle();
        page.send(ActivityPresetsMsg::Name("tooling".into()));
        page.settle();
        page.click(ActivityPresetsMark::Save).settle();
        page.settle();
    }

    #[guinea::test(iterations = 4)]
    fn what_is_shown_is_saved_as_a_preset_in_use_and_kept_for_the_next_run(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        assert!(page.find(ActivityPresetsMark::Empty).is_some(), "{:#?}", page.tree());

        save_tooling(h, &mut page);

        let tooling = Preset {
            name: "tooling".into(),
            filter: chosen(),
        };
        assert_eq!(h.state::<ActivityState>().presets, std::slice::from_ref(&tooling));
        assert_eq!(remembered_presets(&stored(h)), [tooling]);
        assert!(page.find(ActivityPresetsMark::Current).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityPresetsMark::Empty).is_none(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn an_applied_preset_brings_its_choices_back_and_a_deleted_one_is_forgotten(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        save_tooling(h, &mut page);
        h.act::<ActivityState>(Unhide(tools())).settle();
        page.settle();
        assert!(page.find(ActivityPresetsMark::Current).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityPresetsMark::Apply).is_some(), "{:#?}", page.tree());

        page.click(ActivityPresetsMark::Apply).settle();
        page.settle();
        assert_eq!(h.state::<ActivityState>().filter, chosen());

        page.click(ActivityPresetsMark::Delete).settle();
        page.settle();
        assert!(h.state::<ActivityState>().presets.is_empty());
        assert!(remembered_presets(&stored(h)).is_empty());
        assert!(page.find(ActivityPresetsMark::Empty).is_some(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn what_is_shown_again_in_the_preset_in_use_is_shown_again_on_the_page(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        save_tooling(h, &mut page);
        assert!(page.find(ActivityPresetsMark::Hidden).is_some(), "{:#?}", page.tree());

        page.click(ActivityPresetsMark::Hidden).settle();
        page.settle();

        let state = h.state::<ActivityState>();
        assert!(state.presets[0].filter.hidden.is_empty(), "{:#?}", state.presets);
        assert!(state.filter.hidden.is_empty(), "{:#?}", state.filter);
        assert!(page.find(ActivityPresetsMark::Current).is_some(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_preset_saved_under_a_used_name_replaces_it(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        save_tooling(h, &mut page);
        h.act::<ActivityState>(ShowWent(true)).settle();

        page.send(ActivityPresetsMsg::Name("tooling".into()));
        page.click(ActivityPresetsMark::Save).settle();
        page.settle();

        let presets = &h.state::<ActivityState>().presets;
        assert_eq!(presets.len(), 1, "{presets:#?}");
        assert!(presets[0].filter.went, "{presets:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn presets_kept_last_run_are_there_from_the_start(h: &mut Harness) {
        base(h);
        let quiet = Preset {
            name: "quiet".into(),
            filter: chosen(),
        };
        remember_presets(&stored(h), std::slice::from_ref(&quiet)).unwrap();

        h.feature(ActivityFeature).unwrap();

        assert_eq!(h.state::<ActivityState>().presets, [quiet]);
    }
}
