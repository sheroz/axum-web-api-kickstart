use std::collections::HashSet;

use crate::application::app_const::USER_ROLE_ADMIN;

pub fn is_role_admin(roles: &str) -> bool {
    if roles.is_empty() {
        return false;
    }
    roles
        .split(',')
        .map(|s| s.trim())
        .collect::<HashSet<&str>>()
        .contains(USER_ROLE_ADMIN)
}
