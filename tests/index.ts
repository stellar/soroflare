import { xdr, Address, scValToNative } from "@stellar/stellar-sdk";

const path = "./snapshot.json";
const file = Bun.file(path);
const snapshot = await file.json();

// const contract_a = Address.fromScAddress(xdr.ScAddress.scAddressTypeContract(Buffer.from(new Array(32).fill(0)))).toString();

// const contract_b_array = new Array(32).fill(1)
// const contract_b = Address.fromScAddress(xdr.ScAddress.scAddressTypeContract(Buffer.from(contract_b_array))).toString();

// const json: any = {
//     contract_id: contract_b_array,
//     fname: "add_with",
//     ledger_sequence: 0,
//     params: [
//         {
//             address: contract_a
//         },
//         {
//             u32: 5
//         },
//         {
//             u32: 15
//         }
//     ],
//     keys: [],
//     vals: []
// }

// const swaps = [
//     ['CBKMUZNFQIAL775XBB2W2GP5CNHBM5YGH6C3XB7AY6SUVO2IBU3VYK2V', contract_a],
//     ['CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFCT4', contract_b],
//     // ['61f3ec45073b00a29bce98268932a9dcfa4f803efd2a425e0938d44047608b4d', '61f3ec45073b00a29bce98268932a9dcfa4f803efd2a425e0938d44047608b4d'], // contract_a (unsure why this is identical between testing and soroflare but contract_b isn't)
//     ['e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855', 'ba45dcc500ee79e4e6dad8017a74b25341ff527529677d5c24948cbfb7676e94'] // contract_b
// ]

// snapshot.ledger.ledger_entries.forEach((element: any, index: number) => {
//     let [key, val] = element

//     if (
//         key.contract_code
//         || JSON.stringify(key).includes('"durability":"temporary"')
//     ) return

//     swaps.forEach(([a, b]) => {
//         key = JSON.parse(JSON.stringify(key).replace(new RegExp(a, 'g'), b))
//         val = JSON.parse(JSON.stringify(val).replace(new RegExp(a, 'g'), b))
//     })

//     let [entry, live_until] = val

//     json.keys.push(key)
//     json.vals.push({
//         entry,
//         live_until
//     })
// });

const json = {
    "adjustment_config": {
        "instructions": {
            "additive_factor": 50000,
            "multiplicative_factor": 1.02
        },
        "read_bytes": {
            "additive_factor": 0,
            "multiplicative_factor": 1.0
        },
        "refundable_fee": {
            "additive_factor": 0,
            "multiplicative_factor": 1.15
        },
        "tx_size": {
            "additive_factor": 500,
            "multiplicative_factor": 1.1
        },
        "write_bytes": {
            "additive_factor": 0,
            "multiplicative_factor": 1.0
        }
    },
    "contract_id": [
        98,
        128,
        3,
        251,
        171,
        144,
        167,
        231,
        12,
        233,
        182,
        179,
        60,
        187,
        105,
        132,
        91,
        174,
        59,
        69,
        181,
        166,
        40,
        152,
        72,
        3,
        184,
        5,
        204,
        157,
        39,
        17
    ],
    "fname": "add_with",
    "ledger_entries": snapshot.ledger.ledger_entries,
    "ledger_sequence": 0,
    "network": "Test SDF Network ; September 2015",
    "network_config": null,
    "params": [
        {
            "address": "CBKMUZNFQIAL775XBB2W2GP5CNHBM5YGH6C3XB7AY6SUVO2IBU3VYK2V"
        },
        {
            "u32": 5
        },
        {
            "u32": 15
        }
    ],
    "source": [
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0
    ]
}

fetch('http://localhost:8787/executesnapshot', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify(json)
})
    .then(response => response.json())
    .then((data: any) => {
        console.log(JSON.stringify(data, null, 2));
        // console.log(
        //     scValToNative(xdr.ScVal.fromXDR(data.opt.results.xdr, 'base64'))
        // );
    })