SELECT
    id,
    email,
    password_hash,
    username AS "username?",
    created_at,
    updated_at
FROM account
WHERE email = $1