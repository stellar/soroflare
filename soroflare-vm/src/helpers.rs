use soroban_env_host::xdr::{BytesM, Error, ScObject, ScVal, ScVec};

// create [u8; 32] from single u64, first 24 entires will be zero.
#[macro_export]
macro_rules! contract_id {
    ($num:expr) => {
        [
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
            (($num as u64) >> (8 * 7) & 0xff) as u8,
            (($num as u64) >> (8 * 6) & 0xff) as u8,
            (($num as u64) >> (8 * 5) & 0xff) as u8,
            (($num as u64) >> (8 * 4) & 0xff) as u8,
            (($num as u64) >> (8 * 3) & 0xff) as u8,
            (($num as u64) >> (8 * 2) & 0xff) as u8,
            (($num as u64) >> (8 * 1) & 0xff) as u8,
            (($num as u64) >> (8 * 0) & 0xff) as u8,
        ]
    };
}

pub struct ScValHelper(ScVal);

impl From<ScVal> for ScValHelper {
    fn from(value: ScVal) -> Self {
        ScValHelper(value)
    }
}

impl From<ScValHelper> for ScVal {
    fn from(value: ScValHelper) -> Self {
        value.0
    }
}

// BytesN<32>

impl From<[u8; 32]> for ScValHelper {
    fn from(value: [u8; 32]) -> Self {
        ScVal::Object(Some(ScObject::Bytes(
            BytesM::try_from(value.to_vec()).unwrap(),
        )))
        .into()
    }
}

impl<T> TryFrom<Vec<T>> for ScValHelper
where
    T: Into<ScVal> + Clone,
{
    type Error = soroban_env_host::xdr::Error;
    fn try_from(value: Vec<T>) -> Result<Self, Error>
    where
        T: Into<ScVal> + Clone,
    {
        let v = value
            .iter()
            .map(move |b| b.to_owned().into())
            .collect::<Vec<ScVal>>();
        Ok(ScValHelper(ScVal::Object(Some(ScObject::Vec(
            ScVec::try_from(v)?,
        )))))
    }
}
