//! Serde serialization and deserialization coverage tests.

#![cfg(feature = "serde")]

use serde::de::Deserialize;
use serde::de::value::{BorrowedStrDeserializer, StrDeserializer};
use serde::ser::{self, Serialize};

use fy_i18n_lang_tag::{
    Fixed, LanguageView, NormalTag, NormalTagView, OtherExtensionView, PrivateUseTag,
    PrivateUseTagView, RegionView, ScriptView, TExtensionView, UExtensionView,
};

#[cfg(feature = "alloc")]
use fy_i18n_lang_tag::Grow;

#[derive(Default)]
struct TestSerializer {
    output: String,
}

#[derive(Debug)]
struct SerError;
impl core::fmt::Display for SerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SerError")
    }
}
impl std::error::Error for SerError {}
impl ser::Error for SerError {
    fn custom<T: core::fmt::Display>(_msg: T) -> Self {
        SerError
    }
}

impl ser::Serializer for &mut TestSerializer {
    type Ok = ();
    type Error = SerError;
    type SerializeSeq = ser::Impossible<(), Self::Error>;
    type SerializeTuple = ser::Impossible<(), Self::Error>;
    type SerializeTupleStruct = ser::Impossible<(), Self::Error>;
    type SerializeTupleVariant = ser::Impossible<(), Self::Error>;
    type SerializeMap = ser::Impossible<(), Self::Error>;
    type SerializeStruct = ser::Impossible<(), Self::Error>;
    type SerializeStructVariant = ser::Impossible<(), Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output = v.to_string();
        Ok(())
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.output.push(v);
        Ok(())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(ser::Error::custom("unsupported"))
    }

    fn collect_str<T: ?Sized + core::fmt::Display>(
        self,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        self.output = format!("{}", value);
        Ok(())
    }
}

fn serialize_to_str<T: Serialize>(val: &T) -> String {
    let mut ser = TestSerializer::default();
    val.serialize(&mut ser).unwrap();
    ser.output
}

#[test]
fn test_serde_all_views_and_tags_roundtrip() {
    type DeError = serde::de::value::Error;

    // 1. LanguageView
    let lv = LanguageView::parse("zh").unwrap();
    assert_eq!(serialize_to_str(&lv), "zh");
    let de_lv = LanguageView::deserialize(BorrowedStrDeserializer::<DeError>::new("zh")).unwrap();
    assert_eq!(de_lv, lv);

    // 2. ScriptView
    let sv = ScriptView::parse("Hans").unwrap();
    assert_eq!(serialize_to_str(&sv), "Hans");
    let de_sv = ScriptView::deserialize(BorrowedStrDeserializer::<DeError>::new("Hans")).unwrap();
    assert_eq!(de_sv, sv);

    // 3. RegionView
    let rv = RegionView::parse("CN").unwrap();
    assert_eq!(serialize_to_str(&rv), "CN");
    let de_rv = RegionView::deserialize(BorrowedStrDeserializer::<DeError>::new("CN")).unwrap();
    assert_eq!(de_rv, rv);

    // 4. OtherExtensionView
    let ov = OtherExtensionView::parse("a-foo-bar").unwrap();
    assert_eq!(serialize_to_str(&ov), "a-foo-bar");
    let de_ov =
        OtherExtensionView::deserialize(BorrowedStrDeserializer::<DeError>::new("a-foo-bar"))
            .unwrap();
    assert_eq!(de_ov, ov);

    // 5. UExtensionView & TExtensionView
    let uv = UExtensionView::parse("u-ca-gregory").unwrap();
    assert_eq!(serialize_to_str(&uv), "u-ca-gregory");
    let tv = TExtensionView::parse("t-it-m0-ungegn").unwrap();
    assert_eq!(serialize_to_str(&tv), "t-it-m0-ungegn");

    // 6. PrivateUseTagView & PrivateUseTag
    let puv = PrivateUseTagView::parse("x-my-priv").unwrap();
    assert_eq!(serialize_to_str(&puv), "x-my-priv");
    let de_puv =
        PrivateUseTagView::deserialize(BorrowedStrDeserializer::<DeError>::new("x-my-priv"))
            .unwrap();
    assert_eq!(de_puv, puv);

    let put: PrivateUseTag<Fixed<32>> = PrivateUseTag::parse("x-my-priv").unwrap();
    assert_eq!(serialize_to_str(&put), "x-my-priv");
    let de_put: PrivateUseTag<Fixed<32>> =
        PrivateUseTag::deserialize(StrDeserializer::<DeError>::new("x-my-priv")).unwrap();
    assert_eq!(de_put, put);

    // 7. NormalTagView
    let ntv = NormalTagView::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    assert_eq!(serialize_to_str(&ntv), "zh-Hans-CN-u-ca-gregory");
    let de_ntv = NormalTagView::deserialize(BorrowedStrDeserializer::<DeError>::new(
        "zh-Hans-CN-u-ca-gregory",
    ))
    .unwrap();
    assert_eq!(de_ntv, ntv);

    // 8. NormalTag (Fixed)
    let nt_fixed: NormalTag<Fixed<32>> = NormalTag::parse("zh-Hans-CN-u-ca-gregory").unwrap();
    assert_eq!(serialize_to_str(&nt_fixed), "zh-Hans-CN-u-ca-gregory");
    let de_nt: NormalTag<Fixed<32>> =
        NormalTag::deserialize(StrDeserializer::<DeError>::new("zh-Hans-CN-u-ca-gregory")).unwrap();
    assert_eq!(de_nt, nt_fixed);

    // 9. NormalTag (Grow)
    #[cfg(feature = "alloc")]
    {
        let nt_grow: NormalTag<Grow> =
            NormalTag::parse_with("zh-Hans-CN-u-ca-gregory", Grow::default()).unwrap();
        assert_eq!(serialize_to_str(&nt_grow), "zh-Hans-CN-u-ca-gregory");
        let de_grow: NormalTag<Grow> =
            NormalTag::deserialize(StrDeserializer::<DeError>::new("zh-Hans-CN-u-ca-gregory"))
                .unwrap();
        assert_eq!(de_grow, nt_grow);
    }
}

