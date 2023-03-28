use std::{sync::Arc, time::Duration};

use my_grpc_extensions::{GrpcChannel, GrpcClientSettings};
use tonic::transport::Channel;

use crate::accounts_manager_grpc::{
    accounts_manager_grpc_service_client::AccountsManagerGrpcServiceClient,
    AccountManagerGetClientAccountGrpcRequest, AccountManagerGetClientAccountGrpcResponse,
    AccountManagerUpdateAccountBalanceGrpcRequest, AccountManagerUpdateAccountBalanceGrpcResponse,
};

struct AccountsManagerSettingsGrpcUrl(String);

impl AccountsManagerSettingsGrpcUrl {
    pub fn new(url: String) -> Self {
        Self(url)
    }
}

#[tonic::async_trait]
impl GrpcClientSettings for AccountsManagerSettingsGrpcUrl {
    async fn get_grpc_url(&self, _: &'static str) -> String {
        self.0.clone()
    }
}

pub struct AccountsManagerGrpcClient {
    channel: GrpcChannel,
}

impl AccountsManagerGrpcClient {
    pub async fn new(grpc_address: String) -> Self {
        Self {
            channel: GrpcChannel::new(
                Arc::new(AccountsManagerSettingsGrpcUrl::new(grpc_address)),
                "accounts_manager",
                Duration::from_secs(10),
            ),
        }
    }

    async fn create_grpc_service(&self) -> AccountsManagerGrpcServiceClient<Channel> {
        return AccountsManagerGrpcServiceClient::new(self.channel.get_channel().await.unwrap());
    }

    pub async fn get_client_account(
        &self,
        trader_id: &str,
        account_id: &str,
    ) -> AccountManagerGetClientAccountGrpcResponse {
        let mut grpc_client = self.create_grpc_service().await;
        let request = AccountManagerGetClientAccountGrpcRequest {
            trader_id: trader_id.to_string(),
            account_id: account_id.to_string(),
        };
        return grpc_client
            .get_client_account(tonic::Request::new(request))
            .await
            .unwrap()
            .into_inner();
    }

    pub async fn update_client_balance(
        &self,
        trader_id: &str,
        account_id: &str,
        balance_delta: f64,
        process_id: &str,
    ) -> AccountManagerUpdateAccountBalanceGrpcResponse {
        let mut grpc_client = self.create_grpc_service().await;

        return grpc_client
            .update_client_account_balance(AccountManagerUpdateAccountBalanceGrpcRequest {
                trader_id: trader_id.to_string(),
                account_id: account_id.to_string(),
                delta: balance_delta,
                comment: "Open position balance charge".to_string(),
                process_id: process_id.to_string(),
                allow_negative_balance: false,
            })
            .await
            .unwrap()
            .into_inner();
    }
}
