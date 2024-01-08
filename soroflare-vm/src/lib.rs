use soroban_env_host::xdr::{Limits, ScSymbol, ScVal, ScVec, VecM, WriteXdr};

pub mod helpers;
pub mod soroban_cli;
pub mod soroban_vm;
pub mod soroflare_utils;
pub mod token;


// Easy and simple way of printing XDR arguments to supply to Soroflare.n
#[test]
fn generate_scvec() {
    let symbol = ScVal::Symbol(ScSymbol("tdep".to_string().try_into().unwrap()));
    let vec = ScVec(VecM::try_from(vec![symbol]).unwrap());
    println!("{:?}", vec.to_xdr_base64(Limits::none()).unwrap());
}
