ALTER TABLE app_user ADD COLUMN vk_uid INTEGER;
UPDATE app_user SET vk_uid=vk_user.vk_uid FROM vk_user WHERE app_user.id=vk_user.app_user_id;
ALTER TABLE app_user ALTER COLUMN vk_uid SET NOT NULL;

DROP TABLE vk_user;
DROP TABLE device;