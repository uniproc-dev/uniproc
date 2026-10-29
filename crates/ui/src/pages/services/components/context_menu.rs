use app_contracts::features::services::ServiceColumn;
use guicons::icon;
use guinea::Mark;
use windows_reactor::{Border, Callback, View};

use super::column_layout::ColumnLayout;
use super::columns::column_label;
use crate::l10n::L10n;
use crate::pages::services::ServicesMark;
use crate::theme::{size, Palette};
use crate::widgets::popup_menu::{popup_menu, MenuLine, PopupMenu};

struct MenuMark(&'static str);

impl Mark for MenuMark {
    fn name(&self) -> &'static str { self.0 }
}

pub(crate) fn context_menu(
    (x, y): (f64, f64), layout: &ColumnLayout, l10n: &L10n, palette: Palette,
    on_command: Callback<Option<ServiceColumn>>, on_dismiss: Callback<()>,
) -> View {
    let mut lines: Vec<_> = layout.placed().into_iter().map(|column| {
        let image = if layout.config(column).visible {
            icon!(checkmark).size(size::Icon).build_element()
        } else { Border::new().width(size::Icon).height(size::Icon).into() };
        MenuLine::entry(MenuMark(column.name()), image, column_label(l10n, column), Some(column))
            .enabled_if(column != ServiceColumn::Name)
    }).collect();
    lines.push(MenuLine::Separator);
    lines.push(MenuLine::entry(MenuMark(ServicesMark::MenuMoreColumns.name()),
        icon!(more_horizontal).size(size::Icon).build_element(),
        l10n.services_menu_more_columns(), None));
    popup_menu(PopupMenu { x, y, lines,
        card: MenuMark(ServicesMark::Menu.name()), backdrop: MenuMark(ServicesMark::MenuBackdrop.name()),
        palette, on_command, on_dismiss,
    })
}
