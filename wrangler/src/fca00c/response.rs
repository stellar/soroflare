use serde::Serialize;

pub type BasicJsonResponse<T> = JsonResponse<T, ()>;

#[derive(Debug, Serialize)]
pub struct JsonResponse<T, K = ()>
where
    T: Serialize,
    K: Serialize,
{
    message: T,
    status: u16,
    opt: Option<K>,
}

impl<T: Serialize, K: Serialize> JsonResponse<T, K> {
    pub fn new(message: T, status: u16) -> Self {
        Self {
            message,
            status,
            opt: None,
        }
    }

    pub fn with_opt(mut self, opt: K) -> Self {
        self.opt = Some(opt);
        self
    }
}

impl<T: Serialize, K: Serialize> From<JsonResponse<T, K>> for worker::Result<worker::Response> {
    fn from(value: JsonResponse<T, K>) -> Self {
        Ok(worker::Response::from_json(&value)?.with_status(value.status))
    }
}

impl<T: Serialize, K: Serialize> TryFrom<JsonResponse<T, K>> for worker::Response {
    type Error = worker::Error;
    fn try_from(value: JsonResponse<T, K>) -> Result<Self, Self::Error> {
        Ok(worker::Response::from_json(&value)?.with_status(value.status))
    }
}
