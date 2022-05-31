const nearAPI = require("near-api-js");
const fs = require('fs');
// creates keyStore from a private key string
// you can define your key here or use an environment variable

// creates keyStore from a private key string
// you can define your key here or use an environment variable

const { keyStores, KeyPair } = nearAPI;
const keyStore = new keyStores.InMemoryKeyStore();
const PRIVATE_KEY =
    "4GrZBkRSEp8YT6ztXHUu9wzrDb3qrpFpyTzEsFR5yovjbGqt16aKQVR7WHoMUdBoNwe2NJRGZ22mt1o3j2wda1jk";
const SENDER_ADDRESS = "cuongcute.testnet";
// creates a public / private key pair using the provided private key
const keyPair = KeyPair.fromString(PRIVATE_KEY);
console.log({keyPair});
const { connect } = nearAPI;
const axios = require('axios');

(async () => {
    // adds the keyPair you created to keyStore
    await keyStore.setKey("testnet", SENDER_ADDRESS, keyPair);

    const config = {
        networkId: "testnet",
        keyStore,
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
    };
    const near = await connect(config);
    const senderAccount = await near.account(SENDER_ADDRESS)

    

    console.log({senderAddress: senderAccount});

    const contractId = "496add2c24e17711d9512172901b5502df37e10493d247c371eb8dc3e4b173fc";
    console.log(contractId);

    const contract = new nearAPI.Contract(
        senderAccount, // account object đang kết nối
        contractId,
        {
          changeMethods: ["withdraw"], 
          sender: senderAccount,
        }
      );

    let unshieldInfo = {};
    try {
        const burnProof = await axios.post(
            "http://127.0.0.1:9334",
            {
                "jsonrpc": "1.0",
                "method": "getnearburnproof",
                "params": [
                    "c6182a66fd38c4820cc3b9c2c856731a5df29d8bc8e106502c177a76d4ceb1db"
                ],
                "id": 1
            }
        );
        let res = burnProof.data.Result;
        unshieldInfo.inst = res.Instruction;
        unshieldInfo.height = Number('0x' + res.BeaconHeight);
        let inst_paths = [];
        for (const inst_path of res.BeaconInstPath) {
            inst_paths.push(to32Bytes(inst_path));
        }
        unshieldInfo.inst_paths = inst_paths;
        unshieldInfo.inst_path_is_lefts = res.BeaconInstPathIsLeft;
        unshieldInfo.inst_root = to32Bytes(res.BeaconInstRoot);
        unshieldInfo.blk_data = to32Bytes(res.BeaconBlkData);
        unshieldInfo.indexes = res.BeaconSigIdxs;
        let s_r = [], v = [];
        for (const signature of res.BeaconSigs) {
            s_r.push(signature.slice(0, 128));
            v.push(Number(signature.slice(128)));
        }
        unshieldInfo.signatures = s_r;
        unshieldInfo.vs = v;

        console.log({unshieldInfo: unshieldInfo});
    } catch (e) {
        console.log(e);
    }
      
    await contract.withdraw(
        {
            unshield_info: unshieldInfo,
        },
        "300000000000000",
        "0"
    );

})();

function to32Bytes(hexStr) {
    const bytes = Buffer.from(hexStr, "hex");
    const padded = Buffer.alloc(32);
    bytes.copy(padded);
    const arr = [...padded];
    return arr;
}