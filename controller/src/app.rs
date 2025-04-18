use std::fs;

#[derive(Clone)]
pub struct AppImpl {
    status_filename: String
}

impl AppImpl {
    pub fn new(filename: &String) -> Self {
        Self {
            status_filename: filename.clone()
        }
    }
    
    pub fn get_status(self) -> String {
        match fs::read_to_string(&self.status_filename) {
            Ok(status) => status,
            _ => "Invalid".to_string()
        }
    }
    
    pub fn activate(self) -> String {
        unimplemented!()
    }
}
