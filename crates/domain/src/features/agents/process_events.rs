use std::sync::Arc;

use app_contracts::features::agents::{
    ProcessCame, ProcessEvent, ProcessInstance, ProcessWent, ScheduledTask, WindowsProcessEvents,
};
use uniproc_windows_agent::api::{ProcessEvent as Told, ProcessEventBatch, ProcessEventKind};

fn task(path: &str) -> Option<ScheduledTask> {
    (!path.is_empty()).then(|| ScheduledTask {
        name: path.rsplit('\\').next().unwrap_or(path).into(),
        path: path.into(),
    })
}

fn event(told: &Told) -> ProcessEvent {
    let instance = ProcessInstance {
        pid: told.pid,
        sequence: told.sequence_number,
    };
    match &told.kind {
        ProcessEventKind::Started(started) => ProcessEvent::Came(ProcessCame {
            instance,
            parent: ProcessInstance {
                pid: started.parent_pid,
                sequence: started.parent_sequence_number,
            },
            at: told.time,
            image_path: started.image_path.as_str().into(),
            command_line: started.command_line.as_str().into(),
            working_dir: started.working_directory.as_str().into(),
            user: started.user.as_str().into(),
            session_id: started.session_id,
            elevated: started.elevated,
            scheduled_task: task(&started.scheduled_task),
            parent_services: started.parent_services.iter().map(|name| Arc::from(name.as_str())).collect(),
        }),
        ProcessEventKind::Exited(exited) => ProcessEvent::Went(ProcessWent {
            instance,
            at: told.time,
            exit_code: exited.exit_code,
            cpu_cycles: exited.cpu_cycles,
            io_read_ops: exited.io_read_ops,
            io_write_ops: exited.io_write_ops,
            io_read_bytes: exited.io_read_bytes,
            io_write_bytes: exited.io_write_bytes,
            peak_commit_bytes: exited.peak_commit,
            handles: exited.handles,
            hard_faults: exited.hard_faults,
            image_path: exited.image_path.as_str().into(),
            image_name: exited.image_name.as_str().into(),
            started_at: (exited.start_time != 0).then_some(exited.start_time),
        }),
    }
}

pub fn batch(batch: ProcessEventBatch) -> WindowsProcessEvents {
    WindowsProcessEvents {
        history_from: (batch.history_from != 0).then_some(batch.history_from),
        events: batch.events.iter().map(|told| event(told)).collect(),
        lost: batch.lost,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::agents::{ProcessEvent, ProcessInstance, ScheduledTask};
    use uniproc_windows_agent::api::{
        ProcessEvent as Told, ProcessEventKind, ProcessExited, ProcessStarted,
    };

    use super::*;

    fn told(pid: u32, time: u64, kind: ProcessEventKind) -> Arc<Told> {
        Arc::new(Told {
            pid,
            sequence_number: 40,
            time,
            kind,
        })
    }

    #[test]
    fn a_start_and_an_exit_come_over_with_their_instance() {
        let started = ProcessStarted {
            parent_pid: 4,
            parent_sequence_number: 3,
            session_id: 1,
            image_path: r"C:\Windows\System32\cmd.exe".into(),
            command_line: "cmd /d /c exit 7".into(),
            user: r"PC\ignat".into(),
            elevated: Some(false),
            working_directory: r"C:\Users\ignat".into(),
            scheduled_task: r"\Vendor\Updater Daily".into(),
            parent_services: vec!["Schedule".into()],
            ..ProcessStarted::default()
        };
        let exited = ProcessExited {
            exit_code: 7,
            cpu_cycles: 900,
            peak_commit: 4096,
            handles: 92,
            ..ProcessExited::default()
        };

        let events = batch(ProcessEventBatch {
            history_from: 0,
            events: vec![
                told(20, 100, ProcessEventKind::Started(started)),
                told(20, 200, ProcessEventKind::Exited(exited)),
            ],
            lost: 2,
        });

        assert_eq!(events.history_from, None);
        assert_eq!(events.lost, 2);
        let [ProcessEvent::Came(came), ProcessEvent::Went(went)] = &*events.events else {
            panic!("a start and an exit: {:#?}", events.events);
        };
        let me = ProcessInstance { pid: 20, sequence: 40 };
        assert_eq!(came.instance, me);
        assert_eq!(came.parent, ProcessInstance { pid: 4, sequence: 3 });
        assert_eq!(came.at, 100);
        assert_eq!(&*came.command_line, "cmd /d /c exit 7");
        assert_eq!(&*came.working_dir, r"C:\Users\ignat");
        assert_eq!(came.elevated, Some(false));
        assert_eq!(
            came.scheduled_task,
            Some(ScheduledTask {
                name: "Updater Daily".into(),
                path: r"\Vendor\Updater Daily".into(),
            })
        );
        assert_eq!(&*came.parent_services, [Arc::from("Schedule")]);
        assert_eq!(went.instance, me);
        assert_eq!((went.at, went.exit_code, went.handles, went.peak_commit_bytes), (200, 7, 92, 4096));
    }

    #[test]
    fn an_exit_carries_its_image_and_when_it_started() {
        let exited = ProcessExited {
            image_path: r"C:\Windows\System32\cmd.exe".into(),
            image_name: "cmd.exe".into(),
            start_time: 100,
            ..ProcessExited::default()
        };

        let events = batch(ProcessEventBatch {
            history_from: 0,
            events: vec![told(20, 200, ProcessEventKind::Exited(exited))],
            lost: 0,
        });

        let [ProcessEvent::Went(went)] = &*events.events else {
            panic!("an exit: {:#?}", events.events);
        };
        assert_eq!(&*went.image_path, r"C:\Windows\System32\cmd.exe");
        assert_eq!(&*went.image_name, "cmd.exe");
        assert_eq!(went.started_at, Some(100));
    }

    #[test]
    fn an_exit_without_a_start_time_has_none() {
        let events = batch(ProcessEventBatch {
            history_from: 0,
            events: vec![told(20, 200, ProcessEventKind::Exited(ProcessExited::default()))],
            lost: 0,
        });

        let [ProcessEvent::Went(went)] = &*events.events else {
            panic!("an exit: {:#?}", events.events);
        };
        assert_eq!(went.started_at, None);
    }

    #[test]
    fn the_first_batch_says_where_history_starts_and_no_task_is_none() {
        let events = batch(ProcessEventBatch {
            history_from: 55,
            events: vec![told(20, 100, ProcessEventKind::Started(ProcessStarted::default()))],
            lost: 0,
        });

        assert_eq!(events.history_from, Some(55));
        let [ProcessEvent::Came(came)] = &*events.events else {
            panic!("a start: {:#?}", events.events);
        };
        assert_eq!(came.scheduled_task, None);
    }
}
