use amethystate::ReactiveMap;
use app_contracts::features::processes::{ColumnConfig, ProcessColumn};
use guinea::Mark;
use guinea_widgets::table::{ColumnOrder, ColumnWidths, Reordered, Resized};

struct MinWidth;

#[expect(non_upper_case_globals)]
impl MinWidth {
    const Name: f64 = 140.0;
    const Metric: f64 = 64.0;

    fn of(column: ProcessColumn) -> f64 {
        if column == ProcessColumn::Name {
            Self::Name
        } else {
            Self::Metric
        }
    }
}

pub(crate) struct ColumnState {
    pub(crate) column: ProcessColumn,
    pub(crate) width: f64,
    pub(crate) min_width: f64,
    pub(crate) visible: bool,
}

#[derive(Default)]
pub(crate) struct ColumnLayout {
    configs: Option<ReactiveMap<String, ColumnConfig>>,
    widths: ColumnWidths,
    ranks: Option<ReactiveMap<String, u32>>,
    order: ColumnOrder,
}

fn kept_order(ranks: &ReactiveMap<String, u32>) -> ColumnOrder {
    let mut ranked: Vec<(u32, ProcessColumn)> = ProcessColumn::ALL
        .into_iter()
        .filter_map(|column| ranks.get(column.id()).map(|rank| (rank, column)))
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ColumnOrder::new(ranked.into_iter().map(|(_, column)| column.name()).collect())
}

impl ColumnLayout {
    pub(crate) fn new(
        configs: Option<ReactiveMap<String, ColumnConfig>>,
        ranks: Option<ReactiveMap<String, u32>>,
    ) -> Self {
        let mut widths = ColumnWidths::default();
        if let Some(configs) = &configs {
            for column in ProcessColumn::ALL {
                if let Some(config) = configs.get(column.id()) {
                    widths.apply(Resized {
                        column: column.name(),
                        width: config.width as f64,
                    });
                }
            }
        }
        let order = ranks.as_ref().map(kept_order).unwrap_or_default();
        Self {
            configs,
            widths,
            ranks,
            order,
        }
    }

    pub(crate) fn widths(&self) -> &ColumnWidths {
        &self.widths
    }

    pub(crate) fn order(&self) -> &ColumnOrder {
        &self.order
    }

    pub(crate) fn reorder(&mut self, moved: Reordered) {
        let hidden: Vec<&'static str> = self
            .order
            .names()
            .iter()
            .copied()
            .filter(|name| !moved.order.contains(name))
            .collect();
        self.order.apply(moved);
        let names: Vec<&'static str> = self.order.names().iter().copied().chain(hidden).collect();
        self.order = ColumnOrder::new(names);

        let Some(ranks) = &self.ranks else {
            return;
        };
        for (rank, name) in self.order.names().iter().enumerate() {
            let Some(column) = ProcessColumn::from_mark(name) else {
                continue;
            };
            if let Err(err) = ranks.insert(column.id().to_string(), &(rank as u32)) {
                tracing::warn!(column = column.id(), ?err, "column order write failed");
            }
        }
    }

    fn config(&self, column: ProcessColumn) -> ColumnConfig {
        self.configs
            .as_ref()
            .and_then(|configs| configs.get(column.id()))
            .unwrap_or_else(|| column.default_config())
    }

    pub(crate) fn toggle(&mut self, column: ProcessColumn) {
        let config = self.config(column);
        self.store(column, ColumnConfig {
            visible: !config.visible,
            ..config
        });
    }

    fn store(&self, column: ProcessColumn, config: ColumnConfig) {
        let Some(configs) = &self.configs else {
            return;
        };
        let id = column.id();
        let result = if configs.contains_key(id) {
            configs.update(id, &config)
        } else {
            configs.insert(id.to_string(), &config)
        };
        if let Err(err) = result {
            tracing::warn!(column = id, ?err, "column config write failed");
        }
    }

    pub(crate) fn columns(&self) -> Vec<ColumnState> {
        ProcessColumn::ALL
            .into_iter()
            .map(|column| {
                let config = self.config(column);
                ColumnState {
                    column,
                    width: self.widths.get(column.name()).unwrap_or(config.width as f64),
                    min_width: MinWidth::of(column),
                    visible: config.visible,
                }
            })
            .collect()
    }

    pub(crate) fn resize(&mut self, drag: Resized) {
        self.widths.apply(drag);

        let Some(column) = ProcessColumn::from_mark(drag.column) else {
            tracing::warn!(column = drag.column, "resize of a column this page does not know");
            return;
        };
        let width = drag.width.round().max(0.0) as u64;
        self.store(column, ColumnConfig {
            width,
            ..self.config(column)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(columns: &[ProcessColumn]) -> Vec<&'static str> {
        columns.iter().map(|column| column.name()).collect()
    }

    #[test]
    fn a_hidden_column_is_not_dropped_from_the_order_by_a_move_without_it() {
        use ProcessColumn::*;
        let mut layout = ColumnLayout::default();
        layout.reorder(Reordered { order: names(&[Name, Pid, Memory, Cpu]) });
        layout.reorder(Reordered { order: names(&[Name, Cpu, Memory]) });
        assert_eq!(layout.order().names(), names(&[Name, Cpu, Memory, Pid]));
    }
}
