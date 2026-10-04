#[inline]
pub unsafe fn ClosePackageInfo(packageinforeference: *const _PACKAGE_INFO_REFERENCE) -> i32 {
    windows_core::link!("kernel32.dll" "system" fn ClosePackageInfo(packageinforeference : *const _PACKAGE_INFO_REFERENCE) -> i32);
    unsafe { ClosePackageInfo(packageinforeference) }
}
#[inline]
pub unsafe fn CoInitializeEx(
    pvreserved: Option<*const core::ffi::c_void>,
    dwcoinit: u32,
) -> windows_core::HRESULT {
    windows_core::link!("ole32.dll" "system" fn CoInitializeEx(pvreserved : *const core::ffi::c_void, dwcoinit : u32) -> windows_core::HRESULT);
    unsafe { CoInitializeEx(pvreserved.unwrap_or(core::mem::zeroed()) as _, dwcoinit) }
}
#[inline]
pub unsafe fn DeleteObject(ho: HGDIOBJ) -> windows_core::BOOL {
    windows_core::link!("gdi32.dll" "system" fn DeleteObject(ho : HGDIOBJ) -> windows_core::BOOL);
    unsafe { DeleteObject(ho) }
}
#[inline]
pub unsafe fn DestroyIcon(hicon: HICON) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn DestroyIcon(hicon : HICON) -> windows_core::BOOL);
    unsafe { DestroyIcon(hicon) }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[inline]
