CREATE TABLE paired_partners (
  id SERIAL PRIMARY KEY,
  partner1_user_id INTEGER NOT NULL REFERENCES app_user(id),
  partner2_user_id INTEGER NOT NULL REFERENCES app_user(id),
  pairing_state INTEGER NOT NULL,
  pairing_start_time BIGINT NOT NULL,
  unique(partner1_user_id, partner2_user_id));

GRANT SELECT ON TABLE paired_partners TO recipe_calculator_client;
GRANT INSERT ON TABLE paired_partners TO recipe_calculator_client;
GRANT DELETE ON TABLE paired_partners TO recipe_calculator_client;
GRANT SELECT ON TABLE paired_partners_id_seq TO recipe_calculator_client;
GRANT UPDATE ON TABLE paired_partners_id_seq TO recipe_calculator_client;

CREATE INDEX paired_partners_partner1_user_id_index ON paired_partners(partner1_user_id);
CREATE INDEX paired_partners_partner2_user_id_index ON paired_partners(partner2_user_id);
CREATE INDEX paired_partners_pairing_start_time_index ON paired_partners(pairing_start_time);
