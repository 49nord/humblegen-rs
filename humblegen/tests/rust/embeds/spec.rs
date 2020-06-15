#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct EmbeddedStruct {
    #[doc = ""]
    pub foo: String,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub struct MyStruct {
    #[doc = ""]
    pub bar: i32,
    #[doc = ""]
    pub foo: String,
}
#[derive(Debug, Clone, serde :: Deserialize, serde :: Serialize)]
#[doc = ""]
pub enum MyEnum {
    #[doc = ""]
    AnonymousStructVariant {
        #[doc = ""]
        bar: i32,
        #[doc = ""]
        foo: String,
    },
}
