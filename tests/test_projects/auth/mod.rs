use fuel_tx::{Salt, Transaction};
use fuels_abigen_macro::abigen;
use fuels_contract::{contract::Contract, script::Script, parameters::TxParameters};
use fuel_types::ContractId;
use fuel_core::service::{Config};
use fuels_signers::provider::Provider;
use fuels_signers::util::test_helpers::setup_test_provider_and_wallet;
use std::fs::read;


abigen!(AuthContract, "test_artifacts/auth_testing_contract/out/debug/auth_testing_contract-abi.json");
abigen!(AuthCallerContract, "test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-abi.json");

#[tokio::test]
async fn is_external_from_internal() {
    let (auth_instance, _, _, _) = get_contracts().await;
    let result = auth_instance
        .is_caller_external()
        .call()
        .await
        .unwrap();

    assert_eq!(result.value, false);

}

// #[tokio::test]
// #[should_panic]
// async fn is_external_from_external() {
//     let (auth_instance, auth_id, caller_instance, caller_id) = get_contracts().await;

//     let auth_sway_id= authcallercontract_mod::ContractId {
//         value: auth_id.into(),
//     };

//     let result = caller_instance
//         .call_auth_contract(auth_sway_id)
//         .call()
//         .await
//         .unwrap();

//         assert_eq!(result.value, true);
// }

#[tokio::test]
async fn msg_sender_from_sdk() {
    let (auth_instance, auth_id, caller_instance, caller_id) = get_contracts().await;
    let zero_address = authcontract_mod::ContractId {
        value: [0u8; 32]
    };

    let result = auth_instance
        .returns_msg_sender()
        .call()
        .await
        .unwrap();

        // TODO: Fix this, should be returning a `Result`
        assert_eq!(result.value, zero_address);
}

#[tokio::test]
async fn msg_sender_from_contract() {
    let (auth_instance, auth_id, caller_instance, caller_id) = get_contracts().await;

    let caller_sway_id= authcallercontract_mod::ContractId {
        value: caller_id.into(),
    };

    let auth_sway_id= authcallercontract_mod::ContractId {
        value: auth_id.into(),
    };

    let result = caller_instance
        .call_auth_contract(auth_sway_id)
        .call()
        .await
        .unwrap();

        assert_eq!(result.value, caller_sway_id);
}

#[tokio::test]
async fn msg_sender_from_script() {
    // let expected_receipt = Receipt::Return {
    //     id: ContractId::new([0u8; 32]),
    //     val: 0,
    //     pc: receipts[0].pc().unwrap(),
    //     is: 464,
    // };
    let path_to_bin = "test_artifacts/auth_caller_script/out/debug/auth_caller_script.bin";
    let return_val = ez_script(path_to_bin).await;
    assert_eq!(0, return_val);
}

async fn get_contracts() -> (
    AuthContract,
    ContractId,
    AuthCallerContract,
    ContractId,
) {
    let salt = Salt::from([0u8; 32]);
    let (provider, wallet) = setup_test_provider_and_wallet().await;
    let compiled_1 =
        Contract::load_sway_contract("test_artifacts/auth_testing_contract/out/debug/auth_testing_contract.bin", salt).unwrap();
    let compiled_2 = Contract::load_sway_contract("test_artifacts/auth_caller_contract/out/debug/auth_caller_contract-abi.bin",
        salt,
    )
    .unwrap();

    let id_1 = Contract::deploy(&compiled_1, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();
    let id_2 = Contract::deploy(&compiled_2, &provider, &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance_1 = AuthContract::new(id_1.to_string(), provider.clone(), wallet.clone());
    let instance_2 = AuthCallerContract::new(id_2.to_string(), provider.clone(), wallet.clone());

    (instance_1, id_1, instance_2, id_2)
}

async fn ez_script(bin_path: &str) -> u64 {
    let bin = read(bin_path);
    let client = Provider::launch(Config::local_node()).await.unwrap();

    let tx = Transaction::Script {
        gas_price: 0,
        gas_limit: 1_000_000,
        maturity: 0,
        byte_price: 0,
        receipts_root: Default::default(),
        script: bin.unwrap(), // Here we pass the compiled script into the transaction
        script_data: vec![],
        inputs: vec![],
        outputs: vec![],
        witnesses: vec![vec![].into()],
        metadata: None,
    };

    let script = Script::new(tx);
    let receipts = script.call(&client).await.unwrap();

    receipts[0].val().unwrap()
}
