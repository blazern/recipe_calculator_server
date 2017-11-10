#[macro_export]
macro_rules! insert {
    ( $ResultType:ty, $new_item:ident, $table:path, $connection:expr ) => {
        {
            use diesel::LoadDsl;
            use diesel;
            use error;

            let result: Result<$ResultType, diesel::result::Error> = diesel::insert(&$new_item)
                .into($table)
                .get_result($connection);

            let result: Result<$ResultType, error::Error> = match result {
                Ok(val) => {
                    Ok(val)
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
            use error;

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
            let result =
                diesel::delete(
                    $table.filter(
                        $column.eq($value)))
                    .execute($connection);
            result
        }
    };
}

#[cfg(test)]
#[path = "./diesel_macro_test.rs"]
mod diesel_macro_test;