use app_contracts::features::system::{ForgetTool, GetTool, OpenTool, PinTool, SystemState, SystemTool, ToolGroup};
use guicons::icon;
use guinea::prelude::Dispatch;
use guinea::winui::MarkExt;
use windows_reactor::{
    Button, ButtonStyle, Grid, Orientation, ResourceOverrides, StackPanel, Thickness, TooltipExt, View,
};

use super::super::marks::{SystemMark, ToolMark};
use crate::l10n::L10n;
use crate::theme::{setting, size, space, Palette};
use crate::widgets::link_card::{link_card, LinkCard, Trailing};

struct CardButton;

#[expect(non_upper_case_globals)]
impl CardButton {
    const Padding: f64 = 6.0;
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
    icon.size(setting::Icon).build()
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

pub(in crate::pages::system) fn group_title(l10n: &L10n, group: ToolGroup) -> String {
    match group {
        ToolGroup::Windows => l10n.system_section_windows(),
        ToolGroup::Settings => l10n.system_section_settings(),
        ToolGroup::Sysinternals => l10n.system_section_sysinternals(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::pages::system) enum CardPlace {
    Favourites,
    Catalog,
}

fn card_button(mark: SystemMark, glyph: View, hint: String, on_click: impl Fn() + 'static) -> View {
    Button::new()
        .mark(mark)
        .style(ButtonStyle::Subtle)
        .resource_overrides(ResourceOverrides::new().set("ButtonPadding", Thickness::uniform(CardButton::Padding)))
        .on_click(on_click)
        .content(glyph)
        .tooltip(hint)
}

fn pin_button(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, tool: SystemTool) -> View {
    let pinned = state.is_pinned(tool);
    let dispatch = dispatch.clone();
    let (glyph, hint) = if pinned {
        (icon!(pin_filled), l10n.system_unpin())
    } else {
        (icon!(pin), l10n.system_pin())
    };
    card_button(SystemMark::Pin, glyph.size(size::Icon).build(), hint, move || {
        dispatch.emit(PinTool(tool, !pinned))
    })
}

fn forget_button(dispatch: &Dispatch, l10n: &L10n, tool: SystemTool) -> View {
    let dispatch = dispatch.clone();
    card_button(
        SystemMark::Forget,
        icon!(dismiss).size(size::Icon).build(),
        l10n.system_forget(),
        move || dispatch.emit(ForgetTool(tool)),
    )
}

fn accessory(state: &SystemState, dispatch: &Dispatch, l10n: &L10n, tool: SystemTool, place: CardPlace) -> View {
    let mut buttons = vec![pin_button(state, dispatch, l10n, tool)];
    if place == CardPlace::Favourites {
        buttons.push(forget_button(dispatch, l10n, tool));
    }
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(space::Compact)
        .children(buttons)
        .into()
}

pub(in crate::pages::system) fn tool_card(
    state: &SystemState,
    dispatch: &Dispatch,
    l10n: &L10n,
    palette: Palette,
    tool: SystemTool,
    place: CardPlace,
) -> View {
    let (title, description) = tool_words(l10n, tool);
    let missing = state.found(tool) == Some(false);
    let open = dispatch.clone();
    let card = link_card(
        SystemMark::Open,
        LinkCard {
            icon: Some(tool_icon(tool)),
            title,
            description: Some(description),
            trailing: if missing { Trailing::Download } else { Trailing::External },
            accessory: (!missing).then(|| accessory(state, dispatch, l10n, tool, place)),
        },
        palette,
        move || {
            if missing {
                open.emit(GetTool(tool));
            } else {
                open.emit(OpenTool(tool));
            }
        },
    );
    Grid::new().mark(ToolMark(tool)).children((card,)).into()
}
