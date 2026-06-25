SELECT conname, pg_get_constraintdef(c.oid) as def FROM pg_constraint c JOIN pg_class t ON c.conrelid = t.oid WHERE t.relname = 'mxx_system_menu' AND c.contype IN ('u','p');
