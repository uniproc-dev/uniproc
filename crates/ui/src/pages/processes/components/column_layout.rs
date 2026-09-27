use amethystate::ReactiveMap;
use app_contracts::features::processes::{ColumnConfig, ProcessColumn};
use guinea::Mark;
use guinea_widgets::table::{ColumnWidths, Resized};

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
}

impl ColumnLayout {
    pub(crate) fn new(configs: Option<ReactiveMap<String, ColumnConfig>>) -> Self {
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
        Self { configs, widths }
    }

    pub(crate) fn widths(&self) -> &ColumnWidths {
        &self.widths
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
