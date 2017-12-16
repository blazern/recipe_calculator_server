#[macro_export]
macro_rules! insert {
    ( $ResultType:ty, $new_item:expr, $table:path, $connection:expr ) => {
        {
            use diesel;
            use diesel::LoadDsl;
            use diesel::result::DatabaseErrorKind as DieselDatabaseErrorKind;
            use diesel::result::Error as DieselError;
            use diesel::result::Error::DatabaseError as DieselDatabaseError;
            use db::error;

            let result: Result<$ResultType, DieselError> = diesel::insert(&$new_item)
                .into($table)
                .get_result($connection);

            let result: Result<$ResultType, error::Error> = match result {
                Ok(val) => {
                    Ok(val)
                }
                Err(error @ DieselDatabaseError(DieselDatabaseErrorKind::UniqueViolation, _)) => {
                    Err(error::ErrorKind::UniqueViolation(error).into())
                }
                Err(error) => {
                    Err(error.into())
                }
            };
            result
        }
    };
}

#[macro_export]
macro_rules! select_by_column {
    ( $Type:ty, $table:path, $column:path, $value:expr, $connection:expr ) => {
        {
            use diesel::ExpressionMethods;
            use diesel::FirstDsl;
            use diesel::FilterDsl;
            use db::error;

            let result = $table.filter($column.eq($value)).first::<$Type>($connection);
            let result: Result<Option<$Type>, error::Error> = match result {
                Err(diesel::result::Error::NotFound) => {
                    Ok(None)
                }
                Err(error) => {
                    Err(error.into())
                }
                Ok(val) => {
                    Ok(Some(val))
                }
            };
            result
        }
    };
}

#[macro_export]
macro_rules! delete_by_column {
    ( $table:path, $column:path, $value:expr, $connection:expr ) => {
        {
            use diesel::ExecuteDsl;
            use diesel::ExpressionMethods;
            use diesel::FilterDsl;
            use db::error;

            let result =
                diesel::delete(
                    $table.filter(
                        $column.eq($value)))
                    .execute($connection);

            let result: Result<(), error::Error> = match result {
                Ok(_) => {
                    Ok(())
                }
                Err(err) => {
                    Err(err.into())
                }
            };
            result
        }
    };
}

#[macro_export]
macro_rules! update_column {
    ( $Type:ty,
      $table:path,
      $searched_column:path,
      $searched_value:expr,
      $updated_column:path,
      $updated_value:expr,
      $connection:expr ) => {
        {
            use diesel::ExpressionMethods;
            use diesel::FilterDsl;
            use diesel::LoadDsl;
            use db::error;

            let target = $table.filter($searched_column.eq($searched_value));
            let results: Result<Vec<$Type>, diesel::result::Error> =
                diesel::update(target).set(
                    $updated_column.eq($updated_value))
                .get_results($connection);

            let results: Result<Vec<$Type>, error::Error> = match results {
                Err(diesel::result::Error::NotFound) => {
                    Ok(Vec::with_capacity(0))
                }
                Err(error) => {
                    Err(error.into())
                }
                Ok(val) => {
                    Ok(val)
                }
            };
            results
        }
    };
}

#[cfg(test)]
#[path = "./diesel_macro_test.rs"]
mod diesel_macro_test;