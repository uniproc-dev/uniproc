use std::sync::Arc;

pub type RowKey = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Disabled,
    Success,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Weight {
    #[default]
    Normal,
    Strong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Icon {
    Svg(&'static [u8]),
    Rgba { width: u32, height: u32, pixels: Arc<[u8]> },
}

impl Icon {
    pub fn identity(&self) -> usize {
        match self {
            Icon::Svg(bytes) => bytes.as_ptr() as usize,
            Icon::Rgba { pixels, .. } => pixels.as_ptr() as usize,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Chevron {
    #[default]
    None,
    Slot,
    Collapsed,
    Expanded,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub text: String,
    pub note: String,
    pub tone: Tone,
    pub weight: Weight,
    pub align: Align,
    pub heat: Option<Rgba>,
    pub icon: Option<Icon>,
    pub chevron: Chevron,
    pub indent: f32,
    pub dim: bool,
}

impl Cell {
    pub fn clear(&mut self) {
        self.text.clear();
        self.note.clear();
        self.tone = Tone::Primary;
        self.weight = Weight::Normal;
        self.align = Align::Start;
        self.heat = None;
        self.icon = None;
        self.chevron = Chevron::None;
        self.indent = 0.0;
        self.dim = false;
    }
}

pub trait Source {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn key(&self, at: usize) -> RowKey;
    fn height(&self, at: usize) -> f32;
    fn columns(&self) -> usize;
    fn cell(&self, at: usize, column: usize, out: &mut Cell);
}
