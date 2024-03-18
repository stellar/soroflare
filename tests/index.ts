import { xdr, Address, scValToNative } from "@stellar/stellar-sdk";

const path = "./snapshot.json";
const file = Bun.file(path);
const snapshot = await file.json();

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
    "contract": [
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
    "args": [
        {
            "address": "CBKMUZNFQIAL775XBB2W2GP5CNHBM5YGH6C3XB7AY6SUVO2IBU3VYK2V"
        },
        {
            "u32": 5
        },
        {
            "u32": 95
        }
    ],
    "source_account": [
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