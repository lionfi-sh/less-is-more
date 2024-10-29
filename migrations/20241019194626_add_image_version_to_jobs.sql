-- +goose Up
-- +goose StatementBegin
ALTER TABLE jobs RENAME COLUMN model_version_id TO image_version_id;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE jobs RENAME COLUMN image_version_id TO model_version_id;
-- +goose StatementEnd
