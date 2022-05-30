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

    unshieldInfo = {
        inst: "9d01280000000000000000000000000000000000000000000000003030303030303030303030303030303030303030303030303030303030303030303030303030303140000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063756f6e67637574652e746573746e657400000000000000000000000000000000000000000000000000000000000003e86b2bae0bf520a667e39db4fdbabf22909a023e0c197bec4e52ba3b84f35b53855508e0f98d4ad4b72e06600dc3e9e09f6e5f8baf8b84a46e168d0ac42dd4d8ff",
        height: 16,
        inst_paths: [
            to32Bytes("23abf9d3acf3fde6246cce9e392c2154ab8423d8d2e01053e74db7f6d17aea4f"),
        ],
        inst_path_is_lefts: [
            false,
        ],
        inst_root: to32Bytes("45e6d8d759bc5993097236e5f2d17053969f0b769bb1d0f8e222b6c40a0f6af3"),
        blk_data: to32Bytes("eff9f595401e37992a3a1fb0c1908e0d4bb2105eae42c0ef6499483b991f2c91"),
        indexes: [0, 1, 2, 3],
        signatures: [
            "3ba689cfbcbfe81d10f47c0becd911ece7fd1c99ce3bf84c61cf20f3bfc2979438251b39a913e934bd6b61def19fac8da98808cce9b8f428809885364a49d81c",
            "fbb1705370519af0e89fa86ced533123a8a33db842a3d90a7c8c69ee82ce20c44ecf63c0f1646d7f2d173b7d4dae99c16e29af1bedcc5ee1a88e15c132f27136",
            "0cb23956deaaf8070c9dbc36e2035d1b641112d8b75187c7ee834f1dd00adf165c2a88fc3a356c795f6e4df4cf52c81f091d7a4fde215dba1eec47768da7b7ae",
            "6801dc29a7d1784f57c511369f84d68f04630bc7afcaa2b92c03272af26430fb7b93aaae22ce4f44818acb3345db276252ef71c7442cf1fe94d1d230191208cb",
        ],
        vs: [0, 0, 1, 1],
    },
    console.log({unshieldInfo: unshieldInfo});
      
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