use std::future::Future;
use std::sync::Arc;

use crate::config::Config;
use crate::outside::gp;
use crate::outside::http_client::HttpClient;
use crate::outside::vk;
use crate::server::error::Error;
use crate::server::error::ErrorKind::GPTokenCheckError;
use crate::server::error::ErrorKind::GPTokenCheckUnknownError;
use crate::server::error::ErrorKind::UnsupportedSocialNetwork;
use crate::server::error::ErrorKind::VKTokenCheckError;
use crate::server::error::ErrorKind::VKTokenCheckFail;

use super::user_data_generators::new_gp_token_checker_for;
use super::user_data_generators::new_vk_token_checker_for;
use super::user_data_generators::GpTokenChecker;
use super::user_data_generators::VkTokenChecker;

pub enum TokenChecker {
    VK(Box<dyn VkTokenChecker + Send>),
    GP(Box<dyn GpTokenChecker + Send>),
}

pub enum TokenCheckSuccess {
    VK { uid: String },
    GP { uid: String },
}

pub async fn check_token(
    social_network_type: String,
    social_network_token: String,
    overrides: &str,
    http_client: Arc<HttpClient>,
    config: Config,
) -> Result<TokenCheckSuccess, Error> {
    let token_checker = match social_network_type.as_ref() {
        "vk" => TokenChecker::VK(new_vk_token_checker_for(
            &overrides,
            social_network_token,
            config.vk_server_token().to_string(),
            http_client,
        )),
        "gp" => TokenChecker::GP(new_gp_token_checker_for(
            &overrides,
            social_network_token,
            http_client,
        )),
        _ => return Err(UnsupportedSocialNetwork(social_network_type).into()),
    };

    run_token_checker(token_checker).await
}

async fn run_token_checker(token_checker: TokenChecker) -> Result<TokenCheckSuccess, Error> {
    match token_checker {
        TokenChecker::VK(vk_checker) => check_vk_token(vk_checker).await,
        TokenChecker::GP(gp_checker) => check_gp_token(gp_checker).await,
    }
}

fn check_vk_token(
    vk_checker: Box<dyn VkTokenChecker + Send>,
) -> impl Future<Output = Result<TokenCheckSuccess, Error>> {
    // NOTE: we moved the token checker out of an async context so the returned
    // Future would be Sync
    let check_result = vk_checker.check_token();
    async {
        let check_result = check_result.await?;
        match check_result {
            vk::CheckResult::Success { user_id } => Ok(TokenCheckSuccess::VK { uid: user_id }),
            vk::CheckResult::Fail => Err(VKTokenCheckFail.into()),
            vk::CheckResult::Error {
                error_code,
                error_msg,
            } => Err(VKTokenCheckError(error_code, error_msg).into()),
        }
    }
}

fn check_gp_token(
    gp_checker: Box<dyn GpTokenChecker + Send>,
) -> impl Future<Output = Result<TokenCheckSuccess, Error>> {
    // NOTE: we moved the token checker out of an async context so the returned
    // Future would be Sync
    let check_result = gp_checker.check_token();
    async {
        let check_result = check_result.await?;
        match check_result {
            gp::CheckResult::Success { user_id } => Ok(TokenCheckSuccess::GP { uid: user_id }),
            gp::CheckResult::UnknownError => Err(GPTokenCheckUnknownError.into()),
            gp::CheckResult::Error {
                error_title,
                error_descr,
            } => Err(GPTokenCheckError(error_title, error_descr).into()),
        }
    }
}
