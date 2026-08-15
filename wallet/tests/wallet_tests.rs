use btclib::crypto::PrivateKey;
use btclib::types::TransactionOutput;
use btclib::util::Saveable;
use tempfile::tempdir;
use tokio::sync::mpsc;
use wallet::core::{Config, Core, FeeConfig, FeeType, Key, Recipient};

#[test]
fn dummy_config_round_trips_as_toml() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("wallet_config.toml");
    Config::dummy().save(&path).unwrap();
    let loaded = Config::load(&path).unwrap();
    assert_eq!(loaded.default_node, "127.0.0.1:9000");
    assert_eq!(loaded.contacts[0].name, "Bob");
}

#[tokio::test]
async fn wallet_creates_transaction_with_change_and_percent_fee() {
    let dir = tempdir().unwrap();
    let owner = PrivateKey::new_key();
    let recipient = PrivateKey::new_key();
    let owner_pub = dir.path().join("owner.pub");
    let owner_priv = dir.path().join("owner.priv");
    let recipient_pub = dir.path().join("recipient.pub");
    owner.public_key().save_to_file(&owner_pub).unwrap();
    owner.save_to_file(&owner_priv).unwrap();
    recipient.public_key().save_to_file(&recipient_pub).unwrap();

    let config = Config {
        my_keys: vec![Key {
            public: owner_pub,
            private: owner_priv,
        }],
        contacts: vec![Recipient {
            name: "Recipient".to_string(),
            key: recipient_pub,
        }],
        default_node: "127.0.0.1:9000".to_string(),
        fee_config: FeeConfig {
            fee_type: FeeType::Percent,
            value: 1.0,
        },
    };
    let (sender, _receiver) = mpsc::unbounded_channel();
    let core = Core::from_config(config, sender).unwrap();
    core.set_utxos_for_test(vec![TransactionOutput::new(1_000, owner.public_key())])
        .await;
    let tx = core
        .create_transaction(&recipient.public_key(), 500)
        .await
        .unwrap();
    assert_eq!(tx.inputs.len(), 1);
    assert_eq!(tx.outputs[0].value, 500);
    assert_eq!(tx.outputs[1].value, 495);
}
