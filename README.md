# Soroflare
> Careful:  
> This repository is an early stage of development.   
> It is not recommended to use this code in an production enviroment!

This repository contains the environment and virtual machine running as the backbone for the [FCA00C][fca00c] contest.  

## Virtual Machine 
The virtual machine contained in [soroflare-vm] is designed as a standalone rust crate.  
This allows an easy implementation of Soroban contract execution in arbitrary applications.

## FCA00C backend
A modified version of the actual [FCA00C][fca00c] backend is given in [soroflare-wrangler].  
The backend is built using the Cloudflare Wrangler stack and uses the [worker-rs] framework to compile to
WebAssembly.  
The [soroflare-wrangler] can is as an exemplary implementation of the [soroflare-vm].


[fca00c]: https://fca00c.com
[soroflare-vm]: ./soroflare-vm/
[soroflare-wrangler]: ./soroflare-wrangler/
[worker-rs]: https://github.com/cloudflare/workers-rs


# Usage

In the newly updated soroflare version, you have the possibility of setting up an API for running and testing webassembly modules (more specifically Soroban contracts) from your clients.

Soroflare enables you to define every variable within your testing workflow, and can be specifically useful for testing contracts from your clients while being able to manipulate
the context in which your contracts get executed (for instance, test agains state expiration). At the current stage, you can build and populate the execution context of a contract call with all the
entries the invocation you send to soroflare encompasses.

To make requests lighter and reduce overhead, you can also deploy your binaries to soroflare and then reference them within the snapshot you're sending.

## API

### Upload your binary

POST request to `/uploadwasm`, with the WASM binary as raw body.


### Execute invocation with ledger snapshot

POST request to `/executesnapshot` with the snapshot as JSON body:

For example:

```json
{
    "contract_id": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    "fname": "hello",
    "keys": [
        {
            "contract_data": {
                "contract": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
                "durability": "persistent",
                "key": "ledger_key_contract_instance"
            }
        }
    ],
    "ledger_sequence": 500,
    "params": [
        {
            "symbol": "tdep"
        }
    ],
    "vals": [
        {
            "entry": {
                "data": {
                    "contract_data": {
                        "contract": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
                        "durability": "persistent",
                        "ext": "v0",
                        "key": "ledger_key_contract_instance",
                        "val": {
                            "contract_instance": {
                                "executable": {
                                    "wasm": "cb212e08157def179b96989e9178d8cae62ce7b2155497ade08b08156f1921e8"
                                },
                                "storage": null
                            }
                        }
                    }
                },
                "ext": "v0",
                "last_modified_ledger_seq": 0
            },
            "live_until": 100
        }
    ]
}
```

> Note: We're testing entry expiration here as we've set the snapshot's `ledger_sequence` to `500` and our contract instance entry's `live_until` to ledger `100`. This means that we'll receive an `Expired entries` error in the response along with the information about the expired entries.

> Note: above we're not providing the `LedgerEntry::ContractCode` entry in the snapshot, that's because I've already installed in Soroflare the binaries used by the contracts we're invoking in the call (`cb212e08157def179b96989e9178d8cae62ce7b2155497ade08b08156f1921e8`).
> If you try to run this without having installed the contract code in soroflare, you will receive an error about the module not being found.

## Generating Snapshots

The snapshot format that soroflare accepts is the following:

```rust
pub struct WithSnapshotInput {
    ledger_sequence: u32,
    keys: Vec<LedgerKey>,
    vals: Vec<EntryWithLifetime>,
    contract_id: [u8; 32],
    fname: String,
    params: Vec<ScVal>,
}
```

With `LedgerKey` and `ScVal` being Stellar XDR objects and `EntryWithLifetime` defined as follows:

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct EntryWithLifetime {
    pub entry: LedgerEntry,
    pub live_until: Option<u32>,
}
```

To build a correctly-formed JSON snapshot, you can either serialize to JSON a `WithSnapshotInput` struct (for example, with `serde_json`) or simply follow the objects structure. 

Examples of built Snapshots can be found in the `soroflare-wrangler/src/routes/generic.rs` `test` module.
