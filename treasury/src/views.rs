use crate::*;

#[near_bindgen]
impl Contract {
    /// Returns semver of this contract.
    ///
    /// ```bash
    /// near view treasury.testnet version
    /// ```
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
