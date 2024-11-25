use std::path::Path;

use super::App;

#[derive(Debug, Default)]
pub struct PreRenderConfig {
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
}

impl PreRenderConfig {}

pub fn pre_render(app: &App, destination_dir: impl AsRef<Path>, config: PreRenderConfig) {
    todo!()
}
