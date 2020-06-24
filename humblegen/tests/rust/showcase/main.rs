include!("spec.rs");

use std::iter::FromIterator;
use std::str::FromStr;

fn main() {
    let customer = Customer {
        name: "somename".to_owned(),
        id: -23,
        net_worth: 0.123456,
        join_date: ::humblegen_rt::chrono::prelude::Utc::now(),
        birthday: ::humblegen_rt::chrono::prelude::Utc::now()
            .naive_utc()
            .date(),
        is_vip: true,
        favorite_color: Color::Blue,
        aliases: vec!["SomeName", "Some Name"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect(),
        coords: (23, 42),
        email: Some("mail".to_owned()),
        bets: std::collections::HashMap::from_iter(
            vec![("foo", 1.234), ("bar", -0.123)]
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v)),
        ),
        empty: (),
        unique_id: ::humblegen_rt::uuid::Uuid::from_str("db05098d-ecca-478c-8447-cb0a822f9a56").expect("parse uuid"),
        profile_pic: Vec::<u8>::from(r#"raw bytes"#),
    };

    let serialized = serde_json::to_string(&customer).expect("serialize customer");
    println!("serialized:\n{}", serialized);
    let deserialized: Customer = serde_json::from_str(&serialized).expect("deserialize customer");

    assert_eq!(customer.name, deserialized.name);
    assert_eq!(customer.id, deserialized.id);
    assert_eq!(customer.net_worth, deserialized.net_worth);
    assert_eq!(customer.is_vip, deserialized.is_vip);
    assert_eq!(customer.aliases, deserialized.aliases);
    assert_eq!(customer.coords, deserialized.coords);
    assert_eq!(customer.email, deserialized.email);
    assert_eq!(customer.bets, deserialized.bets);
    assert_eq!(customer.unique_id, deserialized.unique_id);
    assert_eq!(customer.profile_pic, deserialized.profile_pic);
}