#[test]
fn test_serde_deserialization_errors() {
    type DeError = serde::de::value::Error;

    // Syntax error deserialization
    assert!(LanguageView::deserialize(BorrowedStrDeserializer::<DeError>::new("123")).is_err());
    assert!(ScriptView::deserialize(BorrowedStrDeserializer::<DeError>::new("Han")).is_err());
    assert!(RegionView::deserialize(BorrowedStrDeserializer::<DeError>::new("C")).is_err());
    assert!(
        OtherExtensionView::deserialize(BorrowedStrDeserializer::<DeError>::new("x-bad")).is_err()
    );
    assert!(
        PrivateUseTagView::deserialize(BorrowedStrDeserializer::<DeError>::new("not-private"))
            .is_err()
    );
    assert!(
        PrivateUseTag::<Fixed<32>>::deserialize(StrDeserializer::<DeError>::new("not-private"))
            .is_err()
    );
    assert!(
        NormalTagView::deserialize(BorrowedStrDeserializer::<DeError>::new("-invalid-")).is_err()
    );
    assert!(
        NormalTag::<Fixed<32>>::deserialize(StrDeserializer::<DeError>::new("-invalid-")).is_err()
    );

    // Borrowed views reject visit_str from non-borrowed data sources
    assert!(LanguageView::deserialize(StrDeserializer::<DeError>::new("zh")).is_err());
    assert!(ScriptView::deserialize(StrDeserializer::<DeError>::new("Hans")).is_err());
    assert!(RegionView::deserialize(StrDeserializer::<DeError>::new("CN")).is_err());
    assert!(OtherExtensionView::deserialize(StrDeserializer::<DeError>::new("a-foo")).is_err());
    assert!(PrivateUseTagView::deserialize(StrDeserializer::<DeError>::new("x-priv")).is_err());
    assert!(NormalTagView::deserialize(StrDeserializer::<DeError>::new("zh-CN")).is_err());

    // Type mismatch triggers Visitor::expecting
    use serde::de::IntoDeserializer;
    fn num_de() -> serde::de::value::I32Deserializer<DeError> {
        123i32.into_deserializer()
    }
    assert!(LanguageView::deserialize(num_de()).is_err());
    assert!(ScriptView::deserialize(num_de()).is_err());
    assert!(RegionView::deserialize(num_de()).is_err());
    assert!(OtherExtensionView::deserialize(num_de()).is_err());
    assert!(PrivateUseTagView::deserialize(num_de()).is_err());
    assert!(PrivateUseTag::<Fixed<32>>::deserialize(num_de()).is_err());
    assert!(NormalTagView::deserialize(num_de()).is_err());
    assert!(NormalTag::<Fixed<32>>::deserialize(num_de()).is_err());
}

