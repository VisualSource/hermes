use actix::{Actor, Context, Handler};

use oxide_auth::{
    endpoint::{Endpoint, OwnerConsent, OwnerSolicitor, Scope, Solicitation, WebResponse},
    frontends::simple::endpoint::{ErrorInto, FnSolicitor, Generic, Vacant},
    primitives::{
        issuer::TokenMap,
        prelude::{AuthMap, Client, ClientMap, RandomGenerator},
    },
};
use oxide_auth_actix::{OAuthMessage, OAuthOperation, OAuthRequest, OAuthResponse, WebError};

pub type HermesEndpoint = Generic<
    ClientMap,
    AuthMap<RandomGenerator>,
    TokenMap<RandomGenerator>,
    Vacant,
    Vec<Scope>,
    fn() -> OAuthResponse,
>;

pub struct OAuthState {
    endpoint: HermesEndpoint,
}

pub enum Extras {
    Get,
    Post(uuid::Uuid),
    Nothing,
    AuthoriztionCode
}

impl OAuthState {
    pub fn preconfigured() -> Self {
        Self {
            endpoint: Generic {
                registrar: vec![Client::confidential(
                    "SomeClientId",
                    "hermes://oauth"
                        .parse::<url::Url>()
                        .expect("failed to parse")
                        .into(),
                    "read".parse().expect("failed to parse scopes"),
                    "ClientSecret".as_bytes(),
                )]
                .into_iter()
                .collect(),
                authorizer: AuthMap::new(RandomGenerator::new(16)),
                issuer: TokenMap::new(RandomGenerator::new(16)),
                solicitor: Vacant,
                scopes: vec!["read".parse().expect("failed to parse scope")],
                response: OAuthResponse::ok,
            },
        }
    }

    pub fn with_solicitor<'a, S>(
        &'a mut self,
        solicitor: S,
    ) -> impl Endpoint<OAuthRequest, Error = WebError> + 'a
    where
        S: OwnerSolicitor<OAuthRequest> + 'static,
    {
        ErrorInto::new(Generic {
            authorizer: &mut self.endpoint.authorizer,
            registrar: &mut self.endpoint.registrar,
            issuer: &mut self.endpoint.issuer,
            solicitor,
            scopes: &mut self.endpoint.scopes,
            response: OAuthResponse::ok,
        })
    }
}

impl Actor for OAuthState {
    type Context = Context<Self>;
}

impl<Op> Handler<OAuthMessage<Op, Extras>> for OAuthState
where
    Op: OAuthOperation,
{
    type Result = Result<Op::Item, Op::Error>;

    fn handle(&mut self, msg: OAuthMessage<Op, Extras>, _ctx: &mut Self::Context) -> Self::Result {
        let (op, ex) = msg.into_inner();

        match ex {
            Extras::Get => {
                let solicitor = FnSolicitor(
                    move |_: &mut OAuthRequest, pre_grant: Solicitation| {
                        let grant = pre_grant.pre_grant();
                        let state = pre_grant.state();
                        let mut response = OAuthResponse::default();

                        response
                            .redirect(
                                format!(
                                    "http://localhost:7433/login?response_type=code&client_id={}&redirect_uri={}&scope={}{}",
                                    grant.client_id,
                                    grant.redirect_uri,
                                    grant.scope,
                                    state
                                        .and_then(|x| Some(format!("&state={}", x)))
                                        .unwrap_or("".to_string())
                                )
                                .parse::<url::Url>()
                                .expect("failed to parse url"),
                            )
                            .expect("failed to set redirect");

                        OwnerConsent::InProgress(response)
                    },
                );

                op.run(self.with_solicitor(solicitor))
            }
            Extras::Post(uuid) => {
                let solicitor = FnSolicitor(move |_: &mut OAuthRequest, _: Solicitation| {
                    OwnerConsent::Authorized(uuid.to_string())
                });

                op.run(self.with_solicitor(solicitor))
            }

            Extras::AuthoriztionCode => {
                let solicitor = FnSolicitor(move |_: &mut OAuthRequest, solicitation: Solicitation|{
                    OwnerConsent::Authorized( solicitation.pre_grant().client_id.clone())
                });

                op.run(self.with_solicitor(solicitor))
            }

            _ => op.run(&mut self.endpoint),
        }
    }
}
