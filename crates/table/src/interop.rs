#![allow(non_snake_case)]

use core::ffi::c_void;

use windows_core::{IUnknown, IUnknown_Vtbl, Interface, Result, GUID, HRESULT};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub cx: i32,
    pub cy: i32,
}

windows_core::imp::define_interface!(ICompositorInterop, ICompositorInterop_Vtbl, 0xfab19398_6d19_4d8a_b752_8f096c396069);
windows_core::imp::interface_hierarchy!(ICompositorInterop, IUnknown);

impl ICompositorInterop {
    pub unsafe fn CreateGraphicsDevice(&self, rendering_device: &IUnknown) -> Result<IUnknown> {
        unsafe {
            let mut result = core::ptr::null_mut();
            (Interface::vtable(self).CreateGraphicsDevice)(Interface::as_raw(self), Interface::as_raw(rendering_device), &mut result)
                .and_then(|| windows_core::imp::Type::from_abi(result))
        }
    }
}

#[repr(C)]
pub struct ICompositorInterop_Vtbl {
    pub base__: IUnknown_Vtbl,
    pub CreateGraphicsDevice: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut *mut c_void) -> HRESULT,
}

windows_core::imp::define_interface!(
    ICompositionDrawingSurfaceInterop,
    ICompositionDrawingSurfaceInterop_Vtbl,
    0x2d6355c2_ad57_4eae_92e4_4c3eff65d578
);
windows_core::imp::interface_hierarchy!(ICompositionDrawingSurfaceInterop, IUnknown);

impl ICompositionDrawingSurfaceInterop {
    pub unsafe fn BeginDraw<T: Interface>(&self, update: Option<&Rect>, offset: &mut Point) -> Result<T> {
        unsafe {
            let mut result = core::ptr::null_mut();
            (Interface::vtable(self).BeginDraw)(
                Interface::as_raw(self),
                update.map_or(core::ptr::null(), |rect| rect as *const Rect),
                &T::IID,
                &mut result,
                offset,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result))
        }
    }

    pub unsafe fn EndDraw(&self) -> HRESULT {
        unsafe { (Interface::vtable(self).EndDraw)(Interface::as_raw(self)) }
    }

    pub unsafe fn Resize(&self, size: Size) -> HRESULT {
        unsafe { (Interface::vtable(self).Resize)(Interface::as_raw(self), size) }
    }
}

#[repr(C)]
pub struct ICompositionDrawingSurfaceInterop_Vtbl {
    pub base__: IUnknown_Vtbl,
    pub BeginDraw: unsafe extern "system" fn(*mut c_void, *const Rect, *const GUID, *mut *mut c_void, *mut Point) -> HRESULT,
    pub EndDraw: unsafe extern "system" fn(*mut c_void) -> HRESULT,
    pub Resize: unsafe extern "system" fn(*mut c_void, Size) -> HRESULT,
    Scroll: usize,
    ResumeDraw: usize,
    SuspendDraw: usize,
}
