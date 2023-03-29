use abstract_sdk::core::objects::dependency::StaticDependency;
use dex::EXCHANGE;

const DEX_DEP: StaticDependency = StaticDependency::new(EXCHANGE, &[">=0.3.0"]);

/// Dependencies for the app
pub const BALANCER_DEPS: &[StaticDependency] = &[DEX_DEP];

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Comparator;

    #[test]
    fn test_dependencies() {
        BALANCER_DEPS.iter().for_each(|dep| {
            dep.version_req.iter().for_each(|req| {
                Comparator::parse(req).unwrap();
            });
        });
    }
}
