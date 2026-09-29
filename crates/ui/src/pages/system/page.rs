use app_contracts::features::system::{GetTool, OpenTool, SystemState, SystemTool, ToolGroup};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::Mark;
use windows_reactor::{KeyedView, LayoutControl, Thickness, View};

use crate::l10n::L10n;
use crate::theme::{space, Palette};
use crate::widgets::page::{action_button, settings_column, settings_section};
use crate::widgets::setting_card::{setting_card, SettingCard, SettingCardSize};
use crate::widgets::text::{caption, subtitle};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToolMark(pub SystemTool);

impl guinea::Mark for ToolMark {
    fn name(&self) -> &'static str {
        match self.0 {
            SystemTool::TaskManager => "Tool.TaskManager",
            SystemTool::ResourceMonitor => "Tool.ResourceMonitor",
            SystemTool::PerformanceMonitor => "Tool.PerformanceMonitor",
            SystemTool::ReliabilityMonitor => "Tool.ReliabilityMonitor",
            SystemTool::SystemInformation => "Tool.SystemInformation",
            SystemTool::DirectXDiagnostic => "Tool.DirectXDiagnostic",
            SystemTool::EventViewer => "Tool.EventViewer",
            SystemTool::Services => "Tool.Services",
            SystemTool::TaskScheduler => "Tool.TaskScheduler",
            SystemTool::DeviceManager => "Tool.DeviceManager",
            SystemTool::DiskManagement => "Tool.DiskManagement",
            SystemTool::ComputerManagement => "Tool.ComputerManagement",
            SystemTool::RegistryEditor => "Tool.RegistryEditor",
            SystemTool::SystemProperties => "Tool.SystemProperties",
            SystemTool::EnvironmentVariables => "Tool.EnvironmentVariables",
            SystemTool::StartupApps => "Tool.StartupApps",
            SystemTool::InstalledApps => "Tool.InstalledApps",
            SystemTool::Storage => "Tool.Storage",
            SystemTool::Power => "Tool.Power",
            SystemTool::WindowsUpdate => "Tool.WindowsUpdate",
            SystemTool::About => "Tool.About",
            SystemTool::ProcessExplorer => "Tool.ProcessExplorer",
            SystemTool::ProcessMonitor => "Tool.ProcessMonitor",
            SystemTool::Autoruns => "Tool.Autoruns",
            SystemTool::TcpView => "Tool.TcpView",
            SystemTool::RamMap => "Tool.RamMap",
            SystemTool::VmMap => "Tool.VmMap",
        }
    }
}

fn tool_icon(tool: SystemTool) -> View {
    let icon = match tool {
        SystemTool::TaskManager => icon!(tool_task_manager),
        SystemTool::ResourceMonitor => icon!(tool_resource_monitor),
        SystemTool::PerformanceMonitor => icon!(tool_performance_monitor),
        SystemTool::ReliabilityMonitor => icon!(tool_reliability_monitor),
        SystemTool::SystemInformation => icon!(tool_system_information),
        SystemTool::DirectXDiagnostic => icon!(tool_directx_diagnostic),
        SystemTool::EventViewer => icon!(tool_event_viewer),
        SystemTool::Services => icon!(tool_services),
        SystemTool::TaskScheduler => icon!(tool_task_scheduler),
        SystemTool::DeviceManager => icon!(tool_device_manager),
        SystemTool::DiskManagement => icon!(tool_disk_management),
        SystemTool::ComputerManagement => icon!(tool_computer_management),
        SystemTool::RegistryEditor => icon!(tool_registry_editor),
        SystemTool::SystemProperties => icon!(tool_system_properties),
        SystemTool::EnvironmentVariables => icon!(tool_environment_variables),
        SystemTool::StartupApps => icon!(tool_startup_apps),
        SystemTool::InstalledApps => icon!(tool_installed_apps),
        SystemTool::Storage => icon!(tool_storage),
        SystemTool::Power => icon!(tool_power),
        SystemTool::WindowsUpdate => icon!(tool_windows_update),
        SystemTool::About => icon!(tool_about),
        SystemTool::ProcessExplorer => icon!(tool_process_explorer),
        SystemTool::ProcessMonitor => icon!(tool_process_monitor),
        SystemTool::Autoruns => icon!(tool_autoruns),
        SystemTool::TcpView => icon!(tool_tcpview),
        SystemTool::RamMap => icon!(tool_rammap),
        SystemTool::VmMap => icon!(tool_vmmap),
    };
    icon.size(SettingCardSize::Icon).build()
}

