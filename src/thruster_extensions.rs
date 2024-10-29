pub(crate) trait TestResponseExt {
    fn json<T: serde::de::DeserializeOwned>(&self) -> T;
}

impl TestResponseExt for thruster::testing::TestResponse {
    fn json<T: serde::de::DeserializeOwned>(&self) -> T {
        serde_json::from_slice(&self.body).expect("Could not deserialize test response correctly")
    }
}
