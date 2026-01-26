pub trait AxProvider {
    fn bundle_ids(&self) -> Vec<String>;
}

#[derive(Default)]
pub struct MockAx {
    bundles: Vec<String>,
}

impl MockAx {
    pub fn with_bundle(bundle_id: &str) -> Self {
        Self {
            bundles: vec![bundle_id.to_string()],
        }
    }

    pub fn add_bundle(&mut self, bundle_id: &str) {
        self.bundles.push(bundle_id.to_string());
    }
}

impl AxProvider for MockAx {
    fn bundle_ids(&self) -> Vec<String> {
        self.bundles.clone()
    }
}

pub fn find_wechat_app(provider: &dyn AxProvider) -> Option<String> {
    provider
        .bundle_ids()
        .into_iter()
        .find(|bundle| bundle == "com.tencent.xinWeChat")
}