pub unsafe fn GetClassLongPtrW(hwnd: HWND, nindex: i32) -> usize {
    windows_core::link!("user32.dll" "system" fn GetClassLongPtrW(hwnd : HWND, nindex : i32) -> usize);
    unsafe { GetClassLongPtrW(hwnd, nindex) }
}
#[inline]
pub unsafe fn GetDC(hwnd: Option<HWND>) -> HDC {
    windows_core::link!("user32.dll" "system" fn GetDC(hwnd : HWND) -> HDC);
    unsafe { GetDC(hwnd.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GetDIBits(
    hdc: HDC,
    hbm: HBITMAP,
    start: u32,
    clines: u32,
    lpvbits: Option<*mut core::ffi::c_void>,
    lpbmi: *mut BITMAPINFO,
    usage: u32,
) -> i32 {
    windows_core::link!("gdi32.dll" "system" fn GetDIBits(hdc : HDC, hbm : HBITMAP, start : u32, clines : u32, lpvbits : *mut core::ffi::c_void, lpbmi : *mut BITMAPINFO, usage : u32) -> i32);
    unsafe {
        GetDIBits(
            hdc,
            hbm,
            start,
            clines,
            lpvbits.unwrap_or(core::mem::zeroed()) as _,
            lpbmi as _,
            usage,
        )
    }
}
#[inline]
pub unsafe fn GetIconInfo(hicon: HICON, piconinfo: *mut ICONINFO) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetIconInfo(hicon : HICON, piconinfo : *mut ICONINFO) -> windows_core::BOOL);
    unsafe { GetIconInfo(hicon, piconinfo as _) }
}
#[inline]
pub unsafe fn GetObjectW(h: HANDLE, c: i32, pv: Option<*mut core::ffi::c_void>) -> i32 {
    windows_core::link!("gdi32.dll" "system" fn GetObjectW(h : HANDLE, c : i32, pv : *mut core::ffi::c_void) -> i32);
    unsafe { GetObjectW(h, c, pv.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GetPackageApplicationIds(
    packageinforeference: *const _PACKAGE_INFO_REFERENCE,
    bufferlength: *mut u32,
    buffer: Option<*mut u8>,
    count: Option<*mut u32>,
) -> i32 {
    windows_core::link!("kernel32.dll" "system" fn GetPackageApplicationIds(packageinforeference : *const _PACKAGE_INFO_REFERENCE, bufferlength : *mut u32, buffer : *mut u8, count : *mut u32) -> i32);
    unsafe {
        GetPackageApplicationIds(
            packageinforeference,
            bufferlength as _,
            buffer.unwrap_or(core::mem::zeroed()) as _,
            count.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn GetPackagesByPackageFamily<P0>(
    packagefamilyname: P0,
    count: *mut u32,
    packagefullnames: Option<*mut windows_core::PWSTR>,
    bufferlength: *mut u32,
    buffer: Option<*mut u16>,
) -> i32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetPackagesByPackageFamily(packagefamilyname : windows_core::PCWSTR, count : *mut u32, packagefullnames : *mut windows_core::PWSTR, bufferlength : *mut u32, buffer : *mut u16) -> i32);
    unsafe {
        GetPackagesByPackageFamily(
            packagefamilyname.param().abi(),
            count as _,
            packagefullnames.unwrap_or(core::mem::zeroed()) as _,
            bufferlength as _,
            buffer.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn OpenPackageInfoByFullName<P0>(
    packagefullname: P0,
    reserved: Option<u32>,
    packageinforeference: *mut PACKAGE_INFO_REFERENCE,
) -> i32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn OpenPackageInfoByFullName(packagefullname : windows_core::PCWSTR, reserved : u32, packageinforeference : *mut PACKAGE_INFO_REFERENCE) -> i32);
    unsafe {
        OpenPackageInfoByFullName(
            packagefullname.param().abi(),
            reserved.unwrap_or(core::mem::zeroed()) as _,
            packageinforeference as _,
        )
    }
}
#[inline]
pub unsafe fn PackageFamilyNameFromFullName<P0>(
    packagefullname: P0,
    packagefamilynamelength: *mut u32,
    packagefamilyname: Option<windows_core::PWSTR>,
) -> i32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn PackageFamilyNameFromFullName(packagefullname : windows_core::PCWSTR, packagefamilynamelength : *mut u32, packagefamilyname : windows_core::PWSTR) -> i32);
    unsafe {
        PackageFamilyNameFromFullName(
            packagefullname.param().abi(),
            packagefamilynamelength as _,
            packagefamilyname.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn PrivateExtractIconsW<P0>(
    szfilename: P0,
    niconindex: i32,
    cxicon: i32,
    cyicon: i32,
    phicon: Option<*mut HICON>,
    piconid: Option<*mut u32>,
    nicons: u32,
    flags: u32,
) -> u32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn PrivateExtractIconsW(szfilename : windows_core::PCWSTR, niconindex : i32, cxicon : i32, cyicon : i32, phicon : *mut HICON, piconid : *mut u32, nicons : u32, flags : u32) -> u32);
    unsafe {
        PrivateExtractIconsW(
            szfilename.param().abi(),
            niconindex,
            cxicon,
            cyicon,
            phicon.unwrap_or(core::mem::zeroed()) as _,
            piconid.unwrap_or(core::mem::zeroed()) as _,
            nicons,
            flags,
        )
    }
}
#[inline]
pub unsafe fn ReleaseDC(hwnd: Option<HWND>, hdc: HDC) -> i32 {
    windows_core::link!("user32.dll" "system" fn ReleaseDC(hwnd : HWND, hdc : HDC) -> i32);
    unsafe { ReleaseDC(hwnd.unwrap_or(core::mem::zeroed()) as _, hdc) }
}
#[inline]
pub unsafe fn SHCreateItemInKnownFolder<P2, T>(
    kfid: *const KNOWNFOLDERID,
    dwkfflags: u32,
    pszitem: P2,
) -> windows_core::Result<T>
where
    P2: windows_core::Param<windows_core::PCWSTR>,
    T: windows_core::Interface,
{
    windows_core::link!("shell32.dll" "system" fn SHCreateItemInKnownFolder(kfid : *const KNOWNFOLDERID, dwkfflags : u32, pszitem : windows_core::PCWSTR, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        SHCreateItemInKnownFolder(
            kfid,
            dwkfflags,
            pszitem.param().abi(),
            &T::IID,
            &mut result__,
        )
        .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHGetFileInfoW<P0>(
    pszpath: P0,
    dwfileattributes: u32,
    psfi: Option<*mut SHFILEINFOW>,
    cbfileinfo: u32,
    uflags: u32,
) -> usize
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("shell32.dll" "system" fn SHGetFileInfoW(pszpath : windows_core::PCWSTR, dwfileattributes : u32, psfi : *mut SHFILEINFOW, cbfileinfo : u32, uflags : u32) -> usize);
    unsafe {
        SHGetFileInfoW(
            pszpath.param().abi(),
            dwfileattributes,
            psfi.unwrap_or(core::mem::zeroed()) as _,
            cbfileinfo,
            uflags,
        )
    }
}
#[inline]
pub unsafe fn SendMessageTimeoutW(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    fuflags: u32,
    utimeout: u32,
    lpdwresult: Option<*mut usize>,
) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn SendMessageTimeoutW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM, fuflags : u32, utimeout : u32, lpdwresult : *mut usize) -> LRESULT);
    unsafe {
        SendMessageTimeoutW(
            hwnd,
            msg,
            wparam,
            lparam,
            fuflags,
            utimeout,
            lpdwresult.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BITMAP {
    pub bmType: i32,
    pub bmWidth: i32,
    pub bmHeight: i32,
    pub bmWidthBytes: i32,
    pub bmPlanes: u16,
    pub bmBitsPixel: u16,
    pub bmBits: *mut core::ffi::c_void,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}
impl Default for BITMAPINFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}
pub const BI_RGB: i32 = 0;
pub type COINIT = i32;
pub const COINIT_APARTMENTTHREADED: COINIT = 2;
pub const DIB_RGB_COLORS: i32 = 0;
pub const ERROR_INSUFFICIENT_BUFFER: i32 = 122;
pub const ERROR_SUCCESS: i32 = 0;
pub const GCLP_HICON: i32 = -14;
pub const GCLP_HICONSM: i32 = -34;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HBITMAP(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HDC(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HGDIOBJ(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HICON(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWND(pub *mut core::ffi::c_void);
windows_core::imp::define_interface!(
    IBindCtx,
    IBindCtx_Vtbl,
    0x0000000e_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IBindCtx, windows_core::IUnknown);
#[repr(C)]
pub struct IBindCtx_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    RegisterObjectBound: usize,
    RevokeObjectBound: usize,
    ReleaseBoundObjects: usize,
    SetBindOptions: usize,
    GetBindOptions: usize,
    GetRunningObjectTable: usize,
    RegisterObjectParam: usize,
    GetObjectParam: usize,
    EnumObjectParam: usize,
    RevokeObjectParam: usize,
}
impl windows_core::RuntimeName for IBindCtx {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ICONINFO {
    pub fIcon: windows_core::BOOL,
    pub xHotspot: u32,
    pub yHotspot: u32,
    pub hbmMask: HBITMAP,
    pub hbmColor: HBITMAP,
}
pub const ICON_BIG: i32 = 1;
pub const ICON_SMALL: i32 = 0;
pub const ICON_SMALL2: i32 = 2;
windows_core::imp::define_interface!(
    IShellItem,
    IShellItem_Vtbl,
    0x43826d1e_e718_42ee_bc55_a1e261c37bfe
);
windows_core::imp::interface_hierarchy!(IShellItem, windows_core::IUnknown);
impl IShellItem {
    pub unsafe fn BindToHandler<P0, T>(
        &self,
        pbc: P0,
        bhid: *const windows_core::GUID,
    ) -> windows_core::Result<T>
    where
        P0: windows_core::Param<IBindCtx>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).BindToHandler)(
                windows_core::Interface::as_raw(self),
                pbc.param().abi(),
                bhid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetParent(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetParent)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub unsafe fn GetDisplayName(
        &self,
        sigdnname: SIGDN,
    ) -> windows_core::Result<windows_core::PWSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDisplayName)(
                windows_core::Interface::as_raw(self),
                sigdnname,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn GetAttributes(&self, sfgaomask: SFGAOF) -> windows_core::Result<SFGAOF> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAttributes)(
                windows_core::Interface::as_raw(self),
                sfgaomask,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub unsafe fn Compare<P0>(&self, psi: P0, hint: SICHINTF) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<Self>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Compare)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
                hint,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IShellItem_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub BindToHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetParent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SIGDN,
        *mut windows_core::PWSTR,
    ) -> windows_core::HRESULT,
    pub GetAttributes: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SFGAOF,
        *mut SFGAOF,
    ) -> windows_core::HRESULT,
    pub Compare: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        SICHINTF,
        *mut i32,
    ) -> windows_core::HRESULT,
}
pub trait IShellItem_Impl: windows_core::IUnknownImpl {
    fn BindToHandler(
        &self,
        pbc: windows_core::Ref<IBindCtx>,
        bhid: *const windows_core::GUID,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetParent(&self) -> windows_core::Result<IShellItem>;
    fn GetDisplayName(&self, sigdnname: SIGDN) -> windows_core::Result<windows_core::PWSTR>;
    fn GetAttributes(&self, sfgaomask: SFGAOF) -> windows_core::Result<SFGAOF>;
    fn Compare(
        &self,
        psi: windows_core::Ref<IShellItem>,
        hint: SICHINTF,
    ) -> windows_core::Result<i32>;
}
impl IShellItem_Vtbl {
    pub const fn new<Identity: IShellItem_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn BindToHandler<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pbc: *mut core::ffi::c_void,
            bhid: *const windows_core::GUID,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellItem_Impl::BindToHandler(
                    this,
                    core::mem::transmute_copy(&pbc),
                    core::mem::transmute_copy(&bhid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetParent<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppsi: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::GetParent(this) {
                    Ok(ok__) => {
                        ppsi.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetDisplayName<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            sigdnname: SIGDN,
            ppszname: *mut windows_core::PWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::GetDisplayName(this, core::mem::transmute_copy(&sigdnname)) {
                    Ok(ok__) => {
                        ppszname.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetAttributes<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            sfgaomask: SFGAOF,
            psfgaoattribs: *mut SFGAOF,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::GetAttributes(this, core::mem::transmute_copy(&sfgaomask)) {
                    Ok(ok__) => {
                        psfgaoattribs.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Compare<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
            hint: SICHINTF,
            piorder: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::Compare(
                    this,
                    core::mem::transmute_copy(&psi),
                    core::mem::transmute_copy(&hint),
                ) {
                    Ok(ok__) => {
                        piorder.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            BindToHandler: BindToHandler::<Identity, OFFSET>,
            GetParent: GetParent::<Identity, OFFSET>,
            GetDisplayName: GetDisplayName::<Identity, OFFSET>,
            GetAttributes: GetAttributes::<Identity, OFFSET>,
            Compare: Compare::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellItem as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellItem {}
windows_core::imp::define_interface!(
    IShellItemImageFactory,
    IShellItemImageFactory_Vtbl,
    0xbcc18b79_ba16_442f_80c4_8a59c30c463b
);
windows_core::imp::interface_hierarchy!(IShellItemImageFactory, windows_core::IUnknown);
impl IShellItemImageFactory {
    pub unsafe fn GetImage(&self, size: SIZE, flags: SIIGBF) -> windows_core::Result<HBITMAP> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetImage)(
                windows_core::Interface::as_raw(self),
                size,
                flags,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IShellItemImageFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetImage: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SIZE,
        SIIGBF,
        *mut HBITMAP,
    ) -> windows_core::HRESULT,
}
pub trait IShellItemImageFactory_Impl: windows_core::IUnknownImpl {
    fn GetImage(&self, size: &SIZE, flags: SIIGBF) -> windows_core::Result<HBITMAP>;
}
impl IShellItemImageFactory_Vtbl {
    pub const fn new<Identity: IShellItemImageFactory_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetImage<
            Identity: IShellItemImageFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            size: SIZE,
            flags: SIIGBF,
            phbm: *mut HBITMAP,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItemImageFactory_Impl::GetImage(
                    this,
                    core::mem::transmute(&size),
                    core::mem::transmute_copy(&flags),
                ) {
                    Ok(ok__) => {
                        phbm.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetImage: GetImage::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellItemImageFactory as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellItemImageFactory {}
pub type KNOWNFOLDERID = windows_core::GUID;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LPARAM(pub isize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LRESULT(pub isize);
pub type PACKAGE_INFO_REFERENCE = *mut _PACKAGE_INFO_REFERENCE;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RGBQUAD {
    pub rgbBlue: u8,
    pub rgbGreen: u8,
    pub rgbRed: u8,
    pub rgbReserved: u8,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SFGAOF(pub u32);
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct SHFILEINFOW {
    pub hIcon: HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}
#[cfg(target_arch = "x86")]
impl Default for SHFILEINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SHFILEINFOW {
    pub hIcon: HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for SHFILEINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const SHGFI_ICON: i32 = 256;
pub const SHGFI_SMALLICON: i32 = 1;
pub type SICHINTF = u32;
pub type SIGDN = i32;
pub type SIIGBF = i32;
pub const SIIGBF_RESIZETOFIT: SIIGBF = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}
pub const SMTO_ABORTIFHUNG: i32 = 2;
pub const SMTO_BLOCK: i32 = 1;
pub const WM_GETICON: i32 = 127;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WPARAM(pub usize);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct _PACKAGE_INFO_REFERENCE {
    pub reserved: *mut core::ffi::c_void,
}
