-- +goose Up
-- +goose StatementBegin
  ALTER TABLE images ADD COLUMN image_url TEXT NOT NULL;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
  ALTER TABLE images DROP COLUMN image_url;
-- +goose StatementEnd
