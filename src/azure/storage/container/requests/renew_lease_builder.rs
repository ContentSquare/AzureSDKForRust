use azure::core::errors::{check_status_extract_headers_and_body, AzureError};
use azure::core::headers::LEASE_ACTION;
use azure::core::lease::LeaseId;
use azure::core::{
    ClientRequestIdOption, ClientRequestIdSupport, ClientRequired, ContainerNameRequired, ContainerNameSupport, LeaseIdRequired,
    LeaseIdSupport, TimeoutOption, TimeoutSupport,
};
use azure::core::{No, ToAssign, Yes};
use azure::storage::client::Client;
use azure::storage::container::responses::RenewLeaseResponse;
use futures::future::{done, Future};
use hyper::{Method, StatusCode};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    client: &'a Client,
    p_container_name: PhantomData<ContainerNameSet>,
    p_lease_id: PhantomData<LeaseIdSet>,
    container_name: Option<&'a str>,
    client_request_id: Option<&'a str>,
    timeout: Option<u64>,
    lease_id: Option<&'a LeaseId>,
}

impl<'a> RenewLeaseBuilder<'a, No, No> {
    pub(crate) fn new(client: &'a Client) -> RenewLeaseBuilder<'a, No, No> {
        RenewLeaseBuilder {
            client,
            p_container_name: PhantomData {},
            container_name: None,
            p_lease_id: PhantomData {},
            lease_id: None,
            client_request_id: None,
            timeout: None,
        }
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> ClientRequired<'a> for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    fn client(&self) -> &'a Client {
        self.client
    }
}

impl<'a, LeaseIdSet> ContainerNameRequired<'a> for RenewLeaseBuilder<'a, Yes, LeaseIdSet>
where
    LeaseIdSet: ToAssign,
{
    fn container_name(&self) -> &'a str {
        self.container_name.unwrap()
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> ClientRequestIdOption<'a> for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    fn client_request_id(&self) -> Option<&'a str> {
        self.client_request_id
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> TimeoutOption for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    fn timeout(&self) -> Option<u64> {
        self.timeout
    }
}

impl<'a, ContainerNameSet> LeaseIdRequired<'a> for RenewLeaseBuilder<'a, ContainerNameSet, Yes>
where
    ContainerNameSet: ToAssign,
{
    fn lease_id(&self) -> &'a LeaseId {
        self.lease_id.unwrap()
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> ContainerNameSupport<'a> for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    type O = RenewLeaseBuilder<'a, Yes, LeaseIdSet>;

    fn with_container_name(self, container_name: &'a str) -> Self::O {
        RenewLeaseBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_lease_id: PhantomData {},
            container_name: Some(container_name),
            client_request_id: self.client_request_id,
            timeout: self.timeout,
            lease_id: self.lease_id,
        }
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> ClientRequestIdSupport<'a> for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    type O = RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>;

    fn with_client_request_id(self, client_request_id: &'a str) -> Self::O {
        RenewLeaseBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_lease_id: PhantomData {},
            container_name: self.container_name,
            client_request_id: Some(client_request_id),
            timeout: self.timeout,
            lease_id: self.lease_id,
        }
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> TimeoutSupport for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    type O = RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>;

    fn with_timeout(self, timeout: u64) -> Self::O {
        RenewLeaseBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_lease_id: PhantomData {},
            container_name: self.container_name,
            client_request_id: self.client_request_id,
            timeout: Some(timeout),
            lease_id: self.lease_id,
        }
    }
}

impl<'a, ContainerNameSet, LeaseIdSet> LeaseIdSupport<'a> for RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{
    type O = RenewLeaseBuilder<'a, ContainerNameSet, Yes>;

    fn with_lease_id(self, lease_id: &'a LeaseId) -> Self::O {
        RenewLeaseBuilder {
            client: self.client,
            p_container_name: PhantomData {},
            p_lease_id: PhantomData {},
            container_name: self.container_name,
            client_request_id: self.client_request_id,
            timeout: self.timeout,
            lease_id: Some(lease_id),
        }
    }
}

// methods callable regardless
impl<'a, ContainerNameSet, LeaseIdSet> RenewLeaseBuilder<'a, ContainerNameSet, LeaseIdSet>
where
    ContainerNameSet: ToAssign,
    LeaseIdSet: ToAssign,
{}

impl<'a> RenewLeaseBuilder<'a, Yes, Yes> {
    pub fn finalize(self) -> impl Future<Item = RenewLeaseResponse, Error = AzureError> {
        let mut uri = format!(
            "https://{}.blob.core.windows.net/{}?comp=lease&restype=container",
            self.client().account(),
            self.container_name()
        );

        if let Some(nm) = TimeoutOption::to_uri_parameter(&self) {
            uri = format!("{}&{}", uri, nm);
        }

        let req = self.client().perform_request(
            &uri,
            &Method::PUT,
            |ref mut request| {
                ClientRequestIdOption::add_header(&self, request);
                LeaseIdRequired::add_header(&self, request);
                request.header(LEASE_ACTION, "renew");
            },
            Some(&[]),
        );

        done(req)
            .from_err()
            .and_then(move |future_response| check_status_extract_headers_and_body(future_response, StatusCode::OK))
            .and_then(|(headers, _body)| done(RenewLeaseResponse::from_headers(&headers)))
    }
}
