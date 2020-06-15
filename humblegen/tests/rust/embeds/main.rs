include!("spec.rs");

fn main() {
    let _ = MyEnum::AnonymousStructVariant {
        bar: 23,
        foo: "foo".to_owned(),
    };
    let _ = MyStruct {
        bar: 23,
        foo: "foo".to_owned(),
    };
}
