use super::ConnectionPool;
use crate::testing_utils::config_in_tests;

#[test]
fn can_borrow_and_put_back() {
    let config = config_in_tests();

    let mut pool = ConnectionPool::for_client_user(config);

    let _connection = pool.borrow_connection().unwrap();
}

#[test]
fn connections_are_reused() {
    let config = config_in_tests();

    let mut pool = ConnectionPool::for_client_user(config);
    let initial_connections_count = pool.pooled_connections_count();

    {
        let _connection1 = pool.borrow_connection().unwrap();
        assert_eq!(initial_connections_count, pool.pooled_connections_count());
    }
    assert_eq!(
        initial_connections_count + 1,
        pool.pooled_connections_count()
    );

    {
        let _connection2 = pool.borrow_connection().unwrap();
        assert_eq!(initial_connections_count, pool.pooled_connections_count());
    }

    assert_eq!(
        initial_connections_count + 1,
        pool.pooled_connections_count()
    );
}

#[test]
fn borrowed_connection_can_be_cloned() {
    let config = config_in_tests();

    let mut pool = ConnectionPool::for_client_user(config);
    let initial_connections_count = pool.pooled_connections_count();

    {
        let connection1 = pool.borrow_connection().unwrap();
        let _connection2 = connection1.try_clone().unwrap();
    }

    assert_eq!(
        initial_connections_count + 2,
        pool.pooled_connections_count()
    );
}
