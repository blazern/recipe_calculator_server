CREATE TABLE foodstuff (
	id SERIAL PRIMARY KEY,
	app_user_id INTEGER NOT NULL REFERENCES app_user(id),
	app_user_foodstuff_id INTEGER NOT NULL,
	name TEXT NOT NULL,
	protein REAL NOT NULL,
	fats REAL NOT NULL,
	carbs REAL NOT NULL,
	calories REAL NOT NULL,
	is_listed BOOLEAN NOT NULL,
	unique(app_user_id, app_user_foodstuff_id));

GRANT SELECT ON TABLE foodstuff TO recipe_calculator_client;
GRANT INSERT ON TABLE foodstuff TO recipe_calculator_client;
GRANT UPDATE ON TABLE foodstuff TO recipe_calculator_client;
GRANT SELECT ON TABLE foodstuff_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE foodstuff_id_seq TO recipe_calculator_client;