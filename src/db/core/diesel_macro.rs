#[macro_export]
macro_rules! insert {
    ( $ResultType:ty, $new_item:expr, $table:path, $connection:expr ) => {{
        use db::core::error;
        use diesel;
        use diesel::result::DatabaseErrorKind as DieselDatabaseErrorKind;
        use diesel::result::Error as DieselError;
        use diesel::result::Error::DatabaseError as DieselDatabaseError;
        use diesel::RunQueryDsl;

        let result: Result<$ResultType, DieselError> = diesel::insert_into($table)
            .values(&$new_item)
            .get_result($connection);

        let result: Result<$ResultType, error::Error> = match result {
            Ok(val) => Ok(val),
            Err(error @ DieselDatabaseError(DieselDatabaseErrorKind::UniqueViolation, _)) => {
                Err(error::ErrorKind::UniqueViolation(error).into())
            }
            Err(error) => Err(error.into()),
        };
        result
    }};
}

#[macro_export]
macro_rules! select_by_column {
    ( $Type:ty, $table:path, $column:path, $value:expr, $connection:expr ) => {{
        use db::core::transform_diesel_single_result;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let result = $table
            .filter($column.eq($value))
            .first::<$Type>($connection);
        transform_diesel_single_result(result)
    }};
}

#[macro_export]
macro_rules! delete_by_column {
    ( $table:path, $column:path, $value:expr, $connection:expr ) => {{
        use db::core::error;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let result = diesel::delete($table.filter($column.eq($value))).execute($connection);

        let result: Result<(), error::Error> = match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into()),
        };
        result
    }};
}

#[macro_export]
macro_rules! update_column {
    ( $Type:ty,
      $table:path,
      $searched_column:path,
      $searched_value:expr,
      $updated_column:path,
      $updated_value:expr,
      $connection:expr ) => {{
        use db::core::error;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let target = $table.filter($searched_column.eq($searched_value));
        let results: Result<Vec<$Type>, diesel::result::Error> = diesel::update(target)
            .set($updated_column.eq($updated_value))
            .get_results($connection);

        let results: Result<Vec<$Type>, error::Error> = match results {
            Err(diesel::result::Error::NotFound) => Ok(Vec::with_capacity(0)),
            Err(error) => Err(error.into()),
            Ok(val) => Ok(val),
        };
        results
    }};
}

#[cfg(test)]
#[path = "./diesel_macro_test.rs"]
mod diesel_macro_test;