#[test]
fn test_u_t_borrowed_deserialization_contract() {
    type E = serde::de::value::Error;
    for input in ["u-ATTR-CA-gregory", "ca-gregory-nu-latn"] {
        let view = UExtensionView::deserialize(BorrowedStrDeserializer::<E>::new(input)).unwrap();
        assert_eq!(view, UExtensionView::parse(input).unwrap());
        let encoded = serialize_to_str(&view);
        let restored =
            UExtensionView::deserialize(BorrowedStrDeserializer::<E>::new(&encoded)).unwrap();
        assert_eq!(restored, view);
    }
    for input in ["t-EN-Latn-US-m0-ungegn", "m0-ungegn-k0-dvorak"] {
        let view = TExtensionView::deserialize(BorrowedStrDeserializer::<E>::new(input)).unwrap();
        assert_eq!(view, TExtensionView::parse(input).unwrap());
        let encoded = serialize_to_str(&view);
        let restored =
            TExtensionView::deserialize(BorrowedStrDeserializer::<E>::new(&encoded)).unwrap();
        assert_eq!(restored, view);
    }
    for invalid in ["u-ca-", "u-ca-gregory-ca-latn", "u-\u{4e2d}"] {
        assert!(UExtensionView::deserialize(BorrowedStrDeserializer::<E>::new(invalid)).is_err());
    }
    for invalid in ["t-m0", "t-m0-ungegn-m0-foo", "t-\u{4e2d}"] {
        assert!(TExtensionView::deserialize(BorrowedStrDeserializer::<E>::new(invalid)).is_err());
    }
    assert!(UExtensionView::deserialize(StrDeserializer::<E>::new("u-ca-gregory")).is_err());
    assert!(TExtensionView::deserialize(StrDeserializer::<E>::new("t-en")).is_err());
}

#[test]
fn test_serde_borrowed_struct_derive() {
    #[derive(serde::Deserialize, PartialEq, Debug)]
    struct BorrowContainer<'a> {
        #[serde(borrow)]
        tag: NormalTagView<'a>,
        #[serde(borrow)]
        priv_tag: PrivateUseTagView<'a>,
        #[serde(borrow)]
        lang: LanguageView<'a>,
        #[serde(borrow)]
        script: ScriptView<'a>,
        #[serde(borrow)]
        region: RegionView<'a>,
        #[serde(borrow)]
        u_ext: UExtensionView<'a>,
        #[serde(borrow)]
        t_ext: TExtensionView<'a>,
        #[serde(borrow)]
        other_ext: OtherExtensionView<'a>,
    }

    // Verifies derive usage under 'de: 'a lifetime bounds
    fn assert_deserializable_borrow<'de: 'a, 'a, T: serde::Deserialize<'de>>() {}
    assert_deserializable_borrow::<BorrowContainer<'_>>();
    assert_deserializable_borrow::<NormalTagView<'_>>();
    assert_deserializable_borrow::<PrivateUseTagView<'_>>();
    assert_deserializable_borrow::<LanguageView<'_>>();
    assert_deserializable_borrow::<ScriptView<'_>>();
    assert_deserializable_borrow::<RegionView<'_>>();
    assert_deserializable_borrow::<UExtensionView<'_>>();
    assert_deserializable_borrow::<TExtensionView<'_>>();
    assert_deserializable_borrow::<OtherExtensionView<'_>>();
}
