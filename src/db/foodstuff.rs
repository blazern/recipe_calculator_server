use diesel;
use diesel::pg::PgConnection;

use schema;
use schema::foodstuff;
use error::Error;
use super::app_user::AppUser;

#[derive(Insertable)]
#[table_name="foodstuff"]
pub struct NewFoodstuff {
    app_user_id: i32,
    app_user_foodstuff_id: i32,
    name: String,
    protein: f32,
    fats: f32,
    carbs: f32,
    calories: f32,
    is_listed: bool,
}

#[derive(Debug, PartialEq, Queryable)]
pub struct Foodstuff {
    id: i32,
    app_user_id: i32,
    app_user_foodstuff_id: i32,
    name: String,
    protein: f32,
    fats: f32,
    carbs: f32,
    calories: f32,
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

    pub fn protein(&self) -> f32 {
        return self.protein;
    }

    pub fn fats(&self) -> f32 {
        return self.fats;
    }

    pub fn carbs(&self) -> f32 {
        return self.carbs;
    }

    pub fn calories(&self) -> f32 {
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
        protein: f32,
        fats: f32,
        carbs: f32,
        calories: f32,
        is_listed: bool) -> NewFoodstuff {
    NewFoodstuff {
        app_user_id: app_user.id(),
        app_user_foodstuff_id: app_user_foodstuff_id,
        name: name,
        protein: protein,
        fats: fats,
        carbs: carbs,
        calories: calories,
        is_listed: is_listed
    }
}
pub fn insert(foodstuff: NewFoodstuff, connection: &PgConnection) -> Result<Foodstuff, Error> {
    return insert!(Foodstuff, foodstuff, schema::foodstuff::table, connection);
}

pub fn select_by_id(id: i32, connection: &PgConnection) -> Result<Option<Foodstuff>, Error> {
    return select_by_column!(
        Foodstuff,
        schema::foodstuff::table,
        schema::foodstuff::id,
        id,
        connection);
}

pub fn unlist(foodstuff: Foodstuff, connection: &PgConnection) -> Result<Foodstuff, Error> {
    let result =
        update_column!(
            Foodstuff,
            schema::foodstuff::table,
            schema::foodstuff::id,
            foodstuff.id(),
            schema::foodstuff::is_listed,
            false,
            connection);

    return match result {
        Ok(mut vec) => {
            if vec.len() > 1 {
                panic!("Count of unlisted foodstuffs is {}! Data in DB most likely was just corrupted!", vec.len());
            }
            Ok(vec.pop().expect("Expect 1 foodstuff"))
        }
        Err(err) => {
            Err(err)
        }
    }
}

#[cfg(test)]
#[path = "./foodstuff_test.rs"]
mod foodstuff_test;