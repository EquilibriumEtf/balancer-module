pub mod contract;
mod dependencies;
pub mod error;
mod handlers;
pub mod msg;
mod state;

// TODO; FIX
// #[cfg(test)]
// #[cfg(not(target_arch = "wasm32"))]
// mod tests;

#[cfg(feature = "boot")]
pub mod boot {
    use crate::contract::BALANCER_ID;
    use crate::msg::{
        BalancerExecuteMsg, BalancerInstantiateMsg, BalancerMigrateMsg, BalancerQueryMsg,
    };
    use boot_core::{Contract, ContractWrapper, CwEnv};

    /// Contract wrapper for deploying with BOOT
    #[boot_core::contract(
        BalancerInstantiateMsg,
        BalancerExecuteMsg,
        BalancerQueryMsg,
        BalancerMigrateMsg
    )]
    pub struct BalancerApp;

    impl<Chain: CwEnv> BalancerApp<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(
                Contract::new(name, chain)
                    .with_wasm_path(BALANCER_ID)
                    .with_mock(Box::new(ContractWrapper::new_with_empty(
                        crate::contract::execute,
                        crate::contract::instantiate,
                        crate::contract::query,
                    ))),
            )
        }
    }
}
