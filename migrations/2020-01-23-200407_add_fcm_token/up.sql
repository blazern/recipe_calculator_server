CREATE TABLE fcm_token (
  id SERIAL PRIMARY KEY,
  token_value varchar UNIQUE NOT NULL,
  app_user_id INTEGER UNIQUE NOT NULL REFERENCES app_user(id));

GRANT SELECT ON TABLE fcm_token TO recipe_calculator_client;
GRANT INSERT ON TABLE fcm_token TO recipe_calculator_client;
GRANT DELETE ON TABLE fcm_token TO recipe_calculator_client;
GRANT SELECT ON TABLE fcm_token_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE fcm_token_id_seq TO recipe_calculator_client;