use std::collections::HashMap;

use typemap_rev::TypeMapKey;

pub struct CommandCounter;

impl TypeMapKey for CommandCounter {
	type Value = HashMap<String, u64>;
}

pub struct UrlCounter;

impl TypeMapKey for UrlCounter {
	type Value = HashMap<String, u64>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
