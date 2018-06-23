ALTER TABLE foodstuff ALTER COLUMN protein TYPE real USING (protein::real / 1000000);
ALTER TABLE foodstuff ALTER COLUMN fats TYPE real USING (fats::real / 1000000);
ALTER TABLE foodstuff ALTER COLUMN carbs TYPE real USING (carbs::real / 1000000);
ALTER TABLE foodstuff ALTER COLUMN calories TYPE real USING (calories::real / 1000000);