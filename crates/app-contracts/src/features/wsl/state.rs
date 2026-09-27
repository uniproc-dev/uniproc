use guinea::prelude::*;
use std::rc::Rc;

use super::messages::WslMsg;
use super::model::{DistroRow, LinuxMachineSummary};

#[derive(Clone, PartialEq, Debug)]
pub struct WslState {
    pub distros: Load<Rc<[DistroRow]>>,
    pub machine: Option<LinuxMachineSummary>,
}

impl Default for WslState {
    fn default() -> Self {
        Self {
            distros: Load::Loading,
            machine: None,
        }
    }
}

impl WslState {
    pub fn distros(&self) -> &[DistroRow] {
        self.distros.ready().map(|d| d.as_ref()).unwrap_or(&[])
    }

    pub fn total(&self) -> usize {
        self.distros().len()
    }

    pub fn running(&self) -> usize {
        self.distros().iter().filter(|d| d.running).count()
    }
}

#[reducer]
fn wsl(this: &mut WslState, WslMsg::Set { distros, machine }: WslMsg) {
    this.distros = Load::Ready(distros);
    this.machine = machine;
}
