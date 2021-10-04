# IOTA address generation

https://thoralf-m.github.io/tangleproof/

```bash
npm i
npm run serve
```

and then open http://localhost:8080

Example proof with `https://api.lb-0.h.chrysalis-devnet.iota.cafe/`:
```
{"latestOutputId":"bb35db97ec63be34a980c6baeaf4922f7c76c03b789cb40a11f23a9281abeec30000","message":{"networkId":"6514788332515804015","parentMessageIds":["4119ce0179160ad6046c4b49505d0936e7cfd7813a69f7a021b40b8dc8ba9235"],"payload":{"type":1,"index":729387,"timestamp":1633117449,"parentMessageIds":["4119ce0179160ad6046c4b49505d0936e7cfd7813a69f7a021b40b8dc8ba9235"],"inclusionMerkleProof":"0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8","nextPoWScore":0,"nextPoWScoreMilestoneIndex":0,"publicKeys":["31ea3169abd40ee4096dfa4940962c64c8500066ca2ec1eedb2c5ee9aa0db5d8","5a1f0250640fb1ba82315b8aee4ebbe466da083a13adf77347c4516376cb4431"],"receipt":null,"signatures":["74f01d3d70c1b3edc85b3c8c6fd3c59310363634b853384b1429d76af26734e52c993f57e5f06d1f9c7897a79d57be3bc1602cefa56b305bc8d062c677aa4703","bfc716ef5862d10770f7dc5410b94f11dd2dd6caaff58cdcc5a17cfd9283c0ee66930854194b160a39855d9910acacf72ff9fdfe24a5d5ddfe06cfe94bc85003"]},"nonce":"383617"},"transactions":[{"type":0,"essence":{"type":0,"inputs":[{"type":0,"transactionId":"7e70e07d6a8a6a60a9086d48ae5c1dede1f76f65ac1631aef93b509a7346f51e","transactionOutputIndex":5}],"outputs":[{"type":0,"address":{"type":0,"address":"6cfe631c248a751a029b5206779992fa80df40dd69c88eced1e81e02c274e601"},"amount":1000000}],"payload":{"type":2,"index":"4368726f6e697374","data":"7b2274696d65223a22323032312d31302d30315431393a34343a33342e3430383135315a222c226d6573736167655f696473223a5b2266323361336138303864613966333735383534333464656334313531383238626635346333633766616135623537656532373665633139343935663638323161225d7d"}},"unlockBlocks":[{"type":0,"signature":{"type":0,"publicKey":"6954e403dde107bdfa93ae9e04a5b96a1310e7be0324f656022276273f700cf3","signature":"dc84732cfbf7252a58d632edda0857f6adba5234f94356a2bcc29d4b0de80878c09b5f1f470836814d0ed80210bdabfc0a097ee608cbbd8280f81a77aaf76a0b"}}]}]}
```
