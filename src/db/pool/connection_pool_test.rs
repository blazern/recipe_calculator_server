use super::ConnectionPool;
use testing_utils::testing_config;

#[test]
fn can_borrow_and_put_back() {
    let config = testing_config();

    let mut pool = ConnectionPool::for_client_user(config);

    let _connection = pool.borrow().unwrap();
}

#[test]
fn connections_are_reused() {
    let config = testing_config();

    let mut pool = ConnectionPool::for_client_user(config);
    let initial_connections_count = pool.pooled_connections_count();

    {
        let _connection1 = pool.borrow().unwrap();
        assert_eq!(initial_connections_count, pool.pooled_connections_count());
    }
    assert_eq!(
        initial_connections_count + 1,
        pool.pooled_connections_count()
    );

    {
        let _connection2 = pool.borrow().unwrap();
        assert_eq!(initial_connections_count, pool.pooled_connections_count());
    }

    assert_eq!(
        initial_connections_count + 1,
        pool.pooled_connections_count()
    );
}

#[test]
fn borrowed_connection_can_be_cloned() {
    let config = testing_config();

    let mut pool = ConnectionPool::for_client_user(config);
    let initial_connections_count = pool.pooled_connections_count();

    {
        let connection1 = pool.borrow().unwrap();
        let _connection2 = connection1.try_clone().unwrap();
    }

    assert_eq!(
        initial_connections_count + 2,
        pool.pooled_connections_count()
    );
}

#[test]
fn connections_pool_is_send_and_sync() {
    let try_build_testcase = trybuild::TestCases::new();
    try_build_testcase.pass("src/db/pool/connection_pool_is_send_and_sync.rs");
}
