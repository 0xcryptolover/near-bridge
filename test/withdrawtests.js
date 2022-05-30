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
        inst: "9d0115000000000000000000000000000000000000000000000000000000000000000000000000000000000000006674302e63756f6e67637574652e746573746e657411000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000063756f6e67637574652e746573746e6574000000000000000000000000000000000000000000000000000000003b9aca00940860e23318c9abc38340a9f2dfae513aa1371958739c484feffccabe2771475308e0f98d4ad4b72e06600dc3e9e09f6e5f8baf8b84a46e168d0ac42dd4d8ff",
        height: 304,
        inst_paths: [
            to32Bytes("82a8c4d7dcdcf1e28ec58e7218155c8f2e75cdc2aded968d63da53efb8848abb"),

        ],
        inst_path_is_lefts: [
            false,
        ],
        inst_root: to32Bytes("fd64d3bd7f578bbb58ee9088949d96f4186b04b3d4b5751ce0104399d7ba4b7c"),
        blk_data: to32Bytes("b2f85d2ee41b2fc42a7e06dc90ca5ddec6d3b08e84a97519c9f9709315155681"),
        indexes: [1, 2, 3],
        signatures: [
            "af503e8cc61c73d6ae5728d5b37d04ec7fa7aff190040cbcffc213c7e2046e721a8d3fe9c12e62e9802f1bedaf1d38b58ea11ed2e38ec4301dd798c5be4ae469",
            "869f5225d790484190ec4bf5113dadc568e8232c90bb711909f3154ff54b477c3b96040e5d041e76dccad008c6453fdfbee8295119bf641f7b9bee8e0d79aa6d",
            "c3622371e7355ab5e4f48097e0e39b435b444f0557b6269c55cd427aac932fb327f5592748f839bbb98666613ebcb1e7100ea02f26a2becb960ba1ef8d909460",
        ],
        vs: [1, 0, 0],
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