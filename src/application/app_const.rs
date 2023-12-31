pub const API_VERSION: &str = "1";

// user roles
pub const USER_ROLE_ADMIN: &str = "admin";
pub const USER_ROLE_GUEST: &str = "guest";

// JWT related constants
pub const JWT_REDIS_REVOKE_GLOBAL_BEFORE_KEY: &str = "jwt.revoke.global.before";
pub const JWT_REDIS_REVOKE_USER_BEFORE_KEY: &str = "jwt.revoke.user.before";
pub const JWT_REDIS_REVOKED_TOKENS_KEY: &str = "jwt.revoked.tokens";