fn tool_words(l10n: &L10n, tool: SystemTool) -> (String, String) {
    match tool {
        SystemTool::TaskManager => (l10n.system_task_manager(), l10n.system_task_manager_description()),
        SystemTool::ResourceMonitor => (l10n.system_resource_monitor(), l10n.system_resource_monitor_description()),
        SystemTool::PerformanceMonitor => {
            (l10n.system_performance_monitor(), l10n.system_performance_monitor_description())
        }
        SystemTool::ReliabilityMonitor => {
            (l10n.system_reliability_monitor(), l10n.system_reliability_monitor_description())
        }
        SystemTool::SystemInformation => {
            (l10n.system_system_information(), l10n.system_system_information_description())
        }
        SystemTool::DirectXDiagnostic => {
            (l10n.system_directx_diagnostic(), l10n.system_directx_diagnostic_description())
        }
        SystemTool::EventViewer => (l10n.system_event_viewer(), l10n.system_event_viewer_description()),
        SystemTool::Services => (l10n.system_services(), l10n.system_services_description()),
        SystemTool::TaskScheduler => (l10n.system_task_scheduler(), l10n.system_task_scheduler_description()),
        SystemTool::DeviceManager => (l10n.system_device_manager(), l10n.system_device_manager_description()),
        SystemTool::DiskManagement => (l10n.system_disk_management(), l10n.system_disk_management_description()),
        SystemTool::ComputerManagement => {
            (l10n.system_computer_management(), l10n.system_computer_management_description())
        }
        SystemTool::RegistryEditor => (l10n.system_registry_editor(), l10n.system_registry_editor_description()),
        SystemTool::SystemProperties => {
            (l10n.system_system_properties(), l10n.system_system_properties_description())
        }
        SystemTool::EnvironmentVariables => {
            (l10n.system_environment_variables(), l10n.system_environment_variables_description())
        }
        SystemTool::StartupApps => (l10n.system_startup_apps(), l10n.system_startup_apps_description()),
        SystemTool::InstalledApps => (l10n.system_installed_apps(), l10n.system_installed_apps_description()),
        SystemTool::Storage => (l10n.system_storage(), l10n.system_storage_description()),
        SystemTool::Power => (l10n.system_power(), l10n.system_power_description()),
        SystemTool::WindowsUpdate => (l10n.system_windows_update(), l10n.system_windows_update_description()),
        SystemTool::About => (l10n.system_about(), l10n.system_about_description()),
        SystemTool::ProcessExplorer => (l10n.system_process_explorer(), l10n.system_process_explorer_description()),
        SystemTool::ProcessMonitor => (l10n.system_process_monitor(), l10n.system_process_monitor_description()),
        SystemTool::Autoruns => (l10n.system_autoruns(), l10n.system_autoruns_description()),
        SystemTool::TcpView => (l10n.system_tcpview(), l10n.system_tcpview_description()),
        SystemTool::RamMap => (l10n.system_rammap(), l10n.system_rammap_description()),
        SystemTool::VmMap => (l10n.system_vmmap(), l10n.system_vmmap_description()),
    }
}

fn group_title(l10n: &L10n, group: ToolGroup) -> String {
    match group {
        ToolGroup::Windows => l10n.system_section_windows(),
        ToolGroup::Settings => l10n.system_section_settings(),
        ToolGroup::Sysinternals => l10n.system_section_sysinternals(),
    }
}

fn tool_control(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, tool: SystemTool) -> View {
    let dispatch = dispatch.clone();
    match state.found(tool) {
        Some(true) => action_button(
            ToolMark(tool),
            l10n.system_open(),
            Some(icon!(open).size(SettingCardSize::Icon).build()),
            true,
            move || dispatch.emit(OpenTool(tool)),
        ),
        Some(false) => action_button(
            ToolMark(tool),
            l10n.system_get(),
            Some(icon!(download_regular).size(SettingCardSize::Icon).build()),
            true,
            move || dispatch.emit(GetTool(tool)),
        ),
        None => View::empty(),
    }
}

fn tool_card(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, palette: Palette, tool: SystemTool) -> View {
    let (title, description) = tool_words(l10n, tool);
    setting_card(
        SettingCard {
            icon: Some(tool_icon(tool)),
            title,
            description,
            control: tool_control(state, dispatch, l10n, tool),
        },
        palette,
    )
}

fn group_section(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, palette: Palette, group: ToolGroup) -> View {
    let hint = match group {
        ToolGroup::Sysinternals => caption(l10n.system_sysinternals_hint())
            .foreground(palette.secondary_text)
            .margin(Thickness::new(1.0, 0.0, 0.0, space::Control))
            .into(),
        _ => View::empty(),
    };
    let cards = SystemTool::in_group(group)
        .map(|tool| KeyedView::new(ToolMark(tool).name(), tool_card(state, dispatch, l10n, palette, tool)));
    settings_section(group_title(l10n, group), (hint, View::keyed_fragment(cards)))
}

pub fn system_view(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, palette: Palette) -> View {
    let [windows, settings, sysinternals] =
        ToolGroup::ALL.map(|group| group_section(state, dispatch, l10n, palette, group));
    settings_column((subtitle(l10n.system_title()), windows, settings, sysinternals))
}
