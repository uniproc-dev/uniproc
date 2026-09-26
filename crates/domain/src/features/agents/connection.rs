use std::time::Duration;

use app_contracts::features::agents::AgentConnectionState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionEvent {
    BeginConnect,
    ConnectSucceeded,
    ConnectFailed,
    RetryDelayElapsed,
    ConnectionLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionEffect {
    None,
    ScheduleRetry,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transition {
    pub from: AgentConnectionState,
    pub event: ConnectionEvent,
    pub to: AgentConnectionState,
    pub effect: TransitionEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidTransition {
    pub state: AgentConnectionState,
    pub event: ConnectionEvent,
}

pub fn retry_in(window: Duration, spent: Duration) -> Duration {
    window.saturating_sub(spent)
}

pub struct Attempts;

#[expect(non_upper_case_globals)]
impl Attempts {
    pub const BeforeGivingUp: u32 = 5;
}

#[derive(Debug)]
pub struct ConnectionMachine {
    state: AgentConnectionState,
    failures: u32,
}

impl Default for ConnectionMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionMachine {
    pub fn new() -> Self {
        Self {
            state: AgentConnectionState::Disconnected,
            failures: 0,
        }
    }

    pub fn apply(&mut self, event: ConnectionEvent) -> Result<Transition, InvalidTransition> {
        let from = self.state;

        let (to, effect) = match (self.state, event) {
            (AgentConnectionState::Disconnected, ConnectionEvent::BeginConnect) => {
                (AgentConnectionState::Connecting, TransitionEffect::None)
            }
            (AgentConnectionState::Connecting, ConnectionEvent::ConnectSucceeded) => {
                self.failures = 0;
                (AgentConnectionState::Connected, TransitionEffect::None)
            }
            (AgentConnectionState::Connecting, ConnectionEvent::ConnectFailed) => {
                self.failures = self.failures.saturating_add(1);
                (AgentConnectionState::WaitingRetry, TransitionEffect::ScheduleRetry)
            }
            (AgentConnectionState::WaitingRetry, ConnectionEvent::RetryDelayElapsed) => {
                (AgentConnectionState::Connecting, TransitionEffect::None)
            }
            (AgentConnectionState::Connected, ConnectionEvent::ConnectionLost) => {
                self.failures = 0;
                (AgentConnectionState::Disconnected, TransitionEffect::None)
            }
            _ => {
                return Err(InvalidTransition { state: self.state, event });
            }
        };

        self.state = to;
        Ok(Transition { from, event, to, effect })
    }

    pub fn state(&self) -> AgentConnectionState {
        if self.state != AgentConnectionState::Connected && self.failures >= Attempts::BeforeGivingUp {
            return AgentConnectionState::GaveUp;
        }
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_retry_leads_back_to_connecting() {
        let mut machine = ConnectionMachine::new();
        machine.apply(ConnectionEvent::BeginConnect).unwrap();
        machine.apply(ConnectionEvent::ConnectFailed).unwrap();

        assert!(
            machine.apply(ConnectionEvent::BeginConnect).is_err(),
            "BeginConnect is not the way out of WaitingRetry"
        );
        let t = machine
            .apply(ConnectionEvent::RetryDelayElapsed)
            .expect("the retry timer must move the machine on");
        assert_eq!(t.to, AgentConnectionState::Connecting);
    }

    #[test]
    fn a_lost_connection_can_be_reconnected() {
        let mut machine = ConnectionMachine::new();
        machine.apply(ConnectionEvent::BeginConnect).unwrap();
        machine.apply(ConnectionEvent::ConnectSucceeded).unwrap();

        let t = machine.apply(ConnectionEvent::ConnectionLost).unwrap();
        assert_eq!(t.to, AgentConnectionState::Disconnected);
        assert_eq!(
            machine.apply(ConnectionEvent::BeginConnect).unwrap().to,
            AgentConnectionState::Connecting
        );
    }

    fn fail_once(machine: &mut ConnectionMachine) {
        machine.apply(ConnectionEvent::ConnectFailed).unwrap();
        machine.apply(ConnectionEvent::RetryDelayElapsed).unwrap();
    }

    #[test]
    fn it_gives_up_after_the_budget_and_keeps_trying() {
        let mut machine = ConnectionMachine::new();
        machine.apply(ConnectionEvent::BeginConnect).unwrap();

        for _ in 1..Attempts::BeforeGivingUp {
            fail_once(&mut machine);
            assert_ne!(machine.state(), AgentConnectionState::GaveUp);
        }
        fail_once(&mut machine);
        assert_eq!(machine.state(), AgentConnectionState::GaveUp);

        let t = machine.apply(ConnectionEvent::ConnectFailed).unwrap();
        assert_eq!(t.effect, TransitionEffect::ScheduleRetry, "still trying");
        assert_eq!(machine.state(), AgentConnectionState::GaveUp);

        machine.apply(ConnectionEvent::RetryDelayElapsed).unwrap();
        machine.apply(ConnectionEvent::ConnectSucceeded).unwrap();
        assert_eq!(machine.state(), AgentConnectionState::Connected);
    }

    #[test]
    fn a_lost_connection_gets_a_fresh_budget() {
        let mut machine = ConnectionMachine::new();
        machine.apply(ConnectionEvent::BeginConnect).unwrap();
        for _ in 0..Attempts::BeforeGivingUp {
            fail_once(&mut machine);
        }
        machine.apply(ConnectionEvent::ConnectSucceeded).unwrap();
        machine.apply(ConnectionEvent::ConnectionLost).unwrap();
        machine.apply(ConnectionEvent::BeginConnect).unwrap();

        fail_once(&mut machine);
        assert_eq!(machine.state(), AgentConnectionState::Connecting);
    }

    #[test]
    fn an_attempt_that_used_its_window_is_retried_at_once() {
        let window = Duration::from_secs(3);
        assert_eq!(retry_in(window, Duration::from_secs(3)), Duration::ZERO);
        assert_eq!(retry_in(window, Duration::from_secs(4)), Duration::ZERO);
    }

    #[test]
    fn a_fast_failure_waits_out_the_rest_of_its_window() {
        let window = Duration::from_secs(3);
        assert_eq!(retry_in(window, Duration::from_millis(200)), Duration::from_millis(2800));
    }
}
