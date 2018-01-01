use super::ConnectionPool;

include!("../../testing_config.rs.inc");

#[test]
fn can_borrow_and_put_back() {
    let config = get_testing_config();

    let mut pool = ConnectionPool::for_client_user(config);

    let connection = pool.borrow().unwrap();
    pool.put_back(connection);
}

#[test]
fn connections_are_reused() {
    let config = get_testing_config();

    let mut pool = ConnectionPool::for_client_user(config);
    let initial_connections_count = pool.pooled_connections_count();

    let connection1 = pool.borrow().unwrap();
    assert_eq!(initial_connections_count, pool.pooled_connections_count());

    pool.put_back(connection1);
    assert_eq!(initial_connections_count + 1, pool.pooled_connections_count());

    let connection2 = pool.borrow().unwrap();
    assert_eq!(initial_connections_count, pool.pooled_connections_count());

    pool.put_back(connection2);
    assert_eq!(initial_connections_count + 1, pool.pooled_connections_count());
}