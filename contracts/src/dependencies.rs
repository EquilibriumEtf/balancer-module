use abstract_sdk::os::EXCHANGE;
use abstract_sdk::os::objects::dependency::StaticDependency;

const DEX_DEP: StaticDependency = StaticDependency::new(EXCHANGE, &[">=0.3.0"]);

/// Dependencies for the app
pub const BALANCER_DEPS: &[StaticDependency] = &[DEX_DEP];

#[cfg(test)]
mod tests {
    use semver::Comparator;
    use super::*;

    #[test]
    fn test_dependencies() {
        BALANCER_DEPS.iter().for_each(|dep| {
            dep.version_req.iter().for_each(|req| {
                Comparator::parse(req).unwrap();
            });
        });
    }
}
