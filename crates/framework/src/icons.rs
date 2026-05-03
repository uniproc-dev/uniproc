#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IconKey(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IconFamily(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IconVariant(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconRef<'a> {
    Key(IconKey),
    Name(&'a str),
    FamilyVariant {
        family: IconFamily,
        variant: Option<IconVariant>,
    },
    DynamicFamilyVariant {
        family: &'a str,
        variant: Option<&'a str>,
    },
}

impl IconKey {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl IconFamily {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl IconVariant {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl<'a> IconRef<'a> {
    pub const fn family_variant(family: IconFamily, variant: Option<IconVariant>) -> Self {
        Self::FamilyVariant { family, variant }
    }

    pub const fn dynamic_family_variant(family: &'a str, variant: Option<&'a str>) -> Self {
        Self::DynamicFamilyVariant { family, variant }
    }
}

impl<'a> From<IconKey> for IconRef<'a> {
    fn from(value: IconKey) -> Self {
        Self::Key(value)
    }
}

impl<'a> From<&'a str> for IconRef<'a> {
    fn from(value: &'a str) -> Self {
        Self::Name(value)
    }
}
