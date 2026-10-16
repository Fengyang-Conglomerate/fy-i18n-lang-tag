//! `serde` serialization and deserialization support.
//!
//! Serializes and deserializes language tags as strings according to BCP 47.
//! Supports `no_std` environments by default.

use core::fmt;
use core::marker::PhantomData;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::normal::{
    LanguageView, NormalTag, NormalTagView, OtherExtensionView, RegionView, ScriptView,
    TExtensionView, UExtensionView,
};
use crate::private_use::{PrivateUseTag, PrivateUseTagView};
use crate::storage::Storage;

// ======================== NormalTagView ========================

impl Serialize for NormalTagView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for NormalTagView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NormalTagViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for NormalTagViewVisitor<'a> {
            type Value = NormalTagView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 normal language tag borrowed string slice")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NormalTagView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "borrowed views only support deserialization from zero-copy borrowed data; for non-borrowed or streaming data, use owned NormalTag",
                ))
            }
        }

        deserializer.deserialize_str(NormalTagViewVisitor(PhantomData))
    }
}

// ======================== NormalTag ========================

impl<S: Storage> Serialize for NormalTag<S> {
    #[inline]
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de, S: Storage> Deserialize<'de> for NormalTag<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NormalTagVisitor<S>(PhantomData<S>);

        impl<'de, S: Storage> Visitor<'de> for NormalTagVisitor<S> {
            type Value = NormalTag<S>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 normal language tag string")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                NormalTag::parse(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(NormalTagVisitor(PhantomData))
    }
}

// ======================== PrivateUseTagView ========================

impl Serialize for PrivateUseTagView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for PrivateUseTagView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PrivateUseTagViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for PrivateUseTagViewVisitor<'a> {
            type Value = PrivateUseTagView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 private-use language tag borrowed string slice")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                PrivateUseTagView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "borrowed views only support deserialization from zero-copy borrowed data; for non-borrowed or streaming data, use owned PrivateUseTag",
                ))
            }
        }

        deserializer.deserialize_str(PrivateUseTagViewVisitor(PhantomData))
    }
}

// ======================== PrivateUseTag ========================

impl<S: Storage> Serialize for PrivateUseTag<S> {
    #[inline]
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de, S: Storage> Deserialize<'de> for PrivateUseTag<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PrivateUseTagVisitor<S>(PhantomData<S>);

        impl<'de, S: Storage> Visitor<'de> for PrivateUseTagVisitor<S> {
            type Value = PrivateUseTag<S>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 private-use language tag string")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                PrivateUseTag::parse(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(PrivateUseTagVisitor(PhantomData))
    }
}

// ======================== Components Views ========================

impl Serialize for LanguageView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for LanguageView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LanguageViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for LanguageViewVisitor<'a> {
            type Value = LanguageView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 primary language subtag borrowed string slice")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                LanguageView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "LanguageView borrowed view only supports deserialization from borrowed data",
                ))
            }
        }

        deserializer.deserialize_str(LanguageViewVisitor(PhantomData))
    }
}

impl Serialize for ScriptView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for ScriptView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ScriptViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for ScriptViewVisitor<'a> {
            type Value = ScriptView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 script subtag borrowed string slice")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                ScriptView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "ScriptView borrowed view only supports deserialization from borrowed data",
                ))
            }
        }

        deserializer.deserialize_str(ScriptViewVisitor(PhantomData))
    }
}

impl Serialize for RegionView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for RegionView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RegionViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for RegionViewVisitor<'a> {
            type Value = RegionView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 region subtag borrowed string slice")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                RegionView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "RegionView borrowed view only supports deserialization from borrowed data",
                ))
            }
        }

        deserializer.deserialize_str(RegionViewVisitor(PhantomData))
    }
}

// ======================== Extension Views ========================

impl Serialize for UExtensionView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl Serialize for TExtensionView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl Serialize for OtherExtensionView<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for OtherExtensionView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OtherExtensionViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for OtherExtensionViewVisitor<'a> {
            type Value = OtherExtensionView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 single-letter extension slice (e.g. a-foo)")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                OtherExtensionView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "OtherExtensionView borrowed view only supports deserialization from borrowed data",
                ))
            }
        }

        deserializer.deserialize_str(OtherExtensionViewVisitor(PhantomData))
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for UExtensionView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UExtensionViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for UExtensionViewVisitor<'a> {
            type Value = UExtensionView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 u extension slice (e.g. u-ca-gregory)")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                UExtensionView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "UExtensionView borrowed view only supports deserialization from borrowed data",
                ))
            }
        }

        deserializer.deserialize_str(UExtensionViewVisitor(PhantomData))
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for TExtensionView<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TExtensionViewVisitor<'a>(PhantomData<&'a ()>);

        impl<'de: 'a, 'a> Visitor<'de> for TExtensionViewVisitor<'a> {
            type Value = TExtensionView<'a>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a BCP 47 t extension slice (e.g. t-it-m0-ungegn)")
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                TExtensionView::parse(v).map_err(E::custom)
            }

            #[inline]
            fn visit_str<E>(self, _v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Err(E::custom(
                    "TExtensionView borrowed view only supports deserialization from borrowed data",
                ))
            }
        }

        deserializer.deserialize_str(TExtensionViewVisitor(PhantomData))
    }
}
