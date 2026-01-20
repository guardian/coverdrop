use crate::define_role;

// The admin keys are used for updating the status endpoint, updating service logging configurations
// and other system administration tasks.
define_role!(Admin, "Admin", "admin");
