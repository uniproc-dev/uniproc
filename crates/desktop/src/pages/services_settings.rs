use crate::routes::ServicesRoute;
use guinea::feature::Install;
use guinea::winui::{Page, PageCx, UpdateCx, page};
use ui::pages::services::{ServicesSettingsMsg, ServicesSettingsPage};
use ui::theme::{Palette, scheme_context};
use windows_reactor::{Callback, View};

pub struct ServicesSettings(ServicesSettingsPage);

#[page]
impl Page for ServicesSettings {
    type Message = ServicesSettingsMsg;

    fn install(_cx: &Install, store: amethystate::Store) -> anyhow::Result<Self> {
        Ok(Self(ServicesSettingsPage::new(
            super::services::open_settings(&store)?,
        )))
    }

    fn update(&mut self, message: ServicesSettingsMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ServicesSettingsMsg| message);
        let nav = cx.navigate::<ServicesRoute>();
        self.0.view(
            &l10n,
            palette,
            forward,
            Callback::new(move |()| {
                let _ = nav.to(ServicesRoute::Overview);
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::Route;
    use app_contracts::features::services::ServiceColumn;
    use domain::features::services::settings::ServicesSettings as Stored;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, PropertyId, PropertyValue};
    use guinea_plugin_store::StoreAccess;
    use ui::pages::services::{ServicesMsg, ServicesPage, ServicesSettingsMark};

    #[guinea::test(iterations = 4)]
    fn column_settings_survive_remount_and_reset(h: &mut Harness) {
        let store = crate::test_support::install(h);
        let settings = store.try_settings::<Stored>().unwrap();
        let route = Route::Services(ServicesRoute::Settings);
        let mut page: Mounted<'_, ServicesSettings> =
            Mounted::mount_at(h.child(), (), route.clone()).unwrap();
        page.send(ServicesSettingsMsg::Expand(true));
        page.settle();
        let node = page
            .within(ServiceColumn::Name)
            .find(ServicesSettingsMark::Shown)
            .unwrap();
        assert_eq!(
            page.property(node, PropertyId::IsEnabled),
            Some(&PropertyValue::Bool(false))
        );
        page.within(ServiceColumn::ImagePath)
            .click(ServicesSettingsMark::Shown)
            .settle();
        assert!(
            settings
                .columns()
                .get(ServiceColumn::ImagePath.id())
                .unwrap()
                .visible
        );
        page.send(ServicesSettingsMsg::MoveColumn(
            ServiceColumn::ImagePath,
            false,
        ));
        page.settle();
        let rank = settings.order().get(ServiceColumn::ImagePath.id()).unwrap();
        let mut overview =
            ServicesPage::new(super::super::services::open_settings(&store).unwrap());
        overview.update(ServicesMsg::Resized(ui::widgets::table::Resized {
            column: guinea::Mark::name(&ServiceColumn::ImagePath),
            width: 410.0,
        }));
        assert_eq!(
            settings
                .columns()
                .get(ServiceColumn::ImagePath.id())
                .unwrap()
                .width,
            410
        );
        drop(page);
        let mut page: Mounted<'_, ServicesSettings> =
            Mounted::mount_at(h.child(), (), route).unwrap();
        page.send(ServicesSettingsMsg::Expand(true));
        page.settle();
        let node = page
            .within(ServiceColumn::ImagePath)
            .find(ServicesSettingsMark::Shown)
            .unwrap();
        assert_eq!(
            page.property(node, PropertyId::IsOn),
            Some(&PropertyValue::Bool(true))
        );
        assert_eq!(
            settings.order().get(ServiceColumn::ImagePath.id()),
            Some(rank)
        );
        page.click(ServicesSettingsMark::ResetColumns).settle();
        assert!(
            settings
                .columns()
                .get(ServiceColumn::ImagePath.id())
                .is_none()
        );
        assert!(
            settings
                .order()
                .get(ServiceColumn::ImagePath.id())
                .is_none()
        );
    }
}
