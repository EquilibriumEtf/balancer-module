use abstract_core::app::BaseInstantiateMsg;
use abstract_core::manager::QueryMsgFns;
use abstract_core::objects::{AccountId, AssetEntry, DexAssetPairing};
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::objects::price_source::UncheckedPriceSource;
use abstract_core::proxy;
use abstract_core::proxy::ExecuteMsgFns as ProxyEexecFns;
use abstract_core::version_control::ExecuteMsgFns;
use abstract_dex_adapter::adapter::DexAdapter;
use abstract_dex_adapter::EXCHANGE;
use abstract_dex_adapter::msg::DexInstantiateMsg;
use abstract_interface::{Abstract, AbstractAccount, AccountDetails, AdapterDeployer, AppDeployer};
use abstract_testing::prelude::*;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use cosmwasm_std::Empty;
use wyndex_bundle::{WynDex, WYNDEX};
use abstract_balancer_app::contract::interface::BalancerApp;
use abstract_balancer_app::msg::{BALANCER, BalancerExecuteMsgFns, BalancerInstantiateMsg, BalancerQueryMsgFns, WeightedAsset};
use speculoos::prelude::*;
use abstract_balancer_app::contract::MODULE_VERSION;

type AResult<T = ()> = anyhow::Result<T>;

pub struct BalancerEnv<Env: CwEnv> {
    pub account: AbstractAccount<Env>,
    pub balancer: BalancerApp<Env>,
    pub abstr: Abstract<Env>,
    pub env: Env,
    pub wynd: WynDex,
}

/// Returns an account with the necessary setup
/// Registeres EUR as the base asset
fn setup_new_account<Env: CwEnv>(
    abstr_deployment: &Abstract<Env>,
    namespace: impl ToString,
) -> anyhow::Result<AbstractAccount<Env>> {
    // TODO: might need to move this
    let signing_account = abstr_deployment.account_factory.get_chain().sender();

    // Create a new account to install the app onto
    let account = abstr_deployment
        .account_factory
        .create_new_account(AccountDetails {
            name: "Test".to_string(),
            description: None,
            base_asset: Some(EUR.into()),
            ..Default::default()
        }, GovernanceDetails::Monarchy {
            monarch: signing_account.into_string(),
        }, None)
        .unwrap();

    // claim the namespace so app can be deployed
    abstr_deployment
        .version_control
        .claim_namespace(account.id().unwrap(), namespace.to_string())
        .unwrap();

    // register base asset!
    // account.proxy.call_as(&abstr_deployment.account_factory.get_chain().sender).update_assets(vec![(AssetEntry::from(ISSUE_ASSET), UncheckedPriceSource::None)], vec![]).unwrap();

    Ok(account)
}

fn balancer_init_msg(asset_weights: Vec<(AssetEntry, WeightedAsset)>) -> BalancerInstantiateMsg {
    BalancerInstantiateMsg {
        asset_weights,
        deviation: Decimal::percent(DEFAULT_DEVIATION_PERCENT),
        dex: WYNDEX.to_string(),
    }
}

// const BET_TOKEN_ANS_ID: &str = "testing>bet";
// const BET_TOKEN_DENOM: &str = "factory/xxx/betting";

// fn setup_default_assets<Env: CwEnv>(abstr: &Abstract<Env>) {
//     // register juno as an asset
//     abstr
//         .ans_host
//         .update_asset_addresses(
//             vec![(
//                 BET_TOKEN_ANS_ID.to_string(),
//                 AssetInfoUnchecked::from_str(&format!("native:{}", BET_TOKEN_DENOM)).unwrap(),
//             )],
//             vec![],
//         )
//         .unwrap();
// }

const ADMIN_ACCOUNT_SEQ: u32 = 1;

const DEFAULT_DEVIATION_PERCENT: u64 = 5;

const EQILIRIUM_NAMESPACE: &'static str = "equilibrium";

impl BalancerEnv<Mock> {
    fn setup() -> anyhow::Result<Self> {
        let owner = Addr::unchecked(TEST_ADMIN);

        // create testing environment
        let mock = Mock::new(&owner);


        // With funds
        mock.add_balance(&owner, coins(6_000_000_000, EUR))?;
        mock.add_balance(&owner, coins(6_000_000_000, USD))?;

        let abstr = Abstract::deploy_on(mock.clone(), mock.sender().to_string()).unwrap();
        let wynd = WynDex::deploy_on(mock.clone(), Empty {})?;
        let account = setup_new_account(&abstr, EQILIRIUM_NAMESPACE)?;

        let balancer_app = BalancerApp::new(BALANCER, mock.clone());

        // Deploy dex adapter to the mock
        let dex_adapter = abstract_dex_adapter::interface::DexAdapter::new(EXCHANGE, mock.clone());

        dex_adapter.deploy(
            abstract_dex_adapter::contract::CONTRACT_VERSION.parse()?,
            DexInstantiateMsg {
                swap_fee: Decimal::percent(1),
                recipient_account: 0,
            },
        )?;

        balancer_app.deploy(MODULE_VERSION.parse().unwrap())?;


        account.install_adapter(dex_adapter, &DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        }, None)?;

