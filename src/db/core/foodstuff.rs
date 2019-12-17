use diesel;

use super::app_user::AppUser;
use super::connection::DBConnection;
use super::diesel_connection;
use super::error::Error;

table! {
    foodstuff {
        id -> Integer,
        app_user_id -> Integer,
        app_user_foodstuff_id -> Integer,
        name -> VarChar,
        protein -> Integer,
        fats -> Integer,
        carbs -> Integer,
        calories -> Integer,
        is_listed -> Bool,
    }
}
use self::foodstuff as foodstuff_schema;

#[derive(Insertable)]
#[table_name = "foodstuff"]
pub struct NewFoodstuff {
    app_user_id: i32,
    app_user_foodstuff_id: i32,
    name: String,
    protein: i32,
    fats: i32,
    carbs: i32,
    calories: i32,
    is_listed: bool,
}

#[derive(Debug, PartialEq, Queryable)]
pub struct Foodstuff {
    id: i32,
    app_user_id: i32,
    app_user_foodstuff_id: i32,
    name: String,
    protein: i32,
    fats: i32,
    carbs: i32,
    calories: i32,
    is_listed: bool,
}

impl Foodstuff {
    pub fn id(&self) -> i32 {
        return self.id;
    }

    pub fn app_user_id(&self) -> i32 {
        return self.app_user_id;
    }

    pub fn app_user_foodstuff_id(&self) -> i32 {
        return self.app_user_foodstuff_id;
    }

    pub fn name(&self) -> &str {
        return &self.name;
    }

    pub fn protein(&self) -> i32 {
        return self.protein;
    }

    pub fn fats(&self) -> i32 {
        return self.fats;
    }

    pub fn carbs(&self) -> i32 {
        return self.carbs;
    }

    pub fn calories(&self) -> i32 {
        return self.calories;
    }

    pub fn is_listed(&self) -> bool {
        return self.is_listed;
    }
}

pub fn new(
    app_user: &AppUser,
    app_user_foodstuff_id: i32,
    name: String,
    protein: i32,
    fats: i32,
    carbs: i32,
    calories: i32,
    is_listed: bool,
) -> NewFoodstuff {
    let app_user_id = app_user.id();
    NewFoodstuff {
        app_user_id,
        app_user_foodstuff_id,
        name,
        protein,
        fats,
        carbs,
        calories,
        is_listed,
    }
}
pub fn insert(foodstuff: NewFoodstuff, connection: &dyn DBConnection) -> Result<Foodstuff, Error> {
    return insert!(
        Foodstuff,
        foodstuff,
        foodstuff_schema::table,
        diesel_connection(connection)
    );
}

pub fn select_by_id(id: i32, connection: &dyn DBConnection) -> Result<Option<Foodstuff>, Error> {
    return select_by_column!(
        Foodstuff,
        foodstuff_schema::table,
        foodstuff_schema::id,
        id,
        diesel_connection(connection)
    );
}

pub fn unlist(foodstuff: Foodstuff, connection: &dyn DBConnection) -> Result<Foodstuff, Error> {
    let result = update_column!(
        Foodstuff,
        foodstuff_schema::table,
        foodstuff_schema::id,
        foodstuff.id(),
        foodstuff_schema::is_listed,
        false,
        diesel_connection(connection)
    );

    return match result {
        Ok(mut vec) => {
            if vec.len() > 1 {
                panic!("Count of unlisted foodstuffs is {}! Data in DB most likely was just corrupted!", vec.len());
            }
            Ok(vec.pop().expect("Expect 1 foodstuff"))
        }
        Err(err) => Err(err),
    };
}

#[cfg(test)]
#[path = "./foodstuff_test.rs"]
mod foodstuff_test;
