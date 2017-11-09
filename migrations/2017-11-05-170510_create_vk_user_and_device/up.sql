CREATE TABLE device (id SERIAL PRIMARY KEY, uuid UUID UNIQUE NOT NULL, app_user_id INTEGER UNIQUE NOT NULL REFERENCES app_user(id));

CREATE TABLE vk_user (id SERIAL PRIMARY KEY, vk_uid INTEGER UNIQUE NOT NULL, app_user_id INTEGER UNIQUE NOT NULL REFERENCES app_user(id));
INSERT INTO vk_user(vk_uid, app_user_id) SELECT vk_uid, id FROM app_user;
ALTER TABLE app_user DROP COLUMN vk_uid;

GRANT SELECT ON TABLE vk_user TO recipe_calculator_client;
GRANT INSERT ON TABLE vk_user TO recipe_calculator_client;
GRANT SELECT ON TABLE vk_user_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE vk_user_id_seq TO recipe_calculator_client;

GRANT SELECT ON TABLE device TO recipe_calculator_client;
GRANT INSERT ON TABLE device TO recipe_calculator_client;
GRANT SELECT ON TABLE device_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE device_id_seq TO recipe_calculator_client;
