CREATE TABLE app_user (id SERIAL PRIMARY KEY, uid UUID UNIQUE NOT NULL, vk_uid INTEGER UNIQUE NOT NULL);

GRANT SELECT ON TABLE app_user TO recipe_calculator_client;
GRANT INSERT ON TABLE app_user TO recipe_calculator_client;
GRANT SELECT ON TABLE app_user_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE app_user_id_seq TO recipe_calculator_client;