        Ok(Self {
            env: mock,
            account,
            wynd,
            balancer: balancer_app,
            abstr,
        })
    }

    fn register_assets(&self, assets: Vec<(AssetEntry, UncheckedPriceSource)>) -> AResult<()> {
        // hack not to have to do exec_on_module
        self.account.proxy.call_as(&Addr::unchecked(self.account.manager.address()?)).update_assets(assets, vec![])?;

        Ok(())
    }

    fn setup_balancer(&self, init_msg: Option<BalancerInstantiateMsg>) -> AResult<()> {
        self.account.install_app(
            self.balancer.clone(),
            &init_msg.unwrap_or(balancer_init_msg(vec![(EUR.into(), WeightedAsset {
                weight: 50,
            }), (USD.into(), WeightedAsset {
                weight: 50
            })])),
            None,
        )?;

        let modules = self.account.manager.module_infos(None, None)?;
        self.balancer.clone().set_address(&modules.module_infos.iter().find(|m| m.id == BALANCER).unwrap().address);

        Ok(())
    }

    fn account(&self, seq: u32) -> AResult<AbstractAccount<Mock>> {
        Ok(AbstractAccount::new(
            &self.abstr,
            Some(AccountId::local(seq.into())),
        ))
    }

    fn admin_account(&self) -> AResult<AbstractAccount<Mock>> {
        self.account(ADMIN_ACCOUNT_SEQ)
    }

    fn admin_account_addr(&self) -> AResult<Addr> {
        Ok(self.admin_account()?.manager.address()?)
    }

    fn balancer_account_addr(&self) -> AResult<Addr> {
        Ok(self.account.proxy.address()?)
    }

    fn balancer_account_balance(&self, denom: &str) -> AResult<Uint128> {
        Ok(self.env.query_balance(&self.balancer_account_addr()?, denom)?)
    }

    // fn register_assets(&self, asset_weights: Vec<(AssetEntry, WeightedAsset)>) -> Result<()> {
    //     Ok(self.account.proxy.update_assets(asset_weights.into_iter().map((|(a, _)| a).collect(), vec![])))
    // }
}

#[test]
fn test_init_config() -> AResult {
    let test_env = BalancerEnv::setup()?;
    test_env.setup_balancer(Some(balancer_init_msg(vec![])))?;

    let config = BalancerQueryMsgFns::config(&test_env.balancer)?;

    assert_that!(config.max_deviation).is_equal_to(Decimal::percent(DEFAULT_DEVIATION_PERCENT));

    Ok(())
}

/// Empty rebalances should have no effect even with weights, as the assets will all be zero
#[test]
fn test_empty_rebalance() -> AResult {
    let test_env = BalancerEnv::setup()?;
    test_env.setup_balancer(Some(balancer_init_msg(vec![])))?;

    test_env.balancer.rebalance()?;

    Ok(())
}

#[test]
#[should_panic]
fn test_non_existent_asset() -> () {
    let test_env = BalancerEnv::setup().unwrap();

    test_env.setup_balancer(Some(balancer_init_msg(vec![("fart".into(), WeightedAsset { weight: 5 })]))).unwrap();
}


/// Rebalancing one asset with no assets should have no effect
#[test]
fn test_rebalance_one_asset_with_no_balances() -> AResult {
    let test_env = BalancerEnv::setup()?;

    test_env.register_assets(vec![(USD.into(), UncheckedPriceSource::Pair(DexAssetPairing::new(EUR.into(), USD.into(), WYNDEX)))])?;
    test_env.setup_balancer(Some(balancer_init_msg(vec![(USD.into(), WeightedAsset { weight: 5 })])))?;

    test_env.balancer.rebalance()?;

    Ok(())
}

/// Rebalancing with a few assets and no balances should be fine
#[test]
fn test_rebalance_one_asset_with_few_assets() -> AResult {
    // this is 50/50
    let test_env = BalancerEnv::setup()?;
    test_env.setup_balancer(Some(balancer_init_msg(vec![(USD.into(), WeightedAsset { weight: 5 }), (EUR.into(), WeightedAsset { weight: 5 })])))?;

    test_env.balancer.rebalance()?;

    Ok(())
}


/// Rebalancing with a few assets and no balances should be fine
#[test]
fn test_rebalance_swap_all_to_other() -> AResult {
    // this is 50/50


    let test_env = BalancerEnv::setup()?;
    test_env.register_assets(vec![(USD.into(), UncheckedPriceSource::Pair(DexAssetPairing::new(EUR.into(), USD.into(), WYNDEX)))])?;
    test_env.setup_balancer(Some(balancer_init_msg(vec![(USD.into(), WeightedAsset { weight: 5 }), (EUR.into(), WeightedAsset { weight: 5 })])))?;


    // set the balance of the proxy to 5 USD
    let account_proxy = test_env.account.proxy.address()?;

    let initial_usd_balance = 5_000_000;
    let expected_usd_balance = initial_usd_balance / 2;
    test_env.env.set_balance(&account_proxy, coins(initial_usd_balance, USD))?;

    assert_that!(test_env.balancer_account_balance(USD)?.u128()).is_equal_to(initial_usd_balance);

    test_env.balancer.rebalance()?;

    assert_that!(test_env.balancer_account_balance(USD)?.u128()).is_equal_to(expected_usd_balance);

    Ok(())
}
