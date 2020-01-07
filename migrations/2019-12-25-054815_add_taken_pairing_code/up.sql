CREATE TABLE taken_pairing_code (
  id SERIAL PRIMARY KEY,
  app_user_id INTEGER NOT NULL,
  val INTEGER NOT NULL,
  creation_time BIGINT NOT NULL,
  family TEXT NOT NULL,
  unique(family, val),
  unique(family, app_user_id));

GRANT SELECT ON TABLE taken_pairing_code TO recipe_calculator_client;
GRANT INSERT ON TABLE taken_pairing_code TO recipe_calculator_client;
GRANT DELETE ON TABLE taken_pairing_code TO recipe_calculator_client;
GRANT SELECT ON TABLE taken_pairing_code_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE taken_pairing_code_id_seq TO recipe_calculator_client;

CREATE INDEX taken_pairing_code_val_index ON taken_pairing_code(val);
CREATE INDEX taken_pairing_code_creation_time_index ON taken_pairing_code(creation_time);
