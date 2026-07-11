-- Update preset type icons from emoji to Tabler icon names
-- Only updates if the icon still matches the original preset emoji (safe for customizations)
UPDATE item_types SET icon = 'file' WHERE id = 1 AND icon = '📄';
UPDATE item_types SET icon = 'checkbox' WHERE id = 2 AND icon = '✅';
