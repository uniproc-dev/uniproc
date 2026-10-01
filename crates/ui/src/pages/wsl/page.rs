use app_contracts::features::settings::ByteUnits;
use app_contracts::features::wsl::WslState;
use guinea::prelude::Load;
use crate::widgets::table::{table, ColumnWidths, Resized};
use windows_reactor::{Callback, View};

use super::components::columns::build_columns;
use crate::l10n::L10n;
use crate::theme::Palette;
use crate::widgets::page::{loading, page_frame, page_title, status_text};
use crate::widgets::text::text;

pub enum WslMsg {
    Resized(Resized),
}

#[derive(Default)]
pub struct WslPage {
    widths: ColumnWidths,
}

impl WslPage {
    pub fn update(&mut self, message: WslMsg) {
        match message {
            WslMsg::Resized(drag) => self.widths.apply(drag),
        }
    }

    pub fn view(
        &self,
        state: &WslState,
        l10n: &L10n,
        palette: Palette,
        forward: Callback<WslMsg>,
        units: ByteUnits,
    ) -> View {
        let body = match &state.distros {
            Load::Ready(rows) => table(
                rows.to_vec(),
                build_columns(l10n, palette, units),
            )
            .widths(&self.widths)
            .on_resize(move |drag: Resized| {
                let _ = forward.call(WslMsg::Resized(drag));
            })
            .build(),
            Load::Failed(err) => text(l10n.wsl_failed(err.to_string())).into(),
            _ => loading(),
        };

        page_frame(
            page_title(l10n.wsl_title()),
            body,
            status_text(l10n.wsl_status(state.total() as i64, state.running() as i64), palette),
            palette,
            None,
        )
    }
}
