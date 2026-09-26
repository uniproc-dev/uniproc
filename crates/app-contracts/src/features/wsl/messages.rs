use std::rc::Rc;

use super::model::{DistroRow, LinuxMachineSummary};

#[derive(Clone)]
pub enum WslMsg {
    Set {
        distros: Rc<[DistroRow]>,
        machine: Option<LinuxMachineSummary>,
    },
}
