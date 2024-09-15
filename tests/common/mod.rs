pub fn setup() {
    dotenvy::from_filename(".env.test").ok();
}
