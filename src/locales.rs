use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct Locales {
    locales: HashMap<String, HashMap<String, String>>,
}

impl Locales {
    pub fn contains_category<T: AsRef<str>>(&self, category: T) -> bool {
        self.locales.contains_key(category.as_ref())
    }

    pub fn contains_key_in_category<C: AsRef<str>, K: AsRef<str>>(
        &self,
        category: C,
        key: K,
    ) -> bool {
        self.locales
            .get(category.as_ref())
            .is_some_and(|category| category.contains_key(key.as_ref()))
    }

    pub fn contains_key<T: AsRef<str>>(&self, key: T) -> bool {
        self.locales
            .iter()
            .any(|(_, category)| category.contains_key(key.as_ref()))
    }

    pub fn get_category<T: AsRef<str>>(&self, category: T) -> Option<&HashMap<String, String>> {
        self.locales.get(category.as_ref())
    }

    pub fn find_category_by_key<K: AsRef<str>>(&self, key: K) -> Option<&str> {
        self.locales.iter().find_map(|(name, category)| {
            category.contains_key(key.as_ref()).then_some(name.as_str())
        })
    }

    pub fn find_in_categories_by_key<C: AsRef<str>, K: AsRef<str>>(
        &self,
        categories: &[C],
        key: K,
    ) -> Option<&str> {
        let categories = categories.iter().map(AsRef::as_ref).collect::<Vec<_>>();

        self.locales
            .iter()
            .filter(|(name, _)| categories.contains(&name.as_str()))
            .find_map(|(name, category)| {
                category.contains_key(key.as_ref()).then_some(name.as_str())
            })
    }

    pub fn get<C: AsRef<str>, K: AsRef<str>>(&self, category: C, key: K) -> Option<&str> {
        self.locales
            .get(category.as_ref())
            .and_then(|category| category.get(key.as_ref()))
            .map(String::as_str)
    }

    pub fn load_dir<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();

        for file in path.join("en").read_dir().unwrap().filter_map(|file| {
            file.ok().map(|value| value.path()).filter(|value| {
                value.is_file() && value.extension().is_some_and(|ext| ext == "cfg")
            })
        }) {
            self.load(file);
        }
    }

    pub fn load(&mut self, path: PathBuf) {
        println!(
            "[{}] Loading locale at {}",
            "Locales".bright_blue(),
            path.display().bright_green()
        );

        let cfg = fs::read_to_string(path).unwrap();

        let mut current = "default";

        for value in cfg.split('\n') {
            let value = value.trim();

            if value.is_empty() || value.starts_with(';') || value.starts_with('#') {
                continue;
            }

            if let Some(value) = value
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
            {
                current = value;

                if !self.locales.contains_key(value) {
                    self.locales.insert(value.to_string(), HashMap::new());
                }
            } else {
                let mut values = value.splitn(2, '=');

                if let Some((key, value)) = values
                    .next()
                    .and_then(|key| values.next().map(|value| (key, value)))
                {
                    if let Some(current) = self.locales.get_mut(current) {
                        current.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }
}
