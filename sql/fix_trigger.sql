CREATE OR REPLACE FUNCTION update_modified_column() 
RETURNS TRIGGER AS $$ 
BEGIN 
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = TG_TABLE_NAME AND column_name = 'update_time') THEN 
        NEW.update_time = CURRENT_TIMESTAMP; 
    ELSIF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = TG_TABLE_NAME AND column_name = 'updated_time') THEN 
        NEW.updated_time = CURRENT_TIMESTAMP; 
    END IF; 
    RETURN NEW; 
END; 
$$ LANGUAGE plpgsql;