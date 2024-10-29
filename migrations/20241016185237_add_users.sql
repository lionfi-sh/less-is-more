-- +goose Up
-- +goose StatementBegin
CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL,
  password_hash TEXT DEFAULT '',
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE images (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  nickname TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()  
);

CREATE TABLE image_versions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  image_id UUID NOT NULL,
  hash TEXT NOT NULL,
  version_number TEXT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()  
);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP TABLE users;
DROP TABLE images;
DROP TABLE image_versions;
-- +goose StatementEnd
