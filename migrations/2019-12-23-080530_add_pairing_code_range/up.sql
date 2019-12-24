CREATE TABLE pairing_code_range (
  id SERIAL PRIMARY KEY,
  left_code INTEGER NOT NULL,
  right_code INTEGER NOT NULL,
  family TEXT NOT NULL);

GRANT SELECT ON TABLE pairing_code_range TO recipe_calculator_client;
GRANT INSERT ON TABLE pairing_code_range TO recipe_calculator_client;
GRANT UPDATE ON TABLE pairing_code_range TO recipe_calculator_client;
GRANT DELETE ON TABLE pairing_code_range TO recipe_calculator_client;
GRANT SELECT ON TABLE pairing_code_range_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE pairing_code_range_id_seq TO recipe_calculator_client;

CREATE INDEX pairing_code_range_left_code_index ON pairing_code_range(left_code);
CREATE INDEX pairing_code_range_right_code_index ON pairing_code_range(right_code);
