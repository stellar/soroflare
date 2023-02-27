# FCA00C Wrangler
This wrangler will host the backend for the FCA00C challenges.  
It exposes endpoints where users can submit solutions, get personal stats and the global leaderboard.

When a user submits a solution a local `soroban-host-env` is spun up in which the uploaded WASM is sandboxed and executed.

## Running locally
As the wrangler is written in rust we need to have the rust toolchain as well as cargo installed.

The wrangler utilizes a `build.rs` to automatically source all required dependency which are given as `WASM` and includes them as handy constants in the
`fca00c:embedded:contracts` package.
It will also embed all html files given in `html/` as `fc00c:embedded:html`.

The wrangler depends on the different dependencies being located in the `{parent}/target/wasm32-unknown-unknown` directory, it is necessary to firstly compile
all required depenencies (e.g. `soroban-asteroids-game-engine`).  
To do this automatically run `make all` in `{parent}`.

To compile the rust code into a wrangler readable format we need to have `worker-build` installed.
The instlalation of this binary is already included in the `make all` script.
Alternativy we could do this manually using `cargo install worker-build`.

NOTE: We are actually using a modifiled version as this bug is annoying https://github.com/cloudflare/workers-rs/pull/256
So just use the Makefile...

After we set up everything (by just running `make all`) we are able to deploy the wrangler using the default `wrangler` cli when located in the `{parent}/wrangler` subdirectory:

```
wrangler dev --local # Run locally with _everything_ local. (miniflare) 
wrangler dev # Run with online dev D1 and KV
```

Note: It seems than when running using the `--local` flag no data is retained between calls, thus endpoints like the leaderboard will always appear empty.

## API Endpoints

In general: 

TaksID:
| ID | Name      |
|----|-----------|
| 0  | Asteroids |

BucketID:

| ID | Description                           |
|----|---------------------------------------|
| 0  | Fastest Submission                    | 
| 1  | Lowest WASM size (smallest)           |
| 2  | Lowest Resources [CPU/MEM] (cheapest) |

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
    "message": "Successfully completed challenge.",
    "status": 200,
    "opt": {
    "improvement": [
        false,
        true,
        false
    ],
    "live": [
        true,
        true,
        true
    ],
    "submission": {
        "mem": 33006915,
        "cpu": 605964886,
        "size": 4786,
        "submission_time": 1675978724893,
        "interface_version": 27
    }
}
```

The `opt.submission` object contains the raw information about the submission.  

The `opt.live` field shows if the submission is eligible for potential rewards during the live quest for each bucket.  

The `opt.improvement` contains information if the solution submitted was an improvement compared to a user's previous submission.  
The array index corresponds to the bucket id.  
A users initial submission will always return `[false, false, false]` (i.e. the initial submission is not an improvement).

If already submitted solution recently (429):
```json
{
    "message": "User has already submitted a solution to this task in the last 60 seconds",
    "status": 419,
    "opt": 54
}
```
The `opt` field shows how many seconds need to pass until the user may submit a new solution to said (task, bucket) tuple.

The `opt` map contains the raw data used to evaluate the submission.
`submission_time` is given as UNIX-millis.
Other parameters are given as returned by the soroban environment.

### GET `/leaderboard?task=&bucket=&limit=&live=`
The parameters `live` and `limit` are all optional.
If not specified `limit` will default to `50`.
If not specified `live` will default to `false`

Authentication is optional and will allow to determine if a leaderboard entry belongs to the authenticated user.

`task`, `bucket` and `live` are used to filter the leaderboard results.

`task` and `bucket` are as defined above.
`live` will filter for submission done during the 'live' phase of the task.


The result will be of the form `[LeaderboardEntry]` (`Vec<LeaderboardEntry>`).

`LeaderboardEntry` consists of:

| Field             | Type           | Description                                                                 |
|-------------------|----------------|-----------------------------------------------------------------------------|
| bucket_id         | int            | As defined above                                                            |
| task_id           | int            | As defined above                                                            |
| CPU               | int            | CPU metric as returned by soroban                                           |
| MEM               | int            | CPU metric as returned by soroban                                           |
| SIZE              | int            | WASM size in bytes                                                          |
| submission_date   | int            | date submitted in UNIX-millis                                               |
| interface_version | int            | soroban version submission was made with                                    |
| rank              | int            | Rank of submission in given bucket; Starting at 1                           |
| display           | string \| null | Displayname of user if set; `null` if anonymous                             |
| anon_index        | int \| null    | Unique identifier to each anonymous account `null` if `display` is set      |
| profile_url       | string \| null | Profile URL of user if set; `null` if none                                  |
| live              | bool           | `true` if submitted while quest was live                                    |
| improved          | bool           | `true` if this was at least the 2nd submission, improving the initial one   |
| me                | bool           | `true` auth header was set and this entry belongs to the authenticated user |


Note: The array index of any given entry in any given task/bucket entry does not necessarily correspond to their rank! (If user is authenticated but not ranked within `limit` they will appear in the array at index `limit+1` but their `rank` may be different.)

```json
[
    {
        "bucket_id": 0,
        "task_id": 0,
        "CPU": 611943909,
        "MEM": 39894755,
        "SIZE": 4786,
        "submission_date": 1675177622546,
        "interface_version": 27,
        "rank": 1,
        "anon_index": 5,
        "display": null,
        "profile_url": null,
        "live": true,
        "improved": false,
        "me" : false
    },
    {
        "bucket_id": 0,
        "task_id": 0,
        "CPU": 1090785050,
        "MEM": 66043230,
        "SIZE": 5044,
        "submission_date": 1675380354090,
        "interface_version": 27,
        "rank": 2,
        "anon_index": 6,
        "display": null,
        "profile_url": null,
        "live": true,
        "improved": false,
        "me": true
    },
    {
        "bucket_id": 0,
        "task_id": 0,
        "CPU": 393119176,
        "MEM": 15825415,
        "SIZE": 806,
        "submission_date": 1675391611277,
        "interface_version": 27,
        "rank": 3,
        "anon_index": 7,
        "display": "Hello world",
        "profile_url": "https://stellar.org",
        "live": true,
        "improved": false,
        "me": false
    }
]
```

### GET `/personal?task=&bucket=&live=`
Required authorization.

`live` is optional and used to filter the leaderboard results. If not specified no filtering will occur.

`task` and `bucket` are as defined above.
`live` will filter for submission done during the 'live' phase of the task.

This endpoint will return only the personal positions in the leaderboard.

The structure of the result will be a single `LeaderboardEntry` as defined above.
```json
{
    "bucket_id": 0,
    "task_id": 0,
    "CPU": 1090785050,
    "MEM": 66043230,
    "SIZE": 5044,
    "submission_date": 1675380354090,
    "interface_version": 27,
    "rank": 2,
    "anon_index": 0,
    "display": null,
    "profile_url": null,
    "live": true,
    "improved": false,
    "me" : true
}
```
NOTE: In the `/personal` endpoint `me` will always be true.

## Structure
The `lib.rs` is the handler of the incoming worker request and delegates it to the coresponding function in `fca00c:router`.
If the function is `submit` it will take the given taksID and look up the verify function to execute within the `fca00c:mod.rs`  
Tasks are registered modually such that the package `fca00c:tasks:` will include all checks required for each task.
A new task may be registered by implementing the trait `fca00c:tasks:Task` for any struct (typically just a `pub struct TaskName;`).
To register a task add it to the `setup` function in `fca00c:tasks:mod.rs`.
