#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl windows_core::imp::TypeKind for Color {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Color {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.Color;u1;u1;u1;u1)");
}
windows_core::imp::define_interface!(
    IUISettings,
    IUISettings_Vtbl,
    0x85361600_1c63_4627_bcb1_3a89e0bc9c55
);
impl windows_core::RuntimeType for IUISettings {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
#[repr(C)]
pub struct IUISettings_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IUISettings3,
    IUISettings3_Vtbl,
    0x03021be4_5254_4781_8194_5168f7d06d7b
);
impl windows_core::RuntimeType for IUISettings3 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
}
impl IUISettings3 {
    pub fn GetColorValue(&self, desiredcolor: UIColorType) -> windows_core::Result<Color> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetColorValue)(
                windows_core::Interface::as_raw(self),
                desiredcolor,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IUISettings3_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub GetColorValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        UIColorType,
        *mut Color,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UIColorType(pub i32);
impl UIColorType {
    pub const Background: Self = Self(0);
    pub const Foreground: Self = Self(1);
    pub const AccentDark3: Self = Self(2);
    pub const AccentDark2: Self = Self(3);
    pub const AccentDark1: Self = Self(4);
    pub const Accent: Self = Self(5);
    pub const AccentLight1: Self = Self(6);
    pub const AccentLight2: Self = Self(7);
    pub const AccentLight3: Self = Self(8);
    pub const Complement: Self = Self(9);
}
impl windows_core::imp::TypeKind for UIColorType {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for UIColorType {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.UI.ViewManagement.UIColorType;i4)",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UISettings(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    UISettings,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl UISettings {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<
        R,
        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            UISettings,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for UISettings {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IUISettings>();
}
unsafe impl windows_core::Interface for UISettings {
    type Vtable = <IUISettings as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IUISettings as windows_core::Interface>::IID;
}
impl core::ops::Deref for UISettings {
    type Target = IUISettings;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
impl windows_core::RuntimeName for UISettings {
    const NAME: &'static str = "Windows.UI.ViewManagement.UISettings";
}
unsafe impl Send for UISettings {}
unsafe impl Sync for UISettings {}
