use futures::done;
use futures::future::ok;
use futures::Future;
use std::sync::Arc;
use uuid::Uuid;

use config::Config;
use db::core::app_user;
use db::core::connection::DBConnection;
use db::core::error::Error as DBError;
use db::core::error::ErrorKind as DBErrorKind;
use db::core::gp_user;
use db::core::transaction;
use db::core::vk_user;
use outside::gp;
use outside::http_client::HttpClient;
use outside::vk;
use server::error::Error;
use server::error::ErrorKind::GPTokenCheckError;
use server::error::ErrorKind::GPTokenCheckUnknownError;
use server::error::ErrorKind::GPUidDuplicationError;
use server::error::ErrorKind::UniqueUuidCreationError;
use server::error::ErrorKind::UnsupportedSocialNetwork;
use server::error::ErrorKind::VKTokenCheckError;
use server::error::ErrorKind::VKTokenCheckFail;
use server::error::ErrorKind::VKUidDuplicationError;

use super::user_data_generators::new_gp_token_checker_for;
use super::user_data_generators::new_user_uuid_generator_for;
use super::user_data_generators::new_vk_token_checker_for;
use super::user_data_generators::GpTokenChecker;
use super::user_data_generators::UserUuidGenerator;
use super::user_data_generators::VkTokenChecker;

pub struct UserRegistrationResult {
    pub uid: Uuid,
    pub client_token: Uuid,
}

enum TokenChecker {
    VK(Box<dyn VkTokenChecker + Send>),
    GP(Box<dyn GpTokenChecker + Send>),
}

enum TokenCheckSuccess {
    VK { uid: String },
    GP { uid: String },
}

pub fn register_user<Conn>(
    user_name: String,
    social_network_type: String,
    social_network_token: String,
    overrides: String,
    config: Config,
    db_connection: Conn,
    http_client: Arc<HttpClient>,
) -> impl Future<Item = UserRegistrationResult, Error = Error> + Send
where
    Conn: DBConnection + Send,
{
    let user_uuid_generator = new_user_uuid_generator_for(&overrides);

    let token_checker = match social_network_type.as_ref() {
        "vk" => Ok(TokenChecker::VK(new_vk_token_checker_for(
            &overrides,
            social_network_token,
            config.vk_server_token().to_string(),
            http_client,
        ))),
        "gp" => Ok(TokenChecker::GP(new_gp_token_checker_for(
            &overrides,
            social_network_token,
            http_client,
        ))),
        _ => Err(UnsupportedSocialNetwork(social_network_type).into()),
    };

    ok(user_uuid_generator).join(done(token_checker)).and_then(
        |(user_uuid_generator, token_checker)| {
            register_user_impl(user_name, db_connection, user_uuid_generator, token_checker)
        },
    )
}

fn register_user_impl<Conn>(
    user_name: String,
    db_connection: Conn,
    user_uuid_generator: Box<dyn UserUuidGenerator>,
    token_checker: TokenChecker,
) -> impl Future<Item = UserRegistrationResult, Error = Error> + Send
where
    Conn: DBConnection + Send,
{
    run_token_checker(token_checker).and_then(move |checked_token| {
        let db_connection_ref = &db_connection;

        let result = transaction::start(&db_connection, move || {
            let uid = user_uuid_generator.generate();
            let client_token = Uuid::new_v4();

            let app_user = app_user::insert(
                app_user::new(uid, user_name.to_string(), client_token),
                db_connection_ref,
            );
            let app_user = app_user.map_err(extract_uuid_duplication_error)?;

            match checked_token {
                TokenCheckSuccess::VK { uid } => {
                    let vk_user = vk_user::new(uid, &app_user);
                    let vk_user_insertion = vk_user::insert(vk_user, db_connection_ref);
                    vk_user_insertion.map_err(extract_vk_uid_duplication_error)?;
                }
                TokenCheckSuccess::GP { uid } => {
                    let gp_user = gp_user::new(uid, &app_user);
                    let gp_user_insertion = gp_user::insert(gp_user, db_connection_ref);
                    gp_user_insertion.map_err(extract_gp_uid_duplication_error)?;
                }
            };

            Ok(UserRegistrationResult {
                uid: *app_user.uid(),
                client_token: *app_user.client_token(),
            })
        });
        done(result)
    })
}

fn run_token_checker(
    token_checker: TokenChecker,
) -> Box<dyn Future<Item = TokenCheckSuccess, Error = Error> + Send> {
    match token_checker {
        TokenChecker::VK(vk_checker) => Box::new(
            vk_checker
                .check_token()
                .map_err(|err| err.into())
                .and_then(|check_result| {
                    let result = match check_result {
                        vk::CheckResult::Success { user_id } => {
                            Ok(TokenCheckSuccess::VK { uid: user_id })
                        }
                        vk::CheckResult::Fail => Err(VKTokenCheckFail.into()),
                        vk::CheckResult::Error {
                            error_code,
                            error_msg,
                        } => Err(VKTokenCheckError(error_code, error_msg).into()),
                    };
                    done(result)
                }),
        ),
        TokenChecker::GP(gp_checker) => Box::new(
            gp_checker
                .check_token()
                .map_err(|err| err.into())
                .and_then(|check_result| {
                    let result = match check_result {
                        gp::CheckResult::Success { user_id } => {
                            Ok(TokenCheckSuccess::GP { uid: user_id })
                        }
                        gp::CheckResult::UnknownError => Err(GPTokenCheckUnknownError.into()),
                        gp::CheckResult::Error {
                            error_title,
                            error_descr,
                        } => Err(GPTokenCheckError(error_title, error_descr).into()),
                    };
                    done(result)
                }),
        ),
    }
}

fn extract_uuid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        error @ DBError(DBErrorKind::UniqueViolation(_), _) => {
            UniqueUuidCreationError(error).into()
        }
        error => error.into(),
    }
}

fn extract_vk_uid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        DBError(DBErrorKind::UniqueViolation(_), _) => VKUidDuplicationError {}.into(),
        error => error.into(),
    }
}

fn extract_gp_uid_duplication_error(db_error: DBError) -> Error {
    match db_error {
        DBError(DBErrorKind::UniqueViolation(_), _) => GPUidDuplicationError {}.into(),
        error => error.into(),
    }
}
