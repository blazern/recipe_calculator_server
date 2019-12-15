ALTER TABLE vk_user ALTER COLUMN vk_uid TYPE integer USING (vk_uid::integer);
