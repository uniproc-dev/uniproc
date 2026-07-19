use forsl_macros::port;

use super::model::WslDistroDto;

#[derive(Clone, Debug, PartialEq)]
pub enum UiEnvironmentsPortMsg {
    SetHostIconByKey(String),
    SetWslDistros(Vec<WslDistroDto>),
    SetHostName(String),
    SetSelectedEnv(String),
    SetHasWsl(bool),
    SetWslIsLoading(bool),
    SetWslDistrosIsLoading(bool),
}

#[port]
pub trait UiEnvironmentsPort: 'static {
    fn send(&self, msg: UiEnvironmentsPortMsg);
}
