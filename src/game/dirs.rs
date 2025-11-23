use directories::{ProjectDirs, UserDirs};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("com", "kneelawk", "physball")
        .expect("Unable to find project directories");
    pub static ref USER_DIRS: UserDirs = UserDirs::new().expect("Unable to find user directories");
}
