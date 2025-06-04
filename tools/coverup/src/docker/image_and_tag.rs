#[derive(Debug, Clone)]
pub struct ImageAndTag(String);

impl ImageAndTag {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for ImageAndTag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');

        let Some(name) = parts.next() else {
            return Err(format!("Image name not found in '{}'", s));
        };
        if name.is_empty() {
            return Err("Image name cannot be empty".to_string());
        }

        let Some(tag) = parts.next() else {
            return Err(format!("Image tag not found in '{}'", s));
        };

        if tag.is_empty() {
            return Err("Tag cannot be empty".to_string());
        }

        Ok(ImageAndTag(s.to_string()))
    }
}
