use ethers::prelude::abigen;

abigen!(
    MonadAdWall,
    "./src/abi/MonadAdWall.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
