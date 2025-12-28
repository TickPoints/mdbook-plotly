use crate::preprocessor::config::PlotyInputType;

pub fn handle(raw_code: String, input_type: &PlotyInputType) -> String {
    match input_type {
        PlotyInputType::SandBoxScript => raw_code,
    }
}
