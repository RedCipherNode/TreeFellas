use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Drive {
    pub name: String,
    pub path: PathBuf,
}

pub fn drives() -> Vec<Drive> {
    let mut drives = Vec::new();

    for letter in b'A'..=b'Z' {
        let path = format!("{}:\\", letter as char);

        if PathBuf::from(&path).exists() {
            drives.push(Drive {
                name: path.clone(),
                path: PathBuf::from(path),
            });
        }
    }

    drives
}
