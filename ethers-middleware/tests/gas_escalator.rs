#![cfg(not(target_arch = "wasm32"))]
use std::convert::TryFrom;

use ethers_core::{
    types::{transaction::eip1559, *},
    utils::Anvil,
};
use ethers_middleware::{
    gas_escalator::{Frequency, GasEscalatorMiddleware, GeometricGasPrice},
    SignerMiddleware,
};
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer};
use instant::Duration;
use tokio::time::sleep;

#[tokio::test]
#[tracing_test::traced_test]
async fn gas_escalator_legacy_works() {
    // TODO: show tracing logs in the test

    let anvil = Anvil::new().port(8545u16).block_time(10u64).spawn();
    let chain_id = anvil.chain_id();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    // wrap with signer
    let wallet: LocalWallet = anvil.keys().first().unwrap().clone().into();
    let wallet = wallet.with_chain_id(chain_id);
    let address = wallet.address();
    let provider = SignerMiddleware::new(provider, wallet);

    // wrap with escalator
    // escalate every 2 seconds. We should only see 4-5 escalations in total
    let escalator = GeometricGasPrice::new(1.1, 2u64, Some(2_000_000_000_000u64));
    let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::Duration(300));

    // TODO: get this to work
    // set the gas price to 10 gwei, so we need to escalate twice
    // this works but the tx still goes through regardless of its gas price for some reason
    // reqwest::Client::new()
    //     .post(&format!("{}/", anvil.endpoint()))
    //     .json(&json!({
    //         "jsonrpc": "2.0",
    //         "method": "anvil_setMinGasPrice",
    //         "params": [10_000_000_000u64],
    //         "id": 1
    //     }))
    //     .send()
    //     .await
    //     .unwrap();

    let nonce = provider.get_transaction_count(address, None).await.unwrap();
    // 1 gwei default base fee
    let gas_price = U256::from(1_000_000_000_u64);
    let tx = TransactionRequest::pay(Address::zero(), 1u64)
        .gas_price(gas_price)
        .nonce(nonce)
        .chain_id(chain_id);

    let pending = provider.send_transaction(tx, None).await.expect("could not send");
    let receipt = pending.await;
    sleep(Duration::from_secs(2)).await;
    println!("receipt gas price: , hardcoded_gas_price: {}, receipt: {:?}", gas_price, receipt);
}

#[tokio::test]
#[tracing_test::traced_test]
async fn gas_escalator_1559_works() {
    // TODO: show tracing logs in the test

    let anvil = Anvil::new().port(8545u16).block_time(10u64).spawn();
    let chain_id = anvil.chain_id();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    // wrap with signer
    let wallet: LocalWallet = anvil.keys().first().unwrap().clone().into();
    let wallet = wallet.with_chain_id(chain_id);
    let address = wallet.address();
    let provider = SignerMiddleware::new(provider, wallet);

    // wrap with escalator
    // escalate every 2 seconds. We should only see 4-5 escalations in total
    let escalator = GeometricGasPrice::new(1.1, 2u64, Some(2_000_000_000_000u64));
    let provider = GasEscalatorMiddleware::new(provider, escalator, Frequency::Duration(300));

    // TODO: get this to work
    // set the gas price to 10 gwei, so we need to escalate twice
    // this works but the tx still goes through regardless of its gas price for some reason
    // reqwest::Client::new()
    //     .post(&format!("{}/", anvil.endpoint()))
    //     .json(&json!({
    //         "jsonrpc": "2.0",
    //         "method": "anvil_setMinGasPrice",
    //         "params": [10_000_000_000u64],
    //         "id": 1
    //     }))
    //     .send()
    //     .await
    //     .unwrap();

    let nonce = provider.get_transaction_count(address, None).await.unwrap();
    // 1 gwei default base fee
    let max_fee_per_gas = U256::from(1_000_000_000_u64);
    let max_priority_fee_per_gas = U256::from(500_000_000_u64);
    let tx = eip1559::Eip1559TransactionRequest {
        to: Some(Address::zero().into()),
        value: Some(1u64.into()),
        max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
        max_fee_per_gas: Some(max_fee_per_gas),
        nonce: Some(nonce),
        chain_id: Some(chain_id.into()),
        ..Default::default()
    };

    let pending = provider.send_transaction(tx, None).await.expect("could not send");
    let receipt = pending.await;
    sleep(Duration::from_secs(2)).await;
    println!(
        "receipt gas price: , hardcoded_gas_price: {}, receipt: {:?}",
        max_fee_per_gas, receipt
    );
}
