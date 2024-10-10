INSERT INTO account (
    email, password_hash
)
VALUES (
    $1, $2
)
RETURNING
    id,
    email,
    password_hash,
    username AS "username?",
    created_at,
    updated_at