use crate::dependencies::BALANCER_DEPS;
use abstract_app::export_endpoints;
use abstract_app::AppContract;

use cosmwasm_std::Response;

use crate::error::BalancerError;
use crate::handlers::{self};
use crate::msg::{
    BalancerExecuteMsg, BalancerInstantiateMsg, BalancerMigrateMsg, BalancerQueryMsg, BALANCER,
};

// As an app writer, the only changes necessary to this file are with the handlers and API dependencies on the `TEMPLATE_APP` const.
pub type BalancerApp = AppContract<
    BalancerError,
    BalancerInstantiateMsg,
    BalancerExecuteMsg,
    BalancerQueryMsg,
    BalancerMigrateMsg,
>;

pub type BalancerResult<T = Response> = Result<T, BalancerError>;

/// The namespace for the app, like "abstract" -> "abstract:template"
pub const MODULE_NAMESPACE: &str = "equilibrium";
/// The name of the app, excluding the namespace
pub const BALANCER_ID: &str = "balancer";
/// The initial version of the app, which will use the package version if not altered
const MODULE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Used as the foundation for building your app.
/// All entrypoints are executed through this const (`instantiate`, `query`, `execute`, `migrate`)
/// The `dependencies` are Abstract API dependencies in the format: Vec(`namespace:contract_name`)
const APP: BalancerApp = BalancerApp::new(BALANCER, MODULE_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_query(handlers::query_handler)
    .with_execute(handlers::execute_handler)
    .with_migrate(handlers::migrate_handler)
    .with_dependencies(BALANCER_DEPS);

// Export the endpoints for this contract
export_endpoints!(APP, BalancerApp);
