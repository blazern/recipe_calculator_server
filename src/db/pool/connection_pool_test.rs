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
    assert_eq!(initial_connections_count + 1, pool.pooled_connections_count());

    {
        let _connection2 = pool.borrow().unwrap();
        assert_eq!(initial_connections_count, pool.pooled_connections_count());
    }

    assert_eq!(initial_connections_count + 1, pool.pooled_connections_count());
}