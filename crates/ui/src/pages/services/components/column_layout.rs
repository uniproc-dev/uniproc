use amethystate::ReactiveMap;
use app_contracts::features::services::{ServiceColumn, ServiceColumnConfig};
use guinea::Mark;

use crate::widgets::table::{ColumnOrder, ColumnWidths, Reordered, Resized};

#[derive(Clone)]
pub struct ServicesSettingsMaps {
    pub columns: ReactiveMap<String, ServiceColumnConfig>,
    pub order: ReactiveMap<String, u32>,
}

#[derive(Default)]
pub(crate) struct ColumnLayout {
    settings: Option<ServicesSettingsMaps>,
    widths: ColumnWidths,
}

impl ColumnLayout {
    pub(crate) fn new(settings: Option<ServicesSettingsMaps>) -> Self {
        Self {
            settings,
            widths: ColumnWidths::default(),
        }
    }

    pub(crate) fn config(&self, column: ServiceColumn) -> ServiceColumnConfig {
        let mut config = self
            .settings
            .as_ref()
            .and_then(|s| s.columns.get(column.id()))
            .unwrap_or_else(|| column.default_config());
        if column == ServiceColumn::Name {
            config.visible = true;
        }
        config
    }

    pub(crate) fn placed(&self) -> Vec<ServiceColumn> {
        let mut columns = ServiceColumn::ALL.to_vec();
        columns[1..].sort_by_key(|column| {
            self.settings
                .as_ref()
                .and_then(|s| s.order.get(column.id()))
                .unwrap_or(u32::MAX)
        });
        columns
    }

    pub(crate) fn order(&self) -> ColumnOrder {
        ColumnOrder::new(self.placed().into_iter().map(|c| c.name()).collect())
    }

    pub(crate) fn widths(&self) -> &ColumnWidths {
        &self.widths
    }

    pub(crate) fn is_default(&self) -> bool {
        self.placed() == ServiceColumn::ALL
            && ServiceColumn::ALL
                .into_iter()
                .all(|column| self.config(column) == column.default_config())
    }

    fn store(&self, column: ServiceColumn, config: ServiceColumnConfig) {
        if let Some(settings) = &self.settings
            && let Err(err) = settings.columns.upsert(column.id(), |_| config)
        {
            tracing::warn!(%err, "could not keep service column appearance");
        }
    }

    pub(crate) fn show(&self, column: ServiceColumn, visible: bool) {
        if column != ServiceColumn::Name {
            self.store(
                column,
                ServiceColumnConfig {
                    visible,
                    ..self.config(column)
                },
            );
        }
    }

    pub(crate) fn resize(&mut self, drag: Resized) {
        self.widths.apply(drag);
        if let Some(column) = ServiceColumn::ALL
            .into_iter()
            .find(|c| c.name() == drag.column)
        {
            self.store(
                column,
                ServiceColumnConfig {
                    width: drag.width.round().max(24.0) as u64,
                    ..self.config(column)
                },
            );
        }
    }

    pub(crate) fn reorder(&self, moved: Reordered) {
        let Some(settings) = &self.settings else {
            return;
        };
        let old = self.placed();
        let mut columns: Vec<_> = moved
            .order
            .iter()
            .filter_map(|name| old.iter().copied().find(|c| c.name() == *name))
            .collect();
        for column in old {
            if !columns.contains(&column) {
                columns.push(column);
            }
        }
        for (rank, column) in columns.into_iter().enumerate() {
            if let Err(err) = settings.order.upsert(column.id(), |_| rank as u32) {
                tracing::warn!(%err, "could not keep service column order");
            }
        }
    }

    pub(crate) fn shift(&self, column: ServiceColumn, down: bool) {
        let mut columns = self.placed();
        let Some(at) = columns.iter().position(|c| *c == column) else {
            return;
        };
        let to = if down { at + 1 } else { at.saturating_sub(1) };
        if at == 0 || to == 0 || to >= columns.len() {
            return;
        }
        columns.swap(at, to);
        self.reorder(Reordered {
            order: columns.into_iter().map(|c| c.name()).collect(),
        });
    }

    pub(crate) fn reset(&mut self) {
        self.widths = ColumnWidths::default();
        if let Some(settings) = &self.settings {
            if let Err(err) = settings.columns.clear() {
                tracing::warn!(%err, "could not reset service columns");
            }
            if let Err(err) = settings.order.clear() {
                tracing::warn!(%err, "could not reset service column order");
            }
        }
    }
}
