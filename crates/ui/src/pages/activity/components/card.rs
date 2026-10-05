use app_contracts::features::activity::ActivityRow;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, CornerRadius, KeyedView, StackPanel, TextTrimming, TextWrapping, Thickness, View, keyed,
};

use super::super::marks::ActivityMark;
use super::lasted::lasted;
use super::rows::launcher_text;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{Palette, radius, space};
use crate::widgets::text::{body_strong, caption};

fn line(text: String, palette: Palette) -> View {
    caption(text)
        .foreground(palette.secondary_text)
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
        .into()
}

pub fn card(row: &ActivityRow, l10n: &L10n, palette: Palette) -> View {
    let lines: Vec<KeyedView> = match row {
        ActivityRow::Came(came) => {
            let lived = came.exit.as_ref().map_or_else(
                || l10n.activity_still_running(),
                |exit| lasted(l10n, exit.lived),
            );
            let mut lines = vec![
                keyed("name", body_strong(came.name.to_string())),
                keyed(
                    "when",
                    line(
                        l10n.activity_hover_came(format::clock(came.at), lived),
                        palette,
                    ),
                ),
            ];
            if let Some(from) = launcher_text(came, l10n) {
                lines.push(keyed("from", line(from, palette)));
            }
            if !came.command_line.is_empty() {
                lines.push(keyed(
                    "command",
                    line(came.command_line.to_string(), palette),
                ));
            }
            lines
        }
        ActivityRow::Went(went) => {
            let name = went.name.as_deref().map_or_else(
                || l10n.activity_unknown_process(i64::from(went.key.pid)),
                str::to_string,
            );
            vec![
                keyed("name", body_strong(name)),
                keyed(
                    "when",
                    line(
                        l10n.activity_hover_went(format::clock(went.exit.at)),
                        palette,
                    ),
                ),
            ]
        }
        ActivityRow::Series(_) => Vec::new(),
    };
    Border::new()
        .mark(ActivityMark::Card)
        .background(palette.menu_fill)
        .border_brush(palette.menu_stroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(radius::Control))
        .padding(Thickness::xy(space::Cell, space::Control))
        .content(StackPanel::new().keyed_children(lines))
        .into()
}
