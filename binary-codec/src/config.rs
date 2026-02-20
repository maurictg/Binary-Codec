use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct SerializerConfig<T = ()>
where
    T: Clone,
{
    toggle_keys: HashMap<String, bool>,
    length_keys: HashMap<String, usize>,
    variant_keys: HashMap<String, u8>,
    multi_disc_config: HashMap<String, HashMap<u8, String>>,
    multi_disc_list: HashMap<String, Vec<u8>>,
    pub discriminator: Option<u8>,
    pub data: Option<T>,
}

impl Default for SerializerConfig {
    fn default() -> Self {
        Self::new(None::<()>)
    }
}

impl<T: Clone> SerializerConfig<T> {
    pub fn new(data: Option<T>) -> Self {
        Self {
            toggle_keys: HashMap::new(),
            length_keys: HashMap::new(),
            variant_keys: HashMap::new(),
            multi_disc_config: HashMap::new(),
            multi_disc_list: HashMap::new(),
            discriminator: None,
            data,
        }
    }

    pub fn configure_multi_disc(&mut self, enum_name: &str, disc: u8, multi_by: &str) {
        let entry = self
            .multi_disc_config
            .entry(enum_name.to_string())
            .or_insert_with(HashMap::new);

        entry.insert(disc, multi_by.to_string());
    }

    pub fn get_multi_disc_size(&self, enum_name: &str) -> usize {
        self.get_toggled_multi_discs(enum_name).len()
    }

    pub fn get_next_multi_disc(&mut self, field: &str, enum_name: &str) -> Option<u8> {
        let discs = self.get_toggled_multi_discs(enum_name);
        let entry = self
            .multi_disc_list
            .entry(format!("{}.{}", field, enum_name))
            .or_insert(discs);

        let res = entry.pop();

        // Cleanup empty entries to prevent issues with multiple calls to get_next_multi_disc for the same field and enum_name
        if entry.is_empty() {
            self.multi_disc_list.remove(&format!("{}.{}", field, enum_name));
        }

        res
    }

    fn get_toggled_multi_discs(&self, enum_name: &str) -> Vec<u8> {
        let mut discs = Vec::new();
        if let Some(disc_map) = self.multi_disc_config.get(enum_name) {
            for (disc, toggle) in disc_map.iter() {
                if self.get_toggle(toggle).unwrap_or(false) {
                    discs.push(*disc);
                }
            }
        }
        discs.sort();
        discs.reverse();
        discs
    }

    pub fn set_toggle(&mut self, key: &str, value: bool) {
        self.toggle_keys.insert(key.to_string(), value);
    }

    pub fn set_length(&mut self, key: &str, value: usize) {
        self.length_keys.insert(key.to_string(), value);
    }

    pub fn set_variant(&mut self, key: &str, value: u8) {
        self.variant_keys.insert(key.to_string(), value);
    }

     /// Variant toggle setting: key_name=1|2|3|4
    pub fn get_variant_toggle(&mut self, setting: &str) -> Option<bool> {
        let mut parts = setting.split("=");
        let key = parts.next().expect("key=discriminators");
        let discs: Vec<u8> = parts.next().expect("key=discriminators").split("|")
            .map(|v| v.parse().expect("a valid u8")).collect();

        let variant = self.get_variant(key)?;
        Some(discs.contains(&variant))
    }

    pub fn get_toggle(&self, key: &str) -> Option<bool> {
        if key.starts_with('!') {
            let key = &key[1..];
            return self.toggle_keys.get(key).map(|v| !*v);
        }

        self.toggle_keys.get(key).copied()
    }

    pub fn get_length(&self, key: &str) -> Option<usize> {
        self.length_keys.get(key).copied()
    }

    pub fn get_variant(&self, key: &str) -> Option<u8> {
        self.variant_keys
            .get(key)
            .copied()
            .or_else(|| self.toggle_keys.get(key).map(|t| if *t { 1 } else { 0 }))
    }

    pub fn reset(&mut self) {
        self.discriminator = None;
    }
}
