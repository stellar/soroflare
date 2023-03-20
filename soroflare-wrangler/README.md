# FCA00C Wrangler
This repository contains a modified version of the CloudFlare wrangler deployed in the [Stellar FCA00C Challenge](https://fastcheapandoutofcontrol.com). 

Unlike the actual deployment, this implementation only contains the logic for gathering the users' performance (namely `CPU` and `size`).

When a user submits a solution, a local `soroban-host-env` is spun up in which the uploaded WASM is sandboxed and executed.

## Running locally
As the wrangler is written in Rust we need to have the rust toolchain and cargo installed.

The wrangler utilizes a `build.rs` to automatically source all required dependencies, which are given as `WASM` files, and includes them as handy constants in the
`fca00c:embedded:contracts` package.  
To compile the rust code into a wrangler readable format, we need to have `worker-build` installed.
The installation of this binary is already included in the `make all` script.

After we set up everything (by just running `make all`), we are able to deploy the wrangler using the default `wrangler`.

To run the wrangler locally, one can utilize `wrangler dev`, or alternatively `make local`.

## API Endpoints

In general: 

TaksID:
| ID | Name      |
|----|-----------|
| 0  | Asteroids |


All endpoints return either `200`, `400`, `401`, `404`, `419`, or `500`.
If the result is not `200`: the field `message` will contain further information.

### POST `/submit?task=`
This endpoint requires authorization.  

The parameter `task` is as defined above.

The body of this request must contain the soroban contract to be submitted in WASM format.
The body is to be submitted as binary blob.

If successful (200):
```json
{
    "message": "Successfully completed challenge",
    "status": 200,
    "opt": {
    "submission": {
        "mem": 33006915,
        "cpu": 605964886,
        "size": 4786,
        "submission_time": 1675978724893,
        "interface_version": 29
    }
}
```

The `opt.submission` object contains the raw information about the submission.  
