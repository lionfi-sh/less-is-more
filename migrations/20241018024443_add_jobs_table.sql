-- +goose Up
-- +goose StatementBegin
CREATE TYPE "JobStatus" AS ENUM (
  'Pending',
  'Completed',
  'Failed'
);

CREATE TABLE jobs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  model_version_id UUID NOT NULL,
  status "JobStatus" NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DROP TABLE jobs;
DROP TYPE "JobStatus";
-- +goose StatementEnd
