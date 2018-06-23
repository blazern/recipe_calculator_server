ALTER TABLE foodstuff ALTER COLUMN protein TYPE integer USING ((protein * 1000000)::integer);
ALTER TABLE foodstuff ALTER COLUMN fats TYPE integer USING ((fats * 1000000)::integer);
ALTER TABLE foodstuff ALTER COLUMN carbs TYPE integer USING ((carbs * 1000000)::integer);
ALTER TABLE foodstuff ALTER COLUMN calories TYPE integer USING ((calories * 1000000)::integer);