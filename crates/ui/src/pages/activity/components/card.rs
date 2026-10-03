use app_contracts::features::activity::ActivityRow;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, ChildrenControl, ContentControl, CornerRadius, StackPanel, TextTrimming, TextWrapping,
    Thickness, View,
};

use super::super::marks::ActivityMark;
use super::lasted::lasted;
use super::rows::launcher_text;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{radius, space, Palette};
use crate::widgets::text::{body_strong, caption};

fn line(text: String, palette: Palette) -> View {
    caption(text)
        .foreground(palette.secondary_text)
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
        .into()
}

pub fn card(row: &ActivityRow, l10n: &L10n, palette: Palette) -> View {
    let lines: Vec<(String, View)> = match row {
        ActivityRow::Came(came) => {
            let lived = came
                .exit
                .as_ref()
                .map_or_else(|| l10n.activity_still_running(), |exit| lasted(l10n, exit.lived));
            let mut lines = vec![
                ("name".to_string(), body_strong(came.name.to_string()).into()),
                (
                    "when".to_string(),
                    line(l10n.activity_hover_came(format::clock(came.at), lived), palette),
                ),
            ];
            if let Some(from) = launcher_text(came, l10n) {
                lines.push(("from".to_string(), line(from, palette)));
            }
            if !came.command_line.is_empty() {
                lines.push(("command".to_string(), line(came.command_line.to_string(), palette)));
            }
            lines
        }
        ActivityRow::Went(went) => {
            let name = went
                .name
                .as_deref()
                .map_or_else(|| l10n.activity_unknown_process(i64::from(went.key.pid)), str::to_string);
            vec![
                ("name".to_string(), body_strong(name).into()),
                ("when".to_string(), line(l10n.activity_hover_went(format::clock(went.exit.at)), palette)),
            ]
        }
        ActivityRow::Burst(_) => Vec::new(),
    };
    Border::new()
        .mark(ActivityMark::Card)
        .background(palette.menu_fill)
        .border_brush(palette.menu_stroke)
        .border_thickness(Thickness::uniform(1.0))
        .corner_radius(CornerRadius::uniform(radius::Control))
        .padding(Thickness::xy(space::Cell, space::Control))
        .content(StackPanel::new().children((View::keyed_fragment(lines),)))
}
