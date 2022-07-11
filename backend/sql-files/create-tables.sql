-- noinspection SqlNoDataSourceInspectionForFile

CREATE TABLE IF NOT EXISTS auth (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    contact_number VARCHAR(255),
    user_password TEXT NOT NULL,
    password_version DOUBLE PRECISION NOT NULL,
    UNIQUE (username),
    UNIQUE (email),
    UNIQUE (contact_number)
);

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    auth_id BIGINT NOT NULL REFERENCES auth(id),
    full_name TEXT NOT NULL,
    profile_picture TEXT,
    description TEXT,
    location_or_region TEXT,
    following TEXT,
    followers TEXT,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS ratings (
    id BIGSERIAL PRIMARY KEY,
    upvotes BIGINT DEFAULT 0,
    views BIGINT DEFAULT 0,
    upvoted_by TEXT
);

CREATE TABLE IF NOT EXISTS buzz (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    description TEXT NOT NULL,
    image_link TEXT,
    video_link TEXT,
    buzz_words TEXT,
    mentioned_users TEXT,
    ratings_id BIGINT REFERENCES ratings(id),
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS reply(
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id),
    buzz_id BIGINT NOT NULL REFERENCES buzz(id),
    reply_content TEXT NOT NULL,
    buzz_words TEXT,
    mentioned_users TEXT,
    ratings_id BIGINT REFERENCES ratings(id),
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS trending (
    id BIGSERIAL PRIMARY KEY,
    trending_id BIGINT,
    description TEXT,
    buzz_words TEXT
);
