use crate::manager::AssetsManager;

#[derive(Debug, Default)]
pub struct Dependencie {
    pub source: String,
    pub deps: Vec<String>,
    pub missing_list: Vec<String>,
}

impl Dependencie {
    pub fn is_valid(&self) -> bool {
        self.missing_list.is_empty()
    }
}

#[derive(Default, Debug)]
pub struct DependencieManager {
    deps: Vec<Dependencie>,
}

impl DependencieManager {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn load_file(&mut self, assets_manager: &mut AssetsManager, filename: &str) {
        let mut data = assets_manager.get(filename);
        match data {
            None => {
                assets_manager.load(filename).unwrap();
                data = assets_manager.get(filename);
            }
            _ => (),
        }

        let content = String::from_utf8(data.unwrap()).unwrap();
        let content = json::parse(&content).unwrap();

        for (name, value) in content.entries() {
            if name == "dependencies" {
                for (name, value) in value.entries() {
                    let mut dep = Dependencie::default();
                    dep.source = String::from(name);
                    if value.is_array() {
                        for file in value.members() {
                            let filename = file.as_str().unwrap().to_string();
                            dep.deps.push(filename.clone());
                        }
                    }
                    self.deps.push(dep);
                }
                break;
            }
        }
    }

    pub fn update(&mut self, assets_manager: &mut AssetsManager) {
        for dep in self.deps.iter_mut() {
            for file in dep.deps.iter() {
                if !assets_manager.have_file(&file) {
                    dep.missing_list.push(file.clone());
                }
            }
        }
    }

    pub fn check_if_valid(&self, filename: &str) -> bool {
        for dep in self.deps.iter() {
            if dep.source == filename {
                return dep.is_valid();
            }
        }

        true
    }

    pub fn get_missing_dependencies(&self, filename: &str) -> Vec<String> {
        let mut result = Vec::<String>::new();

        for dep in self.deps.iter() {
            if dep.source == filename {
                dep.missing_list.iter().for_each(|file| {
                    result.push(file.clone());
                });
            }
        }

        result
    }
}
