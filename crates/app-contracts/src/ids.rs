macro_rules! ids {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $(#[$variant_meta:meta])* $variant:ident => $id:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $( $(#[$variant_meta])* $variant ),+
        }

        impl $name {
            pub const ALL: [Self; [$(stringify!($variant)),+].len()] = [$(Self::$variant),+];

            pub fn id(self) -> &'static str {
                match self {
                    $(Self::$variant => $id),+
                }
            }

            pub fn from_id(id: &str) -> Option<Self> {
                Self::ALL.into_iter().find(|item| item.id() == id)
            }
        }
    };
}

pub(crate) use ids;
