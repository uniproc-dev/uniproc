#[allow(clippy::all, non_snake_case, non_camel_case_types, dead_code)]
mod bindings;

use std::cell::Cell;
use std::ffi::c_void;

use app_contracts::features::window::PressedAway;
use guinea::prelude::GlobalEventBus;
use guinea::Mark;
use ui::widgets::selection::SelectionMark;
use windows_core::imp::{ConstBuffer, DelegateBox};
use windows_core::{HRESULT, IInspectable, IUnknown_Vtbl, Interface, Ref, Result, RuntimeType};
use windows_reference::IReference;

use bindings::{
    AutomationProperties, DependencyObject, FocusManager, FocusManagerGotFocusEventArgs,
    PointerRoutedEventArgs, RoutedEventArgs, UIElement, VisualTreeHelper,
};

thread_local! {
    static WATCHING: Cell<bool> = const { Cell::new(false) };
    static ROOT: Cell<usize> = const { Cell::new(0) };
}

pub fn install() {
    if WATCHING.replace(true) {
        return;
    }
    match FocusManager::GotFocus(on_focus) {
        Ok(revoker) => revoker.forget(),
        Err(err) => tracing::warn!(%err, "could not watch focus to find the window root"),
    }
}

fn on_focus(_: Ref<IInspectable>, args: Ref<FocusManagerGotFocusEventArgs>) {
    match listen_at_root(&args) {
        Ok(()) => {}
        Err(err) if err.code().is_ok() => tracing::trace!("the newly focused element has no window root"),
        Err(err) => tracing::warn!(%err, "could not listen to presses on the window root"),
    }
}

fn listen_at_root(args: &Ref<FocusManagerGotFocusEventArgs>) -> Result<()> {
    let Ok(element) = args.ok()?.NewFocusedElement()?.cast::<UIElement>() else {
        return Ok(());
    };
    let root = element.XamlRoot()?.Content()?;
    let identity = root.as_raw() as usize;
    if ROOT.get() == identity {
        return Ok(());
    }
    let handler = IInspectable::from(IReference::from(pressed_handler()));
    root.AddHandler(&UIElement::PointerPressedEvent()?, &handler, true)?;
    ROOT.set(identity);
    tracing::debug!("listening to presses on the window root");
    Ok(())
}

#[repr(C)]
pub struct PressedHandlerVtbl {
    base: IUnknown_Vtbl,
    invoke: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    PressedHandler,
    PressedHandlerVtbl,
    0xa48a71e1_8bb4_5597_9e31_903a3f6a04fb
);

impl RuntimeType for PressedHandler {
    const SIGNATURE: ConstBuffer = ConstBuffer::new()
        .push_slice(b"delegate(")
        .push_other(ConstBuffer::for_interface::<Self>())
        .push_slice(b")");
}

type PressedBox = DelegateBox<PressedHandler, ()>;

const PRESSED: PressedHandlerVtbl = PressedHandlerVtbl {
    base: IUnknown_Vtbl {
        QueryInterface: PressedBox::QueryInterface,
        AddRef: PressedBox::AddRef,
        Release: PressedBox::Release,
    },
    invoke: invoke_pressed,
};

fn pressed_handler() -> PressedHandler {
    let raw = Box::into_raw(Box::new(PressedBox::new(&PRESSED, ())));
    unsafe { PressedHandler::from_raw(raw as *mut c_void) }
}

unsafe extern "system" fn invoke_pressed(
    _this: *mut c_void,
    _sender: *mut c_void,
    args: *mut c_void,
) -> HRESULT {
    let args = unsafe { PointerRoutedEventArgs::from_raw_borrowed(&args) };
    if let Some(args) = args
        && let Err(err) = on_pressed(args)
    {
        tracing::warn!(%err, "could not read a window press");
    }
    HRESULT(0)
}

fn on_pressed(args: &PointerRoutedEventArgs) -> Result<()> {
    let source = args
        .cast::<RoutedEventArgs>()?
        .OriginalSource()?
        .cast::<DependencyObject>()?;
    let away = is_away(automation_ids(source));
    tracing::debug!(away, "window pressed");
    if away {
        GlobalEventBus::publish(PressedAway);
    }
    Ok(())
}

fn automation_ids(from: DependencyObject) -> impl Iterator<Item = String> {
    std::iter::successors(Some(from), |node| VisualTreeHelper::GetParent(node).ok())
        .map(|node| AutomationProperties::GetAutomationId(&node).unwrap_or_default())
}

fn is_away(mut ids: impl Iterator<Item = String>) -> bool {
    let keeper = SelectionMark::Keeper.name();
    !ids.any(|id| id == keeper)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(ids: &[&str]) -> impl Iterator<Item = String> {
        ids.iter().map(|id| id.to_string())
    }

    #[test]
    fn a_press_inside_a_keeper_keeps_the_selection() {
        let keeper = SelectionMark::Keeper.name();
        assert!(!is_away(path(&["", "Row", keeper, "", "Navigation"])));
    }

    #[test]
    fn a_press_with_no_keeper_on_its_way_to_the_root_is_away() {
        assert!(is_away(path(&["", "Row", "", "Navigation"])));
        assert!(is_away(path(&[])));
    }
}
