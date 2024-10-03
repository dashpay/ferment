use std::fmt::Debug;
use async_trait::async_trait;
use crate::asyn::dapi_request::DapiRequest;
use crate::asyn::mock::MockResponse;
use crate::asyn::proof::FromProof;
use crate::asyn::query::{Query, RequestSettings, TransportRequest};
use crate::asyn::sdk::Sdk;
use crate::nested::{Identifier, ProtocolError};



#[async_trait]
#[ferment_macro::export]
pub trait Fetch
    where
        Self: Sized
        + Debug
        + MockResponse
        + FromProof<
            <Self as Fetch>::Request,
            Request = <Self as Fetch>::Request,
            Response = <<Self as Fetch>::Request as DapiRequest>::Response,
        > {
    type Request: TransportRequest + Into<<Self as FromProof<<Self as Fetch>::Request>>::Request>;
    async fn fetch<Q: Query<<Self as Fetch>::Request>>(
        sdk: &Sdk,
        query: Q,
    ) -> Result<Option<Self>, ProtocolError> {
        let request = query.query(sdk.prove())?;
        let response = request
            .clone()
            .execute(sdk, RequestSettings::default())
            .await?;
        let object: Option<Self> = sdk.parse_proof(request, response)?;
        match object {
            Some(item) => Ok(item.into()),
            None => Ok(None),
        }
    }
    async fn fetch_by_identifier(sdk: &Sdk, id: Identifier) -> Result<Option<Self>, ProtocolError>
        where
            Identifier: Query<<Self as Fetch>::Request>,
    {
        Self::fetch(sdk, id).await
    }
}

// pub struct Fetch_VTable {
//     pub fetch : unsafe extern "C" fn (
//         sdk : * mut crate :: fermented :: types :: asyn :: sdk :: Sdk ,
//         query : * mut crate :: fermented :: types :: asyn :: query:: TransportRequest ,)
//         -> * mut crate :: fermented :: generics :: Result_ok_Option_crate_asyn_fetch_Fetch_err_crate_nested_ProtocolError ,
//     pub fetch_by_identifier : unsafe extern "C" fn (
//         sdk : * mut crate :: fermented :: types :: asyn :: sdk :: Sdk ,
//         id : * mut crate :: fermented :: types :: nested :: Identifier ,)
//         -> * mut crate :: fermented :: generics :: Result_ok_Option_crate_asyn_fetch_Fetch_err_crate_nested_ProtocolError , }


// pub trait TransportClient: Send + Sized {
//     type Error: CanRetry + Send;
//     fn with_uri(uri: Uri) -> Self;
// }

