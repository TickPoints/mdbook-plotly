use crate::preprocessor::config::PlotlyInputType;

pub fn handle(raw_code: String, input_type: &PlotlyInputType) -> String {
    match input_type {
        PlotlyInputType::SandBoxScript => raw_code,
    }
}
