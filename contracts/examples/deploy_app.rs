use abstract_boot::VersionControl;
use boot_core::DaemonOptionsBuilder;
use std::env;
use std::sync::Arc;

use boot_core::{instantiate_daemon_env, networks::juno::UNI_6};
use cosmwasm_std::Addr;

use balancer::boot::BalancerApp;

use balancer::contract::{BALANCER_ID, MODULE_NAMESPACE};
use semver::Version;

// To deploy the app we need to get the memory and then register it
// We can then deploy a test OS that uses that new app

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn deploy_app() -> anyhow::Result<()> {
    let rt = Arc::new(tokio::runtime::Runtime::new()?);
    let options = DaemonOptionsBuilder::default().network(UNI_6).build()?;
    let app_version = Version::parse(APP_VERSION)?;

    // Setup the environment
    let (_sender, chain) = instantiate_daemon_env(&rt, options)?;

    // Load Abstract Version Control
    let version_control_address: String =
        env::var("VERSION_CONTROL_ADDRESS").expect("VERSION_CONTROL_ADDRESS must be set");

    // let version_control = VersionControl::load(chain.clone(), &Addr::unchecked(version_control_address));

    // // Upload and register your module
    // let app_name = format!("{}:{}", MODULE_NAMESPACE, MODULE_NAME);
    // let mut app = BalancerApp::new(&app_name, &chain);
    // version_control.upload_and_register_module(&mut app, &app_version)?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    if let Err(ref err) = deploy_app() {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}
