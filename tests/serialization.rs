use tangleproof::inclusion_proof::InclusionProof;

#[test]
fn serde_inclusion_proof() {
    let proof_json = r#"{"latestOutputId":"7702ea0f2cd6af3206b894c3f2fe4362b23f0f4828857d31e733103b09db25840000","message":{"networkId":"7712883261355838377","parentMessageIds":["429b7d4a6d9e7de6e6601e3d69af6b04d80bb10b3fcd1605bffc8db4f35e6e33","892adbd6b903f9167fe9fa0a43aa01b641814d88e07e2fc0b0778a2092d79248","a1a9449048c38681a78af588a0786c1c1b91ea3db6698acb4c7cfeab29767c97","b5f1c92995db5818e854d5a8c43ad5dad4b773bef79246b1a31f9b3502a9afd9"],"payload":{"type":0,"essence":{"type":0,"inputs":[{"type":0,"transactionId":"a513e340debe6f7a5007da20a029e68984fbd511781bfd8ec115b0fed70b4c44","transactionOutputIndex":0}],"outputs":[{"type":0,"address":{"type":0,"address":"ab1ffcb1392ae0d6590507c5de2e90ee562c8fd1b5949cd2091d46128973ecb9"},"amount":10000000}],"payload":{"type":2,"index":"74616e676c6570726f6f660d0a","data":"74616e676c6570726f6f66206d6573736167650d0a"}},"unlockBlocks":[{"type":0,"signature":{"type":0,"publicKey":"1d8267fad616a9ca4ccbe1119c87a958d3844574b59a10945ca13f8ab05ffa43","signature":"facb2f8d2f07730490a5a6a94011d64d66c9f07d053308b1334597a1c038c48b55e8504cf88c08bc9ed2cf2e748d91ddfa5f81771db2dc1487ae6593e26a9409"}}]},"nonce":"9223372036854944079"},"transactionMessages":[{"networkId":"7712883261355838377","parentMessageIds":["429b7d4a6d9e7de6e6601e3d69af6b04d80bb10b3fcd1605bffc8db4f35e6e33","892adbd6b903f9167fe9fa0a43aa01b641814d88e07e2fc0b0778a2092d79248","a1a9449048c38681a78af588a0786c1c1b91ea3db6698acb4c7cfeab29767c97","b5f1c92995db5818e854d5a8c43ad5dad4b773bef79246b1a31f9b3502a9afd9"],"payload":{"type":0,"essence":{"type":0,"inputs":[{"type":0,"transactionId":"a513e340debe6f7a5007da20a029e68984fbd511781bfd8ec115b0fed70b4c44","transactionOutputIndex":0}],"outputs":[{"type":0,"address":{"type":0,"address":"ab1ffcb1392ae0d6590507c5de2e90ee562c8fd1b5949cd2091d46128973ecb9"},"amount":10000000}],"payload":{"type":2,"index":"74616e676c6570726f6f660d0a","data":"74616e676c6570726f6f66206d6573736167650d0a"}},"unlockBlocks":[{"type":0,"signature":{"type":0,"publicKey":"1d8267fad616a9ca4ccbe1119c87a958d3844574b59a10945ca13f8ab05ffa43","signature":"facb2f8d2f07730490a5a6a94011d64d66c9f07d053308b1334597a1c038c48b55e8504cf88c08bc9ed2cf2e748d91ddfa5f81771db2dc1487ae6593e26a9409"}}]},"nonce":"9223372036854944079"}]}"#;
    let inclusion_proof: InclusionProof = serde_json::from_str(proof_json).unwrap();
    let inclusion_proof_string = serde_json::to_string(&inclusion_proof).unwrap();
    assert_eq!(proof_json, &inclusion_proof_string);
}