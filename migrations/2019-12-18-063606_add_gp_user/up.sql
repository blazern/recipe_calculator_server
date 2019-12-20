CREATE TABLE gp_user (id SERIAL PRIMARY KEY, gp_uid varchar UNIQUE NOT NULL, app_user_id INTEGER UNIQUE NOT NULL REFERENCES app_user(id));

GRANT SELECT ON TABLE gp_user TO recipe_calculator_client;
GRANT INSERT ON TABLE gp_user TO recipe_calculator_client;
GRANT SELECT ON TABLE gp_user_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE gp_user_id_seq TO recipe_calculator_client;